use crate::crypto::SupportedKey;
use crate::encoding::encode_b64;
use crate::error::KeyError::JWKFormattingError;
use crate::error::{CryptoError, KeyError};
use crate::jwk::fast_padded_coordinate_vector;
use crate::jws::JWSHeader;
use crate::ssl_common::{get_big_num, get_big_num_context};
use openssl::ec::{EcGroup, EcKey};
use openssl::ecdsa::EcdsaSig;
use openssl::pkey::{PKey, Private};
use openssl::rsa::Rsa;
use openssl::sign::Signer;
use serde_json::json;

pub struct PrivateKey {
    pub kt: SupportedKey,
    pub k: PKey<Private>
}

impl PrivateKey {
    pub fn from_supported_type(key_type: SupportedKey) -> Result<Self, KeyError> {
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

    pub fn get_jwk(&self) -> Result<serde_json::Value, KeyError> {
        match self.kt {
            SupportedKey::Rsa2048 | SupportedKey::Rsa4096 => Ok(self.rsa_jwk()?),
            SupportedKey::EcP256 | SupportedKey::EcP384 | SupportedKey::EcP521 => Ok(self.ec_jwk()?),
            SupportedKey::Ed25519 => Ok(self.ed_jwk()?)
        }
    }

    pub (crate) fn rsa_jwk(&self) -> Result<serde_json::Value, KeyError> {
        let rsa = self.k.rsa()
            .map_err(|e| KeyError::UnableToCast(e, self.kt.to_string().to_string()))?;
        Ok(json!({
            "kty": self.kt.get_kty(),
            "alg": self.kt.get_key_alg().to_string(),
            "e": encode_b64(&rsa.e().to_vec()),
            "n": encode_b64(&rsa.n().to_vec()),
        }))
    }
    pub (crate) fn ec_jwk(&self) -> Result<serde_json::Value, KeyError> {
        let ec = self.k.ec_key()
            .map_err(|e| KeyError::UnableToCast(e, self.kt.to_string().to_string()))?;
        // "padding" but really - sizes according to RFC 7517
        let (padding, crv) = match self.kt {
            SupportedKey::EcP256 => (32, "P-256"),
            SupportedKey::EcP384 => (48, "P-384"),
            SupportedKey::EcP521 => (66, "P-521"),
            _ => return Err(KeyError::from(CryptoError::UnknownKeyTypeError(self.kt.to_string().to_string())))
        };
        let mut x = get_big_num()?;
        let mut y = get_big_num()?;
        let mut ctx = get_big_num_context()?;
        let group = EcGroup::from_curve_name(self.kt.get_nid())
            .map_err(|e| KeyError::EcGroupError(e, self.kt.get_nid().as_raw()))?;
        ec.public_key().affine_coordinates(&group, &mut x, &mut y, &mut ctx)
            .map_err(|e| KeyError::AffineCoordinatesError(e))?;
        Ok(json!({
            "kty": self.kt.get_kty(),
            "alg": self.kt.get_key_alg().to_string(),
            "crv": crv,
            "x": encode_b64(&x.to_vec_padded(padding)
                .map_err(|e| KeyError::UnableToPadVector(e))?),
            "y": encode_b64(&y.to_vec_padded(padding)
                .map_err(|e| KeyError::UnableToPadVector(e))?),
        }))
    }
    pub (crate) fn ed_jwk(&self) -> Result<serde_json::Value, KeyError> {
        // Neither google nor siri knew what the fuck to do
        // chat-gpt suggested throwing everything in a temp-file and re-reading that
        // but that sounds too fucking nasty
        let pem = self.k.public_key_to_pem()
            .map_err(|e| KeyError::UnableToSerializeToPem(e, self.kt.to_string().to_string()))?;
        let pem = match String::from_utf8(pem) {
            Ok(pem) => pem,
            Err(e) => return Err(KeyError::UnableToReadPemFromUtf8Error(e))
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

    pub fn sign(&self, header: &JWSHeader, jws_data: &String) -> Result<Vec<u8>, KeyError> {
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

    pub fn get_pem_bytes(&self) -> Result<Vec<u8>, KeyError> {

        let a = self.k.ec_key()
            .map_err(|e| KeyError::UnableToCast(e, self.kt.to_string().to_string()))?
            .private_key_to_pem()
            .map_err(|e| KeyError::UnableToReadKeyFromPemError(e))?;
        Ok(a)
    }

    pub fn load_private_bytes(pem: &Vec<u8>, supported_key: SupportedKey) -> Result<Self, KeyError> {
        let key = PKey::private_key_from_pem(pem)
            .map_err(|e| KeyError::UnableToReadKeyFromPemError(e))?;
        Ok(PrivateKey {
            kt: supported_key,
            k: key,
        })
    }

    pub fn jwk_thumbprint(&self) -> Result<String, KeyError> {
        use crate::crypto::SupportedHash;
        let jwk = self.get_jwk()?;
        let canonical = canonical_jwk_string(&jwk)?;
        let hash = SupportedHash::SHA256.hash(canonical.as_bytes());
        Ok(encode_b64(&hash))
    }

    fn sign_rsa(&self, data: &String) -> Result<Vec<u8>, KeyError> {
        let mut signer = Signer::new(self.kt.get_key_alg().get_hash().get_digest(), &self.k)
            .map_err(|e| KeyError::KeySignerError(e, self.kt.to_string().to_string()))?;
        Ok(
            signer.sign_oneshot_to_vec(data.as_bytes())
                .map_err(|e| KeyError::KeySigningError(e, self.kt.to_string().to_string()))?
        )
    }

    fn sign_elliptic_curve(&self, header: &JWSHeader, data: &String) -> Result<Vec<u8>, KeyError> {
        let hash = header.get_alg().get_hash().hash(data.as_bytes());
        let signer = EcdsaSig::sign(&hash, &self.k.ec_key()
            .map_err(|e| KeyError::KeySigningError(e, self.kt.to_string().to_string()))?
            .as_ref())
            .map_err(|e| KeyError::KeySigningError(e, self.kt.to_string().to_string()))?;
        let coordinate_size = self.kt.get_coordinate_size()?;
        let mut r = fast_padded_coordinate_vector(signer.r(), coordinate_size);
        let s = fast_padded_coordinate_vector(signer.s(), coordinate_size);
        r.extend(s);
        Ok(r)
    }

    fn sign_ed(&self, data: &String) -> Result<Vec<u8>, KeyError> {
        let mut t = Signer::new_without_digest(&self.k)
            .map_err(|e| KeyError::KeySignerError(e, self.kt.to_string().to_string()))?;
        Ok(
            t.sign_oneshot_to_vec(data.as_bytes())
                .map_err(|e| KeyError::KeySigningError(e, self.kt.to_string().to_string()))?
        )
    }
}

/// Returns either a private key instance, or an anyhow error describing
pub fn load_private_key(user_id: &str, key_path: &str, key_type: &str) -> anyhow::Result<PrivateKey> {
    let absolute_key_path = key_path.to_owned() + "/" + user_id + ".pem";
    let pem_bytes = std::fs::read(absolute_key_path)
        .map_err(|e| anyhow::anyhow!("Failed to read PEM: {}", e))?;
    let supported_key = SupportedKey::from_str(key_type)
        .map_err(|e| anyhow::anyhow!("Unsupported key type: {}", e))?;
    let private_key = PrivateKey::load_private_bytes(&pem_bytes, supported_key)
        .map_err(|e| anyhow::anyhow!("Failed to load private key: {}", e))?;
    Ok(private_key)
}

pub(crate) fn gen_rsa(key_length: u32) -> Result<PKey<Private>, KeyError> {
    let rsa = Rsa::generate(key_length)
        .map_err(|e| KeyError::RsaGenerationError(e, key_length))?;
    Ok(
        PKey::from_rsa(rsa)
            .map_err(|e| KeyError::PrivateKeyFromConversionError(e, "RSA".to_string()))?
    )
}

pub(crate) fn gen_ec(ec_type: &SupportedKey) -> Result<PKey<Private>, KeyError> {
    let ec = EcKey::generate(
        EcGroup::from_curve_name(ec_type.get_nid())
            .map_err(|e| KeyError::EcCurveError(e, ec_type.get_nid().as_raw()))?
            .as_ref()
    )
        .map_err(|e| KeyError::EcGenerationError(e, ec_type.to_owned()))?;
    Ok(
        PKey::from_ec_key(ec)
            .map_err(|e| KeyError::PrivateKeyFromConversionError(e, ec_type.to_string().to_string()))?
    )
}

pub(crate) fn gen_ed() -> Result<PKey<Private>, KeyError> {
    Ok(
        PKey::generate_ed25519()
            .map_err(|e| KeyError::EdGenerationError(e, SupportedKey::Ed25519))?
    )
}

fn canonical_jwk_string(jwk: &serde_json::Value) -> Result<String, KeyError> {
    let kty = jwk["kty"].as_str().ok_or(JWKFormattingError("JWK missing kty".into()))?;
    match kty {
        "EC" => Ok(format!(
            r#"{{"crv":"{}","kty":"{}","x":"{}","y":"{}"}}"#,
            jwk["crv"].as_str().ok_or(JWKFormattingError("EC JWK missing crv".into()))?,
            kty,
            jwk["x"].as_str().ok_or(JWKFormattingError("EC JWK missing x".into()))?,
            jwk["y"].as_str().ok_or(JWKFormattingError("EC JWK missing y".into()))?,
        )),
        "RSA" => Ok(format!(
            r#"{{"e":"{}","kty":"{}","n":"{}"}}"#,
            jwk["e"].as_str().ok_or(JWKFormattingError("RSA JWK missing e".into()))?,
            kty,
            jwk["n"].as_str().ok_or(JWKFormattingError("RSA JWK missing n".into()))?,
        )),
        "OKP" => Ok(format!(
            r#"{{"crv":"{}","kty":"{}","x":"{}"}}"#,
            jwk["crv"].as_str().ok_or(JWKFormattingError("OKP JWK missing crv".into()))?,
            kty,
            jwk["x"].as_str().ok_or(JWKFormattingError("OKP JWK missing x".into()))?,
        )),
        other => Err(JWKFormattingError(format!("Unsupported JWK kty '{}' for thumbprint", other).into())),
    }
}