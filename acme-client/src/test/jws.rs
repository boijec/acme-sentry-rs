use serde_json::json;
use crate::crypto::SupportedKey;
use crate::jws::{JWSHeader, JWS};
use crate::keys::PrivateKey;

#[test]
fn test_signing_jws() {
    let pkey = PrivateKey::from_supported_type(SupportedKey::EcP521).unwrap();
    let jws = JWS::with_header_and_payload(JWSHeader::with_alg(SupportedKey::EcP521.get_key_alg()), json!({
        "testPayload": "test",
    }));
    let t = jws.finalize(&pkey).unwrap();
    println!("{}", t);
}