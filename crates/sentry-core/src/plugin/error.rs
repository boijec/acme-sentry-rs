use sentry_plugin_engine::error::PluginEngineError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum PluginServiceError {
    #[error("Error computing plugin path :: {0}")]
    PluginPathError(String),
    #[error("Error during engine execution of plugin :: {0}")]
    EngineError(PluginEngineError),
    #[error("Precondition for plugin run failed :: {0}")]
    ConfigPreConditionError(String),
}
