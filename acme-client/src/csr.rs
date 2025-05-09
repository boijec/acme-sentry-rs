use openssl::hash::MessageDigest;
use openssl::pkey::{PKey, Private};
use openssl::x509::{X509NameBuilder, X509Req, X509ReqBuilder};

pub struct CertificateSigningRequest {
    x509req: X509Req
}

pub fn create_new_certificate_request(
    country: &str,
    state: Option<&str>,
    city: &str,
    organization: &str,
    department: &str,
    url: &str,
    pkey: &PKey<Private>
) -> Result<CertificateSigningRequest, Box<dyn std::error::Error>> {
    let mut req_name = X509NameBuilder::new()?;
    req_name.append_entry_by_text("C", &country)?;
    if let Some(s) = state {
        req_name.append_entry_by_text("ST", &s)?;
    }
    req_name.append_entry_by_text("L", &city)?;
    req_name.append_entry_by_text("O", &organization)?;
    req_name.append_entry_by_text("OU", &department)?;
    req_name.append_entry_by_text("CN", &url)?;

    let mut req_builder = X509ReqBuilder::new()?;

    req_builder.set_subject_name(req_name.build().as_ref())?;
    req_builder.sign(pkey, MessageDigest::sha256())?;

    Ok(CertificateSigningRequest {
        x509req: req_builder.build()
    })
}