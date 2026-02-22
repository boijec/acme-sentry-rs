pub mod fs;

#[cfg(test)]
mod test;

use std::slice::Iter;
use std::sync::OnceLock;
use tracing::Level;
use uuid::Uuid;

#[derive(Debug)]
pub struct ApplicationConfig {
    pub application_mode: bool,
    pub base_url: String,
    pub base_dir: String,
    pub output_dir: String,
    pub user_id: String,
    pub user_email: String,
    pub key_type: String,
    pub logging_level: Option<Level>,
}

pub static APPLICATION_CONFIG: OnceLock<ApplicationConfig> = OnceLock::new();

pub trait EnumIterator<T> {
    fn iterator() -> Iter<'static, T>;
}
pub struct InternalIdTooling;
impl InternalIdTooling {
    pub fn new_id() -> Uuid {
        Uuid::new_v4()
    }
    pub fn new_compact_id() -> String {
        InternalIdTooling::new_id().simple().encode_lower(&mut Uuid::encode_buffer()).to_string()
    }
}

