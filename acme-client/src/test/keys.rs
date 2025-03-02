use crate::crypto::SupportedKey;
use crate::keys::{gen_ec, gen_ed, gen_rsa};

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