use std::fs::read_dir;
use crate::plugin_engine::error::PluginEngineError;
use std::path::Path;
use crate::plugin_engine::run_mode::DiscoveryType;

pub fn check_plugins_directory(str_path: &str) -> Result<DiscoveryType, PluginEngineError> {
    let path = Path::new(str_path);
    if path.is_dir() {
        let Ok(entries) = read_dir(&str_path) else {
            return Err(PluginEngineError::DiscoveryTypeError(
                "Directory read not possible".to_string(),
            ));
        };
        for entry in entries.flatten() {
            if entry.path().extension().and_then(|ext| ext.to_str()) == Some("so") {
                return Ok(DiscoveryType::Directory);
            }
        }
        Err(PluginEngineError::DiscoveryTypeError(
            "Directory does not contain plugin files".to_string(),
        ))
    } else if path.is_file() && path.extension().and_then(|ext| ext.to_str()) == Some("so") {
        Ok(DiscoveryType::SoloPlugin)
    } else {
        Err(PluginEngineError::DiscoveryTypeError("Path is not a directory".to_string()))
    }
}