use std::sync::Arc;
use sentry_io::fs::FileSystemOperations;

pub struct JobDependencies {
    file_system: Arc<dyn FileSystemOperations + Send + Sync>,
}
impl JobDependencies {
    pub fn new(
        file_system: Arc<dyn FileSystemOperations + Send + Sync>
    ) -> Self {
        Self { file_system }
    }
    pub fn file_system(&self) -> Arc<dyn FileSystemOperations + Send + Sync> {
        Arc::clone(&self.file_system)
    }
}
