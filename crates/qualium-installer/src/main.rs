#![cfg_attr(windows, windows_subsystem = "windows")]

//! Qualium Quantum Browser v5 — Production Windows Installer
//! Pure native Win32 implementation (Zero WinForms, Zero .NET).
//! Implements the complete 8-step installation pipeline:
//! 1. Welcome
//! 2. License & Notices
//! 3. Install Location (Dynamic disk space validation)
//! 4. Installation Options (Shortcuts, Launch, Startup)
//! 5. Ready to Install
//! 6. Installing (Live progress and component reporting)
//! 7. Verification (Critical file verification & SHA256)
//! 8. Complete & Launch

#[cfg(windows)]
use qualium_installer_lib::engine::{InstallEngine, InstallOptions};
#[cfg(windows)]
use qualium_installer_lib::win32;
#[cfg(windows)]
use qualium_installer_lib::win32_gui;
#[cfg(windows)]
use std::env;
#[cfg(windows)]
use std::path::PathBuf;

#[cfg(windows)]
const EMBEDDED_PAYLOAD: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/payload.zip"));

#[cfg(windows)]
fn main() -> anyhow::Result<()> {
    let args: Vec<String> = env::args().collect();
    let is_silent = args.iter().any(|a| a == "/S" || a == "/silent" || a == "--silent");
    let is_portable = args.iter().any(|a| a == "/PORTABLE" || a == "/portable" || a == "--portable" || a == "-p");
    let dest_dir = get_destination_from_args(&args).unwrap_or_else(|| {
        if is_portable {
            env::current_exe()
                .ok()
                .and_then(|p| p.parent().map(|dir| dir.join("QuantumBrowser")))
                .unwrap_or_else(|| PathBuf::from("QuantumBrowser"))
        } else {
            win32::get_default_install_dir()
        }
    });

    let engine = InstallEngine::new(EMBEDDED_PAYLOAD);

    if is_silent {
        let options = InstallOptions {
            install_dir: dest_dir,
            create_desktop_shortcut: false,
            create_start_menu_shortcut: !is_portable,
            launch_after_install: false,
            start_with_windows: false,
            is_portable,
        };
        match engine.install(&options, |_, _, _, _, _, _| {}) {
            Ok(_) => return Ok(()),
            Err(e) => {
                let _ = std::fs::write(std::env::temp_dir().join("qualium_install_error.log"), format!("Install error: {:#}", e));
                return Err(e);
            }
        }
    }

    win32_gui::run_installer_gui(engine, dest_dir)
}

#[cfg(windows)]
fn get_destination_from_args(args: &[String]) -> Option<PathBuf> {
    for i in 1..args.len() {
        if (args[i] == "-d" || args[i] == "--dir" || args[i] == "/D") && i + 1 < args.len() {
            return Some(PathBuf::from(&args[i + 1]));
        }
        if args[i].starts_with("/D=") {
            return Some(PathBuf::from(&args[i][3..]));
        }
    }
    None
}

#[cfg(not(windows))]
fn main() -> anyhow::Result<()> {
    anyhow::bail!("qualium_installer is only supported on Windows")
}
