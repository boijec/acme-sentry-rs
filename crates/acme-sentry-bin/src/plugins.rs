use crate::toml_config;
use crate::toml_config::save_serialized_config;
use anyhow::anyhow;
use clap::ArgMatches;
use sentry_core::config::ConfigurationMap;
use sentry_core::plugin::tooling::PluginTool;
use sentry_io::fs::FileSystemOperations;
use std::sync::Arc;

enum PluginFeature {
    Install,
    List,
}

impl From<String> for PluginFeature {
    fn from(value: String) -> Self {
        match value.as_ref() {
            "ls" => PluginFeature::List,
            "install" => PluginFeature::Install,
            _ => PluginFeature::List,
        }
    }
}

pub(crate) fn check_plugin_ready(
    fs: Arc<dyn FileSystemOperations + Send + Sync>,
) -> anyhow::Result<bool> {
    let plugin_tool = PluginTool::new(fs);
    let ready = plugin_tool
        .verify_mvp()
        .map_err(|_| anyhow!("Failed to verify mvp"))?;
    Ok(ready)
}

#[allow(unused)]
pub(crate) fn get_default(
    fs: Arc<dyn FileSystemOperations + Send + Sync>,
) -> anyhow::Result<Option<String>> {
    let toml = toml_config::read_current_toml_config(fs)?;
    if let Some(_) = toml.acme_plugin {
        Ok(toml.acme_plugin)
    } else {
        Ok(None)
    }
}

pub(crate) fn list_or_install_plugin(
    matches: &mut ArgMatches,
    fs: Arc<dyn FileSystemOperations + Send + Sync>,
    config: Arc<dyn ConfigurationMap>,
) -> anyhow::Result<()> {
    let (feature, mut command_mat) = match_subcommand(matches)
        .map_err(|e| anyhow!("failed to extract plugin command options: {}", e))?;
    let plugin_tool = PluginTool::new(fs.clone());

    match feature {
        PluginFeature::List => {
            plugin_tool
                .list_plugins(config.get_default_plugin())
                .map_err(|e| anyhow!("failed to list plugin files: {}", e))?;
            Ok(())
        }
        PluginFeature::Install => {
            let path: Option<String> = command_mat.remove_one("path");
            if let Some(path) = path {
                let name = plugin_tool
                    .install_plugin(&path)
                    .map_err(|e| anyhow!("failed to install plugin files: {}", e))?;
                let toml = toml_config::read_current_toml_config(fs.clone())?;
                if let None = toml.acme_plugin {
                    let new = toml.with_default_plugin(name);
                    save_serialized_config(new, fs)
                        .map_err(|e| anyhow!("failed to save acme plugin files: {}", e))?;
                }
            }
            Ok(())
        }
    }
}

fn match_subcommand(matches: &mut ArgMatches) -> anyhow::Result<(PluginFeature, ArgMatches)> {
    let (_, mut plugin_mat) = matches
        .remove_subcommand()
        .ok_or_else(|| anyhow!("missing subcommand"))?;
    let (plugin_command, sub_command_mat) = plugin_mat
        .remove_subcommand()
        .ok_or_else(|| anyhow!("missing subcommand"))?;
    Ok((plugin_command.into(), sub_command_mat))
}
