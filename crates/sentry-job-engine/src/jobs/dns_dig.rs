use crate::dependencies::JobDependencies;
use crate::error::JobEngineError;
use crate::job::Job;
use crate::scheduling::SchedulerHandle;
use async_trait::async_trait;
use tracing::{info, warn};

/// Maximum number of polling retries before giving up on DNS propagation.
const MAX_DIG_RETRIES: u32 = 30;

pub struct DnsDigJob {
    pub order_id: i64,
    pub user_id: String,
    pub challenge_id: i64,
    pub attempt: u32,
}

impl DnsDigJob {
    pub fn new(order_id: i64, user_id: String, challenge_id: i64) -> Self {
        DnsDigJob {
            order_id,
            user_id,
            challenge_id,
            attempt: 0,
        }
    }

    fn next_attempt(&self) -> Self {
        DnsDigJob {
            order_id: self.order_id,
            user_id: self.user_id.clone(),
            challenge_id: self.challenge_id,
            attempt: self.attempt + 1,
        }
    }

    async fn dig_txt(record_name: &str) -> anyhow::Result<Vec<String>> {
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
        .map_err(|e| anyhow::anyhow!("dig task panicked: {}", e))?
        .map_err(|e| anyhow::anyhow!("Failed to execute dig: {}", e))?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let values: Vec<String> = stdout
            .lines()
            .filter(|l| !l.is_empty())
            .map(|l| l.trim_matches('"').to_owned())
            .collect();
        Ok(values)
    }
}

#[async_trait]
impl<'a> Job<'a> for DnsDigJob {
    fn job_type(&self) -> &'a str {
        "dns-dig-job"
    }

    fn inject_dependencies(&mut self, _dependencies: &JobDependencies) {
        return;
    }

    async fn execute(&self, handle: SchedulerHandle<'a>) -> Result<(), JobEngineError> {
        let conn = DatabaseConnection::get_connection()
            .map_err(|e| anyhow::anyhow!("DB connection failed: {}", e))?;

        let challenge = AcmeDnsChallenge::get_for_order(self.order_id, &conn)
            .map_err(|e| anyhow::anyhow!("Failed to load challenge: {}", e))?
            .ok_or_else(|| anyhow::anyhow!("No challenge found for order {}", self.order_id))?;

        // Compute the expected TXT value from key_authorization (base64url of sha256)
        let expected_txt =
            acme_client::crypto::sha256_b64(challenge.key_authorization.as_bytes());

        info!(
            "Dig attempt {}/{} for '{}' expecting TXT='{}'",
            self.attempt + 1,
            MAX_DIG_RETRIES,
            challenge.dns_record_name,
            expected_txt
        );

        let records = Self::dig_txt(&challenge.dns_record_name).await?;
        let propagated = records.iter().any(|r| r == &expected_txt);

        if propagated {
            info!(
                "DNS TXT record for '{}' confirmed! Marking challenge as valid.",
                challenge.dns_record_name
            );
            AcmeDnsChallenge::set_status(challenge.id, "valid", &conn)
                .map_err(|e| anyhow::anyhow!("Failed to update challenge status: {}", e))?;

            handle
                .submit(GenerateCsrJob::new(self.order_id, self.user_id.clone()))
                .await
                .map_err(|e| anyhow::anyhow!("Failed to submit GenerateCsrJob: {}", e))?;
        } else if self.attempt >= MAX_DIG_RETRIES {
            warn!(
                "DNS TXT record for '{}' not found after {} attempts — marking order as invalid.",
                challenge.dns_record_name, MAX_DIG_RETRIES
            );
            AcmeOrder::set_status(self.order_id, "invalid", &conn)
                .map_err(|e| anyhow::anyhow!("Failed to update order status: {}", e))?;
            AcmeDnsChallenge::set_status(challenge.id, "invalid", &conn)
                .map_err(|e| anyhow::anyhow!("Failed to update challenge status: {}", e))?;
        } else {
            warn!(
                "DNS TXT record for '{}' not yet visible (attempt {}/{}). Retrying in 30s.",
                challenge.dns_record_name,
                self.attempt + 1,
                MAX_DIG_RETRIES
            );
            // Brief delay before re-queuing so we don't hammer immediately
            tokio::time::sleep(std::time::Duration::from_secs(30)).await;
            handle
                .submit(self.next_attempt())
                .await
                .map_err(|e| anyhow::anyhow!("Failed to re-submit DnsDigJob: {}", e))?;
        }

        Ok(())
    }
}
