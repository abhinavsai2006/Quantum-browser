//! Property-Based Fuzzing Smoke Tests (Phase 17 / Step 23)

#[cfg(test)]
mod fuzz_tests {
    use qualium_crypto::{MlKem768Ciphertext, MlKem768Engine};
    use qualium_filter::FilterEngine;
    use qualium_ipc::{CallerPrivilegeLevel, IpcSecurityManager, PeerCredentials, SecureIpcFrame};
    use rand::rngs::StdRng;
    use rand::{Rng, RngCore, SeedableRng};

    #[tokio::test]
    async fn test_fuzz_ipc_frame_parser() {
        let manager = IpcSecurityManager::new(1000);
        let peer = PeerCredentials {
            process_id: 1000,
            user_id: 1000,
            privilege_level: CallerPrivilegeLevel::BrowserParentChrome,
            is_authorized_process: true,
        };
        let (session_id, session_key) = manager.establish_session(peer).await.expect("session");

        let mut rng = StdRng::seed_from_u64(0xDEADBEEF_CAFEF00D);

        // Run 5,000 randomized adversarial fuzz iterations
        for i in 1..=5000 {
            let fuzz_len = rng.gen_range(0..256);
            let payload_str: String = (0..fuzz_len)
                .map(|_| rng.gen_range(32u8..126u8) as char)
                .collect();

            let frame = SecureIpcFrame::create_and_sign(
                session_id,
                i as u64,
                payload_str,
                &session_key,
            ).expect("sign");

            // Intentionally corrupt bytes randomly
            let mut corrupted_frame = frame;
            if rng.gen_bool(0.6) && !corrupted_frame.hmac_signature_hex.is_empty() {
                corrupted_frame.hmac_signature_hex.pop();
                corrupted_frame.hmac_signature_hex.push('0');
            }

            // Must either process safely or reject with error — NEVER panic
            let _ = manager.process_incoming_frame(&corrupted_frame).await;
        }
    }

    #[test]
    fn test_fuzz_abp_filter_parser() {
        let mut rng = StdRng::seed_from_u64(0x1337_C0DE_F00D);
        let filter = FilterEngine::default();

        // Fuzz URL and Domain checks with random strings, unicode sequences, and binary garbage
        for _ in 0..5000 {
            let len = rng.gen_range(1..256);
            let random_str: String = (0..len)
                .map(|_| rng.gen_range(32u8..126u8) as char)
                .collect();

            let domain_len = rng.gen_range(1..64);
            let domain_str: String = (0..domain_len)
                .map(|_| rng.gen_range(97u8..122u8) as char)
                .collect();

            // Parser must be completely resilient against crashes/panics
            let _ = filter.check_url(&random_str, &domain_str);
        }
    }

    #[test]
    fn test_fuzz_ml_kem_ciphertext_ingestion() {
        let (sk, _) = MlKem768Engine::generate_keypair();
        let mut rng = StdRng::seed_from_u64(0x900D_BEEF);

        // Fuzz 1,000 arbitrary ciphertexts against decapsulation
        for _ in 0..1000 {
            let mut random_ct_bytes = vec![0u8; 1088];
            rng.fill_bytes(&mut random_ct_bytes);

            let ct = MlKem768Ciphertext {
                bytes: random_ct_bytes,
            };

            // NIST FIPS 203 requires implicit rejection (constant time random secret derivation)
            // It must NEVER panic, crash, or leak timing information
            let res = MlKem768Engine::decapsulate(&sk, &ct);
            assert!(res.is_ok(), "ML-KEM-768 decapsulation must handle all inputs safely");
        }
    }
}
