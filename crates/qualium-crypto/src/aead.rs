//! Authenticated Encryption with Associated Data (AEAD)
//!
//! Provides ChaCha20-Poly1305 (RFC 8439) and AES-256-GCM authenticated cipher operations.

use crate::kem::CryptoError;
use aes_gcm::{Aes256Gcm, KeyInit, Nonce as AesNonce};
use chacha20poly1305::{
    aead::{Aead, Payload},
    ChaCha20Poly1305, Nonce,
};

pub struct QualiumAead;

impl QualiumAead {
    /// Encrypt plaintext using ChaCha20-Poly1305
    pub fn encrypt_chacha(
        key: &[u8; 32],
        nonce_12b: &[u8; 12],
        aad: &[u8],
        plaintext: &[u8],
    ) -> Result<Vec<u8>, CryptoError> {
        let cipher = ChaCha20Poly1305::new_from_slice(key)
            .map_err(|e| CryptoError::AeadError(e.to_string()))?;
        let nonce = Nonce::from_slice(nonce_12b);
        let payload = Payload {
            msg: plaintext,
            aad,
        };
        cipher
            .encrypt(nonce, payload)
            .map_err(|e| CryptoError::AeadError(e.to_string()))
    }

    /// Decrypt ciphertext using ChaCha20-Poly1305
    pub fn decrypt_chacha(
        key: &[u8; 32],
        nonce_12b: &[u8; 12],
        aad: &[u8],
        ciphertext: &[u8],
    ) -> Result<Vec<u8>, CryptoError> {
        let cipher = ChaCha20Poly1305::new_from_slice(key)
            .map_err(|e| CryptoError::AeadError(e.to_string()))?;
        let nonce = Nonce::from_slice(nonce_12b);
        let payload = Payload {
            msg: ciphertext,
            aad,
        };
        cipher
            .decrypt(nonce, payload)
            .map_err(|e| CryptoError::AeadError(e.to_string()))
    }

    /// Encrypt plaintext using AES-256-GCM
    pub fn encrypt_aes_gcm(
        key: &[u8; 32],
        nonce_12b: &[u8; 12],
        aad: &[u8],
        plaintext: &[u8],
    ) -> Result<Vec<u8>, CryptoError> {
        let cipher = Aes256Gcm::new_from_slice(key)
            .map_err(|e| CryptoError::AeadError(e.to_string()))?;
        let nonce = AesNonce::from_slice(nonce_12b);
        let payload = aes_gcm::aead::Payload {
            msg: plaintext,
            aad,
        };
        cipher
            .encrypt(nonce, payload)
            .map_err(|e| CryptoError::AeadError(e.to_string()))
    }

    /// Decrypt ciphertext using AES-256-GCM
    pub fn decrypt_aes_gcm(
        key: &[u8; 32],
        nonce_12b: &[u8; 12],
        aad: &[u8],
        ciphertext: &[u8],
    ) -> Result<Vec<u8>, CryptoError> {
        let cipher = Aes256Gcm::new_from_slice(key)
            .map_err(|e| CryptoError::AeadError(e.to_string()))?;
        let nonce = AesNonce::from_slice(nonce_12b);
        let payload = aes_gcm::aead::Payload {
            msg: ciphertext,
            aad,
        };
        cipher
            .decrypt(nonce, payload)
            .map_err(|e| CryptoError::AeadError(e.to_string()))
    }
}
