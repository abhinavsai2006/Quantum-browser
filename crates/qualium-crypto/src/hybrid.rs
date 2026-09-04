//! Hybrid Post-Quantum Key Establishment (X25519 + ML-KEM-768 + HKDF-SHA384)
//!
//! # Standards Track & Interoperability Context
//! Implements TLS 1.3 X25519MLKEM768 interoperability using the currently applicable
//! IETF standards-track specification (active Internet-Draft `draft-ietf-tls-hybrid-design` /
//! `draft-kwiatkowski-tls-ecdhe-mlkem`) and native Rust cryptography.
//!
//! Note on Cryptographic Transplantation:
//! The IETF draft explicitly notes that hybrid security analysis depends on the TLS 1.3
//! transcript binding and key schedule. The same hybridization cannot be assumed secure when
//! transplanted into other protocols without formal transcript-binding proofs.
//!
//! # Construction Specification:
//! 1. Initiator generates X25519 ephemeral keypair and ML-KEM-768 keypair.
//! 2. Responder encapsulates to ML-KEM public key and performs ECDH with X25519.
//! 3. Shared secret derived via HKDF-Extract(Salt, ss_classical || ss_pq)
//! 4. HKDF-Expand with domain separated transcript context:
//!    Context = "Qualium-PQ-v5.0||HybridKEM||ML-KEM-768||X25519||" || SHA384(pk_c || pk_pq || ct_c || ct_pq)

use crate::kem::{CryptoError, MlKem768Ciphertext, MlKem768Engine, MlKem768PrivateKey, MlKem768PublicKey, X25519Exchange};
use hkdf::Hkdf;
use sha2::{Digest, Sha384};
use x25519_dalek::{PublicKey as X25519PublicKey, StaticSecret};
use zeroize::{Zeroize, ZeroizeOnDrop};

const PROTOCOL_DOMAIN_LABEL: &[u8] = b"Qualium-PQ-v5.0::HybridKEM::X25519::ML-KEM-768";

#[derive(Clone, Zeroize, ZeroizeOnDrop)]
pub struct HybridClientState {
    #[zeroize(skip)]
    pub x25519_secret: StaticSecret,
    pub ml_kem_private: MlKem768PrivateKey,
    #[zeroize(skip)]
    pub client_x25519_public: X25519PublicKey,
    #[zeroize(skip)]
    pub client_ml_kem_public: MlKem768PublicKey,
}

#[derive(Clone)]
pub struct HybridClientOffer {
    pub client_x25519_public: X25519PublicKey,
    pub client_ml_kem_public: MlKem768PublicKey,
    pub supported_versions: Vec<String>,
}

#[derive(Clone)]
pub struct HybridServerResponse {
    pub server_x25519_public: X25519PublicKey,
    pub ml_kem_ciphertext: MlKem768Ciphertext,
    pub selected_version: String,
}

#[derive(Zeroize, ZeroizeOnDrop)]
pub struct HybridSessionKeys {
    pub tx_key: [u8; 32],
    pub rx_key: [u8; 32],
    pub session_id: [u8; 32],
}

pub struct HybridKeyExchange;

impl HybridKeyExchange {
    pub const SUPPORTED_PROTOCOL_VERSION: &'static str = "Qualium-PQ-v5.0";
    pub const IETF_DRAFT_HYBRID_GROUP: &'static str = "X25519MLKEM768 (draft-ietf-tls-hybrid-design)";

    /// Check if peer offers compatible hybrid post-quantum protocol support
    pub fn is_protocol_version_supported(version: &str) -> bool {
        version == Self::SUPPORTED_PROTOCOL_VERSION
    }

    /// Step 1: Client initiates the hybrid handshake offer
    pub fn client_initiate() -> (HybridClientState, HybridClientOffer) {
        let (x_sec, x_pub) = X25519Exchange::generate_keypair();
        let (kem_sec, kem_pub) = MlKem768Engine::generate_keypair();

        let state = HybridClientState {
            x25519_secret: x_sec,
            ml_kem_private: kem_sec,
            client_x25519_public: x_pub,
            client_ml_kem_public: kem_pub.clone(),
        };

        let offer = HybridClientOffer {
            client_x25519_public: x_pub,
            client_ml_kem_public: kem_pub,
            supported_versions: vec!["Qualium-PQ-v5.0".into()],
        };

        (state, offer)
    }

