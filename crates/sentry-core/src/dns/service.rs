use crate::challenge::model::ServiceOrderChallenge;
use crate::config::ApplicationConfig;
use crate::directory::service::DirectoryService;
use crate::dns::error::DnsServiceError;
use sentry_crypto::crypto::sha256_b64;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::info;

pub struct DnsService {
    configuration: Arc<ApplicationConfig>,
}

impl DnsService {
    pub fn new(configuration: Arc<ApplicationConfig>) -> Self {
        DnsService {
            configuration,
        }
    }

    pub async fn compute_and_dig(&self, challenge: &ServiceOrderChallenge) -> Result<bool, DnsServiceError> {
        let expected = sha256_b64(challenge.key_authorization.as_bytes());
        info!(
            "Attempting dns dig for '{}' expecting TXT='{}'",
            challenge.dns_record_name,
            expected
        );
        let records = Self::dig_txt(&challenge.dns_record_name).await?;
        let propagated = records.iter().any(|r| r == &expected);
        Ok(propagated)
    }

    async fn dig_txt(record_name: &str) -> Result<Vec<String>, DnsServiceError> {
        use std::process::Command;
        let output = tokio::task::spawn_blocking({
            let record_name = record_name.to_owned();
            move || {
                Command::new("dig")
                    .args(["+short", "TXT", record_name.as_str()])
                    .output()
            }
        })
            .await
            .map_err(|e| DnsServiceError::DigPanic(e))?
            .map_err(|e| DnsServiceError::InternalError(format!("Failed to execute dig - {}", e.to_string())))?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let values: Vec<String> = stdout
            .lines()
            .filter(|l| !l.is_empty())
            .map(|l| l.trim_matches('"').to_owned())
            .collect();
        Ok(values)
    }
}