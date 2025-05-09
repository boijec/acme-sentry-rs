use std::error::Error;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use crate::encoding::encode_b64;
use crate::keys::PrivateKey;

pub struct JWS {
    header: JWSHeader,
    payload: Value,
    signature: Option<String>
}

#[derive(Deserialize, Serialize, Debug)]
pub struct JWSHeader {
    alg: String,
    kid: Option<String>,
    jwk: Option<Value>,
    nonce: Option<String>,
    url: Option<String>,
}

impl JWSHeader {
    pub fn with_alg(alg: &str) -> Self {
        JWSHeader{
            alg: alg.to_string(),
            kid: None,
            jwk: None,
            nonce: None,
            url: None,
        }
    }
    pub fn serialize_with_pkey(&self, pkey: &PrivateKey) -> Result<String, Box<dyn Error>> {
        Ok(serde_json::to_string(&json!({
            "alg": self.alg,
            "jwk": pkey.get_jwk()?,
            "nonce": "placeholder",
            "url": "",
        }))?)
    }

    pub fn serialize_with_kid(&self, kid: String) -> Result<String, Box<dyn Error>> {
        Ok(serde_json::to_string(&json!({
            "alg": self.alg,
            "kid": Some(kid),
            "nonce": "placeholder",
            "url": "",
        }))?)
    }
}

impl JWS {
    pub fn with_header_and_payload(header: JWSHeader, payload: Value) -> Self {
        JWS {
            header,
            payload,
            signature: None,
        }
    }
    pub fn finalize(&self, pkey: &PrivateKey) -> Result<String, Box<dyn Error>> {
        let prep_string = format!("{}.{}", encode_b64(self.header.serialize_with_pkey(&pkey)?.as_bytes()), encode_b64(self.payload.to_string().as_bytes()));
        let t = pkey.sign(&prep_string)?;
        Ok(format!("{}.{}", prep_string, encode_b64(t.as_ref())))
    }
}