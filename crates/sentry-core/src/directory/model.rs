#[derive(Clone)]
pub struct ServiceUserDirectory {
    pub key_change: String,
    pub new_authz: Option<String>,
    pub new_nonce: String,
    pub new_account: String,
    pub new_order: String,
    pub revoke_cert: String,
}

#[derive(Default)]
pub struct ServiceUserDirectoryBuilder {
    key_change: String,
    new_authz: Option<String>,
    new_nonce: String,
    new_account: String,
    new_order: String,
    revoke_cert: String,
}

impl ServiceUserDirectory {
    pub fn builder() -> ServiceUserDirectoryBuilder {
        ServiceUserDirectoryBuilder::default()
    }
}

impl ServiceUserDirectoryBuilder {
    pub fn key_change(mut self, key_change: &str) -> Self {
        self.key_change = key_change.into();
        self
    }
    pub fn new_authz(mut self, new_authz: &str) -> Self {
        self.new_authz = Some(new_authz.into());
        self
    }
    pub fn new_authz_option(mut self, new_authz: Option<String>) -> Self {
        self.new_authz = new_authz;
        self
    }
    pub fn new_nonce(mut self, new_nonce: &str) -> Self {
        self.new_nonce = new_nonce.into();
        self
    }
    pub fn new_account(mut self, new_account: &str) -> Self {
        self.new_account = new_account.into();
        self
    }
    pub fn new_order(mut self, new_order: &str) -> Self {
        self.new_order = new_order.into();
        self
    }
    pub fn revoke_cert(mut self, revoke_cert: &str) -> Self {
        self.revoke_cert = revoke_cert.into();
        self
    }
    pub fn build(self) -> ServiceUserDirectory {
        ServiceUserDirectory {
            key_change: self.key_change,
            new_authz: self.new_authz,
            new_nonce: self.new_nonce,
            new_account: self.new_account,
            new_order: self.new_order,
            revoke_cert: self.revoke_cert,
        }
    }
}