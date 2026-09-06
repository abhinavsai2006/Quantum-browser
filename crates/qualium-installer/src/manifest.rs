//! Qualium Quantum Browser v5 — Installation Manifest Schema
//! Provides authoritative tracking of all installed files, shortcuts, and registry keys.

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManifestFileEntry {
    pub relative_path: String,
    pub size_bytes: u64,
    pub sha256: String,
    pub component: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstallManifest {
    pub product_name: String,
    pub version: String,
    pub publisher: String,
    pub install_dir: PathBuf,
    pub installed_at_utc: String,
    pub total_installed_bytes: u64,
    pub total_file_count: usize,
    pub files: Vec<ManifestFileEntry>,
    pub shortcuts: Vec<PathBuf>,
    pub registry_key: String,
    pub user_data_dir: PathBuf,
}

impl InstallManifest {
    pub fn new(install_dir: PathBuf, user_data_dir: PathBuf) -> Self {
        Self {
            product_name: "Qualium Quantum Browser".to_string(),
            version: "1.0.0".to_string(),
            publisher: "Qualium AI".to_string(),
            install_dir,
            installed_at_utc: chrono_stub_timestamp(),
            total_installed_bytes: 0,
            total_file_count: 0,
            files: Vec::new(),
            shortcuts: Vec::new(),
            registry_key: r"Software\Microsoft\Windows\CurrentVersion\Uninstall\QualiumQuantumBrowser".to_string(),
            user_data_dir,
        }
    }

    pub fn save_to_dir(&self, dir: &Path) -> anyhow::Result<PathBuf> {
        let path = dir.join("install-manifest.json");
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(&path, json)?;
        Ok(path)
    }

    pub fn load_from_dir(dir: &Path) -> anyhow::Result<Self> {
        let path = dir.join("install-manifest.json");
        let content = std::fs::read_to_string(&path)?;
        let manifest: Self = serde_json::from_str(&content)?;
        Ok(manifest)
    }
}

fn chrono_stub_timestamp() -> String {
    let now = std::time::SystemTime::now();
    let duration = now.duration_since(std::time::UNIX_EPOCH).unwrap_or_default();
    let secs = duration.as_secs();
    format!("{}-09-06T11:30:00Z", 1970 + secs / 31536000)
}
