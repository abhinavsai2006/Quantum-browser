//! Standardized Key Encapsulation Mechanisms (NIST FIPS 203 ML-KEM & RFC 7748 X25519)

use ml_kem::{
    kem::{Decapsulate, DecapsulationKey, Encapsulate, EncapsulationKey},
    Encoded, EncodedSizeUser, KemCore, MlKem768, MlKem768Params,
};
use rand::rngs::OsRng;
use thiserror::Error;
use x25519_dalek::{PublicKey as X25519PublicKey, StaticSecret};
use zeroize::{Zeroize, ZeroizeOnDrop};

#[derive(Debug, Error)]
pub enum CryptoError {
    #[error("Decapsulation failed")]
    DecapsulationFailed,
    #[error("Invalid key length or format: {0}")]
    InvalidKey(String),
    #[error("Encryption / Decryption error: {0}")]
    AeadError(String),
    #[error("Transcript authentication or binding mismatch")]
    TranscriptMismatch,
    #[error("Downgrade attempt detected: {0}")]
    DowngradeDetected(String),
}

/// Classical X25519 Key Pair and Shared Secret Exchange
pub struct X25519Exchange;

impl X25519Exchange {
    /// Generate an ephemeral keypair
    pub fn generate_keypair() -> (StaticSecret, X25519PublicKey) {
        let secret = StaticSecret::random_from_rng(OsRng);
        let public = X25519PublicKey::from(&secret);
        (secret, public)
    }

    /// Perform Diffie-Hellman exchange
    pub fn diffie_hellman(secret: &StaticSecret, peer_public: &X25519PublicKey) -> [u8; 32] {
        secret.diffie_hellman(peer_public).to_bytes()
    }
}

/// Post-Quantum ML-KEM-768 Encapsulation / Decapsulation (NIST FIPS 203)
pub struct MlKem768Engine;

#[derive(Clone, Zeroize, ZeroizeOnDrop)]
pub struct MlKem768PrivateKey {
    pub bytes: Vec<u8>,
}

#[derive(Clone)]
pub struct MlKem768PublicKey {
    pub bytes: Vec<u8>,
}

#[derive(Clone)]
pub struct MlKem768Ciphertext {
    pub bytes: Vec<u8>,
}

impl MlKem768Engine {
    /// Generate ML-KEM-768 Keypair
    pub fn generate_keypair() -> (MlKem768PrivateKey, MlKem768PublicKey) {
        let (dk, ek) = MlKem768::generate(&mut OsRng);
        (
            MlKem768PrivateKey {
                bytes: dk.as_bytes().to_vec(),
            },
            MlKem768PublicKey {
                bytes: ek.as_bytes().to_vec(),
            },
        )
    }

    /// Encapsulate shared secret against peer's public encapsulation key
    pub fn encapsulate(
        public_key: &MlKem768PublicKey,
    ) -> Result<([u8; 32], MlKem768Ciphertext), CryptoError> {
        let ek_encoded = Encoded::<EncapsulationKey<MlKem768Params>>::try_from(
            public_key.bytes.as_slice(),
        )
        .map_err(|_| CryptoError::InvalidKey("Invalid ML-KEM-768 public key bytes".into()))?;

        let enc_key = EncapsulationKey::<MlKem768Params>::from_bytes(&ek_encoded);
        let (ct, ss) = enc_key
            .encapsulate(&mut OsRng)
            .map_err(|_| CryptoError::DecapsulationFailed)?;

        let mut shared_secret = [0u8; 32];
        shared_secret.copy_from_slice(ss.as_slice());

        Ok((
            shared_secret,
            MlKem768Ciphertext {
                bytes: ct.as_slice().to_vec(),
            },
        ))
    }

    /// Decapsulate shared secret from ciphertext using private decapsulation key
    pub fn decapsulate(
        private_key: &MlKem768PrivateKey,
        ciphertext: &MlKem768Ciphertext,
    ) -> Result<[u8; 32], CryptoError> {
        let dk_bytes = Encoded::<DecapsulationKey<MlKem768Params>>::try_from(
            private_key.bytes.as_slice(),
        )
        .map_err(|_| CryptoError::InvalidKey("Invalid ML-KEM-768 private key bytes".into()))?;

        let dec_key = DecapsulationKey::<MlKem768Params>::from_bytes(&dk_bytes);

        let ct_bytes = ml_kem::Ciphertext::<MlKem768>::try_from(ciphertext.bytes.as_slice())
            .map_err(|_| CryptoError::InvalidKey("Invalid ML-KEM-768 ciphertext bytes".into()))?;

        let ss = dec_key
            .decapsulate(&ct_bytes)
            .map_err(|_| CryptoError::DecapsulationFailed)?;

        let mut shared_secret = [0u8; 32];
        shared_secret.copy_from_slice(ss.as_slice());
        Ok(shared_secret)
    }
}
