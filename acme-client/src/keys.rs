use crate::crypto::SupportedKey;
use openssl::ec::{EcGroup, EcKey};
use openssl::pkey::{PKey, Private};
use openssl::rsa::Rsa;
use openssl::ssl::Error;

pub struct PrivateKey {
    kt: SupportedKey,
    pub k: PKey<Private>,
}

impl PrivateKey {
    pub fn from_supported_type(key_type: SupportedKey) -> Result<PrivateKey, Error> {
        match key_type {
            SupportedKey::Rsa2048 => {
                Ok(PrivateKey {
                    kt: key_type,
                    k: gen_rsa(2048)?
                })
            },
            SupportedKey::Rsa4096 => Ok(PrivateKey {
                kt: key_type,
                k: gen_rsa(4096)?
            }),
            SupportedKey::EcP256 | SupportedKey::EcP384 | SupportedKey::EcP521 => Ok(PrivateKey {
                kt: key_type.clone(),
                k: gen_ec(&key_type)?
            }),
            SupportedKey::Ed25519 => Ok(PrivateKey {
                kt: key_type.clone(),
                k: gen_ed()?
            })
        }
    }

    pub fn is_type(&self, key_type: &SupportedKey) -> bool {
        self.kt.eq(key_type)
    }
}

pub(crate) fn gen_rsa(key_length: u32) -> Result<PKey<Private>, Error> {
    let rsa = Rsa::generate(key_length)?;
    Ok(PKey::from_rsa(rsa)?)
}

pub(crate) fn gen_ec(ec_type: &SupportedKey) -> Result<PKey<Private>, Error> {
    let ec = EcKey::generate(EcGroup::from_curve_name(ec_type.get_nid())?.as_ref())?;
    Ok(PKey::from_ec_key(ec)?)
}

pub(crate) fn gen_ed() -> Result<PKey<Private>, Error> {
    Ok(PKey::generate_ed25519()?)
}