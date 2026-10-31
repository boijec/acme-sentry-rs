use crate::error::LoadKeyError;
use sentry_crypto::crypto::SupportedKey;
use sentry_io::fs::FileSystemOperations;
use sentry_jose::keys::PrivateKey;
use std::path::Path;
use std::sync::Arc;

pub fn load_private_key(
    user_id: &String,
    key_path: &String,
    key_type: &String,
    fs_ops: Arc<dyn FileSystemOperations>
) -> Result<PrivateKey, LoadKeyError> {
    let absolute_key_path = format!("{}.pem", user_id);
    if !fs_ops.file_exists(Path::new(key_path).to_path_buf(), absolute_key_path.as_str()) {
        return Err(LoadKeyError::ErrorDeterminingPath(absolute_key_path))
    }
    let pem_bytes = fs_ops.read_from_file(key_path.as_str(), absolute_key_path.as_str())
        .map_err(|e| LoadKeyError::UnableToReadFromKeyFile(e, absolute_key_path.clone()))?;
    let supported_key = SupportedKey::from_str(key_type)
        .map_err(|e| LoadKeyError::UnsupportedKeyType(e, key_type.to_string()))?;
    let private_key = PrivateKey::load_private_bytes(&pem_bytes, supported_key)
        .map_err(|e| LoadKeyError::FailedToInstantiateKey(e, absolute_key_path.clone()))?;
    Ok(private_key)
}