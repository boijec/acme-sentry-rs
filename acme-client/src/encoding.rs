use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};

/// Encode a byte slice to a base64 string.
pub fn encode_b64(input: &[u8]) -> String {
    URL_SAFE_NO_PAD.encode(input)
}

/// Decode a base64 string to a byte vector.
pub fn decode_b64<T: AsRef<[u8]> + ?Sized>(input: &T) -> Result<Vec<u8>, base64::DecodeError> {
    URL_SAFE_NO_PAD.decode(input)
}