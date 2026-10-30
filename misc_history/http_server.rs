use crate::acme_jobs::place_order::PlaceOrderJob;
use crate::job_execution::job_base::SchedulerHandle;
use acme_client::comms::account::AcmeAccountClient;
use acme_client::keys::load_private_key;
use actix_web::{Error, HttpResponse, Responder, web};
use anyhow::anyhow;
use common_utils::APPLICATION_CONFIG;
use persistence::account_link::AcmeAccountLink;
use persistence::database::DatabaseConnection;
use persistence::directory::AcmeDirectory;
use serde::{Deserialize, Serialize};
use serde_json::json;
use tracing::info;

#[derive(Debug, Deserialize, Serialize)]
pub struct NewOrderRequest {
    pub domain: String,
}

pub async fn post_new_order<'a>(
    req: web::Json<NewOrderRequest>,
    handle: web::Data<SchedulerHandle<'a>>,
) -> impl Responder {
    info!("Received new order request: domain='{}'", req.domain);
    let user_id = APPLICATION_CONFIG.get().unwrap().user_id.clone();
    match handle
        .submit(PlaceOrderJob::new(user_id, req.domain.clone()))
        .await
    {
        Ok(_) => HttpResponse::Accepted().json(json!({
            "status": "accepted",
            "message": format!("Order for domain '{}' has been queued.", req.domain)
        })),
        Err(err) => HttpResponse::ServiceUnavailable().json(json!({
            "status": "error",
            "message": format!("Failed to queue order: {}", err)
        })),
    }
}

#[allow(dead_code)]
#[derive(Debug)]
pub enum ApiResponseError {
    ClientBuildError(anyhow::Error),
    DbConnectionEmail(anyhow::Error),
    SubRequestError(anyhow::Error),
    KeyFetchError(anyhow::Error),
    AccountLinkError(anyhow::Error),
    DirectoryError(anyhow::Error),
}

impl std::fmt::Display for ApiResponseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl ApiResponseError {
    #[allow(unused)]
    pub fn as_str(&self) -> String {
        match self {
            ApiResponseError::ClientBuildError(e) => e.root_cause().to_string(),
            ApiResponseError::SubRequestError(e) => e.root_cause().to_string(),
            ApiResponseError::KeyFetchError(e) => e.root_cause().to_string(),
            ApiResponseError::DbConnectionEmail(e) => e.root_cause().to_string(),
            ApiResponseError::AccountLinkError(e) => e.root_cause().to_string(),
            ApiResponseError::DirectoryError(e) => e.root_cause().to_string(),
        }
    }
}

impl actix_web::ResponseError for ApiResponseError {}

pub async fn check_account_status() -> Result<impl Responder, Error> {
    let user_id = APPLICATION_CONFIG.get().unwrap().user_id.clone();
    let conn = DatabaseConnection::get_connection()
        .map_err(|e| ApiResponseError::DbConnectionEmail(anyhow!("DB connection failed: {}", e)))?;

    let account_link = AcmeAccountLink::get_for_user(1, &conn)
        .map_err(|e| ApiResponseError::AccountLinkError(anyhow!("DB connection failed: {}", e)))?
        .ok_or_else(|| ApiResponseError::AccountLinkError(anyhow!("No account link for user '{}'", user_id)))?;

    let dir = AcmeDirectory::get_existing(1, &conn)
        .map_err(|e| ApiResponseError::DirectoryError(e))?
        .ok_or_else(|| ApiResponseError::DirectoryError(anyhow!("No account directory for user '{}'", user_id)))?;

    let private_key = load_private_key(&user_id, "/home/demorith/acme-sentry/cae60c12-2d2f-48d1-adcc-cce5e9f55d9f/login-keys/ec-p521", "ec-p521")
        .map_err(|e| ApiResponseError::KeyFetchError(e))?;

    let acme = AcmeAccountClient::new(private_key, account_link.account_url, true)
        .map_err(|e| ApiResponseError::AccountLinkError(anyhow!("Failed to create ACME client: {}", e)))?;
    let t = acme
        .get_account(&dir.new_nonce)
        .await
        .map_err(|err| ApiResponseError::SubRequestError(err))?;
    Ok(HttpResponse::Ok().json(t))
}
