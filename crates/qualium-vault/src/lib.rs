//! Qualium Local Encrypted Vault & Zero-Retention History Subsystems

pub mod history;
pub mod password;

pub use history::{EncryptedHistoryRecord, HistoryManager};
pub use password::{EncryptedPasswordVault, MasterKey, PasswordEntry, VaultError};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypted_password_vault_lifecycle() {
        let mut vault = EncryptedPasswordVault::new();
        let salt = vault.salt().to_string();
        let master_key =
            EncryptedPasswordVault::derive_master_key("Correct-Horse-Battery-Staple-2026", &salt)
                .expect("derive master key");

        let entry_id = vault
            .add_entry(
                &master_key,
                "https://accounts.qualium.ai",
                "alice@qualium.ai",
                "Super-Secure-PQC-Password!",
            )
            .expect("add entry");

        // Successful decryption with matching master key
        let decrypted = vault
            .get_decrypted_password(&master_key, &entry_id)
            .expect("get decrypted password");
        assert_eq!(decrypted, "Super-Secure-PQC-Password!");

        // Failed decryption with invalid master key
        let wrong_key = MasterKey { key: [0u8; 32] };
        let failed_res = vault.get_decrypted_password(&wrong_key, &entry_id);
        assert!(failed_res.is_err(), "Decryption with wrong key must fail");
    }

    #[test]
    fn test_zero_history_retention_by_default() {
        let mut history = HistoryManager::default();
        assert!(!history.is_enabled(), "Default history retention must be OFF");

        let vault_key = [0x5au8; 32];
        let record = history.record_visit(&vault_key, "https://example.com/sensitive", "Sensitive", "example.com");
        assert!(record.is_none(), "When history is disabled, nothing is stored");
    }
}
