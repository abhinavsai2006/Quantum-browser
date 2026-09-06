//! Qualium Post-Quantum Cryptographic Engine
//!
//! Exposes standardized NIST FIPS 203 ML-KEM-768, RFC 7748 X25519,
//! Hybrid Key Establishment, and AEAD ciphers.

pub mod aead;
pub mod hybrid;
pub mod kem;
pub mod signature;

pub use aead::QualiumAead;
pub use hybrid::{
    HybridClientOffer, HybridClientState, HybridKeyExchange, HybridServerResponse,
    HybridSessionKeys,
};
pub use kem::{
    CryptoError, MlKem1024Ciphertext, MlKem1024Engine, MlKem1024PrivateKey, MlKem1024PublicKey,
    MlKem512Ciphertext, MlKem512Engine, MlKem512PrivateKey, MlKem512PublicKey,
    MlKem768Ciphertext, MlKem768Engine, MlKem768PrivateKey, MlKem768PublicKey,
    X25519Exchange,
};
pub use signature::{
    QualiumPublicKey, QualiumSecretKey, QualiumSignature, QualiumSigner,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_kem_512_roundtrip() {
        let (sk, pk) = MlKem512Engine::generate_keypair();
        let (ss1, ct) = MlKem512Engine::encapsulate(&pk).expect("encapsulation");
        let ss2 = MlKem512Engine::decapsulate(&sk, &ct).expect("decapsulation");
        assert_eq!(ss1, ss2, "Shared secrets from ML-KEM-512 must match");
    }

    #[test]
    fn test_kem_768_roundtrip() {
        let (sk, pk) = MlKem768Engine::generate_keypair();
        let (ss1, ct) = MlKem768Engine::encapsulate(&pk).expect("encapsulation");
        let ss2 = MlKem768Engine::decapsulate(&sk, &ct).expect("decapsulation");
        assert_eq!(ss1, ss2, "Shared secrets from ML-KEM-768 must match");
    }

    #[test]
    fn test_kem_1024_roundtrip() {
        let (sk, pk) = MlKem1024Engine::generate_keypair();
        let (ss1, ct) = MlKem1024Engine::encapsulate(&pk).expect("encapsulation");
        let ss2 = MlKem1024Engine::decapsulate(&sk, &ct).expect("decapsulation");
        assert_eq!(ss1, ss2, "Shared secrets from ML-KEM-1024 must match");
    }

    #[test]
    fn test_hybrid_handshake_roundtrip() {
        let (client_state, offer) = HybridKeyExchange::client_initiate();
        let (response, server_keys) =
            HybridKeyExchange::server_respond(&offer).expect("server respond");
        let client_keys =
            HybridKeyExchange::client_finalize(&client_state, &response).expect("client finalize");

        assert_eq!(
            client_keys.session_id, server_keys.session_id,
            "Session IDs must match"
        );
        assert_eq!(
            client_keys.tx_key, server_keys.rx_key,
            "Client TX must match Server RX"
        );
        assert_eq!(
            client_keys.rx_key, server_keys.tx_key,
            "Client RX must match Server TX"
        );

        // Test encryption over hybrid channel
        let msg = b"Qualium Quantum Browser Secure Frame Payload";
        let nonce = [7u8; 12];
        let aad = b"header-metadata-v5";
        let ct = QualiumAead::encrypt_chacha(&client_keys.tx_key, &nonce, aad, msg)
            .expect("encrypt");
        let pt = QualiumAead::decrypt_chacha(&server_keys.rx_key, &nonce, aad, &ct)
            .expect("decrypt");
        assert_eq!(msg.to_vec(), pt, "Decrypted message must match plaintext");
    }
}
