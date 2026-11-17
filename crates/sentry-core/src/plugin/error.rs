use sentry_io::fs::FileSystemError;
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

#[derive(Error, Debug)]
pub enum PluginToolingError {
    #[error("no default plugin found")]
    NoDefaultFound,
    #[error("could not resolve path to filename")]
    PathResolveError,
    #[error("file system error in tooling :: {0}")]
    FileSystemError(#[from] FileSystemError),
    #[error("error while calling engine in plugin tool :: {0}")]
    EngineError(PluginEngineError),
}
