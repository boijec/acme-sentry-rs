use crate::error::DefaultOperationsError;
use crate::jws::{build_jws_body_kid, build_jws_body_kid_post_as_get};
use sentry_jose::keys::PrivateKey;
use serde_json::{Value, json};

pub trait DefaultOperations {
    fn get_account_url(&self) -> String;
    fn get_private_key(&self) -> PrivateKey;

    fn protected_kid(&self, nonce: String, url: &str) -> Value {
        let alg = self.get_private_key().kt.get_key_alg();
        json!({
            "alg":   alg.to_string(),
            "kid":   self.get_account_url(),
            "nonce": nonce,
            "url":   url,
        })
    }

    fn build_jws_body_kid(
        &self,
        protected: Value,
        payload: &Value,
    ) -> Result<Value, DefaultOperationsError> {
        build_jws_body_kid(protected, payload, &self.get_private_key())
    }

    /// POST-as-GET variant: payload is the empty string (base64url of empty bytes).
    fn build_jws_body_kid_post_as_get(
        &self,
        protected: Value,
    ) -> Result<Value, DefaultOperationsError> {
        build_jws_body_kid_post_as_get(protected, &self.get_private_key())
    }
}

pub(crate) trait ConvertOperation<T: DefaultOperations> {
    fn convert(self) -> T;
}