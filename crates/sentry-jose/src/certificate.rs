use openssl::x509::X509;

// TODO: remove allow
#[allow(dead_code)]
pub struct Certificate {
    x509: X509
}