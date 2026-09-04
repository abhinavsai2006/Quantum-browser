//! Encrypted Local Password Manager Vault

use argon2::{password_hash::SaltString, Argon2, PasswordHasher};
use qualium_crypto::QualiumAead;
use rand::rngs::OsRng;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;
use uuid::Uuid;
use zeroize::{Zeroize, ZeroizeOnDrop};

#[derive(Debug, Error)]
pub enum VaultError {
    #[error("Master password authentication failed")]
    AuthFailed,
    #[error("Vault error: {0}")]
    Generic(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PasswordEntry {
    pub id: Uuid,
    pub origin_url: String,
    pub username: String,
    pub encrypted_password: Vec<u8>,
    pub nonce: [u8; 12],
    pub created_at_epoch_sec: u64,
}

#[derive(Clone, Zeroize, ZeroizeOnDrop)]
pub struct MasterKey {
    pub key: [u8; 32],
}

pub struct EncryptedPasswordVault {
    salt: String,
    entries: HashMap<Uuid, PasswordEntry>,
}

impl EncryptedPasswordVault {
    pub fn new() -> Self {
        let salt = SaltString::generate(&mut OsRng).to_string();
        Self {
            salt,
            entries: HashMap::new(),
        }
    }

    /// Derive 256-bit encryption key from user's master password using Argon2id
    pub fn derive_master_key(master_password: &str, salt_str: &str) -> Result<MasterKey, VaultError> {
        let argon2 = Argon2::default();
        let salt = SaltString::from_b64(salt_str)
            .map_err(|e| VaultError::Generic(e.to_string()))?;
        
        let hash = argon2
            .hash_password(master_password.as_bytes(), &salt)
            .map_err(|e| VaultError::Generic(e.to_string()))?;

        let mut key = [0u8; 32];
        let hash_bytes = hash
            .hash
            .ok_or_else(|| VaultError::Generic("Argon2id output hash missing".into()))?;
        let copy_len = key.len().min(hash_bytes.len());
        key[..copy_len].copy_from_slice(&hash_bytes.as_bytes()[..copy_len]);

        Ok(MasterKey { key })
    }

    pub fn salt(&self) -> &str {
        &self.salt
    }

    /// Add a new encrypted password entry
    pub fn add_entry(
        &mut self,
        master_key: &MasterKey,
        origin_url: &str,
        username: &str,
        plaintext_password: &str,
    ) -> Result<Uuid, VaultError> {
        let mut nonce = [0u8; 12];
        use rand::RngCore;
        OsRng.fill_bytes(&mut nonce);

        let encrypted = QualiumAead::encrypt_aes_gcm(
            &master_key.key,
            &nonce,
            origin_url.as_bytes(),
            plaintext_password.as_bytes(),
        )
        .map_err(|e| VaultError::Generic(e.to_string()))?;

        let id = Uuid::new_v4();
        let entry = PasswordEntry {
            id,
            origin_url: origin_url.to_string(),
            username: username.to_string(),
            encrypted_password: encrypted,
            nonce,
            created_at_epoch_sec: 1724600000,
        };

        self.entries.insert(id, entry);
        Ok(id)
    }

    /// Retrieve and decrypt a password entry
    pub fn get_decrypted_password(
        &self,
        master_key: &MasterKey,
        entry_id: &Uuid,
    ) -> Result<String, VaultError> {
        let entry = self
            .entries
            .get(entry_id)
            .ok_or_else(|| VaultError::Generic("Entry not found".into()))?;

        let decrypted_bytes = QualiumAead::decrypt_aes_gcm(
            &master_key.key,
            &entry.nonce,
            entry.origin_url.as_bytes(),
            &entry.encrypted_password,
        )
        .map_err(|_| VaultError::AuthFailed)?;

        String::from_utf8(decrypted_bytes).map_err(|e| VaultError::Generic(e.to_string()))
    }
}

impl Default for EncryptedPasswordVault {
    fn default() -> Self {
        Self::new()
    }
}
