use crate::job_execution::job_base::{Job, SchedulerHandle};
use crate::plugin_engine::loader::run_plugin;
use crate::plugin_engine::run_mode::{DiscoveryType, RunMode};
use async_trait::async_trait;
use common_utils::APPLICATION_CONFIG;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tracing::instrument;

#[derive(Serialize, Deserialize)]
pub struct CallSetChallengePluginJob {
    pub order_id: i64,
    pub user_id: String,
    pub discovery_type: DiscoveryType,
    pub plugin_path: String,
    pub challenge_token: String,
}

impl CallSetChallengePluginJob {
    pub fn new(order_id: i64, user_id: String, challenge_token: String) -> Self {
        let cnf = APPLICATION_CONFIG.get().unwrap();
        CallSetChallengePluginJob {
            order_id,
            user_id,
            discovery_type: DiscoveryType::as_ds_type(cnf.discovery_type.as_str().clone()).unwrap(),
            plugin_path: cnf.acme_challenge_plugin.to_string(),
            challenge_token,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct CallUnsetChallengePluginJob {
    pub order_id: i64,
    pub user_id: String,
    pub discovery_type: DiscoveryType,
    pub plugin_path: String,
}

impl CallUnsetChallengePluginJob {
    pub fn new(order_id: i64, user_id: String) -> Self {
        let cnf = APPLICATION_CONFIG.get().unwrap();
        CallUnsetChallengePluginJob {
            order_id,
            user_id,
            discovery_type: DiscoveryType::as_ds_type(cnf.discovery_type.as_str().clone()).unwrap(),
            plugin_path: cnf.acme_challenge_plugin.to_string(),
        }
    }
}

#[async_trait]
impl Job for CallSetChallengePluginJob {
    fn job_type(&self) -> &'static str {
        "set-acme-challenge-token"
    }

    fn payload(&self) -> Value {
        serde_json::to_value(self).unwrap()
    }

    #[instrument(level = "trace", name = "set_acme_challenge_token", fields(job_name = %self.job_type()), skip_all)]
    async fn execute(&self, handle: SchedulerHandle) -> anyhow::Result<()> {
        let res = run_plugin(self.plugin_path.clone(), self.discovery_type.clone(), RunMode::Set)
            .map_err(|e| anyhow::anyhow!("Failed to load plugin: {}", e.to_string()))?;
        Ok(res)
    }
}

#[async_trait]
impl Job for CallUnsetChallengePluginJob {
    fn job_type(&self) -> &'static str {
        "unset-acme-challenge-token"
    }

    fn payload(&self) -> Value {
        serde_json::to_value(self).unwrap()
    }

    #[instrument(level = "trace", name = "unset-acme-challenge-token", fields(job_name = %self.job_type()), skip_all)]
    async fn execute(&self, handle: SchedulerHandle) -> anyhow::Result<()> {
        let res = run_plugin(self.plugin_path.clone(), self.discovery_type.clone(), RunMode::Unset)
            .map_err(|e| anyhow::anyhow!("Failed to load plugin: {}", e.to_string()))?;
        Ok(res)
    }
}