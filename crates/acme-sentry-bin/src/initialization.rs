use crate::{UNIVERSAL_BASE_DIR, toml_config};
use clap::ArgMatches;
use sentry_core::initialization::{InitializationMode, create_basic_directories};

pub(crate) fn init_application(matches: &mut ArgMatches) -> Result<(), String> {
    let (mode, base_url, key_type, email) = extract_argument_matches(matches)?;
    let conf_content = toml_config::build_toml_config(base_url, key_type, email)
        .map_err(|e| format!("failed to build toml config: {}", e))?;
    let _ = create_basic_directories(UNIVERSAL_BASE_DIR, mode, conf_content)?;
    println!(
        "Initialization of basic directories and database-file has been completed, you can now run in watch or one-shot mode\n"
    );
    Ok(())
}

fn extract_argument_matches(
    matches: &mut ArgMatches,
) -> Result<(InitializationMode, String, String, String), String> {
    let (_, mut mat) = matches
        .remove_subcommand()
        .ok_or_else(|| "error parsing subcommand")?;
    let mode_str: String = mat
        .remove_one("init-mode")
        .ok_or_else(|| "Initialization mode not specified".to_string())?;
    let mode: InitializationMode = InitializationMode::from(mode_str);
    let base_url: String = mat
        .remove_one("base-url")
        .ok_or_else(|| "base-url not specified".to_string())?;
    let key_type: String = mat
        .remove_one("key-type")
        .ok_or_else(|| "key-type not specified".to_string())?;
    let email: String = mat
        .remove_one("email")
        .ok_or_else(|| "email not specified".to_string())?;
    Ok((mode, base_url, key_type, email))
}
