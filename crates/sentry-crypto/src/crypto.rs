use openssl::hash::MessageDigest;
use openssl::nid::Nid;
use openssl::sha::{sha256, sha384, sha512};
use crate::error::CryptoError;
use serde::{Deserialize, Serialize, Serializer};
use std::fmt::{Display, Formatter};
use crate::encoding::encode_b64;

#[derive(Debug, Clone, PartialEq)]
pub enum SupportedKey {
    Rsa2048,
    Rsa4096,
    EcP256,
    EcP384,
    EcP521,
    Ed25519
}

impl Display for SupportedKey {
    fn fmt(&self, f: &mut Formatter) -> Result<(), std::fmt::Error> {
        write!(f, "{}", self.to_string())
    }
}

impl SupportedKey {
    pub fn get_key_alg(&self) -> SupportedAlgorithm {
        match self {
            SupportedKey::Rsa2048 | SupportedKey::Rsa4096 => SupportedAlgorithm::RS256,
            SupportedKey::EcP256 => SupportedAlgorithm::ES256,
            SupportedKey::EcP384 => SupportedAlgorithm::ES384,
            SupportedKey::EcP521 => SupportedAlgorithm::ES512,
            SupportedKey::Ed25519 => SupportedAlgorithm::EdDSA
        }
    }

    pub fn from_curve(crv: &str) -> Result<SupportedKey, CryptoError> {
        match crv {
            "P-256" => Ok(SupportedKey::EcP256),
            "P-384" => Ok(SupportedKey::EcP384),
            "P-521" => Ok(SupportedKey::EcP521),
            _ => Err(CryptoError::UnknownCurveError(crv.to_string()))
        }
    }

    pub fn from_kty(kty: &str) -> Result<SupportedKey, CryptoError> {
        match kty {
            "RSA" => Ok(SupportedKey::Rsa2048),
            "EC" => Ok(SupportedKey::EcP256),
            "Ed25519" => Ok(SupportedKey::Ed25519),
            _ => Err(CryptoError::UnknownKtyError(kty.to_string())),
        }
    }

    pub fn from_str(string: &str) -> Result<SupportedKey, CryptoError> {
        match string.to_lowercase().as_str() {
            "rsa-2048" => Ok(SupportedKey::Rsa2048),
            "rsa-4096" => Ok(SupportedKey::Rsa4096),
            "ec-p256" => Ok(SupportedKey::EcP256),
            "ec-p384" => Ok(SupportedKey::EcP384),
            "ec-p521" => Ok(SupportedKey::EcP521),
            "ec-ed25519" => Ok(SupportedKey::Ed25519),
            _ => Err(CryptoError::UnknownKeyTypeError(string.to_string())),
        }
    }

    pub fn to_string(&self) -> &str {
        match self {
            SupportedKey::Rsa2048 => "RSA (2048 bytes)",
            SupportedKey::Rsa4096 => "RSA (4096 bytes)",
            SupportedKey::EcP256 => "EllipticCurve P-256",
            SupportedKey::EcP384 => "EllipticCurve P-384",
            SupportedKey::EcP521 => "EllipticCurve P-512",
            SupportedKey::Ed25519 => "Ed DSA (Ed25519)"
        }
    }

    pub fn get_kty(&self) -> &str {
        match self {
            SupportedKey::Rsa2048 | SupportedKey::Rsa4096 => "RSA",
            SupportedKey::EcP256 | SupportedKey::EcP384 | SupportedKey::EcP521 => "EC",
            SupportedKey::Ed25519 => "OKP",
        }
    }

    pub fn get_nid(&self) -> Nid {
        match self {
            SupportedKey::Rsa2048 | SupportedKey::Rsa4096 => Nid::RSA,
            SupportedKey::EcP256 => Nid::X9_62_PRIME256V1,
            SupportedKey::EcP384 => Nid::SECP384R1,
            SupportedKey::EcP521 => Nid::SECP521R1,
            SupportedKey::Ed25519 => Nid::X9_62_PRIME256V1,
        }
    }

    pub fn get_coordinate_size(&self) -> Result<usize, CryptoError> {
        match self {
            SupportedKey::EcP256 => Ok(32),
            SupportedKey::EcP384 => Ok(48),
            SupportedKey::EcP521 => Ok(66),
            kt => Err(CryptoError::CoordinateSizeError(kt.to_string().to_string()))
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
pub enum SupportedAlgorithm {
    RS256,
    ES256,
    ES384,
    ES512,
    EdDSA,
}

impl Display for SupportedAlgorithm {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

impl Serialize for SupportedAlgorithm {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl SupportedAlgorithm {
    pub fn to_string(&self) -> &str {
        match self {
            SupportedAlgorithm::RS256 => "RS256",
            SupportedAlgorithm::ES256 => "ES256",
            SupportedAlgorithm::ES384 => "ES384",
            SupportedAlgorithm::ES512 => "ES512",
            SupportedAlgorithm::EdDSA => "EdDSA",
        }
    }

    pub fn from_str(alg: &str) -> Result<SupportedAlgorithm, CryptoError> {
        match alg {
            "RS256" => Ok(SupportedAlgorithm::RS256),
            "ES256" => Ok(SupportedAlgorithm::ES256),
            "ES384" => Ok(SupportedAlgorithm::ES384),
            "ES512" => Ok(SupportedAlgorithm::ES512),
            "EdDSA" => Ok(SupportedAlgorithm::EdDSA),
             default => Err(CryptoError::AlgorithmSupportError(default.to_string())),
        }
    }

    pub fn get_hash(&self) -> SupportedHash {
        match self {
            SupportedAlgorithm::RS256 => SupportedHash::SHA256,
            SupportedAlgorithm::ES256 => SupportedHash::SHA256,
            SupportedAlgorithm::ES384 => SupportedHash::SHA384,
            SupportedAlgorithm::ES512 => SupportedHash::SHA512,
            SupportedAlgorithm::EdDSA => SupportedHash::SHA256
        }
    }
}

pub enum SupportedHash {
    SHA256,
    SHA384,
    SHA512 // SHA512 is also for EdDSA
}

pub fn sha256_b64(data: &[u8]) -> String {
    encode_b64(&sha256(data))
}

impl Into<String> for &SupportedHash {
    fn into(self) -> String {
        match self {
            SupportedHash::SHA256 => "SHA256".into(),
            SupportedHash::SHA384 => "SHA384".into(),
            SupportedHash::SHA512 => "SHA512".into(),
        }
    }
}

impl SupportedHash {
    pub fn hash(&self, hash_data: &[u8]) -> Vec<u8> {
        match self {
            SupportedHash::SHA256 => sha256(hash_data).to_vec(),
            SupportedHash::SHA384 => sha384(hash_data).to_vec(),
            SupportedHash::SHA512 => sha512(hash_data).to_vec(),
        }
    }

    pub fn get_digest(&self) -> MessageDigest {
        match self {
            SupportedHash::SHA256 => MessageDigest::sha256(),
            SupportedHash::SHA384 => MessageDigest::sha384(),
            SupportedHash::SHA512 => MessageDigest::sha512(),
        }
    }
}