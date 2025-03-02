use std::error::Error;
use openssl::nid::Nid;
use openssl::sha::{sha256, sha384, sha512};

#[derive(Debug, Clone, PartialEq)]
pub enum SupportedKey {
    Rsa2048,
    Rsa4096,
    EcP256,
    EcP384,
    EcP521,
    Ed25519
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
}

pub enum SupportedAlgorithm {
    RS256,
    ES256,
    ES384,
    ES512,
    EdDSA,
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
}

pub enum SupportedHash {
    SHA256,
    SHA384,
    SHA512 // SHA512 is also for EdDSA
}

impl SupportedHash {
    pub fn hash(&self, hash_data: &[u8]) -> Result<Vec<u8>, Box<dyn Error>> {
        match self {
            SupportedHash::SHA256 => Ok(sha256(hash_data).to_vec()),
            SupportedHash::SHA384 => Ok(sha384(hash_data).to_vec()),
            SupportedHash::SHA512 => Ok(sha512(hash_data).to_vec())
        }
    }
}