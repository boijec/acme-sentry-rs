use std::fmt::Display;

#[derive(Debug)]
pub enum PluginEngineError {
    DiscoveryTypeError(String),
    PluginLoadError(String),
    PluginRuntimeError(String),
}

impl Display for PluginEngineError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

impl PluginEngineError {
    pub fn to_string(&self) -> String {
        match self {
            PluginEngineError::DiscoveryTypeError(s) => s.clone(),
            PluginEngineError::PluginLoadError(s) => s.clone(),
            PluginEngineError::PluginRuntimeError(s) => s.clone(),
        }
    }
}
