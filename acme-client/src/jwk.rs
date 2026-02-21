use std::error::Error;
use openssl::bn::{BigNum, BigNumRef};
use openssl::pkey::{PKey, Private, Public};
use openssl::rsa::Rsa;
use serde::Deserialize;
use serde_json::Value;
use crate::crypto::{SupportedAlgorithm, SupportedKey};
use crate::encoding::decode_b64;

#[derive(Deserialize, Debug)]
pub struct GenericJWK {
    alg: String,
    kty: String,
    crv: Option<String>,
    x: Option<String>,
    y: Option<String>,
    e: Option<String>,
    n: Option<String>,
}
pub struct RsaJWK {
    pub alg: SupportedAlgorithm,
    pub kty: SupportedKey,
    pub e: String,
    pub n: String,
}
pub struct EcJWK {
    pub alg: SupportedAlgorithm,
    pub kty: SupportedKey,
    pub crv: String,
    pub x: String,
    pub y: String,
}
pub struct EdJWK {
    pub alg: SupportedAlgorithm,
    pub kty: SupportedKey,
    pub crv: String,
    pub x: String,
}
impl GenericJWK {
    pub fn from_value(jwk: Value) -> Result<Self, Box<dyn Error>> {
        Ok(serde_json::from_value(jwk)?)
    }
    pub fn parse_pub(&self) -> Result<PKey<Public>, Box<dyn Error>> {
        match self.kty.as_str() {
            "RSA" => Ok(self.parse_rsa_pub()?),
            _ => panic!("Unknown kty {}", self.kty),
        }
    }
    fn parse_rsa_pub(&self) -> Result<PKey<Public>, Box<dyn Error>> {
        let n_coordinate_bytes = decode_b64(self.n.clone().unwrap().as_str())?;
        let e_coordinate_bytes = decode_b64(self.e.clone().unwrap().as_str())?;
        let n = BigNum::from_slice(&n_coordinate_bytes)?;
        let e = BigNum::from_slice(&e_coordinate_bytes)?;
        let rsa = Rsa::from_public_components(n, e)?;
        Ok(PKey::from_rsa(rsa)?)
    }
}

/// Size will differ on ES512, EC521 key coordinates should be 66 bytes in length
/// The big number ref returned by the key is 65.
///
/// A fast way to get a new vec with the 66 bytes and the rest copied over is to
/// use the resize method and fill the preceding bytes with padded "0"-s.
///
/// If you really want to lose brain cells, read FIPS 186-2
///
/// TL;DR
///
/// Take a vector of 5 bytes:
/// [215, 215, 215, 215, 215]
///
/// If coordinate needs to have length 7 bytes, the resulting vector *has* to be:
/// [0, 0, 215, 215, 215, 215, 215]
pub(crate) fn fast_padded_coordinate_vector(key_coordinate: &BigNumRef, coordinate_size: usize) -> Vec<u8> {
    let coordinate_vector = key_coordinate.to_vec();
    if coordinate_vector.len() == coordinate_size {
        return coordinate_vector;
    }
    // make new Vec with the full expected capacity.
    let mut padded_vec = Vec::with_capacity(coordinate_size);
    // truncate the bitch
    // ex. 66 (coordinate_size) - 65 (coordinate_vector.len()) == truncate to size 1 and pad the starting "overflow" with 0 bytes
    padded_vec.resize(coordinate_size - coordinate_vector.len(), 0);
    // fill in the blanks and the padded_vec should be of size `expected_coordinate_size`
    padded_vec.extend(coordinate_vector);
    padded_vec
}