    /// Step 2: Server accepts offer, performs DH + KEM encapsulation, and derives session keys
    pub fn server_respond(
        offer: &HybridClientOffer,
    ) -> Result<(HybridServerResponse, HybridSessionKeys), CryptoError> {
        if !offer.supported_versions.contains(&"Qualium-PQ-v5.0".to_string()) {
            return Err(CryptoError::DowngradeDetected(
                "Version negotiation failed or attempted insecure downgrade".into(),
            ));
        }

        // Generate server X25519 ephemeral keypair
        let (server_x_sec, server_x_pub) = X25519Exchange::generate_keypair();

        // Perform classical Diffie-Hellman
        let ss_classical = X25519Exchange::diffie_hellman(&server_x_sec, &offer.client_x25519_public);

        // Perform post-quantum ML-KEM-768 encapsulation
        let (ss_pq, kem_ct) = MlKem768Engine::encapsulate(&offer.client_ml_kem_public)?;

        // Derive hybrid keys with transcript binding
        let session_keys = Self::derive_session_keys(
            &ss_classical,
            &ss_pq,
            &offer.client_x25519_public,
            &offer.client_ml_kem_public,
            &server_x_pub,
            &kem_ct,
            true, // is_server
        )?;

        let response = HybridServerResponse {
            server_x25519_public: server_x_pub,
            ml_kem_ciphertext: kem_ct,
            selected_version: "Qualium-PQ-v5.0".into(),
        };

        Ok((response, session_keys))
    }

    /// Step 3: Client receives server response, decapsulates ML-KEM, and derives matching session keys
    pub fn client_finalize(
        state: &HybridClientState,
        response: &HybridServerResponse,
    ) -> Result<HybridSessionKeys, CryptoError> {
        if response.selected_version != "Qualium-PQ-v5.0" {
            return Err(CryptoError::DowngradeDetected(
                "Unexpected protocol version in server response".into(),
            ));
        }

        // Classical DH
        let ss_classical =
            X25519Exchange::diffie_hellman(&state.x25519_secret, &response.server_x25519_public);

        // PQ Decapsulation
        let ss_pq =
            MlKem768Engine::decapsulate(&state.ml_kem_private, &response.ml_kem_ciphertext)?;

        // Derive keys
        Self::derive_session_keys(
            &ss_classical,
            &ss_pq,
            &state.client_x25519_public,
            &state.client_ml_kem_public,
            &response.server_x25519_public,
            &response.ml_kem_ciphertext,
            false, // is_client
        )
    }

    /// Transcript-bound HKDF derivation combining both secrets
    fn derive_session_keys(
        ss_classical: &[u8; 32],
        ss_pq: &[u8; 32],
        client_x_pub: &X25519PublicKey,
        client_kem_pub: &MlKem768PublicKey,
        server_x_pub: &X25519PublicKey,
        server_kem_ct: &MlKem768Ciphertext,
        is_server: bool,
    ) -> Result<HybridSessionKeys, CryptoError> {
        // Hash the transcript components for binding
        let mut hasher = Sha384::new();
        hasher.update(PROTOCOL_DOMAIN_LABEL);
        hasher.update(client_x_pub.as_bytes());
        hasher.update(&client_kem_pub.bytes);
        hasher.update(server_x_pub.as_bytes());
        hasher.update(&server_kem_ct.bytes);
        let transcript_hash = hasher.finalize();

        // Concatenate classical and PQ secrets
        let mut combined_secret = [0u8; 64];
        combined_secret[0..32].copy_from_slice(ss_classical);
        combined_secret[32..64].copy_from_slice(ss_pq);

        // HKDF-Extract using fixed salt and combined shared secret
        let salt = b"Qualium-v5-Session-Key-Salt-2026";
        let hkdf = Hkdf::<Sha384>::new(Some(salt), &combined_secret);

        let mut key_material = [0u8; 96]; // 32 tx + 32 rx + 32 session_id
        hkdf.expand(&transcript_hash, &mut key_material)
            .map_err(|_| CryptoError::AeadError("HKDF Expand failed".into()))?;

        let mut key_c2s = [0u8; 32];
        let mut key_s2c = [0u8; 32];
        let mut session_id = [0u8; 32];

        key_c2s.copy_from_slice(&key_material[0..32]);
        key_s2c.copy_from_slice(&key_material[32..64]);
        session_id.copy_from_slice(&key_material[64..96]);

        let (tx_key, rx_key) = if is_server {
            (key_s2c, key_c2s)
        } else {
            (key_c2s, key_s2c)
        };

        Ok(HybridSessionKeys {
            tx_key,
            rx_key,
            session_id,
        })
    }
}
