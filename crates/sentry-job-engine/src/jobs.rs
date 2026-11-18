use crate::dependencies::JobDependencies;
use crate::error::PreFlightError;
use crate::jobs::initialize_keys_for_user::LoadLocalUserJob;
use crate::scheduling::SchedulerHandle;

pub mod accept_dns_challenge;
pub mod dns_dig;
pub mod download_certificate;
pub mod generate_csr;
pub mod initialize_keys_for_user;
pub mod login_user;
pub mod place_order;

pub async fn preflight_start_sequence(
    handle: SchedulerHandle<JobDependencies>,
) -> Result<(), PreFlightError> {
    handle.submit(LoadLocalUserJob::instantiate(), None).await?;
    Ok(())
}
