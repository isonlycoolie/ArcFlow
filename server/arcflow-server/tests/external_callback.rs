//! External callback HMAC and idempotency unit tests.

#[cfg(test)]
mod external_callback {
    use arcflow_core::external::{compute_hmac_sha256_hex, verify_webhook_signature};

    #[test]
    fn idempotency_key_format_is_stable() {
        let run_id = "run-1";
        let binding_id = "gov_portal";
        let key = "idem-abc";
        let composite = format!("{run_id}:{binding_id}:{key}");
        assert_eq!(composite, "run-1:gov_portal:idem-abc");
    }

    #[test]
    fn webhook_signature_accepts_sha256_prefix() {
        let body = br#"{"binding_id":"b","status":"success"}"#;
        let secret = "test-secret";
        let sig = format!("sha256={}", compute_hmac_sha256_hex(secret, body));
        assert!(verify_webhook_signature(secret, body, &sig));
    }

    #[test]
    fn webhook_rejects_tampered_body() {
        let body = br#"{"binding_id":"b","status":"success"}"#;
        let secret = "test-secret";
        let sig = format!("sha256={}", compute_hmac_sha256_hex(secret, body));
        let tampered = br#"{"binding_id":"b","status":"failed"}"#;
        assert!(!verify_webhook_signature(secret, tampered, &sig));
    }
}
