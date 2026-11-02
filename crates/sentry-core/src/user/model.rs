pub struct ServiceUser {
    pub id: u32,
    pub user_id: String,
    pub key_type: String,
    pub key_path: String,
    pub key_name: String,
    pub user_dump_path: String,
}

#[derive(Default)]
pub struct ServiceUserBuilder {
    id: u32,
    user_id: String,
    key_type: String,
    key_path: String,
    key_name: String,
    user_dump_path: String,
}

impl ServiceUser {
    pub fn builder() -> ServiceUserBuilder {
        ServiceUserBuilder::default()
    }
}

impl ServiceUserBuilder {
    pub fn id(mut self, id: u32) -> Self {
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
    pub fn build(self) -> ServiceUser {
        ServiceUser {
            id: self.id,
            user_id: self.user_id,
            key_type: self.key_type,
            key_path: self.key_path,
            key_name: self.key_name,
            user_dump_path: self.user_dump_path,
        }
    }
}