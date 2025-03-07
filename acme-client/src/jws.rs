use std::error::Error;
use serde::{Deserialize, Serialize};
use serde_json::json;
use crate::encoding::encode_b64;
use crate::keys::PrivateKey;

pub struct JWS {
    header: JWSHeader,
    payload: String,
    signature: String,
    ready: bool,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct JWSHeader {
    alg: String,
    nonce: String,
    url: String,
    jwk: serde_json::Value,
}

impl JWSHeader {
    fn serialize(&self) -> Result<String, Box<dyn Error>> {
        Ok(serde_json::to_string(&json!({
            "alg": self.alg,
            "nonce": self.nonce,
            "url": self.url,
            "jwk": self.jwk,
        }))?)
    }
}

impl JWS {
    pub fn finalize(&self, _pkey: PrivateKey) -> Result<String, Box<dyn Error>> {
        // TODO: implement additional logic to result in error if JWS is not ready
        if !self.ready {
            return Err("Invalid JWS header".into())
        }
        Ok(format!("{}.{}", encode_b64(self.header.serialize()?.as_bytes()), encode_b64(self.payload.as_bytes())))
        // return head
    }
}