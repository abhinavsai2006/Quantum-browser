#![windows_subsystem = "windows"]

//! Qualium Quantum Browser v5 — Production Windows Uninstaller
//! Pure native Win32 implementation (Zero WinForms, Zero .NET).
//! Implements the complete uninstallation pipeline:
//! 1. Welcome & Running Qualium Process Detection
//! 2. User Data Options (Keep Personal Data vs Purge All Data)
//! 3. Ready to Remove
//! 4. Removing (Shortcut cleanup, Registry deregistration, Manifest deletion)
//! 5. Verification & Clean Self-Deletion

use qualium_installer_lib::uninstaller_engine::UninstallerEngine;
use qualium_installer_lib::win32;
use qualium_installer_lib::win32_gui;
use std::env;
use std::path::PathBuf;

fn get_app_dir() -> PathBuf {
    if let Ok(exe) = env::current_exe() {
        if let Some(parent) = exe.parent() {
            if parent.file_name().and_then(|n| n.to_str()) == Some("uninstall") {
                if let Some(grandparent) = parent.parent() {
                    return grandparent.to_path_buf();
                }
            }
            return parent.to_path_buf();
        }
    }
    win32::get_default_install_dir()
}

fn main() -> anyhow::Result<()> {
    let args: Vec<String> = env::args().collect();
    let is_silent = args.iter().any(|a| a == "/S" || a == "/silent" || a == "--silent");
    let install_dir = get_app_dir();
    let engine = UninstallerEngine::new(install_dir.clone());

    let purge_data = args.iter().any(|a| a == "/purge" || a == "--purge" || a == "/remove-data");
    let keep_data = !purge_data;

    if is_silent {
        let _ = engine.execute_uninstall(keep_data, |_, _, _, _| {});
        return Ok(());
    }

    win32_gui::run_uninstaller_gui(engine, install_dir)
}
