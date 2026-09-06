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
        let message = b"Qualium-Update-Manifest-v1.0.0-Release";

        let sig = QualiumSigner::sign(&sk, message).expect("sign");
        assert!(QualiumSigner::verify(&pk, message, &sig), "Valid signature must verify");

        // Verify with tampered message
        let tampered_msg = b"Qualium-Update-Manifest-v1.0.0-Malicious";
        assert!(!QualiumSigner::verify(&pk, tampered_msg, &sig), "Signature over tampered message must fail");
    }

    #[test]
    fn test_hybrid_handshake_authenticated_channel_payload_flow() {
        // Full integration flow:
        // Client initiate -> Server respond -> Client finalize -> Authenticated Channel (AEAD payload)
        let (client_state, offer) = HybridKeyExchange::client_initiate();
        let (response, server_keys) = HybridKeyExchange::server_respond(&offer).expect("server_respond");
        let client_keys = HybridKeyExchange::client_finalize(&client_state, &response).expect("client_finalize");

        // Verify shared transcript and session ID
        assert_eq!(client_keys.session_id, server_keys.session_id);
        assert_eq!(client_keys.tx_key, server_keys.rx_key);
        assert_eq!(client_keys.rx_key, server_keys.tx_key);

        // Client encrypts request payload with client tx_key
        let nonce = [0x42u8; 12];
        let aad = b"Qualium-Stream-Frame-Header-v5";
        let request_payload = b"GET / HTTP/1.1\r\nHost: qualium.is\r\n\r\n";
        let encrypted_frame = QualiumAead::encrypt_chacha(&client_keys.tx_key, &nonce, aad, request_payload)
            .expect("client encrypt");

        // Server decrypts request with server rx_key
        let server_decrypted = QualiumAead::decrypt_chacha(&server_keys.rx_key, &nonce, aad, &encrypted_frame)
            .expect("server decrypt");
        assert_eq!(request_payload.to_vec(), server_decrypted);

        // Server responds with payload encrypted with server tx_key
        let resp_nonce = [0x43u8; 12];
        let response_payload = b"HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\n\r\nAuthenticated";
        let server_frame = QualiumAead::encrypt_chacha(&server_keys.tx_key, &resp_nonce, aad, response_payload)
            .expect("server encrypt");

        // Client decrypts with client rx_key
        let client_decrypted = QualiumAead::decrypt_chacha(&client_keys.rx_key, &resp_nonce, aad, &server_frame)
            .expect("client decrypt");
        assert_eq!(response_payload.to_vec(), client_decrypted);
    }

    #[test]
    fn test_hybrid_handshake_tamper_rejection() {
        let (client_state, offer) = HybridKeyExchange::client_initiate();
        let (mut response, _server_keys) = HybridKeyExchange::server_respond(&offer).expect("server_respond");

        // MITM Attack 1: Tamper with ML-KEM ciphertext
        response.ml_kem_ciphertext.bytes[0] ^= 0xaa;
        // Under NIST FIPS 203 implicit rejection, client derives a distinct pseudorandom secret,
        // causing HKDF-derived session keys to NOT match the server's session keys.
        let corrupted_client_keys = HybridKeyExchange::client_finalize(&client_state, &response).expect("finalize");
        assert_ne!(
            corrupted_client_keys.session_id,
            _server_keys.session_id,
            "Corrupted ML-KEM ciphertext must NEVER derive matching session ID"
        );
        assert_ne!(
            corrupted_client_keys.tx_key,
            _server_keys.rx_key,
            "Corrupted ML-KEM ciphertext must NEVER derive matching session keys"
        );
    }

    #[test]
    fn test_hybrid_server_response_downgrade_rejection() {
        let (client_state, offer) = HybridKeyExchange::client_initiate();
        let (mut response, _server_keys) = HybridKeyExchange::server_respond(&offer).expect("server_respond");

        // MITM Attack 2: Attempt downgrade in server response selected version
        response.selected_version = "Insecure-Classical-TLS-v1.0".to_string();
        let finalize_res = HybridKeyExchange::client_finalize(&client_state, &response);
        assert!(
            finalize_res.is_err(),
            "Client must reject server response attempting protocol downgrade"
        );
    }
}
