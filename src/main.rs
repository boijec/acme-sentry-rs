use job_execution;
use acme_client;
use acme_client::crypto::SupportedKey;
use acme_client::keys::PrivateKey;

fn main() {
    let pkey = PrivateKey::from_supported_type(SupportedKey::Rsa2048).unwrap();
    let jwk = pkey.get_jwk().unwrap();
    println!("{:?}", jwk);
}
