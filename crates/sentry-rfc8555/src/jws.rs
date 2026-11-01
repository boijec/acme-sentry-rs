use crate::error::DefaultOperationsError;
use sentry_crypto::encoding::encode_b64;
use sentry_jose::jws::{Jws, JwsHeader};
use sentry_jose::keys::PrivateKey;
use serde_json::{Value, json};
use tracing::{info, warn};

pub(crate) fn try_validate_jws_response(body: &Value) {
    let is_flat_jws = body.get("protected").is_some()
        && body.get("payload").is_some()
        && body.get("signature").is_some();

    if is_flat_jws {
        match Jws::parse_flat(body) {
            Ok(Some(_)) => {
                info!("CA response validated as a signed flat JWS successfully.");
            }
            Ok(None) => {}
            Err(e) => {
                warn!(
                    "CA response looks like a flat JWS but validation failed: {}",
                    e
                );
            }
        }
    }
}

pub(crate) fn build_jws_body_jwk(
    protected: Value,
    payload: &Value,
    key: &PrivateKey,
) -> Result<Value, DefaultOperationsError> {
    let header = JwsHeader::with_alg(key.kt.get_key_alg());
    let protected_b64 = encode_b64(protected.to_string().as_bytes());
    let payload_b64 = encode_b64(payload.to_string().as_bytes());
    let signing_input = format!("{}.{}", protected_b64, payload_b64);
    let signature = key
        .sign(&header, &signing_input)
        .map_err(|e| DefaultOperationsError::JswSignatureFailed(e))?;
    Ok(json!({
        "protected": protected_b64,
        "payload":   payload_b64,
        "signature": encode_b64(&signature),
    }))
}

pub(crate) fn build_jws_body_kid(
    protected: Value,
    payload: &Value,
    key: &PrivateKey,
) -> Result<Value, DefaultOperationsError> {
    let header = JwsHeader::with_alg(key.kt.get_key_alg());
    let protected_b64 = encode_b64(protected.to_string().as_bytes());
    let payload_b64 = encode_b64(payload.to_string().as_bytes());
    let signing_input = format!("{}.{}", protected_b64, payload_b64);
    let signature = key
        .sign(&header, &signing_input)
        .map_err(|e| DefaultOperationsError::JswSignatureFailed(e))?;
    Ok(json!({
        "protected": protected_b64,
        "payload":   payload_b64,
        "signature": encode_b64(&signature),
    }))
}

pub(crate) fn build_jws_body_kid_post_as_get(
    protected: Value,
    key: &PrivateKey,
) -> Result<Value, DefaultOperationsError> {
    let header = JwsHeader::with_alg(key.kt.get_key_alg());
    let protected_b64 = encode_b64(protected.to_string().as_bytes());
    // POST-as-GET: payload is the empty string, not even a JSON null
    let payload_b64 = encode_b64(b"");
    let signing_input = format!("{}.{}", protected_b64, payload_b64);
    let signature = key
        .sign(&header, &signing_input)
        .map_err(|e| DefaultOperationsError::JswSignatureFailed(e))?;
    Ok(json!({
        "protected": protected_b64,
        "payload":   payload_b64,
        "signature": encode_b64(&signature),
    }))
}
