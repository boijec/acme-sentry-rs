use crate::config::{ApplicationConfig, ConfigurationMap};
use crate::order::model::ServiceUserOrder;
use crate::plugin::error::PluginServiceError;
use crate::plugin::tooling::PluginTool;
use sentry_plugin_engine::engine::{PluginEngine, SetConfig};
use std::sync::Arc;
use tracing::instrument;

pub struct PluginService {
    configuration: Arc<ApplicationConfig>,
    plugin_tool: Arc<PluginTool>,
    plugin_engine: Arc<PluginEngine>,
}

impl PluginService {
    pub fn new(
        configuration: Arc<ApplicationConfig>,
        plugin_tool: Arc<PluginTool>,
        plugin_engine: Arc<PluginEngine>,
    ) -> Self {
        PluginService {
            configuration,
            plugin_tool,
            plugin_engine,
        }
    }

    #[instrument(skip(self, order))]
    pub fn set_acme_challenge(&self, order: ServiceUserOrder) -> Result<(), PluginServiceError> {
        let challenge_info = order.challenge_info.ok_or_else(|| {
            PluginServiceError::ConfigPreConditionError("ChallengeInfo not found".into())
        })?;
        let run_config = SetConfig::new(
            challenge_info.txt_record_value.clone(),
            order.domain.clone(),
            challenge_info.dns_record_name.clone(),
        );
        let path = self
            .plugin_tool
            .match_default(self.configuration.get_default_plugin())
            .map_err(|_| {
                PluginServiceError::PluginPathError("unable to match default plugin".into())
            })?;
        self.plugin_engine
            .run_set_challenge(path, run_config)
            .map_err(|e| PluginServiceError::EngineError(e))?;
        Ok(())
    }
}
