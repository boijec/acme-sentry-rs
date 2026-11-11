use std::{env, fs};
use std::path::PathBuf;
use crate::clap_config::ConfigurationError;

const USER_BASE_DIR: &str = ".acmes";
const OPT_BASE_DIR: &str = "/opt/acmes";

pub(crate) fn check_surroundings() -> Result<String, ConfigurationError> {
    let user_home = env::home_dir()
        .ok_or_else(|| ConfigurationError::OperationError("Home dir not set / found".into()))?;
    let path_in_user_home = user_home.join(PathBuf::from(USER_BASE_DIR));
    let exists_in_user = fs::exists(&path_in_user_home)
        .map_err(|e| ConfigurationError::OperationError(e.to_string()))?;
    let exists_in_opt = fs::exists(PathBuf::from(OPT_BASE_DIR))
        .map_err(|e| ConfigurationError::OperationError(e.to_string()))?;

    if exists_in_user {
        Ok(path_in_user_home.into_string().unwrap())
    } else if exists_in_opt {
        Ok(OPT_BASE_DIR.into())
    } else {
        Err(ConfigurationError::OperationError("init might not have been ran yet..".into()))
    }
}