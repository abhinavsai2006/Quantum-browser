//! Zero-Retention History Engine & Optional Encrypted History Store

use qualium_crypto::QualiumAead;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedHistoryRecord {
    pub record_id: Uuid,
    pub timestamp_epoch_sec: u64,
    pub domain: String,
    pub encrypted_url_and_title: Vec<u8>,
    pub nonce: [u8; 12],
}

pub struct HistoryManager {
    is_enabled: bool,
    ephemeral_buffer: VecDeque<EncryptedHistoryRecord>,
}

impl HistoryManager {
    pub fn new(is_enabled: bool) -> Self {
        Self {
            is_enabled,
            ephemeral_buffer: VecDeque::new(),
        }
    }

    /// Record a visit if and only if local history is explicitly enabled by the user
    pub fn record_visit(
        &mut self,
        vault_key: &[u8; 32],
        url: &str,
        title: &str,
        domain: &str,
    ) -> Option<Uuid> {
        if !self.is_enabled {
            // Principle of zero retention: Discard immediately
            return None;
        }

        let mut nonce = [0u8; 12];
        use rand::{rngs::OsRng, RngCore};
        OsRng.fill_bytes(&mut nonce);

        let plaintext_payload = serde_json::json!({
            "url": url,
            "title": title,
        })
        .to_string();

        let encrypted = QualiumAead::encrypt_aes_gcm(
            vault_key,
            &nonce,
            domain.as_bytes(),
            plaintext_payload.as_bytes(),
        )
        .ok()?;

        let record_id = Uuid::new_v4();
        let record = EncryptedHistoryRecord {
            record_id,
            timestamp_epoch_sec: 1724600000,
            domain: domain.to_string(),
            encrypted_url_and_title: encrypted,
            nonce,
        };

        self.ephemeral_buffer.push_back(record);
        Some(record_id)
    }

    /// Clear all history records securely on Identity Reset or Session Exit
    pub fn purge_all(&mut self) {
        self.ephemeral_buffer.clear();
    }

    pub fn is_enabled(&self) -> bool {
        self.is_enabled
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.is_enabled = enabled;
        if !enabled {
            self.purge_all();
        }
    }
}

impl Default for HistoryManager {
    fn default() -> Self {
        // DEFAULT IS OFF (Zero Retention)
        Self::new(false)
    }
}
