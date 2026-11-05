use crate::challenge::model::{ChallengeMark, ServiceOrderChallenge};
use crate::challenge::service::ChallengeService;
use crate::config::{ApplicationConfig, ConfigurationMap};
use crate::directory::service::DirectoryService;
use crate::helpers::resources::get_shared_resources;
use crate::order::error::OrderServiceError;
use crate::order::model::{DownloadStatus, ServiceUserOrder};
use crate::user::model::ServiceUser;
use sentry_db::repo::order_repo::{Order, OrderRepo};
use sentry_jose::keys::PrivateKey;
use sentry_plugin_engine::loader::{SetConfig, run_plugin};
use sentry_plugin_engine::run_mode::{DiscoveryType, RunMode};
use sentry_rfc8555::certificate::CertificateOperations;
use sentry_rfc8555::order::OrderOperations;
use std::sync::Arc;
use tracing::{info, instrument};

pub struct OrderService {
    configuration: Arc<ApplicationConfig>,
    order_repo: Arc<dyn OrderRepo + Send + Sync>,

    directory_service: Arc<DirectoryService>,
    challenge_service: Arc<ChallengeService>,
}

impl OrderService {
    pub fn new(
        configuration: Arc<ApplicationConfig>,
        order_repo: Arc<dyn OrderRepo + Send + Sync>,
        directory_service: Arc<DirectoryService>,
        challenge_service: Arc<ChallengeService>,
    ) -> Self {
        OrderService {
            configuration,
            order_repo,
            directory_service,
            challenge_service,
        }
    }

    #[instrument(skip(self, user))]
    pub async fn place_order_for_domain(
        &self,
        domain: String,
        user: &ServiceUser,
    ) -> Result<ServiceUserOrder, OrderServiceError> {
        let directory = self
            .directory_service
            .fetch()
            .await
            .map_err(|e| OrderServiceError::InternalError(e.to_string()))?;
        let (pkey, account_resource) = get_shared_resources(&user)?;
        let order_ops = OrderOperations::new(pkey, account_resource, true)
            .map_err(|e| OrderServiceError::InternalError(e.to_string()))?;
        let order_result = order_ops
            .place_order(&directory.new_nonce, &directory.new_order, domain.as_str())
            .await
            .map_err(|e| OrderServiceError::InternalError(e.to_string()))?;
        info!("Order placement successfully completed successfully");
        let persisted_order = self
            .order_repo
            .create_order(user.id, domain.clone())
            .await?;
        let persisted_order = self
            .order_repo
            .update_order_urls(
                persisted_order.id(),
                order_result.order_url,
                order_result.authorization_url,
                order_result.finalize_url,
            )
            .await?;
        info!(
            "Order record saved (id={}) for domain '{}'",
            persisted_order.id(),
            domain
        );
        Ok(self.build_service_user_order_from_order(&persisted_order))
    }

    #[instrument(skip(self, order_id, user))]
    pub async fn get_order(
        &self,
        order_id: i64,
        user: &ServiceUser,
    ) -> Result<ServiceUserOrder, OrderServiceError> {
        let order = self
            .order_repo
            .get_by_id(order_id)
            .await?
            .ok_or_else(|| OrderServiceError::InternalError("Order not found".into()))?;
        if order.user_id() != user.id {
            return Err(OrderServiceError::InternalError(
                "Illegal state :: order was persisted for another user".to_string(),
            ));
        }

        Ok(self.build_service_user_order_from_order(&order))
    }

    #[instrument(skip(self, order_id))]
    pub async fn __get_order_any(
        &self,
        order_id: i64,
    ) -> Result<ServiceUserOrder, OrderServiceError> {
        let order = self
            .order_repo
            .get_by_id(order_id)
            .await?
            .ok_or_else(|| OrderServiceError::InternalError("Order not found".into()))?;
        Ok(self.build_service_user_order_from_order(&order))
    }

    pub async fn get_order_challenge(
        &self,
        order_id: i64,
    ) -> Result<Option<ServiceOrderChallenge>, OrderServiceError> {
        Ok(self
            .challenge_service
            .__get_challenge_for_order(order_id)
            .await?)
    }

    fn build_service_user_order_from_order(&self, order: &Order) -> ServiceUserOrder {
        ServiceUserOrder::builder()
            .id(order.id())
            .domain(order.domain())
            .order_url_option(order.order_url())
            .authorization_url_option(order.authorization_url())
            .finalize_url_option(order.finalize_url())
            .certificate_url_option(order.certificate_url())
            .status(order.status())
            .created_at(order.created_at())
            .build()
    }

