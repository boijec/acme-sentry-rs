use crate::challenge::error::ChallengeServiceError;
use crate::challenge::model::{ChallengeMark, ServiceOrderChallenge};
use crate::config::ApplicationConfig;
use crate::directory::model::ServiceUserDirectory;
use crate::directory::service::DirectoryService;
use crate::helpers::resources::get_shared_resources;
use crate::order::model::ServiceUserOrder;
use crate::user::model::ServiceUser;
use sentry_db::repo::challenge_repo::{ChallengeRepo, DnsChallenge};
use sentry_jose::keys::PrivateKey;
use sentry_rfc8555::challenge::{ChallengeOperations, FinalizeResult};
use std::sync::Arc;
use tracing::info;

pub struct ChallengeService {
    application_config: Arc<ApplicationConfig>,
    challenge_repo: Arc<dyn ChallengeRepo + Send + Sync>,
    directory_service: Arc<DirectoryService>,
}

impl ChallengeService {
    pub fn new(
        application_config: Arc<ApplicationConfig>,
        challenge_repo: Arc<dyn ChallengeRepo + Send + Sync>,
        directory_service: Arc<DirectoryService>,
    ) -> ChallengeService {
        ChallengeService {
            application_config,
            challenge_repo,
            directory_service,
        }
    }

    pub async fn initiate_challenge(
        &self,
        user: &ServiceUser,
        order: ServiceUserOrder,
    ) -> Result<ServiceUserOrder, ChallengeServiceError> {
        let directory = self
            .directory_service
            .fetch()
            .await
            .map_err(|e| ChallengeServiceError::InternalError(e.to_string()))?;
        let (pkey, account_resource) = get_shared_resources(&user)?;

        let challenge_ops = ChallengeOperations::new(pkey, account_resource, true)
            .map_err(|e| ChallengeServiceError::InternalError(e.to_string()))?;
        let auth_url = order.authorization_url.clone().ok_or_else(|| {
            ChallengeServiceError::InternalError("Order authorization url not found".into())
        })?;

        let challenge_info = challenge_ops
            .fetch_dns01_challenge(&directory.new_nonce, &auth_url)
            .await
            .map_err(|e| ChallengeServiceError::InternalError(e.to_string()))?;

        let challenge = self
            .challenge_repo
            .create_challenge(
                order.id.clone(),
                challenge_info.token,
                challenge_info.key_authorization,
                challenge_info.dns_record_name,
                challenge_info.txt_record_value,
                challenge_info.challenge_url,
            )
            .await
            .map_err(|e| ChallengeServiceError::InternalError(e.to_string()))?;

        Ok(self.build_service_user_order_from_challenge(order, challenge))
    }

    pub async fn __get_challenge_for_order(
        &self,
        order_id: i64,
    ) -> Result<Option<ServiceOrderChallenge>, ChallengeServiceError> {
        let service_order_challenge = self
            .challenge_repo
            .get_for_order(order_id)
            .await
            .map_err(|e| ChallengeServiceError::InternalError(e.to_string()))?;
        if let Some(challenge) = service_order_challenge {
            Ok(Some(
                self.build_service_order_challenge_from_challenge(challenge),
            ))
        } else {
            Ok(None)
        }
    }

    pub async fn mark(
        &self,
        order_id: i64,
        mark: ChallengeMark,
    ) -> Result<(), ChallengeServiceError> {
        let challenge = self
            .__get_challenge_for_order(order_id)
            .await
            .map_err(|e| ChallengeServiceError::InternalError(e.to_string()))?
            .ok_or_else(|| {
                ChallengeServiceError::InternalError("Order challenge not found".into())
            })?;
        match mark {
            ChallengeMark::Verified => self
                .mark_verified(challenge)
                .await
                .map_err(|e| ChallengeServiceError::InternalError(e.to_string()))?,
            ChallengeMark::Failed => self
                .mark_failed(challenge)
                .await
                .map_err(|e| ChallengeServiceError::InternalError(e.to_string()))?,
        }
        Ok(())
    }
    async fn mark_failed(
        &self,
        challenge: ServiceOrderChallenge,
    ) -> Result<(), ChallengeServiceError> {
        self.challenge_repo
            .set_challenge_status(challenge.id, "invalid".to_string())
            .await
            .map_err(|e| ChallengeServiceError::InternalError(e.to_string()))?;
        Ok(())
    }
    async fn mark_verified(
        &self,
        challenge: ServiceOrderChallenge,
    ) -> Result<(), ChallengeServiceError> {
        self.challenge_repo
            .set_challenge_status(challenge.id, "valid".to_string())
            .await
            .map_err(|e| ChallengeServiceError::InternalError(e.to_string()))?;
        Ok(())
    }

    pub async fn finalize(
        &self,
        (pkey, account_resource): (PrivateKey, String),
        challenge: &ServiceOrderChallenge,
        finalize_url: &str,
        domain: &str,
        certificate_private_key: &PrivateKey,
    ) -> Result<FinalizeResult, ChallengeServiceError> {
        let directory = self
            .directory_service
            .fetch()
            .await
            .map_err(|e| ChallengeServiceError::InternalError(e.to_string()))?;
        let challenge_ops = ChallengeOperations::new(pkey, account_resource, true)
            .map_err(|e| ChallengeServiceError::InternalError(e.to_string()))?;
        self.notify_ca(&challenge, &challenge_ops, &directory)
            .await
            .map_err(|e| ChallengeServiceError::InternalError(e.to_string()))?;
        info!("Sending finalize request for domain.");
        tokio::time::sleep(tokio::time::Duration::from_secs(8)).await;
        let t = challenge_ops.finalize(&directory.new_nonce, finalize_url, domain, certificate_private_key)
            .await
            .map_err(|e| ChallengeServiceError::InternalError(e.to_string()))?;
        Ok(t)
    }
    async fn notify_ca(
        &self,
        challenge: &ServiceOrderChallenge,
        challenge_ops: &ChallengeOperations,
        directory: &ServiceUserDirectory,
    ) -> Result<(), ChallengeServiceError> {
        challenge_ops
            .notify_challenge_ready(&directory.new_nonce, &challenge.challenge_ready_url)
            .await
            .map_err(|e| ChallengeServiceError::InternalError(e.to_string()))?;
        info!(
            "DNS challenge (id={}) persisted and CA notified.",
            challenge.id
        );
        Ok(())
    }

    fn build_service_user_order_from_challenge(
        &self,
        order: ServiceUserOrder,
        challenge: DnsChallenge,
    ) -> ServiceUserOrder {
        ServiceUserOrder::builder()
            .id(order.id)
            .domain(order.domain.as_str())
            .order_url_option(order.order_url)
            .authorization_url_option(order.authorization_url)
            .finalize_url_option(order.finalize_url)
            .certificate_url_option(order.certificate_url)
            .status(order.status.as_str())
            .created_at(order.created_at.as_str())
            .challenge_info(self.build_service_order_challenge_from_challenge(challenge))
            .build()
    }

    fn build_service_order_challenge_from_challenge(
        &self,
        challenge: DnsChallenge,
    ) -> ServiceOrderChallenge {
        ServiceOrderChallenge::builder()
            .id(challenge.id())
            .order_id(challenge.order_id())
            .token(challenge.token())
            .key_authorization(challenge.key_authorization())
            .dns_record_name(challenge.dns_record_name())
            .challenge_ready_url(challenge.challenge_ready_url())
            .txt_record_value(challenge.txt_record_value())
            .status(challenge.status())
            .build()
    }
}
