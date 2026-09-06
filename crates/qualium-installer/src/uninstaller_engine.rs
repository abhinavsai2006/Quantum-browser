//! Qualium Quantum Browser v5 — Uninstaller Engine
//! Authoritative removal based on install-manifest.json, Windows registry cleanup, and user data choices.
//!
//! This module is Windows-only: it drives the Win32 registry, shortcut removal,
//! and batch-script self-deletion that only apply on Windows.
#![cfg(windows)]

use crate::manifest::InstallManifest;
use crate::win32;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

pub struct UninstallerEngine {
    install_dir: PathBuf,
}

impl UninstallerEngine {
    pub fn new(install_dir: PathBuf) -> Self {
        Self { install_dir }
    }

    /// Check if any Qualium processes are active
    pub fn check_running_processes(&self) -> Vec<String> {
        win32::get_running_qualium_processes()
    }

    /// Execute the uninstallation workflow
    pub fn execute_uninstall<F>(
        &self,
        keep_personal_data: bool,
        mut on_progress: F,
    ) -> anyhow::Result<()>
    where
        F: FnMut(usize, usize, &str, &str),
    {
        // 1. Load manifest if present
        let manifest = InstallManifest::load_from_dir(&self.install_dir).ok();

        // 2. Remove shortcuts
        on_progress(1, 6, "Removing Windows desktop and Start Menu shortcuts...", "Shortcuts");
        if let Some(ref m) = manifest {
            for sc in &m.shortcuts {
                if sc.exists() {
                    let _ = fs::remove_file(sc);
                }
            }
        }
        // Fallback checks for standard shortcut locations
        if let Some(desktop_lnk) = win32::get_desktop_shortcut_path() {
            if desktop_lnk.exists() {
                let _ = fs::remove_file(&desktop_lnk);
            }
        }
        if let Some(userprofile) = std::env::var("USERPROFILE").ok() {
            let alt_desktop = PathBuf::from(userprofile).join("Desktop").join("Qaulium Quantum Browser.lnk");
            if alt_desktop.exists() {
                let _ = fs::remove_file(&alt_desktop);
            }
        }
        if let Some(start_dir) = win32::get_start_menu_shortcut_dir() {
            if start_dir.exists() {
                let _ = fs::remove_dir_all(&start_dir);
            }
        }

        // 3. Remove Windows Uninstall Registry Registration
        on_progress(2, 6, "Cleaning Windows Installed Apps registry entries...", "Registry");
        let _ = win32::unregister_uninstall();

        // 4. Remove application files listed in manifest or install directory
        on_progress(3, 6, "Removing Qualium runtime and application components...", "Application Files");
        if let Some(ref m) = manifest {
            for entry in &m.files {
                let p = self.install_dir.join(&entry.relative_path);
                if p.exists() {
                    let _ = fs::remove_file(p);
                }
            }
        }

        // Remove known application subdirectories
        let subdirs = ["runtime", "chrome", "resources", "uninstall"];
        for sub in &subdirs {
            let p = self.install_dir.join(sub);
            if p.exists() {
                let _ = fs::remove_dir_all(p);
            }
        }

        // Remove top-level application binaries and logs
        let binaries = [
            "QualiumQuantumBrowser.exe",
            "QauliumQuantumBrowser.exe",
            "qualium-daemon.exe",
            "chrome.manifest",
            "application.ini",
            "install-manifest.json",
        ];
        for b in &binaries {
            let p = self.install_dir.join(b);
            if p.exists() {
                let _ = fs::remove_file(p);
            }
        }

        // 5. Handle user data choice
        on_progress(4, 6, "Processing user data selection...", "User Data");
        if !keep_personal_data {
            let data_dirs = [
                win32::get_user_data_dir(),
                if let Ok(l) = std::env::var("LOCALAPPDATA") {
                    PathBuf::from(l).join("Qaulium")
                } else {
                    PathBuf::from(r"C:\Qaulium")
                },
            ];
            for dd in &data_dirs {
                if dd.exists() {
                    let _ = fs::remove_dir_all(dd);
                }
            }
        }

        // 6. Verify removal
        on_progress(5, 6, "Verifying uninstallation completeness...", "Verification");
        let main_exe = self.install_dir.join("QualiumQuantumBrowser.exe");
        if main_exe.exists() {
            anyhow::bail!("Failed to remove executable: {}", main_exe.display());
        }

        // 7. Schedule self-deletion of uninstaller executable & parent dir
        on_progress(6, 6, "Scheduling cleanup of uninstaller binaries...", "Cleanup");
        self.schedule_self_delete()?;

        Ok(())
    }

    /// Schedule self-deletion via detached cmd process
    fn schedule_self_delete(&self) -> anyhow::Result<()> {
        let current_exe = std::env::current_exe()?;
        let install_dir = self.install_dir.clone();

        // Small batch script in %TEMP% to delete the uninstaller and the install folder after process exits
        let temp_dir = std::env::temp_dir();
        let cleanup_bat = temp_dir.join(format!("qualium_cleanup_{}.bat", std::process::id()));
        let bat_content = format!(
            r#"@echo off
ping 127.0.0.1 -n 2 > nul
del /f /q "{}" > nul 2>&1
rd /s /q "{}" > nul 2>&1
del /f /q "%~f0" > nul 2>&1
"#,
            current_exe.display(),
            install_dir.display()
        );

        fs::write(&cleanup_bat, bat_content)?;

        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            let _ = Command::new("cmd.exe")
                .args(["/c", &cleanup_bat.to_string_lossy()])
                .creation_flags(win32::sys::CREATE_NO_WINDOW)
                .spawn();
        }

        Ok(())
    }
}
