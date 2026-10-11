use crate::acme_jobs::download_certificate::DownloadCertificateJob;
use crate::job_execution::job_base::{Job, SchedulerHandle};
use acme_client::comms::account::AcmeAccountClient;
use acme_client::crypto::SupportedKey;
use acme_client::keys::{load_private_key, PrivateKey};
use async_trait::async_trait;
use persistence::account_link::AcmeAccountLink;
use persistence::database::DatabaseConnection;
use persistence::directory::AcmeDirectory;
use persistence::order::AcmeOrder;
use persistence::user::AcmeUser;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tracing::info;
use persistence::challenge::AcmeDnsChallenge;

#[derive(Serialize, Deserialize)]
pub struct GenerateCsrJob {
    pub order_id: i64,
    pub user_id: String,
}

impl GenerateCsrJob {
    pub fn new(order_id: i64, user_id: String) -> Self {
        GenerateCsrJob { order_id, user_id }
    }
}

#[async_trait]
impl Job for GenerateCsrJob {
    fn job_type(&self) -> &'static str {
        "generate-csr-job"
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

        let finalize_url = order
            .finalize_url
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Order {} has no finalize_url", self.order_id))?
            .clone();

        let account_link = AcmeAccountLink::get_for_user(user.id, &conn)
            .map_err(|e| anyhow::anyhow!("Failed to load account link: {}", e))?
            .ok_or_else(|| anyhow::anyhow!("No account link for user '{}'", self.user_id))?;

        let dir = AcmeDirectory::get_existing(user.id, &conn)?
            .ok_or_else(|| anyhow::anyhow!("No ACME directory for user '{}'", self.user_id))?;

        let private_key = load_private_key(&user.user_id, &user.key_path, &user.key_type)?;

        let acme = AcmeAccountClient::new(private_key, account_link.account_url, true)
            .map_err(|e| anyhow::anyhow!("Failed to create ACME client: {}", e))?;

        let challenge_info = AcmeDnsChallenge::get_for_order(self.order_id, &conn)
            .map_err(|e| anyhow::anyhow!("Failed to load challenge for order: {}", e))?.unwrap();

        // TODO: use challenge url from DB
        acme.notify_challenge_ready(&dir.new_nonce, "")
            .await?;

        info!(
            "DNS challenge (id={}) persisted and CA notified. Submitting DnsDigJob.",
            challenge_info.id
        );

        info!(
            "Sending finalize request for domain '{}' (order {}).",
            order.domain, self.order_id
        );

        let result = acme
            .finalize(&dir.new_nonce, &finalize_url, &order.domain)
            .await?;

        if let Some(cert_url) = result.certificate_url {
            AcmeOrder::set_certificate_url(self.order_id, &cert_url, &conn)
                .map_err(|e| anyhow::anyhow!("Failed to set certificate_url: {}", e))?;
            info!("Certificate URL available: {}", cert_url);
        } else {
            info!("Order is being processed — certificate URL not yet available.");
            AcmeOrder::set_status(self.order_id, "processing", &conn)
                .map_err(|e| anyhow::anyhow!("Failed to update order status: {}", e))?;
        }

        handle
            .submit(DownloadCertificateJob::new(self.order_id, self.user_id.clone()))
            .await
            .map_err(|_| anyhow::anyhow!("Failed to submit DownloadCertificateJob"))?;

        Ok(())
    }
}
