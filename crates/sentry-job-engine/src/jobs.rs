use crate::error::PreFlightError;
use crate::jobs::directory_query::DirectoryUpdateJob;
use crate::jobs::initialize_keys_for_user::InitializeLocalUserJob;
use crate::jobs::login_user::LoginUserJob;
use crate::scheduling::SchedulerHandle;

pub mod accept_dns_challenge;
pub mod directory_query;
pub mod dns_dig;
pub mod download_certificate;
pub mod generate_csr;
pub mod initialize_keys_for_user;
pub mod login_user;
pub mod place_order;

pub async fn preflight_start_sequence(handle: &SchedulerHandle<'_>) -> Result<(), PreFlightError> {
    handle
        .submit(InitializeLocalUserJob::builder().build())
        .await?;
    handle
        .submit(DirectoryUpdateJob::builder().build())
        .await?;
    handle
        .submit(LoginUserJob::builder().build())
        .await?;
    handle
        .submit(DirectoryUpdateJob::builder().fetch_local().build())
        .await?;
    Ok(())
}