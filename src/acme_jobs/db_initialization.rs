use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tracing::instrument;
use persistence::database::DatabaseConnection;
use crate::job_execution::job_base::{Job, SchedulerHandle};

#[derive(Serialize, Deserialize)]
pub struct DbInitializationJob {
    db_loc: String,
}
impl DbInitializationJob {
    pub fn new(db_loc: &str) -> Self {
        DbInitializationJob {
            db_loc: db_loc.to_owned(),
        }
    }
}
#[async_trait]
impl Job for DbInitializationJob {
    fn job_type(&self) -> &'static str {
        "db-initialization-job"
    }
    fn payload(&self) -> Value {
        serde_json::to_value(self).unwrap()
    }
    #[instrument(level = "trace", name = "db_initialization_job", fields(job_name = %self.job_type()), skip_all)]
    async fn execute(&self, _: SchedulerHandle) -> anyhow::Result<()> {
        let db = DatabaseConnection::get_connection(self.db_loc.as_str()).unwrap();
        db.internal_structure_check().unwrap();
        Ok(())
    }
}