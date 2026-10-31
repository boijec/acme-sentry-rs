use crate::dependencies::JobDependencies;
use crate::error::JobEngineError;
use crate::job::Job;
use crate::jobs::dns_dig::DnsDigJob;
use crate::scheduling::SchedulerHandle;
use async_trait::async_trait;
use sentry_core::config::ConfigurationMap;
use sentry_plugin_engine::loader::{SetConfig, run_plugin};
use sentry_plugin_engine::run_mode::{DiscoveryType, RunMode};
use std::sync::Arc;
use tracing::warn;

pub struct AcceptDnsChallengeJob {
    config: Option<Arc<dyn ConfigurationMap + Send + Sync>>,
    // order_id: i64,
    // user_id: String,
}

#[derive(Default)]
pub struct AcceptDnsChallengeJobBuilder {
    order_id: i64,
    user_id: String,
}

impl AcceptDnsChallengeJob {
    pub fn builder() -> AcceptDnsChallengeJobBuilder {
        AcceptDnsChallengeJobBuilder::default()
    }
}

impl AcceptDnsChallengeJobBuilder {
    pub fn order_id(mut self, order_id: i64) -> Self {
        self.order_id = order_id;
        self
    }
    pub fn user_id(mut self, user_id: String) -> Self {
        self.user_id = user_id;
        self
    }
    pub fn build(self) -> AcceptDnsChallengeJob {
        AcceptDnsChallengeJob {
            user_id: self.user_id,
            order_id: self.order_id,
        }
    }
}

#[async_trait]
impl<'a> Job<'a> for AcceptDnsChallengeJob {
    fn job_type(&self) -> &'a str {
        "accept-dns-challenge-job"
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

        let order = AcmeOrder::get_by_id(self.order_id, &conn)
            .map_err(|e| anyhow::anyhow!("Failed to load order {}: {}", self.order_id, e))?
            .ok_or_else(|| anyhow::anyhow!("Order {} not found", self.order_id))?;

        let authorization_url = order
            .authorization_url
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Order {} has no authorization_url set", self.order_id))?
            .clone();

        let account_link = AcmeAccountLink::get_for_user(user.id, &conn)
            .map_err(|e| anyhow::anyhow!("Failed to load account link: {}", e))?
            .ok_or_else(|| anyhow::anyhow!("No account link for user '{}'", self.user_id))?;

        let dir = AcmeDirectory::get_existing(user.id, &conn)?
            .ok_or_else(|| anyhow::anyhow!("No ACME directory for user '{}'", self.user_id))?;

        let private_key = load_private_key(&user.user_id, &user.key_path, &user.key_type)?;

        let acme = AcmeAccountClient::new(private_key, account_link.account_url, true)
            .map_err(|e| anyhow::anyhow!("Failed to create ACME client: {}", e))?;

        let challenge_info = acme
            .fetch_dns01_challenge(&dir.new_nonce, &authorization_url)
            .await?;

        let challenge = AcmeDnsChallenge::create(
            self.order_id,
            &challenge_info.token,
            &challenge_info.key_authorization,
            &challenge_info.dns_record_name,
            &conn,
        )
        .map_err(|e| anyhow::anyhow!("Failed to persist DNS challenge: {}", e))?;
        let config = APPLICATION_CONFIG.get().unwrap();
        let run_config = SetConfig {
            acme_token: challenge_info.txt_record_value.clone(),
            domain: order.domain.clone(),
            record_name: challenge_info.dns_record_name.clone(),
        };

        let _ = run_plugin(
            config.acme_challenge_plugin.clone(),
            DiscoveryType::as_ds_type(config.discovery_type.as_str()).unwrap(),
            RunMode::Set,
            &run_config
        )
            .map_err(|e| anyhow::anyhow!("Failed to load plugin: {}", e.to_string()))?;
        warn!(
            "ACTION REQUIRED: Set DNS TXT record '{}' to value '{}' for order {}.",
            challenge_info.dns_record_name, challenge_info.txt_record_value, self.order_id
        );

        handle
            .submit(DnsDigJob::new(self.order_id, self.user_id.clone(), challenge.id))
            .await
            .map_err(|_| anyhow::anyhow!("Failed to submit DnsDigJob"))?;

        Ok(())
    }
}

