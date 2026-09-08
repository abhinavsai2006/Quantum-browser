//! Qualium Quantum Browser v5 — Installer & Uninstaller Library
//! Provides Win32 native integration, installation engine, manifest tracking, and uninstaller engine.
//!
//! Windows-only modules (`win32`, `win32_gui`, `uninstaller_engine`) are gated with
//! `#[cfg(windows)]` so the crate can be included in the workspace without breaking
//! Linux/macOS builds. The CI additionally uses `--exclude qualium-installer` on those
//! platforms so the Windows-specific engine code is never attempted outside Windows.

// Cross-platform modules
pub mod manifest;
pub mod engine;

// Win32 bindings and GUI wizard
#[cfg(windows)]
pub mod win32;
#[cfg(not(windows))]
pub mod win32 {
    use std::path::{Path, PathBuf};

    pub mod sys {
        pub const CREATE_NO_WINDOW: u32 = 0;
    }

    pub fn get_disk_free_space(_path: &Path) -> Option<(u64, u64)> {
        None
    }

    pub fn get_running_qualium_processes() -> Vec<String> {
        Vec::new()
    }

    pub fn create_shortcut(
        _target_exe: &Path,
        _shortcut_path: &Path,
        _working_dir: &Path,
        _description: &str,
        _icon_path: Option<&Path>,
    ) -> anyhow::Result<()> {
        anyhow::bail!("Windows shortcut creation is only supported on Windows")
    }

    pub fn register_uninstall(
        _install_dir: &Path,
        _uninstaller_exe: &Path,
        _icon_path: &Path,
        _total_size_kb: u64,
    ) -> anyhow::Result<()> {
        Ok(())
    }

    pub fn unregister_uninstall() -> anyhow::Result<()> {
        Ok(())
    }

    pub fn get_default_install_dir() -> PathBuf {
        PathBuf::from("/opt/qualium")
    }

    pub fn get_user_data_dir() -> PathBuf {
        if let Ok(home) = std::env::var("HOME") {
            PathBuf::from(home).join(".qualium")
        } else {
            PathBuf::from("/tmp/.qualium")
        }
    }

    pub fn get_desktop_shortcut_paths() -> Vec<PathBuf> {
        Vec::new()
    }

    pub fn get_desktop_shortcut_path() -> Option<PathBuf> {
        None
    }

    pub fn get_start_menu_shortcut_dir() -> Option<PathBuf> {
        None
    }
}

#[cfg(windows)]
pub mod win32_gui;
#[cfg(not(windows))]
pub mod win32_gui {
    use crate::engine::InstallEngine;
    use crate::uninstaller_engine::UninstallerEngine;
    use std::path::PathBuf;

    pub fn run_installer_gui(_engine: InstallEngine, _default_dest: PathBuf) -> anyhow::Result<()> {
        anyhow::bail!("Windows GUI installer is only supported on Windows")
    }

    pub fn run_uninstaller_gui(_engine: UninstallerEngine, _install_dir: PathBuf) -> anyhow::Result<()> {
        anyhow::bail!("Windows GUI uninstaller is only supported on Windows")
    }
}

#[cfg(windows)]
pub mod uninstaller_engine;
#[cfg(not(windows))]
pub mod uninstaller_engine {
    pub struct UninstallerEngine;
}

pub use manifest::{InstallManifest, ManifestFileEntry};
pub use engine::{InstallEngine, InstallOptions, PayloadMetrics};
#[cfg(windows)]
pub use uninstaller_engine::UninstallerEngine;
