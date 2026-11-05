use crate::error::SharedResourceError;
use crate::user::model::ServiceUser;
use sentry_jose::keys::PrivateKey;

pub fn get_shared_resources(
    user: &ServiceUser,
) -> Result<(PrivateKey, String), SharedResourceError> {
    let pkey = user
        .private_key
        .clone()
        .ok_or_else(|| SharedResourceError::MissingResource("Private Key".into()))?;
    let account_resource = user
        .user_account_resource
        .clone()
        .ok_or_else(|| SharedResourceError::MissingResource("Account URL".into()))?;
    Ok((pkey, account_resource))
}
