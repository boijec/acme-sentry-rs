use sentry_db::pool::DB_FILE_NAME;
use sentry_io::fs::{BasicUnixFileSystem, FileSystemOperations, FileSystemOrigin};
use sentry_plugin_engine::engine::PLUGIN_DIRECTORY;
use serde::Serialize;

pub const CONFIG_FILE_NAME: &str = "config.toml";

#[derive(Clone, Default, Debug, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum InitializationMode {
    #[default]
    Headless,
    Server,
    User,
}

impl From<String> for InitializationMode {
    fn from(mode: String) -> Self {
        match mode.as_ref() {
            "headless" => InitializationMode::Headless,
            "server" => InitializationMode::Server,
            "user" => InitializationMode::User,
            _ => InitializationMode::default(),
        }
    }
}

pub fn create_basic_directories(
    base_dir_name: &str,
    mode: InitializationMode,
    configuration_content: String,
) -> Result<BasicUnixFileSystem, String> {
    let fs: BasicUnixFileSystem = match mode {
        InitializationMode::Headless => {
            BasicUnixFileSystem::initialize(base_dir_name, FileSystemOrigin::CurrentDirectory)
                .map_err(|e| {
                    format!(
                        "Failed to initialize `./.` (current) directory setup: {}",
                        e
                    )
                })?
        }
        InitializationMode::Server => {
            BasicUnixFileSystem::initialize(base_dir_name, FileSystemOrigin::OptionsDirectory)
                .map_err(|e| {
                    format!(
                        "Failed to initialize `/opt/.` (options) directory setup: {}",
                        e
                    )
                })?
        }
        InitializationMode::User => {
            BasicUnixFileSystem::initialize(base_dir_name, FileSystemOrigin::UserHomeDirectory)
                .map_err(|e| {
                    format!(
                        "Failed to initialize `~/.` (user home) directory setup: {}",
                        e
                    )
                })?
        }
    };
    fs.ensure_sub_dir(PLUGIN_DIRECTORY).map_err(|e| {
        format!(
            "Failed to initialize plugins directory (error) sub-dir setup: {}",
            e
        )
    })?;
    fs.touch_file("", DB_FILE_NAME)
        .map_err(|e| format!("Failed to touch `{}`: {}", DB_FILE_NAME, e))?;
    fs.write_to_file("", CONFIG_FILE_NAME, configuration_content.as_bytes())
        .map_err(|e| format!("Failed to write `{}`: {}", CONFIG_FILE_NAME, e))?;
    Ok(fs)
}
