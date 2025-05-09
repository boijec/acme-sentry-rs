use crate::crypto::SupportedKey;
use crate::encoding::encode_b64;
use crate::jws::JWSHeader;
use openssl::bn::{BigNum, BigNumContext, BigNumRef};
use openssl::ec::{EcGroup, EcKey};
use openssl::ecdsa::EcdsaSig;
use openssl::hash::MessageDigest;
use openssl::pkey::{PKey, Private};
use openssl::rsa::Rsa;
use openssl::sign::Signer;
use serde_json::{json, Value};
use std::error::Error;

pub struct PrivateKey {
    kt: SupportedKey,
    pub k: PKey<Private>,
}

impl PrivateKey {
    pub fn from_supported_type(key_type: SupportedKey) -> Result<Self, Box<dyn Error>> {
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

    pub fn get_jwk(&self) -> Result<serde_json::Value, Box<dyn Error>> {
        match self.kt {
            SupportedKey::Rsa2048 | SupportedKey::Rsa4096 => Ok(self.rsa_jwk()?),
            SupportedKey::EcP256 | SupportedKey::EcP384 | SupportedKey::EcP521 => Ok(self.ec_jwk()?),
            SupportedKey::Ed25519 => Ok(self.ed_jwk()?)
        }
    }

    pub (crate) fn rsa_jwk(&self) -> Result<serde_json::Value, Box<dyn Error>> {
        let rsa = self.k.rsa()?;
        Ok(json!({
            "kty": self.kt.get_kty(),
            "alg": self.kt.get_key_alg().to_string(),
            "e": encode_b64(&rsa.e().to_vec()),
            "n": encode_b64(&rsa.n().to_vec()),
        }))
    }
    pub (crate) fn ec_jwk(&self) -> Result<serde_json::Value, Box<dyn Error>> {
        let ec = self.k.ec_key()?;
        // "padding" but really - sizes according to RFC 7517
        let (padding, crv) = match self.kt {
            SupportedKey::EcP256 => (32, "P-256"),
            SupportedKey::EcP384 => (48, "P-384"),
            SupportedKey::EcP521 => (66, "P-521"),
            _ => return Err("Unsupported key type".into())
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
    pub (crate) fn ed_jwk(&self) -> Result<serde_json::Value, Box<dyn Error>> {
        // Neither google nor siri knew what the fuck to do
        // chat-gpt suggested throwing everything in a temp-file and re-reading that
        // but that sounds too fucking nasty
        let pem = self.k.public_key_to_pem()?;
        let pem = match String::from_utf8(pem) {
            Ok(pem) => pem,
            Err(_) => return Err("Could not read Utf-8 string".into())
        };
        let mut x = String::new();
        for line in pem.lines() {
            if line.is_empty() || line.starts_with("-----") {
                continue;
            }
            // don't even want know what the (original dev) poor bastard had to do to make this work in the first place before it was yanked by chat-gpt
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

    pub fn sign(&self, header: &JWSHeader, jws_data: &String) -> Result<Vec<u8>, Box<dyn Error>> {
        match self.kt {
            SupportedKey::Rsa2048 | SupportedKey::Rsa4096 => {
                Ok(self.sign_rsa(jws_data)?)
            }
            SupportedKey::EcP256 | SupportedKey::EcP384 | SupportedKey::EcP521 => {
                Ok(self.sign_elliptic_curve(header, jws_data)?)
            }
            SupportedKey::Ed25519 => {
                Ok(self.sign_ed(jws_data)?)
            }
        }

    }

    fn sign_rsa(&self, data: &String) -> Result<Vec<u8>, Box<dyn Error>> {
        let mut t = Signer::new(MessageDigest::sha256(), &self.k)?;
        Ok(t.sign_oneshot_to_vec(data.as_bytes())?)
    }

    fn sign_elliptic_curve(&self, header: &JWSHeader, data: &String) -> Result<Vec<u8>, Box<dyn Error>> {
        let hash = header.get_alg().get_hash().hash(data.as_bytes())?;
        let ec_sign = EcdsaSig::sign(&hash, &self.k.ec_key()?.as_ref())?;
        let size = self.kt.get_coordinate_size();
        let mut r = fast_padded_coordinate_vector(ec_sign.r(), size);
        let mut s = fast_padded_coordinate_vector(ec_sign.s(), size);
        r.append(&mut s);
        Ok(r)
    }

    fn sign_ed(&self, data: &String) -> Result<Vec<u8>, Box<dyn Error>> {
        let mut t = Signer::new_without_digest(&self.k)?;
        Ok(t.sign_oneshot_to_vec(data.as_bytes())?)
    }
}

/// Size will differ on ES512, EC521 key coordinates should be 66 bytes in length
/// The big number ref returned by the key is 65.
/// As fast way to get a new arr with the 66 bytes and the rest copied over is to
/// use the resize method and fill the preceding bytes with padded "0"-s.
/// If you really want to lose brain cells, read X9.62 or FIPS 186-2
fn fast_padded_coordinate_vector(key_coordinate: &BigNumRef, coordinate_size: usize) -> Vec<u8> {
    let coordinate_vector = key_coordinate.to_vec();
    if coordinate_vector.len() == coordinate_size {
        return coordinate_vector;
    }
    // make new Vec with the full expected capacity.
    let mut padded_vec = Vec::with_capacity(coordinate_size);
    // truncate the bitch, use the leftovers as padding.
    // ex. 66 (coordinate_size) - 65 (coordinate_vector.len()) == truncate to size 1 and pad the starting "overflow" with 0 bytes
    padded_vec.resize(coordinate_size - coordinate_vector.len(), 0);
    // fill in the blanks and the capped_vec should be of size `expected_coordinate_size`
    padded_vec.extend(coordinate_vector);
    padded_vec
}

pub(crate) fn gen_rsa(key_length: u32) -> Result<PKey<Private>, Box<dyn Error>> {
    let rsa = Rsa::generate(key_length)?;
    Ok(PKey::from_rsa(rsa)?)
}

pub(crate) fn gen_ec(ec_type: &SupportedKey) -> Result<PKey<Private>, Box<dyn Error>> {
    let ec = EcKey::generate(EcGroup::from_curve_name(ec_type.get_nid())?.as_ref())?;
    Ok(PKey::from_ec_key(ec)?)
}

pub(crate) fn gen_ed() -> Result<PKey<Private>, Box<dyn Error>> {
    Ok(PKey::generate_ed25519()?)
}