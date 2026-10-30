
#[derive(Debug)]
pub enum PluginEngineError {
    DiscoveryTypeError(String),
    PluginLoadError(String),
}

impl PluginEngineError {
    pub fn to_string(&self) -> String {
        match self {
            PluginEngineError::DiscoveryTypeError(s) => s.clone(),
            PluginEngineError::PluginLoadError(s) => s.clone(),
        }
    }
}