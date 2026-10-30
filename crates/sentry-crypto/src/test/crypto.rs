use openssl::sha;
use crate::crypto::SupportedHash;

#[test]
fn test_hashing() {
    let mut hasher = sha::Sha256::new();
    hasher.update(b"test");
    let hash = hasher.finish();
    let hash_data = b"test";

    let hash_result = SupportedHash::SHA256.hash(hash_data);
    
    assert_eq!(hash_result.len(), 32);
    assert_eq!(hex::encode(hash_result), hex::encode(hash));
}