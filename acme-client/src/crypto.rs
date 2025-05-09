use openssl::nid::Nid;
use openssl::sha::{sha256, sha384, sha512};
use serde::{Deserialize, Serialize, Serializer};
use std::error::Error;
use std::fmt::{Display, Formatter};

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

    pub fn get_coordinate_size(&self) -> usize {
        match self {
            SupportedKey::EcP256 => 32,
            SupportedKey::EcP384 => 48,
            SupportedKey::EcP521 => 66,
            kt => panic!("Coordinate size is not available for key type: {}", kt)
        }
    }
}

#[derive(Debug, Deserialize)]
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

    pub fn from_str(alg: &str) -> SupportedAlgorithm {
        match alg {
            "RS256" => SupportedAlgorithm::RS256,
            "ES256" => SupportedAlgorithm::ES256,
            "ES384" => SupportedAlgorithm::ES384,
            "ES512" => SupportedAlgorithm::ES512,
            "EdDSA" => SupportedAlgorithm::EdDSA,
             default => panic!("Algorithm {} is not supported", default)
        }
    }

    pub fn get_hash(&self) -> SupportedHash {
        match self {
            SupportedAlgorithm::ES256 => SupportedHash::SHA256,
            SupportedAlgorithm::ES384 => SupportedHash::SHA384,
            SupportedAlgorithm::ES512 => SupportedHash::SHA512,
            any => panic!("Hash not supported for alg: {}", any)
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