use crate::job_execution::job_base::{Job, JobDependencies, SchedulerHandle};
use async_trait::async_trait;
use persistence::database::DatabaseConnection;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Serialize, Deserialize)]
pub struct DbInitializationJob {}
impl DbInitializationJob {
    pub fn new() -> Self {
        DbInitializationJob {}
    }
}
#[async_trait]
impl<'a> Job<'a> for DbInitializationJob {
    fn job_type(&self) -> &'a str {
        "db-initialization-job"
    }
    fn payload(&self) -> Value {
        serde_json::to_value(self).unwrap()
    }

    fn inject_dependencies(&mut self, _dependencies: &JobDependencies) {
        return;
    }

    async fn execute(&self, _: SchedulerHandle<'a>) -> anyhow::Result<()> {
        let db = DatabaseConnection::get_connection().unwrap();
        db.internal_structure_check().unwrap();
        Ok(())
    }
}