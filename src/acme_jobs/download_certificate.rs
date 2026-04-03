use crate::job_execution::job_base::{Job, SchedulerHandle};
use acme_client::comms::account::AcmeAccountClient;
use acme_client::crypto::SupportedKey;
use acme_client::keys::PrivateKey;
use async_trait::async_trait;
use common_utils::APPLICATION_CONFIG;
use persistence::account_link::AcmeAccountLink;
use persistence::certificate::AcmeCertificate;
use persistence::database::DatabaseConnection;
use persistence::directory::AcmeDirectory;
use persistence::order::AcmeOrder;
use persistence::user::AcmeUser;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tracing::info;

/// Polling retries before giving up on a processing order.
const MAX_POLL_RETRIES: u32 = 20;

#[derive(Serialize, Deserialize)]
pub struct DownloadCertificateJob {
    pub order_id: i64,
    pub user_id: String,
    pub attempt: u32,
}

impl DownloadCertificateJob {
    pub fn new(order_id: i64, user_id: String) -> Self {
        DownloadCertificateJob {
            order_id,
            user_id,
            attempt: 0,
        }
    }

    fn next_attempt(&self) -> Self {
        DownloadCertificateJob {
            order_id: self.order_id,
            user_id: self.user_id.clone(),
            attempt: self.attempt + 1,
        }
    }
}

#[async_trait]
impl Job for DownloadCertificateJob {
    fn job_type(&self) -> &'static str {
        "download-certificate-job"
    }

    fn payload(&self) -> Value {
        serde_json::to_value(self).unwrap()
    }

    async fn execute(&self, handle: SchedulerHandle) -> anyhow::Result<()> {
        let conn = DatabaseConnection::get_connection()
            .map_err(|e| anyhow::anyhow!("DB connection failed: {}", e))?;

        let user = AcmeUser::get_user(self.user_id.as_str(), &conn)
            .map_err(|e| anyhow::anyhow!("Failed to look up user: {}", e))?
            .ok_or_else(|| anyhow::anyhow!("User '{}' not found", self.user_id))?;

        let order = AcmeOrder::get_by_id(self.order_id, &conn)
            .map_err(|e| anyhow::anyhow!("Failed to load order {}: {}", self.order_id, e))?
            .ok_or_else(|| anyhow::anyhow!("Order {} not found", self.order_id))?;

        let account_link = AcmeAccountLink::get_for_user(user.id, &conn)
            .map_err(|e| anyhow::anyhow!("Failed to load account link: {}", e))?
            .ok_or_else(|| anyhow::anyhow!("No account link for user '{}'", self.user_id))?;

        let dir = AcmeDirectory::get_existing(user.id, &conn)?
            .ok_or_else(|| anyhow::anyhow!("No ACME directory for user '{}'", self.user_id))?;

        let pem_bytes = std::fs::read(&user.key_path)
            .map_err(|e| anyhow::anyhow!("Failed to read PEM: {}", e))?;
        let supported_key = SupportedKey::from_str(&user.key_type)
            .map_err(|e| anyhow::anyhow!("Unsupported key type: {}", e))?;
        let private_key = PrivateKey::load_private_bytes(&pem_bytes, supported_key)
            .map_err(|e| anyhow::anyhow!("Failed to load private key: {}", e))?;

        let acme = AcmeAccountClient::new(private_key, account_link.account_url, true)
            .map_err(|e| anyhow::anyhow!("Failed to create ACME client: {}", e))?;

        // If certificate_url is not yet set, poll the order URL for an update
        let certificate_url = if let Some(cert_url) = &order.certificate_url {
            cert_url.clone()
        } else {
            let order_url = order
                .order_url
                .as_ref()
                .ok_or_else(|| anyhow::anyhow!("Order {} has no order_url to poll", self.order_id))?
                .clone();

            match acme
                .poll_order_certificate_url(&dir.new_nonce, &order_url)
                .await?
            {
                Some(cert_url) => {
                    AcmeOrder::set_certificate_url(self.order_id, &cert_url, &conn)
                        .map_err(|e| anyhow::anyhow!("Failed to persist certificate_url: {}", e))?;
                    cert_url
                }
                None if self.attempt < MAX_POLL_RETRIES => {
                    info!(
                        "Order {} still processing (attempt {}/{}). Retrying in 10s.",
                        self.order_id,
                        self.attempt + 1,
                        MAX_POLL_RETRIES
                    );
                    tokio::time::sleep(std::time::Duration::from_secs(10)).await;
                    handle
                        .submit(self.next_attempt())
                        .await
                        .map_err(|_| {
                            anyhow::anyhow!("Failed to re-submit DownloadCertificateJob")
                        })?;
                    return Ok(());
                }
                None => {
                    return Err(anyhow::anyhow!(
                        "Order {} did not become valid after {} poll attempts.",
                        self.order_id,
                        MAX_POLL_RETRIES
                    ));
                }
            }
        };

        let cert_bytes = acme
            .download_certificate(&dir.new_nonce, &certificate_url)
            .await?;

        let config = APPLICATION_CONFIG.get().unwrap();
        let cert_path = format!(
            "{}/{}.crt.pem",
            config.output_dir,
            order.domain.replace('.', "_")
        );
        std::fs::write(&cert_path, &cert_bytes)
            .map_err(|e| anyhow::anyhow!("Failed to write certificate to '{}': {}", cert_path, e))?;

        info!(
            "Certificate for domain '{}' saved to '{}'.",
            order.domain, cert_path
        );

        AcmeCertificate::record(self.order_id, &cert_path, &conn)
            .map_err(|e| anyhow::anyhow!("Failed to record certificate in DB: {}", e))?;

        Ok(())
    }
}
