use crate::crypto::SupportedAlgorithm;
use crate::encoding::{count_occurrences, decode_b64, encode_b64, remove_first};
use crate::jwk::GenericJWK;
use crate::keys::PrivateKey;
use openssl::hash::MessageDigest;
use openssl::sign::Verifier;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::error::Error;
use openssl::pkey::{PKey, Public};

pub struct JWS {
    header: JWSHeader,
    payload: Value,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct JWSHeader {
    alg: SupportedAlgorithm,
    kid: Option<String>,
    jwk: Option<Value>,
    nonce: Option<String>,
    url: Option<String>,
}

pub trait KeyFetcher {
    fn fetch_key(&self, kid: String) -> Result<PKey<Public>, Box<dyn Error>>;
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

    pub fn from_string(header: String) -> Result<Self, Box<dyn Error>> {
        Ok(serde_json::from_str(&header)?)
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
    pub fn get_payload(&self) -> &Value {
        &self.payload
    }
    pub fn finalize(&self, pkey: &PrivateKey) -> Result<String, Box<dyn Error>> {
        let encoded_header = encode_b64(self.header.serialize_with_pkey(&pkey)?.as_bytes());
        let encoded_payload = encode_b64(self.payload.to_string().as_bytes());
        let jws_data = format!("{}.{}", encoded_header, encoded_payload);
        let signature = pkey.sign(&self.header, &jws_data)?;
        Ok(format!("{}.{}", jws_data, encode_b64(signature.as_ref())))
    }
    pub fn parse(content: &String, key_fetcher: Box<dyn KeyFetcher>) -> Result<Option<JWS>, Box<dyn Error>> {
        if count_occurrences(content, '.') != 2 {
            return Err("Error encountered when parsing JWS, JWS parts formatting is invalid".into())
        }
        let signature_cut = content.rfind(".").unwrap();
        let (header_and_payload, sign) = content.split_at(signature_cut);
        let header_separator = header_and_payload.find('.').unwrap();
        let (header, payload) = header_and_payload.split_at(header_separator);
        let header_byte = decode_b64(header)?;
        let header_string = String::from_utf8(header_byte.clone())?;
        let header = JWSHeader::from_string(header_string.clone())?;
        let h_temp = header.clone();
        let key = match h_temp.jwk {
            Some(jwk) => GenericJWK::from_value(jwk)?.parse_pub(),
            None => {
                if h_temp.kid.is_none() {
                    return Err("JWK and KID parameter not found in JWS header".into())
                }
                Ok(key_fetcher.fetch_key(h_temp.kid.unwrap())?)
            }
        }?;
        Self::validate(key, header_and_payload, decode_b64(remove_first(sign))?)?;
        let payload_byte = decode_b64(remove_first(payload))?;
        let payload = serde_json::from_str(String::from_utf8(payload_byte)?.as_str())?;
        Ok(Some(JWS {
            header,
            payload
        }))
    }

    fn validate(key: PKey<Public>, header_and_payload: &str, signature_byte: Vec<u8>) -> Result<(), Box<dyn Error>> {
        let mut verifier = Verifier::new(MessageDigest::sha256(), &key)?;
        verifier.update(header_and_payload.as_bytes())?;
        let x = verifier.verify(signature_byte.as_slice())?;
        if !x {
            return Err("Signature verification failed".into());
        }
        Ok(())
    }
}