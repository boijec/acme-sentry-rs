use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use persistence::database::DatabaseConnection;
use crate::job_execution::job_base::{Job, SchedulerHandle};

#[derive(Serialize, Deserialize)]
pub struct DbInitializationJob {}
impl DbInitializationJob {
    pub fn new() -> Self {
        DbInitializationJob {}
    }
}
#[async_trait]
impl Job for DbInitializationJob {
    fn job_type(&self) -> &'static str {
        "DbInitializationJob"
    }
    fn payload(&self) -> Value {
        serde_json::to_value(self).unwrap()
    }
    async fn execute(&self, _: SchedulerHandle) -> anyhow::Result<()> {
        let db = DatabaseConnection::get_connection().unwrap();
        db.internal_structure_check().unwrap();
        Ok(())
    }
}