use crate::dependencies::JobDependencies;
use crate::error::JobEngineError;
use crate::job::Job;
use crate::scheduling::SchedulerHandle;
use async_trait::async_trait;
use tracing::info;

pub struct PlaceOrderJob {
    pub user_id: String,
    pub domain: String,
}

impl PlaceOrderJob {
    pub fn new(user_id: String, domain: String) -> Self {
        PlaceOrderJob { user_id, domain }
    }
    pub fn builder() -> PlaceOrderJobBuilder {
        PlaceOrderJobBuilder::default()
    }
}

#[derive(Default)]
pub struct PlaceOrderJobBuilder {
    user_id: String,
    domain: String,
}

impl PlaceOrderJobBuilder {
    pub fn user_id(mut self, user_id: &str) -> Self {
        self.user_id = user_id.to_string();
        self
    }
    pub fn domain(mut self, domain: &str) -> Self {
        self.domain = domain.to_string();
        self
    }
    pub fn build(self) -> PlaceOrderJob {
        PlaceOrderJob {
            user_id: self.user_id,
            domain: self.domain,
        }
    }
}

#[async_trait]
impl<'a> Job<'a> for PlaceOrderJob {
    fn job_type(&self) -> &'a str {
        "place-order-job"
    }

    fn inject_dependencies(&mut self, _dependencies: &JobDependencies) {
        return;
    }

    async fn execute(&self, handle: SchedulerHandle<'a>) -> Result<(), JobEngineError> {
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

