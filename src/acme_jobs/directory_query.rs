use crate::acme_jobs::initialize_keys_for_user::InitializeLocalUserJob;
use crate::job_execution::job_base::{Job, SchedulerHandle};
use acme_client::comms::directory::AcmeDirectoryApi;
use async_trait::async_trait;
use common_utils::CompareFields;
use persistence::data_model::AcmeDirectory;
use persistence::database::DatabaseConnection;
use reqwest::Url;
use serde::{Deserialize, Serialize};
use serde_json::{from_value, Value};
use std::error::Error;
use tracing::{info, instrument};

#[derive(Serialize, Deserialize)]
pub struct DirectoryUpdateJob {
    pub base_url: String,
    pub user_id: String,
}
impl DirectoryUpdateJob {
    pub fn new(base_url: String, user_id: String) -> Result<Self, Box<dyn Error>> {
        let url = Self::validate_url(Some(base_url.clone()))?;
        Ok(DirectoryUpdateJob {
            user_id,
            base_url: url.to_string(),
        })
    }
    pub fn validate_url(base_url: Option<String>) -> Result<Url, Box<dyn Error>> {
        if let Some(base_url) = base_url {
            let x = base_url.to_owned() + "/dir";
            let url = Url::parse(x.as_str())?;
            return Ok(url);
        }
        Err("Acme CA base url could not be parsed!".into())
    }
    async fn call_directory(&self) -> anyhow::Result<Value> {
        let client = reqwest::Client::builder()
            .danger_accept_invalid_certs(true)
            .build()?;
        info!("Calling the requested ACME directory");
        let response = client.get(self.base_url.as_str()).send().await?;
        info!("Response returned: {}", response.status());
        let bytes = response.bytes().await?;
        let slice = bytes.iter().as_slice();
        let value = serde_json::from_slice::<Value>(slice)?;
        Ok(value)
    }
    fn get_existing(
        &self,
        user_id: i64,
        connection: &DatabaseConnection,
    ) -> anyhow::Result<Option<AcmeDirectory>> {
        let sql = r#"
            SELECT * FROM acme_users_directory WHERE user_id = $1;
            "#;
        let mut statement = connection.prepare(sql).unwrap();
        statement.bind((1, user_id))?;
        Ok(AcmeDirectory::scan_statement(statement).unwrap())
    }
    fn refresh_if_diff(
        &self,
        acme_directory: AcmeDirectoryApi,
    ) -> anyhow::Result<Option<AcmeDirectory>> {
        let connection = DatabaseConnection::get_connection().unwrap();
        let user = InitializeLocalUserJob::get_user(self.user_id.as_str(), &connection).unwrap();
        if user.is_none() {
            return Err(anyhow::anyhow!(
                "Could not complete directory job since queried user could not be found!"
            ))?;
        }
        let user = user.unwrap();
        let existing_dir = self.get_existing(user.id, &connection)?;
        let sql;
        if existing_dir.is_none() {
            sql = r#"
            INSERT INTO acme_users_directory(
                user_id,
                new_nonce,
                new_account,
                new_order,
                new_authz,
                revoke_cert,
                key_change
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7) RETURNING *;
            "#;
        } else {
            let existing = existing_dir.unwrap();
            info!("Existing acme directory found for user_id: {} - directory id: {}", user.id, existing.directory_id);
            if existing.is_equal_to(&acme_directory) {
                info!(
                    "Existing acme directory was found to be up to date - skipping refresh..."
                );
                return Ok(Some(existing));
            }
            info!("Acme directory found not to be equal to request - refreshing...");
            sql = self.get_update();
        }
        let mut statement = connection.prepare(sql).unwrap();
        statement.bind((1, user.id))?;
        statement.bind((2, acme_directory.new_nonce.as_str()))?;
        statement.bind((3, acme_directory.new_account.as_str()))?;
        statement.bind((4, acme_directory.new_order.as_str()))?;
        statement.bind((
            5,
            acme_directory.new_authz.unwrap_or("".to_string()).as_str(),
        ))?;
        statement.bind((6, acme_directory.revoke_cert.as_str()))?;
        statement.bind((7, acme_directory.key_change.as_str()))?;
        Ok(AcmeDirectory::scan_statement(statement).unwrap())
    }
    fn get_update(&self) -> &'static str {
        r#"
        UPDATE acme_users_directory SET
            new_nonce = ?2,
            new_account = ?3,
            new_order = ?4,
            new_authz = ?5,
            revoke_cert = ?6,
            key_change = ?7
        WHERE user_id = ?1 RETURNING *;
        "#
    }
}

#[async_trait]
impl Job for DirectoryUpdateJob {
    fn job_type(&self) -> &'static str {
        "directory-update-job"
    }
    fn payload(&self) -> Value {
        serde_json::to_value(self).unwrap()
    }
    #[instrument(level = "trace", name = "directory_update_job", fields(job_name = %self.job_type()), skip_all)]
    async fn execute(&self, _scheduler: SchedulerHandle) -> anyhow::Result<()> {
        let value = self.call_directory().await?;
        let dir: AcmeDirectoryApi = from_value(value.clone())?;
        let t = self.refresh_if_diff(dir)?;
        if let Some(dir) = t {
            info!("Directory refresh returned directory with id: {}", dir.directory_id);
        }
        Ok(())
    }
}
