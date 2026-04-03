use crate::crypto::SupportedAlgorithm;
use crate::encoding::{count_occurrences, decode_b64, encode_b64, remove_first};
use crate::jwk::GenericJWK;
use crate::keys::PrivateKey;
use openssl::bn::BigNum;
use openssl::ecdsa::EcdsaSig;
use openssl::hash::MessageDigest;
use openssl::pkey::{PKey, Public};
use openssl::sign::Verifier;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::error::Error;

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
        JWSHeader {
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
        JWS { header, payload }
    }

    pub fn get_payload(&self) -> &Value {
        &self.payload
    }

    pub fn finalize(&self, pkey: &PrivateKey) -> Result<String, Box<dyn Error>> {
        let encoded_header = encode_b64(self.header.serialize_with_pkey(pkey)?.as_bytes());
        let encoded_payload = encode_b64(self.payload.to_string().as_bytes());
        let jws_data = format!("{}.{}", encoded_header, encoded_payload);
        let signature = pkey.sign(&self.header, &jws_data)?;
        Ok(format!("{}.{}", jws_data, encode_b64(signature.as_ref())))
    }

    pub fn parse(
        content: &String,
        key_fetcher: Box<dyn KeyFetcher>,
    ) -> Result<Option<JWS>, Box<dyn Error>> {
        if count_occurrences(content, '.') != 2 {
            return Err(
                "Error encountered when parsing JWS, JWS parts formatting is invalid".into(),
            );
        }
        let signature_cut = content.rfind('.').unwrap();
        let (header_and_payload, sign) = content.split_at(signature_cut);
        let header_separator = header_and_payload.find('.').unwrap();
        let (header, payload) = header_and_payload.split_at(header_separator);

        let header_bytes = decode_b64(header)?;
        let header_string = String::from_utf8(header_bytes)?;
        let header = JWSHeader::from_string(header_string)?;

        let h_temp = header.clone();
        let key = match h_temp.jwk {
            Some(jwk) => GenericJWK::from_value(jwk)?.parse_pub(),
            None => {
                if h_temp.kid.is_none() {
                    return Err("JWK and KID parameter not found in JWS header".into());
                }
                Ok(key_fetcher.fetch_key(h_temp.kid.unwrap())?)
            }
        }?;

        Self::verify(&header.alg, key, header_and_payload, decode_b64(remove_first(sign))?)?;

        let payload_bytes = decode_b64(remove_first(payload))?;
        let payload = serde_json::from_str(String::from_utf8(payload_bytes)?.as_str())?;
        Ok(Some(JWS { header, payload }))
    }

    pub fn parse_flat(
        value: &Value,
        key_fetcher: Box<dyn KeyFetcher>,
    ) -> Result<Option<JWS>, Box<dyn Error>> {
        let protected_b64 = value["protected"]
            .as_str()
            .ok_or("Flat JWS missing 'protected' field")?;
        let payload_b64 = value["payload"]
            .as_str()
            .ok_or("Flat JWS missing 'payload' field")?;
        let signature_b64 = value["signature"]
            .as_str()
            .ok_or("Flat JWS missing 'signature' field")?;

        let header_bytes = decode_b64(protected_b64)?;
        let header_string = String::from_utf8(header_bytes)?;
        let header = JWSHeader::from_string(header_string)?;

        let h_temp = header.clone();
        let key = match h_temp.jwk {
            Some(jwk) => GenericJWK::from_value(jwk)?.parse_pub(),
            None => {
                if h_temp.kid.is_none() {
                    return Err(
                        "JWK and KID parameter not found in flat JWS protected header".into(),
                    );
                }
                Ok(key_fetcher.fetch_key(h_temp.kid.unwrap())?)
            }
        }?;

        // Signed data in flat serialisation: base64url(protected) + "." + base64url(payload)
        let signing_input = format!("{}.{}", protected_b64, payload_b64);
        let signature_bytes = decode_b64(signature_b64)?;
        Self::verify(&header.alg, key, &signing_input, signature_bytes)?;

        let payload_bytes = decode_b64(payload_b64)?;
        let payload: Value = if payload_bytes.is_empty() {
            Value::Null
        } else {
            serde_json::from_str(String::from_utf8(payload_bytes)?.as_str())?
        };

        Ok(Some(JWS { header, payload }))
    }

    fn verify(
        alg: &SupportedAlgorithm,
        key: PKey<Public>,
        signed_input: &str,
        signature: Vec<u8>,
    ) -> Result<(), Box<dyn Error>> {
        match alg {
            SupportedAlgorithm::RS256 => {
                Self::verify_with_rsa(&key, signed_input, &signature)?;
            }

            SupportedAlgorithm::ES256
            | SupportedAlgorithm::ES384
            | SupportedAlgorithm::ES512 => {
                Self::verify_with_ec(alg, &key, signed_input, &signature)?;
            }

            SupportedAlgorithm::EdDSA => {
                Self::verify_with_ed(&key, signed_input, &signature)?;
            }
        }
        Ok(())
    }

    fn verify_with_rsa(key: &PKey<Public>, signed_input: &str, signature: &Vec<u8>) -> Result<(), Box<dyn Error>> {
        let mut verifier = Verifier::new(MessageDigest::sha256(), &key)?;
        verifier.update(signed_input.as_bytes())?;
        if !verifier.verify(&signature)? {
            return Err("RSA (RS256) signature verification failed".into());
        }
        Ok(())
    }

    fn verify_with_ec(alg: &SupportedAlgorithm, key: &PKey<Public>, signed_input: &str, signature: &Vec<u8>) -> Result<(), Box<dyn Error>> {
        let coord_size: usize = match alg {
            SupportedAlgorithm::ES256 => 32,
            SupportedAlgorithm::ES384 => 48,
            SupportedAlgorithm::ES512 => 66,
            _ => unreachable!(),
        };
        let der_sig = ecdsa_raw_to_der(&signature, coord_size)?;
        let digest = alg.get_hash().get_digest();
        let mut verifier = Verifier::new(digest, &key)?;
        verifier.update(signed_input.as_bytes())?;
        if !verifier.verify(&der_sig)? {
            return Err(
                format!("ECDSA ({}) signature verification failed", alg).into(),
            );
        }
        Ok(())
    }

    fn verify_with_ed(key: &PKey<Public>, signed_input: &str, signature: &Vec<u8>) -> Result<(), Box<dyn Error>> {
        let mut verifier = Verifier::new_without_digest(&key)?;
        if !verifier.verify_oneshot(&signature, signed_input.as_bytes())? {
            return Err("EdDSA signature verification failed".into());
        }
        Ok(())
    }
}

/// Converts a raw ECDSA signature (R‖S concatenation as used in JWS RFC 7518 §3.4)
/// into an ASN.1 DER SEQUENCE, which is the format OpenSSL's verifier expects.
fn ecdsa_raw_to_der(raw: &[u8], coord_size: usize) -> Result<Vec<u8>, Box<dyn Error>> {
    if raw.len() != coord_size * 2 {
        return Err(format!(
            "Invalid raw ECDSA signature length: expected {} bytes ({} per coordinate), got {}",
            coord_size * 2,
            coord_size,
            raw.len()
        )
        .into());
    }
    let r = BigNum::from_slice(&raw[..coord_size])?;
    let s = BigNum::from_slice(&raw[coord_size..])?;
    let sig = EcdsaSig::from_private_components(r, s)?;
    Ok(sig.to_der()?)
}
