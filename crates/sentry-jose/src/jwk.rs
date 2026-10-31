use crate::error::{JwkError, KeyError};
use openssl::bn::BigNumRef;
use openssl::pkey::{PKey, Public};
use openssl::rsa::Rsa;
use sentry_crypto::encoding::decode_b64;
use sentry_crypto::ssl_common::big_num_from_vec;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Serialize, Deserialize, Debug)]
pub struct GenericJWK {
    alg: String,
    kty: String,
    crv: Option<String>,
    x: Option<String>,
    y: Option<String>,
    e: Option<String>,
    n: Option<String>,
}
impl GenericJWK {
    pub fn from_value(jwk: Value) -> Result<Self, JwkError> {
        Ok(serde_json::from_value(jwk)?)
    }
    pub fn parse_pub(&self) -> Result<PKey<Public>, JwkError> {
        match self.kty.as_str() {
            "RSA" => Ok(self.parse_rsa_pub()?),
            _ => Err(JwkError::UnknownKeyTypeError(self.kty.clone())),
        }
    }
    fn parse_rsa_pub(&self) -> Result<PKey<Public>, JwkError> {
        let n_coordinate_bytes = decode_b64(self.n.clone().unwrap().as_str())?;
        let e_coordinate_bytes = decode_b64(self.e.clone().unwrap().as_str())?;
        let n = big_num_from_vec(&n_coordinate_bytes)?;
        let e = big_num_from_vec(&e_coordinate_bytes)?;
        let rsa = Rsa::from_public_components(n, e)
            .map_err(|e| JwkError::PublicKeyComponentsError(e))?;
        Ok(PKey::from_rsa(rsa)
            .map_err(|e| JwkError::from(KeyError::UnableToCast(e, "RSA".to_string())))?)
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
pub fn fast_padded_coordinate_vector(key_coordinate: &BigNumRef, coordinate_size: usize) -> Vec<u8> {
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