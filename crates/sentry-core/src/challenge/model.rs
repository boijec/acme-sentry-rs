pub struct ServiceOrderChallenge {
    pub id: i64,
    pub order_id: i64,
    pub token: String,
    pub key_authorization: String,
    pub dns_record_name: String,
    pub txt_record_value: String,
    pub status: String,
    pub challenge_ready_url: String
}

#[derive(Default)]
pub struct ServiceOrderChallengeBuilder {
    id: i64,
    order_id: i64,
    token: String,
    key_authorization: String,
    dns_record_name: String,
    txt_record_value: String,
    status: String,
    challenge_ready_url: String
}

impl ServiceOrderChallenge {
    pub fn builder() -> ServiceOrderChallengeBuilder {
        ServiceOrderChallengeBuilder::default()
    }
}

impl ServiceOrderChallengeBuilder {
    pub fn id(mut self, id: i64) -> Self {
        self.id = id;
        self
    }
    pub fn order_id(mut self, order_id: i64) -> Self {
        self.order_id = order_id;
        self
    }
    pub fn token(mut self, token: &str) -> Self {
        self.token = token.to_owned();
        self
    }
    pub fn key_authorization(mut self, key_authorization: &str) -> Self {
        self.key_authorization = key_authorization.to_owned();
        self
    }
    pub fn dns_record_name(mut self, dns_record_name: &str) -> Self {
        self.dns_record_name = dns_record_name.to_owned();
        self
    }
    pub fn txt_record_value(mut self, txt_record_value: &str) -> Self {
        self.txt_record_value = txt_record_value.to_owned();
        self
    }
    pub fn status(mut self, status: &str) -> Self {
        self.status = status.to_owned();
        self
    }
    pub fn challenge_ready_url(mut self, challenge_ready_url: &str) -> Self {
        self.challenge_ready_url = challenge_ready_url.to_owned();
        self
    }
    pub fn build(self) -> ServiceOrderChallenge {
        ServiceOrderChallenge {
            id: self.id,
            order_id: self.order_id,
            token: self.token,
            key_authorization: self.key_authorization,
            dns_record_name: self.dns_record_name,
            txt_record_value: self.txt_record_value,
            status: self.status,
            challenge_ready_url: self.challenge_ready_url,
        }
    }
}

pub enum ChallengeMark {
    Verified,
    Failed,
}