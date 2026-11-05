use crate::error::CommonsSslError;
use openssl::bn::{BigNum, BigNumContext};

pub fn big_num_from_vec(bytes: &Vec<u8>) -> Result<BigNum, CommonsSslError> {
    BigNum::from_slice(bytes)
        .map_err(|e| CommonsSslError::BigNumFromSliceError(e, String::from_utf8_lossy(bytes).to_string()))
}
pub fn big_num_from_slice(bytes: &[u8]) -> Result<BigNum, CommonsSslError> {
    BigNum::from_slice(bytes)
        .map_err(|e| CommonsSslError::BigNumFromSliceError(e, String::from_utf8_lossy(bytes).to_string()))
}
pub fn get_big_num() -> Result<BigNum, CommonsSslError> {
    BigNum::new()
        .map_err(|e| CommonsSslError::BigNumError(e))
}
pub fn get_big_num_context() -> Result<BigNumContext, CommonsSslError> {
    BigNumContext::new()
        .map_err(|e| CommonsSslError::BigNumContextError(e))
}