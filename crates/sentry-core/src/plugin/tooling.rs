use sentry_io::fs::FileSystemOperations;
use sentry_plugin_engine::engine::{PLUGIN_DIRECTORY, PluginEngine};
use std::path::PathBuf;
use std::sync::Arc;

pub struct PluginTool {
    fs: Arc<dyn FileSystemOperations + Send + Sync>,
}

impl PluginTool {
    pub fn new(fs: Arc<dyn FileSystemOperations + Send + Sync>) -> Self {
        Self { fs }
    }
    pub fn list_plugins(&self, default: Option<String>) -> Result<(), String> {
        let entries = self
            .fs
            .read_sub_directory(PLUGIN_DIRECTORY)
            .map_err(|e| e.to_string())?;
        println!("=== Installed Plugins ===");
        println!("(*) = Default\n");
        for entry in entries {
            let p = entry.path().clone();
            let filename = p.file_name().and_then(|f| f.to_str()).unwrap_or("");
            if p.extension().and_then(|ext| ext.to_str()) == Some("so") {
                let (name, compatibility) = PluginEngine::load_name_and_check_fingerprint(&p)
                    .map_err(|e| e.to_string())?;
                let default_ptr: String;
                if let Some(def) = default.clone() {
                    if def == filename {
                        default_ptr = "(*)".into();
                    } else {
                        default_ptr = "   ".into();
                    }
                } else {
                    default_ptr = "   ".into();
                }
                println!("\t{} Name: {}, Verification: {}\n", default_ptr, name, compatibility);
            }
        }
        Ok(())
    }

    pub fn match_default(&self, default: Option<String>) -> Result<PathBuf, String> {
        let entries = self
            .fs
            .read_sub_directory(PLUGIN_DIRECTORY)
            .map_err(|e| e.to_string())?;
        for entry in entries {
            let p = entry.path().clone();
            let filename = p.file_name().and_then(|f| f.to_str()).unwrap_or("");
            if p.extension().and_then(|ext| ext.to_str()) == Some("so") {
                if let Some(def) = default.clone() {
                    if def == filename {
                        return Ok(p);
                    } else {
                        continue;
                    }
                } else {
                    continue;
                }
            }
        }
        Err("No default found.".to_string())
    }

    pub fn verify_mvp(&self) -> Result<bool, String> {
        let entries = self
            .fs
            .read_sub_directory(PLUGIN_DIRECTORY)
            .map_err(|e| e.to_string())?;
        for entry in entries {
            let p = entry.path().clone();
            if p.extension().and_then(|ext| ext.to_str()) == Some("so") {
                return Ok(true);
            }
        }
        Ok(false)
    }

    pub fn install_plugin(&self, relative_path: &str) -> Result<String, String> {
        let path_buf = self
            .fs
            .resolve_path(relative_path)
            .map_err(|e| e.to_string())?;
        let filename = path_buf.file_name().and_then(|s| s.to_str());
        if let Some(filename) = filename {
            let _ = self
                .fs
                .copy(
                    path_buf.clone(),
                    format!("{}/{}", PLUGIN_DIRECTORY, filename).as_str(),
                )
                .map_err(|e| e.to_string())?;
            return Ok(filename.to_string());
        }
        Err("Could not resolve path to filename..".into())
    }
}
