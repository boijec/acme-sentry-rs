use crate::config::ApplicationConfig;
use crate::directory::model::ServiceUserDirectory;
use crate::order::error::OrderServiceError;
use crate::order::model::ServiceUserOrder;
use crate::user::model::ServiceUser;
use sentry_db::repo::order_repo::OrderRepo;
use sentry_rfc8555::order::OrderOperations;
use std::sync::Arc;
use tracing::{info, instrument};

pub struct OrderService {
    configuration: Arc<ApplicationConfig>,
    order_repo: Arc<dyn OrderRepo + Send + Sync>,
}

impl OrderService {
    pub fn new(
        configuration: Arc<ApplicationConfig>,
        order_repo: Arc<dyn OrderRepo + Send + Sync>,
    ) -> Self {
        OrderService {
            configuration,
            order_repo,
        }
    }

    #[instrument(skip(self, user, directory))]
    pub async fn place_order_for_domain(
        &self,
        domain: String,
        user: &ServiceUser,
        directory: &ServiceUserDirectory,
    ) -> Result<ServiceUserOrder, OrderServiceError> {
        let pkey = user.private_key.clone().ok_or_else(|| {
            OrderServiceError::InternalError("No private key instance found bound to user".into())
        })?;
        let account_resource = user.user_account_resource.clone().ok_or_else(|| {
            OrderServiceError::InternalError("No account resource found bound to user".into())
        })?;
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
            persisted_order.id(), domain
        );
        Ok(
            ServiceUserOrder::builder()
                .id(persisted_order.id())
                .domain(persisted_order.domain())
                .order_url_option(persisted_order.order_url())
                .authorization_url_option(persisted_order.authorization_url())
                .finalize_url_option(persisted_order.finalize_url())
                .certificate_url_option(persisted_order.certificate_url())
                .status(persisted_order.status())
                .created_at(persisted_order.created_at())
                .build()
        )
    }
}
