use crate::crypto::SupportedKey;
use crate::encoding::encode_b64;
use openssl::bn::{BigNum, BigNumContext};
use openssl::ec::{EcGroup, EcKey};
use openssl::error::ErrorStack;
use openssl::pkey::{PKey, Private};
use openssl::rsa::Rsa;
use openssl::ssl::Error;
use serde_json::json;

pub struct PrivateKey {
    kt: SupportedKey,
    pub k: PKey<Private>,
}

impl PrivateKey {
    pub fn from_supported_type(key_type: SupportedKey) -> Result<PrivateKey, Error> {
        match key_type {
            SupportedKey::Rsa2048 => {
                Ok(PrivateKey {
                    kt: key_type,
                    k: gen_rsa(2048)?
                })
            },
            SupportedKey::Rsa4096 => Ok(PrivateKey {
                kt: key_type,
                k: gen_rsa(4096)?
            }),
            SupportedKey::EcP256 | SupportedKey::EcP384 | SupportedKey::EcP521 => Ok(PrivateKey {
                kt: key_type.clone(),
                k: gen_ec(&key_type)?
            }),
            SupportedKey::Ed25519 => Ok(PrivateKey {
                kt: key_type.clone(),
                k: gen_ed()?
            })
        }
    }

    pub fn is_type(&self, key_type: &SupportedKey) -> bool {
        self.kt.eq(key_type)
    }

    pub fn get_jwk(&self) -> Result<serde_json::Value, Error> {
        match self.kt {
            SupportedKey::Rsa2048 | SupportedKey::Rsa4096 => {
                Ok(self.rsa_thumbprint()?)
            },
            SupportedKey::EcP256 | SupportedKey::EcP384 | SupportedKey::EcP521 => {
                Ok(self.ec_thumbprint()?)
            },
            SupportedKey::Ed25519 => {
                Ok(self.ed_thumbprint()?)
            }
        }
    }
    pub(crate) fn rsa_thumbprint(&self) -> Result<serde_json::Value, Error> {
        let rsa = self.k.rsa()?;
        Ok(json!({
            "kty": self.kt.get_kty(),
            "alg": self.kt.get_key_alg().to_string(),
            "e": encode_b64(&rsa.e().to_vec()),
            "n": encode_b64(&rsa.n().to_vec()),
        }))
    }
    pub(crate) fn ec_thumbprint(&self) -> Result<serde_json::Value, Error> {
        let ec = self.k.ec_key()?;
        let (padding, crv) = match self.kt {
            SupportedKey::EcP256 => (32, "P-256"),
            SupportedKey::EcP384 => (48, "P-384"),
            SupportedKey::EcP521 => (66, "P-521"),
            _ => return Err(Error::from(ErrorStack::get()))
        };
        let mut x = BigNum::new()?;
        let mut y = BigNum::new()?;
        let mut ctx = BigNumContext::new()?;
        let group = EcGroup::from_curve_name(self.kt.get_nid())?;
        ec.public_key().affine_coordinates(&group, &mut x, &mut y, &mut ctx)?;
        Ok(json!({
            "kty": self.kt.get_kty(),
            "alg": self.kt.get_key_alg().to_string(),
            "crv": crv,
            "x": encode_b64(&x.to_vec_padded(padding)?),
            "y": encode_b64(&y.to_vec_padded(padding)?),
        }))
    }
    pub(crate) fn ed_thumbprint(&self) -> Result<serde_json::Value, Error> {
        // Neither google nor siri knew what the fuck to do
        // chat-gpt suggested throwing everything in a temp-file and re-reading that
        // but that sounds too fucking nasty
        let pem = self.k.public_key_to_pem()?;
        let pem = match String::from_utf8(pem) {
            Ok(pem) => pem,
            Err(_) => return Err(Error::from(ErrorStack::get()))
        };
        let mut x = String::new();
        for line in pem.lines() {
            if line.is_empty() || line.starts_with("-----") {
                continue;
            }
            // don't even want know what poor bastard had to make this in the first place before it was yanked by chat-gpt
            x += &line.trim().trim_end_matches('=').replace('/', "_").replace('+', "-");
        }
        x.replace_range(..16, "");
        Ok(json!({
            "kty": self.kt.get_kty(),
            "alg": self.kt.get_key_alg().to_string(),
            "crv": "Ed25519",
            "x": &x,
        }))
    }
}

pub(crate) fn gen_rsa(key_length: u32) -> Result<PKey<Private>, Error> {
    let rsa = Rsa::generate(key_length)?;
    Ok(PKey::from_rsa(rsa)?)
}

pub(crate) fn gen_ec(ec_type: &SupportedKey) -> Result<PKey<Private>, Error> {
    let ec = EcKey::generate(EcGroup::from_curve_name(ec_type.get_nid())?.as_ref())?;
    Ok(PKey::from_ec_key(ec)?)
}

pub(crate) fn gen_ed() -> Result<PKey<Private>, Error> {
    Ok(PKey::generate_ed25519()?)
}