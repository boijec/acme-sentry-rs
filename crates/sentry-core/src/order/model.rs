use crate::challenge::model::ServiceOrderChallenge;

pub struct ServiceUserOrder {
    pub id: i64,
    pub domain: String,
    pub order_url: Option<String>,
    pub authorization_url: Option<String>,
    pub finalize_url: Option<String>,
    pub certificate_url: Option<String>,
    pub status: String,
    pub created_at: String,

    pub challenge_info: Option<ServiceOrderChallenge>,
}

#[derive(Default)]
pub struct ServiceUserOrderBuilder {
    id: i64,
    domain: String,
    order_url: Option<String>,
    authorization_url: Option<String>,
    finalize_url: Option<String>,
    certificate_url: Option<String>,
    status: String,
    created_at: String,

    challenge_info: Option<ServiceOrderChallenge>,
}

pub enum DownloadStatus {
    Result(Vec<u8>),
    ContinuePolling,
}

impl ServiceUserOrder {
    pub fn builder() -> ServiceUserOrderBuilder {
        ServiceUserOrderBuilder::default()
    }
}

impl ServiceUserOrderBuilder {
    pub fn id(mut self, id: i64) -> Self {
        self.id = id;
        self
    }
    pub fn domain(mut self, domain: &str) -> Self {
        self.domain = domain.to_owned();
        self
    }
    pub fn order_url(mut self, order_url: &str) -> Self {
        self.order_url = Some(order_url.to_owned());
        self
    }
    pub fn order_url_option(mut self, order_url: impl ConvertInto<Option<String>>) -> Self {
        self.order_url = order_url.cnv_into();
        self
    }
    pub fn authorization_url(mut self, authorization_url: &str) -> Self {
        self.authorization_url = Some(authorization_url.to_owned());
        self
    }
    pub fn authorization_url_option(
        mut self,
        authorization_url: impl ConvertInto<Option<String>>,
    ) -> Self {
        self.authorization_url = authorization_url.cnv_into();
        self
    }
    pub fn finalize_url(mut self, finalize_url: &str) -> Self {
        self.finalize_url = Some(finalize_url.to_owned());
        self
    }
    pub fn finalize_url_option(mut self, finalize_url: impl ConvertInto<Option<String>>) -> Self {
        self.finalize_url = finalize_url.cnv_into();
        self
    }
    pub fn certificate_url(mut self, certificate_url: &str) -> Self {
        self.certificate_url = Some(certificate_url.to_owned());
        self
    }
    pub fn certificate_url_option(
        mut self,
        certificate_url: impl ConvertInto<Option<String>>,
    ) -> Self {
        self.certificate_url = certificate_url.cnv_into();
        self
    }
    pub fn status(mut self, status: &str) -> Self {
        self.status = status.to_owned();
        self
    }
    pub fn created_at(mut self, created_at: &str) -> Self {
        self.created_at = created_at.to_owned();
        self
    }
    pub fn challenge_info(mut self, challenge_info: ServiceOrderChallenge) -> Self {
        self.challenge_info = Some(challenge_info);
        self
    }
    pub fn build(self) -> ServiceUserOrder {
        ServiceUserOrder {
            id: self.id,
            domain: self.domain,
            order_url: self.order_url,
            authorization_url: self.authorization_url,
            finalize_url: self.finalize_url,
            certificate_url: self.certificate_url,
            status: self.status,
            created_at: self.created_at,

            challenge_info: self.challenge_info,
        }
    }
}

pub trait ConvertInto<T> {
    fn cnv_into(self) -> T;
}

impl ConvertInto<Option<String>> for Option<&str> {
    fn cnv_into(self) -> Option<String> {
        match self {
            Some(str) => Some(str.to_owned()),
            None => None,
        }
    }
}

impl ConvertInto<Option<String>> for Option<String> {
    fn cnv_into(self) -> Option<String> {
        self
    }
}
