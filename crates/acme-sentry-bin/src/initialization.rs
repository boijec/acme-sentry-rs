use clap::ArgMatches;
use tracing::info;
use sentry_db::pool::DB_FILE_NAME;
use sentry_io::fs::{BasicUnixFileSystem, FileSystemOperations, FileSystemOrigin};
use crate::clap_config::InitializationMode;
use crate::UNIVERSAL_BASE_DIR;

pub(crate) fn init_application(matches: ArgMatches) -> Result<(), String> {
    let (_, mat) = matches.subcommand().ok_or_else(|| "error parsing subcommand")?;
    let mode: &InitializationMode = mat.get_one("init-mode")
        .ok_or_else(|| "Initialization mode not specified".to_string())?;
    let fs: BasicUnixFileSystem = match mode {
        InitializationMode::Headless => {
            BasicUnixFileSystem::initialize(UNIVERSAL_BASE_DIR, FileSystemOrigin::CurrentDirectory)
                .map_err(|e| format!("Failed to initialize `./.` (current) directory setup: {}", e))?
        },
        InitializationMode::Server => {
            BasicUnixFileSystem::initialize(UNIVERSAL_BASE_DIR, FileSystemOrigin::OptionsDirectory)
                .map_err(|e| format!("Failed to initialize `/opt/.` (options) directory setup: {}", e))?
        },
        InitializationMode::User => {
            BasicUnixFileSystem::initialize(UNIVERSAL_BASE_DIR, FileSystemOrigin::UserHomeDirectory)
                .map_err(|e| format!("Failed to initialize `~/.` (user home) directory setup: {}", e))?
        },
    };
    fs.touch_file("", DB_FILE_NAME)
        .map_err(|e| format!("Failed to touch `{}`: {}", DB_FILE_NAME, e))?;
    info!(
        "Initialization of basic directories and database-file has been completed, you can now run in watch or one-shot mode"
    );
    Ok(())
}