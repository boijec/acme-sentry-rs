use crate::acme_jobs::accept_dns_challenge::AcceptDnsChallengeJob;
use crate::job_execution::job_base::{Job, SchedulerHandle};
use acme_client::comms::account::AcmeAccountClient;
use acme_client::keys::load_private_key;
use async_trait::async_trait;
use persistence::account_link::AcmeAccountLink;
use persistence::database::DatabaseConnection;
use persistence::directory::AcmeDirectory;
use persistence::order::AcmeOrder;
use persistence::user::AcmeUser;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tracing::info;

#[derive(Serialize, Deserialize)]
pub struct PlaceOrderJob {
    pub user_id: String,
    pub domain: String,
}

impl PlaceOrderJob {
    pub fn new(user_id: String, domain: String) -> Self {
        PlaceOrderJob { user_id, domain }
    }
}

#[async_trait]
impl Job for PlaceOrderJob {
    fn job_type(&self) -> &'static str {
        "place-order-job"
    }

    fn payload(&self) -> Value {
        serde_json::to_value(self).unwrap()
    }

    async fn execute(&self, handle: SchedulerHandle) -> anyhow::Result<()> {
        let conn = DatabaseConnection::get_connection()
            .map_err(|e| anyhow::anyhow!("DB connection failed: {}", e))?;

        let user = AcmeUser::get_user(self.user_id.as_str(), &conn)
            .map_err(|e| anyhow::anyhow!("Failed to look up user: {}", e))?
            .ok_or_else(|| anyhow::anyhow!("User '{}' not found", self.user_id))?;

        let account_link = AcmeAccountLink::get_for_user(user.id, &conn)
            .map_err(|e| anyhow::anyhow!("Failed to load account link: {}", e))?
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "No account link found for user '{}' — has login_user_job run?",
                    self.user_id
                )
            })?;

        let dir = AcmeDirectory::get_existing(user.id, &conn)?
            .ok_or_else(|| anyhow::anyhow!("No ACME directory for user '{}'", self.user_id))?;

        let private_key = load_private_key(&user.user_id, &user.key_path, &user.key_type)?;

        let acme = AcmeAccountClient::new(private_key, account_link.account_url, true)
            .map_err(|e| anyhow::anyhow!("Failed to create ACME client: {}", e))?;

        let result = acme
            .place_order(&dir.new_nonce, &dir.new_order, &self.domain)
            .await?;

        let order = AcmeOrder::create(user.id, self.domain.as_str(), &conn)
            .map_err(|e| anyhow::anyhow!("Failed to create order record: {}", e))?;

        AcmeOrder::update_order_urls(
            order.id,
            &result.order_url,
            &result.authorization_url,
            &result.finalize_url,
            &conn,
        )
        .map_err(|e| anyhow::anyhow!("Failed to update order URLs: {}", e))?;

        info!(
            "Order record saved (id={}) for domain '{}', submitting AcceptDnsChallengeJob.",
            order.id, self.domain
        );
        handle
            .submit(AcceptDnsChallengeJob::new(order.id, self.user_id.clone()))
            .await
            .map_err(|_| anyhow::anyhow!("Failed to submit AcceptDnsChallengeJob"))?;

        Ok(())
    }
}

