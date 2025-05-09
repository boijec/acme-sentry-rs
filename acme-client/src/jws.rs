use std::error::Error;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use crate::crypto::SupportedAlgorithm;
use crate::encoding::encode_b64;
use crate::keys::PrivateKey;

pub struct JWS {
    header: JWSHeader,
    payload: Value,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct JWSHeader {
    alg: SupportedAlgorithm,
    kid: Option<String>,
    jwk: Option<Value>,
    nonce: Option<String>,
    url: Option<String>,
}

impl JWSHeader {
    // bad idea, find something else... this will be impacted by the "directory" communication.
    pub fn with_alg(alg: SupportedAlgorithm) -> Self {
        JWSHeader{
            alg,
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

    pub fn get_alg(&self) -> &SupportedAlgorithm {
        &self.alg
    }
}

impl JWS {
    pub fn with_header_and_payload(header: JWSHeader, payload: Value) -> Self {
        JWS {
            header,
            payload,
        }
    }
    pub fn finalize(&self, pkey: &PrivateKey) -> Result<String, Box<dyn Error>> {
        let encoded_header = encode_b64(self.header.serialize_with_pkey(&pkey)?.as_bytes());
        let encoded_payload = encode_b64(self.payload.to_string().as_bytes());
        let jws_data = format!("{}.{}", encoded_header, encoded_payload);
        let signature = pkey.sign(&self.header, &jws_data)?;
        Ok(format!("{}.{}", jws_data, encode_b64(signature.as_ref())))
    }
}