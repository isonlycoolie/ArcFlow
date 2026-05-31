//! HMAC-SHA256 verification for external callbacks.

use hmac::{Hmac, Mac};
use sha2::Sha256;
use subtle::ConstantTimeEq;

type HmacSha256 = Hmac<Sha256>;

/// Computes hex-encoded HMAC-SHA256 of `body` with `secret`.
pub fn compute_hmac_sha256_hex(secret: &str, body: &[u8]) -> String {
    let mut mac =
        HmacSha256::new_from_slice(secret.as_bytes()).expect("HMAC accepts any key length");
    mac.update(body);
    hex::encode(mac.finalize().into_bytes())
}

/// Verifies `X-ArcFlow-Signature` header value against body.
/// Accepts `sha256=<hex>` or raw hex.
pub fn verify_webhook_signature(secret: &str, body: &[u8], signature_header: &str) -> bool {
    let expected = compute_hmac_sha256_hex(secret, body);
    let provided = signature_header
        .trim()
        .strip_prefix("sha256=")
        .unwrap_or(signature_header.trim());
    if provided.len() != expected.len() {
        return false;
    }
    provided.as_bytes().ct_eq(expected.as_bytes()).into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip_signature() {
        let body = br#"{"binding_id":"b","status":"success"}"#;
        let sig = format!("sha256={}", compute_hmac_sha256_hex("secret", body));
        assert!(verify_webhook_signature("secret", body, &sig));
        assert!(!verify_webhook_signature("wrong", body, &sig));
    }
}
