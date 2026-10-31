use crate::keys::PrivateKey;
use openssl::hash::MessageDigest;
use openssl::x509::{X509NameBuilder, X509Req, X509ReqBuilder};
use std::error::Error;

// TODO: remove allow
#[allow(dead_code)]
pub struct CertificateSigningRequest {
    x509req: X509Req,
}

// TODO: remove allow
#[allow(dead_code)]
pub fn create_new_certificate_request(
    country: &str,
    state: Option<&str>,
    city: &str,
    organization: &str,
    department: &str,
    url: &str,
    pkey: &openssl::pkey::PKey<openssl::pkey::Private>,
) -> Result<CertificateSigningRequest, Box<dyn Error>> {
    let mut req_name = X509NameBuilder::new()?;
    req_name.append_entry_by_text("C", country)?;
    if let Some(s) = state {
        req_name.append_entry_by_text("ST", s)?;
    }
    req_name.append_entry_by_text("L", city)?;
    req_name.append_entry_by_text("O", organization)?;
    req_name.append_entry_by_text("OU", department)?;
    req_name.append_entry_by_text("CN", url)?;

    let mut req_builder = X509ReqBuilder::new()?;
    req_builder.set_subject_name(req_name.build().as_ref())?;
    req_builder.sign(pkey, MessageDigest::sha256())?;

    Ok(CertificateSigningRequest {
        x509req: req_builder.build(),
    })
}

pub fn build_csr_der(cn: &str, key: &PrivateKey) -> Result<Vec<u8>, Box<dyn Error>> {
    let mut name_builder = X509NameBuilder::new()?;
    name_builder.append_entry_by_text("CN", cn)?;
    let name = name_builder.build();

    let pkey = &key.k;
    let mut req_builder = X509ReqBuilder::new()?;
    req_builder.set_subject_name(&name)?;
    req_builder.set_pubkey(pkey)?;
    req_builder.sign(pkey, MessageDigest::sha256())?;

    Ok(req_builder.build().to_der()?)
}
