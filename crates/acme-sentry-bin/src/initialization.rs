use crate::{UNIVERSAL_BASE_DIR, toml_config};
use anyhow::anyhow;
use clap::ArgMatches;
use sentry_core::initialization::{InitializationMode, create_basic_directories};

pub(crate) fn init_application(matches: &mut ArgMatches) -> anyhow::Result<()> {
    let (mode, base_url, key_type, email) = extract_argument_matches(matches)?;
    let conf_content = toml_config::build_toml_config(base_url, key_type, email)
        .map_err(|e| anyhow!("failed to build toml config: {}", e))?;
    let _ = create_basic_directories(UNIVERSAL_BASE_DIR, mode, conf_content)
        .map_err(|e| anyhow!("failed to create temporary directory: {}", e))?;
    println!(
        "Initialization of basic directories and database-file has been completed, you can now run in watch or one-shot mode\n"
    );
    Ok(())
}

fn extract_argument_matches(
    matches: &mut ArgMatches,
) -> anyhow::Result<(InitializationMode, String, String, String)> {
    let (_, mut mat) = matches
        .remove_subcommand()
        .ok_or_else(|| anyhow!("error parsing subcommand"))?;
    let mode_str: String = mat
        .remove_one("init-mode")
        .ok_or_else(|| anyhow!("Initialization mode not specified"))?;
    let mode: InitializationMode = InitializationMode::from(mode_str);
    let base_url: String = mat
        .remove_one("base-url")
        .ok_or_else(|| anyhow!("base-url not specified"))?;
    let key_type: String = mat
        .remove_one("key-type")
        .ok_or_else(|| anyhow!("key-type not specified"))?;
    let email: String = mat
        .remove_one("email")
        .ok_or_else(|| anyhow!("email not specified"))?;
    Ok((mode, base_url, key_type, email))
}
