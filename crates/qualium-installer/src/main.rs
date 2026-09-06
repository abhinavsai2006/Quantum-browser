#![windows_subsystem = "windows"]

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

use qualium_installer_lib::engine::{InstallEngine, InstallOptions};
use qualium_installer_lib::win32;
use qualium_installer_lib::win32_gui;
use std::env;
use std::path::PathBuf;

const EMBEDDED_PAYLOAD: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/payload.zip"));

fn main() -> anyhow::Result<()> {
    let args: Vec<String> = env::args().collect();
    let is_silent = args.iter().any(|a| a == "/S" || a == "/silent" || a == "--silent");
    let dest_dir = get_destination_from_args(&args).unwrap_or_else(win32::get_default_install_dir);

    let engine = InstallEngine::new(EMBEDDED_PAYLOAD);

    if is_silent {
        let options = InstallOptions {
            install_dir: dest_dir,
            create_desktop_shortcut: true,
            create_start_menu_shortcut: true,
            launch_after_install: false,
            start_with_windows: false,
        };
        let _ = engine.install(&options, |_, _, _, _, _, _| {});
        return Ok(());
    }

    win32_gui::run_installer_gui(engine, dest_dir)
}

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