    #[instrument(skip(self, user, order))]
    pub async fn prepare(
        &self,
        user: &ServiceUser,
        order: ServiceUserOrder,
    ) -> Result<(), OrderServiceError> {
        let order = self
            .challenge_service
            .initiate_challenge(user, order)
            .await?;
        let challenge_info = order
            .challenge_info
            .ok_or_else(|| OrderServiceError::InternalError("ChallengeInfo not found".into()))?;

        let run_config = SetConfig {
            acme_token: challenge_info.txt_record_value.clone(),
            domain: order.domain.clone(),
            record_name: challenge_info.dns_record_name.clone(),
        };

        let _ = run_plugin(
            self.configuration.get_acme_challenge_plugin(),
            DiscoveryType::as_ds_type(self.configuration.get_discovery_type().as_str())
                .ok_or_else(|| {
                    OrderServiceError::InternalError("Failed to get discovery type".into())
                })?,
            RunMode::Set,
            &run_config,
        )
        .map_err(|e| OrderServiceError::InternalError(e.to_string()))?;

        Ok(())
    }
    #[instrument(skip(self, order, mark))]
    pub async fn mark(
        &self,
        order: ServiceUserOrder,
        mark: ChallengeMark,
    ) -> Result<(), OrderServiceError> {
        match mark {
            ChallengeMark::Verified => self
                .mark_verified(order)
                .await
                .map_err(|e| OrderServiceError::InternalError(e.to_string()))?,
            ChallengeMark::Failed => self
                .mark_failed(order)
                .await
                .map_err(|e| OrderServiceError::InternalError(e.to_string()))?,
        }
        Ok(())
    }
    async fn mark_failed(&self, order: ServiceUserOrder) -> Result<(), OrderServiceError> {
        self.order_repo
            .set_status(order.id.clone(), "invalid".to_string())
            .await
            .map_err(|e| OrderServiceError::InternalError(e.to_string()))?;
        self.challenge_service
            .mark(order.id, ChallengeMark::Failed)
            .await
            .map_err(|e| OrderServiceError::InternalError(e.to_string()))?;
        Ok(())
    }
    async fn mark_verified(&self, order: ServiceUserOrder) -> Result<(), OrderServiceError> {
        self.order_repo
            .set_status(order.id.clone(), "valid".to_string())
            .await
            .map_err(|e| OrderServiceError::InternalError(e.to_string()))?;
        self.challenge_service
            .mark(order.id, ChallengeMark::Verified)
            .await
            .map_err(|e| OrderServiceError::InternalError(e.to_string()))?;
        Ok(())
    }
    #[instrument(skip(self, order, user, certificate_private_key))]
    pub async fn finalize(
        &self,
        order: &ServiceUserOrder,
        user: &ServiceUser,
        certificate_private_key: &PrivateKey,
    ) -> Result<(), OrderServiceError> {
        let finalize_url = order
            .finalize_url
            .clone()
            .ok_or_else(|| OrderServiceError::InternalError("FinalizeUrl not found".into()))?;
        let challenge = self
            .challenge_service
            .__get_challenge_for_order(order.id)
            .await
            .map_err(|e| OrderServiceError::InternalError(e.to_string()))?
            .ok_or_else(|| OrderServiceError::InternalError("ChallengeInfo not found".into()))?;
        let (pkey, account_resource) = get_shared_resources(user)?;
        let result = self
            .challenge_service
            .finalize(
                (pkey, account_resource),
                &challenge,
                &finalize_url,
                &order.domain,
                certificate_private_key,
            )
            .await
            .map_err(|e| OrderServiceError::InternalError(e.to_string()))?;
        if let Some(cert_url) = result.certificate_url {
            info!("Certificate URL available: {}", cert_url);
            self.order_repo
                .set_certificate_url(order.id, cert_url)
                .await
                .map_err(|e| OrderServiceError::InternalError(e.to_string()))?;
        } else {
            info!("Order is being processed — certificate URL not yet available.");
            self.order_repo
                .set_status(order.id, "processing".to_string())
                .await
                .map_err(|e| OrderServiceError::InternalError(e.to_string()))?;
        }
        Ok(())
    }

    pub async fn try_download_pem(
        &self,
        order: &ServiceUserOrder,
        user: &ServiceUser,
    ) -> Result<DownloadStatus, OrderServiceError> {
        let order_url = order
            .order_url
            .clone()
            .ok_or_else(|| OrderServiceError::InternalError("OrderUrl not found".into()))?;
        let directory = self
            .directory_service
            .fetch()
            .await
            .map_err(|e| OrderServiceError::InternalError(e.to_string()))?;
        let (pkey, account_resource) = get_shared_resources(user)?;
        let ops = CertificateOperations::new(pkey, account_resource, true)
            .map_err(|e| OrderServiceError::InternalError(e.to_string()))?;
        let certificate_url = ops
            .poll_order_certificate_url(&directory.new_nonce, &order_url)
            .await
            .map_err(|e| OrderServiceError::InternalError(e.to_string()))?;
        if let Some(cert_url) = certificate_url {
            let bytes = ops
                .download_certificate(&directory.new_nonce, &cert_url)
                .await
                .map_err(|e| OrderServiceError::InternalError(e.to_string()))?;
            Ok(DownloadStatus::Result(bytes))
        } else {
            Ok(DownloadStatus::ContinuePolling)
        }
    }
}
