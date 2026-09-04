//! Cryptography Verification Suite: KATs & Hybrid Handshake Tests

#[cfg(test)]
mod tests {
    use qualium_crypto::{
        HybridKeyExchange, MlKem768Engine, QualiumAead,
    };

    #[test]
    fn test_ml_kem_768_known_answer_simulation() {
        let (sk, pk) = MlKem768Engine::generate_keypair();
        assert_eq!(pk.bytes.len(), 1184, "ML-KEM-768 public key size must be 1184 bytes");
        assert_eq!(sk.bytes.len(), 2400, "ML-KEM-768 private key size must be 2400 bytes");

        let (ss, ct) = MlKem768Engine::encapsulate(&pk).expect("encapsulate");
        assert_eq!(ct.bytes.len(), 1088, "ML-KEM-768 ciphertext size must be 1088 bytes");
        assert_eq!(ss.len(), 32, "Shared secret must be 32 bytes");

        let recovered_ss = MlKem768Engine::decapsulate(&sk, &ct).expect("decapsulate");
        assert_eq!(ss, recovered_ss, "Decapsulated secret must match");
    }

    #[test]
    fn test_hybrid_handshake_transcript_integrity() {
        let (client_state, offer) = HybridKeyExchange::client_initiate();
        let (response, server_keys) =
            HybridKeyExchange::server_respond(&offer).expect("server_respond");
        let client_keys =
            HybridKeyExchange::client_finalize(&client_state, &response).expect("client_finalize");

        assert_eq!(client_keys.session_id, server_keys.session_id);
        assert_eq!(client_keys.tx_key, server_keys.rx_key);
        assert_eq!(client_keys.rx_key, server_keys.tx_key);
    }

    #[test]
    fn test_hybrid_downgrade_resistance() {
        let (_client_state, mut offer) = HybridKeyExchange::client_initiate();
        // Attacker alters supported versions to downgrade
        offer.supported_versions = vec!["Insecure-Classical-v1.0".to_string()];

        let server_res = HybridKeyExchange::server_respond(&offer);
        assert!(
            server_res.is_err(),
            "Server must reject downgraded or unsupported protocol offers"
        );
    }

    #[test]
    fn test_aead_authenticated_transport() {
        let key = [0x33u8; 32];
        let nonce = [0x11u8; 12];
        let aad = b"Qualium-Frame-Control-v5";
        let plaintext = b"Confidential Browsing Session Payload";

        let ciphertext =
            QualiumAead::encrypt_chacha(&key, &nonce, aad, plaintext).expect("encrypt");
        let decrypted =
            QualiumAead::decrypt_chacha(&key, &nonce, aad, &ciphertext).expect("decrypt");

        assert_eq!(plaintext.to_vec(), decrypted);

        // Tamper with ciphertext
        let mut tampered = ciphertext.clone();
        tampered[0] ^= 0xff;
        let tamper_res = QualiumAead::decrypt_chacha(&key, &nonce, aad, &tampered);
        assert!(tamper_res.is_err(), "Tampered ciphertext must be rejected");
    }

    #[test]
    fn test_ml_kem_ciphertext_tamper_rejection() {
        let (sk, pk) = MlKem768Engine::generate_keypair();
        let (_ss, mut ct) = MlKem768Engine::encapsulate(&pk).expect("encapsulate");

        // Corrupt first byte of ciphertext
        ct.bytes[0] ^= 0x55;

        // NIST FIPS 203 ML-KEM implicit rejection: decapsulating a corrupted ciphertext
        // produces a pseudorandom secret that does not equal the valid shared secret.
        let bogus_ss = MlKem768Engine::decapsulate(&sk, &ct).expect("implicit decapsulation");
        assert_ne!(_ss, bogus_ss, "Decapsulating corrupted ciphertext must NOT match honest shared secret");
    }

    #[test]
    fn test_signature_verification_lifecycle() {
        use qualium_crypto::QualiumSigner;
        let (sk, pk) = QualiumSigner::generate_keypair();
        let message = b"Qualium-Update-Manifest-v5.0.0-Release";

        let sig = QualiumSigner::sign(&sk, message).expect("sign");
        assert!(QualiumSigner::verify(&pk, message, &sig), "Valid signature must verify");

        // Verify with tampered message
        let tampered_msg = b"Qualium-Update-Manifest-v5.0.0-Malicious";
        assert!(!QualiumSigner::verify(&pk, tampered_msg, &sig), "Signature over tampered message must fail");
    }
}
