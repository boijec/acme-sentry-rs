
#[derive(Debug)]
pub enum PluginEngineError {
    DiscoveryTypeError(String),
    PluginLoadError(String),
}