use std::sync::Arc;
use sentry_jose::keys::PrivateKey;

pub struct ServiceUser {
    pub id: i64,
    pub user_id: String,
    pub key_type: String,
    pub key_path: String,
    pub key_name: String,
    pub user_dump_path: String,

    pub private_key: Option<PrivateKey>,
    pub user_account_resource: Option<String>,
}

#[derive(Default)]
pub struct ServiceUserBuilder {
    id: i64,
    user_id: String,
    key_type: String,
    key_path: String,
    key_name: String,
    user_dump_path: String,

    private_key: Option<PrivateKey>,
    user_account_resource: Option<String>,
}

impl ServiceUser {
    pub fn builder() -> ServiceUserBuilder {
        ServiceUserBuilder::default()
    }
    pub fn with_account_resource(mut self, account_resource: &String) -> ServiceUser {
        self.user_account_resource = Some(account_resource.clone());
        self
    }
    pub fn with_private_key(mut self, private_key: &PrivateKey) -> ServiceUser {
        self.private_key = Some(private_key.clone());
        self
    }
}

impl ServiceUserBuilder {
    pub fn id(mut self, id: i64) -> Self {
        self.id = id;
        self
    }
    pub fn user_id(mut self, user_id: &str) -> Self {
        self.user_id = user_id.to_owned();
        self
    }
    pub fn key_type(mut self, key_type: &str) -> Self {
        self.key_type = key_type.to_owned();
        self
    }
    pub fn key_path(mut self, key_path: &str) -> Self {
        self.key_path = key_path.to_owned();
        self
    }
    pub fn key_name(mut self, key_name: &str) -> Self {
        self.key_name = key_name.to_owned();
        self
    }
    pub fn user_dump_path(mut self, user_dump_path: &str) -> Self {
        self.user_dump_path = user_dump_path.to_owned();
        self
    }
    pub fn private_key(mut self, private_key: PrivateKey) -> Self {
        self.private_key = Some(private_key);
        self
    }
    pub fn user_account_resource(mut self, user_account_resource: &str) -> Self {
        self.user_account_resource = Some(user_account_resource.to_owned());
        self
    }
    pub fn build(self) -> ServiceUser {
        ServiceUser {
            id: self.id,
            user_id: self.user_id,
            key_type: self.key_type,
            key_path: self.key_path,
            key_name: self.key_name,
            user_dump_path: self.user_dump_path,
            private_key: self.private_key,
            user_account_resource: self.user_account_resource,
        }
    }
}