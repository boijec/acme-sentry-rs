use crate::crypto::SupportedKey;
use crate::keys::PrivateKey;

#[test]
fn test_rsa_creation_and_length() {
    {
        let key = match PrivateKey::from_supported_type(SupportedKey::Rsa2048) {
            Ok(key) => key,
            Err(e) => panic!("{:?}", e),
        };
        assert!(key.is_type(&SupportedKey::Rsa2048));
        assert_eq!(key.k.bits(), 2048);
    }
    {
        let key = match PrivateKey::from_supported_type(SupportedKey::Rsa4096) {
            Ok(key) => key,
            Err(e) => panic!("{:?}", e),
        };
        assert!(key.is_type(&SupportedKey::Rsa4096));
        assert_eq!(key.k.bits(), 4096);
    }
}

#[test]
fn test_ec_creation_and_length() {
    {
        let key = match PrivateKey::from_supported_type(SupportedKey::EcP256) {
            Ok(key) => key,
            Err(e) => panic!("{:?}", e),
        };
        assert!(key.is_type(&SupportedKey::EcP256));
        assert_eq!(key.k.bits(), 256);
    }
    {
        let key = match PrivateKey::from_supported_type(SupportedKey::EcP384) {
            Ok(key) => key,
            Err(e) => panic!("{:?}", e),
        };
        assert!(key.is_type(&SupportedKey::EcP384));
        assert_eq!(key.k.bits(), 384);
    }
    {
        let key = match PrivateKey::from_supported_type(SupportedKey::EcP521) {
            Ok(key) => key,
            Err(e) => panic!("{:?}", e),
        };
        assert!(key.is_type(&SupportedKey::EcP521));
        assert_eq!(key.k.bits(), 521);
    }
}

#[test]
fn test_ed_creation_and_length() {
    {
        let key = match PrivateKey::from_supported_type(SupportedKey::Ed25519) {
            Ok(key) => key,
            Err(e) => panic!("{:?}", e),
        };
        assert!(key.is_type(&SupportedKey::Ed25519));
        assert_eq!(key.k.bits(), 256);
    }
}