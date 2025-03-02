use crate::crypto::SupportedKey;
use crate::encoding::{decode_b64, encode_b64};
use crate::keys::{gen_ec, gen_ed, gen_rsa, PrivateKey};
use openssl::bn::BigNum;
use openssl::rsa::Rsa;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
struct Jwk {
    kty: String,
    e: String,
    n: String,
}

#[test]
fn test_rsa_creation_and_length() {
    {
        let key = match gen_rsa(2048) {
            Ok(key) => key,
            Err(e) => panic!("{:?}", e),
        };
        assert_eq!(key.bits(), 2048);
    }
    {
        let key = match gen_rsa(4096) {
            Ok(key) => key,
            Err(e) => panic!("{:?}", e),
        };
        assert_eq!(key.bits(), 4096);
    }
}

#[test]
fn test_ec_creation_and_length() {
    {
        let key = match gen_ec(&SupportedKey::EcP256) {
            Ok(key) => key,
            Err(e) => panic!("{:?}", e),
        };
        assert_eq!(key.bits(), 256);
    }
    {
        let key = match gen_ec(&SupportedKey::EcP384) {
            Ok(key) => key,
            Err(e) => panic!("{:?}", e),
        };
        assert_eq!(key.bits(), 384);
    }
    {
        let key = match gen_ec(&SupportedKey::EcP521) {
            Ok(key) => key,
            Err(e) => panic!("{:?}", e),
        };
        assert_eq!(key.bits(), 521);
    }
}

#[test]
fn test_ed_creation_and_length() {
    {
        let key = match gen_ed() {
            Ok(key) => key,
            Err(e) => panic!("{:?}", e),
        };
        assert_eq!(key.bits(), 256);
    }
}

#[test]
fn test_rsa_thumbprint() {
    let key = match PrivateKey::from_supported_type(SupportedKey::Rsa4096) {
        Ok(key) => key,
        Err(e) => panic!("{:?}", e),
    };
    let jwk = match key.get_jwk() {
        Ok(key) => key,
        Err(e) => panic!("{:?}", e),
    };
    let rsa = key.k.rsa().unwrap();
    let e = encode_b64(&rsa.e().to_vec());
    let n = encode_b64(&rsa.n().to_vec());
    let j: Jwk = serde_json::from_value(jwk).unwrap();
    assert_eq!(j.kty, "RSA");
    assert_eq!(j.e, e);
    assert_eq!(j.n, n);
}

#[test]
fn test_rsa_public_key_thumbprint() {
    let key = match PrivateKey::from_supported_type(SupportedKey::Rsa2048) {
        Ok(key) => key,
        Err(e) => panic!("{:?}", e),
    };
    let jwk = match key.get_jwk() {
        Ok(key) => key,
        Err(e) => panic!("{:?}", e),
    };
    let rsa = key.k.rsa().unwrap();
    let j: Jwk = serde_json::from_value(jwk).unwrap();
    let e_bytes = decode_b64(j.e.as_bytes()).unwrap();
    let n_bytes = decode_b64(j.n.as_bytes()).unwrap();
    let e = BigNum::from_slice(&e_bytes).unwrap();
    let n = BigNum::from_slice(&n_bytes).unwrap();
    assert_eq!(rsa.e().to_vec(), e.to_vec());
    assert_eq!(rsa.n().to_vec(), n.to_vec());

    let r = Rsa::from_public_components(n, e).unwrap();

    assert_eq!(rsa.e(), r.e());
    assert_eq!(rsa.n(), r.n());
}