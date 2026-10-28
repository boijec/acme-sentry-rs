use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use crate::error::EncodingError;

/// Encode a byte slice to a base64 string.
pub fn encode_b64(input: &[u8]) -> String {
    URL_SAFE_NO_PAD.encode(input)
}

/// Decode a base64 string to a byte vector.
pub fn decode_b64<T: AsRef<[u8]> + ?Sized>(input: &T) -> Result<Vec<u8>, EncodingError> {
    URL_SAFE_NO_PAD.decode(input)
        .map_err(EncodingError::from)
}

pub fn remove_last(value: &str) -> &str {
    let mut chars = value.chars();
    chars.next_back();
    chars.as_str()
}

pub fn remove_first(value: &str) -> &str {
    let mut chars = value.chars();
    chars.next();
    chars.as_str()
}

pub fn count_occurrences(value: &str, character: char) -> usize {
    value.chars().filter(|c| *c == character).count()
}