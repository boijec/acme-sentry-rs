use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};

/// Encode a byte slice to a base64 string.
pub fn b64_encode<T: AsRef<[u8]>>(input: &T) -> String {
    URL_SAFE_NO_PAD.encode(input)
}

/// Decode a base64 string to a byte vector.
pub fn b64_decode<T: AsRef<[u8]> + ?Sized>(input: &T) -> Result<Vec<u8>, base64::DecodeError> {
    URL_SAFE_NO_PAD.decode(input)
}