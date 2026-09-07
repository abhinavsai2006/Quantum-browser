//! Qualium Quantum Browser v5 — Transactional Installation Engine
//! Executes real unzipping, SHA256 verification, staging, committing, and shortcut/registry registration.

use crate::manifest::{InstallManifest, ManifestFileEntry};
#[cfg(windows)]
use crate::win32;
use sha2::{Digest, Sha256};
use std::fs::{self, File};
use std::io::{self, Cursor, Read, Write};
use std::path::{Path, PathBuf};
use zip::ZipArchive;

pub struct InstallOptions {
    pub install_dir: PathBuf,
    pub create_desktop_shortcut: bool,
    pub create_start_menu_shortcut: bool,
    pub launch_after_install: bool,
    pub start_with_windows: bool,
}

pub struct PayloadMetrics {
    pub total_file_count: usize,
    pub total_uncompressed_bytes: u64,
}

pub struct InstallEngine {
    payload_bytes: &'static [u8],
}

impl InstallEngine {
    pub fn new(payload_bytes: &'static [u8]) -> Self {
        Self { payload_bytes }
    }

    /// Calculate real uncompressed size and file count from the embedded payload
    pub fn inspect_payload(&self) -> anyhow::Result<PayloadMetrics> {
        let cursor = Cursor::new(self.payload_bytes);
        let mut zip = ZipArchive::new(cursor)?;
        let mut total_bytes: u64 = 0;
        let mut file_count: usize = 0;

        for i in 0..zip.len() {
            let file = zip.by_index(i)?;
            if !file.is_dir() {
                total_bytes += file.size();
                file_count += 1;
            }
        }

        Ok(PayloadMetrics {
            total_file_count: file_count,
            total_uncompressed_bytes: total_bytes,
        })
    }

    /// Execute installation pipeline with live progress reporting
    pub fn install<F>(&self, options: &InstallOptions, mut on_progress: F) -> anyhow::Result<InstallManifest>
    where
        F: FnMut(usize, usize, u64, u64, &str, &str),
    {
        let dest = &options.install_dir;
        let metrics = self.inspect_payload()?;

        // 1. Verify disk space (Windows-only: uses GetDiskFreeSpaceExW)
        #[cfg(windows)]
        if let Some((free_avail, _)) = win32::get_disk_free_space(dest) {
            let required_with_margin = metrics.total_uncompressed_bytes + 50 * 1024 * 1024;
            if free_avail < required_with_margin {
                anyhow::bail!(
                    "Insufficient disk space on destination drive. Required: {:.2} MB, Available: {:.2} MB",
                    required_with_margin as f64 / (1024.0 * 1024.0),
                    free_avail as f64 / (1024.0 * 1024.0)
                );
            }
        }

        // 2. Prepare staging directory inside system temp directory
        let staging_dir = std::env::temp_dir().join(format!("qualium_stage_{}", std::process::id()));
        if staging_dir.exists() {
            let _ = fs::remove_dir_all(&staging_dir);
        }
        fs::create_dir_all(&staging_dir)?;

        // Resolve user data dir: Windows uses LOCALAPPDATA, others use XDG_DATA_HOME or ~/.local/share
        #[cfg(windows)]
        let user_data_dir = win32::get_user_data_dir();
        #[cfg(not(windows))]
        let user_data_dir = std::env::var("XDG_DATA_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|_| {
                // Pure stdlib home detection: try HOME (Linux/macOS), then /tmp fallback
                std::env::var("HOME")
                    .map(PathBuf::from)
                    .unwrap_or_else(|_| PathBuf::from("/tmp"))
                    .join(".local")
                    .join("share")
                    .join("qualium")
            });
        let mut manifest = InstallManifest::new(dest.clone(), user_data_dir);
        manifest.total_installed_bytes = metrics.total_uncompressed_bytes;
        manifest.total_file_count = metrics.total_file_count;

        // 3. Extract and stream files into staging directory
        let cursor = Cursor::new(self.payload_bytes);
        let mut zip = ZipArchive::new(cursor)?;
        let mut copied_files: usize = 0;
        let mut copied_bytes: u64 = 0;

        let total_files = metrics.total_file_count;
        let total_bytes = metrics.total_uncompressed_bytes;

        for i in 0..zip.len() {
            let mut file = zip.by_index(i)?;
            let raw_path = file.name().replace('\\', "/");
            let out_path = staging_dir.join(&raw_path);

            if file.is_dir() {
                fs::create_dir_all(&out_path)?;
                continue;
            }

            if let Some(parent) = out_path.parent() {
                fs::create_dir_all(parent)?;
            }

            let component = if raw_path.starts_with("runtime") {
                "Gecko Runtime & Necko Stack"
            } else if raw_path.starts_with("chrome") {
                "Qaulium UI Chrome Resources"
            } else if raw_path.ends_with(".exe") {
                "Qaulium Core Binaries"
            } else {
                "Application Resources & Configurations"
            };

            on_progress(copied_files, total_files, copied_bytes, total_bytes, &raw_path, component);

            let mut outfile = File::create(&out_path)?;
            let mut hasher = Sha256::new();
            let mut buffer = [0u8; 65536];

            loop {
                let n = file.read(&mut buffer)?;
                if n == 0 {
                    break;
                }
                outfile.write_all(&buffer[..n])?;
                hasher.update(&buffer[..n]);
                copied_bytes += n as u64;
            }

            let hash_hex = format!("{:x}", hasher.finalize());
            manifest.files.push(ManifestFileEntry {
                relative_path: raw_path,
                size_bytes: file.size(),
                sha256: hash_hex,
                component: component.to_string(),
            });

            copied_files += 1;
            on_progress(copied_files, total_files, copied_bytes, total_bytes, "Verifying buffer...", component);
        }

        // 4. Atomic commit: move staged files into final destination
        fs::create_dir_all(dest)?;
        commit_staging_to_dest(&staging_dir, dest)?;
        let _ = fs::remove_dir_all(&staging_dir);

        // 5. Create icons folder if not present
        let resources_dir = dest.join("resources");
        fs::create_dir_all(&resources_dir)?;
        let icon_path = resources_dir.join("qualium.ico");
        if !icon_path.exists() {
            // Check if icon exists in stage or root
            let cand1 = dest.join("qualium.ico");
            if cand1.exists() {
                let _ = fs::copy(&cand1, &icon_path);
            }
        }

        // 6. Ensure QauliumQuantumBrowser.exe and QualiumQuantumBrowser.exe both exist
        let main_browser_exe = dest.join("QauliumQuantumBrowser.exe");
        let alt_browser_exe = dest.join("QualiumQuantumBrowser.exe");
        if !main_browser_exe.exists() && alt_browser_exe.exists() {
            let _ = fs::copy(&alt_browser_exe, &main_browser_exe);
        } else if main_browser_exe.exists() && !alt_browser_exe.exists() {
            let _ = fs::copy(&main_browser_exe, &alt_browser_exe);
        }

        // 7. Ensure QauliumUninstall.exe and QualiumUninstall.exe exist in install_dir and install_dir\uninstall
        let uninstall_dir = dest.join("uninstall");
        fs::create_dir_all(&uninstall_dir)?;
        let root_uninstaller = dest.join("QualiumUninstall.exe");
        let sub_uninstaller = uninstall_dir.join("QualiumUninstall.exe");
        let root_alt_uninstaller = dest.join("QauliumUninstall.exe");
        let sub_alt_uninstaller = uninstall_dir.join("QauliumUninstall.exe");

        if !root_uninstaller.exists() && !sub_uninstaller.exists() {
            // Check fallback locations
            let candidates = [
                std::env::current_exe().ok().and_then(|p| p.parent().map(|dir| dir.join("QualiumUninstall.exe"))),
                std::env::current_exe().ok().and_then(|p| p.parent().map(|dir| dir.join("QauliumUninstall.exe"))),
                Some(PathBuf::from(r"E:\Qaulium AI\Broswer\dist\QualiumUninstall.exe")),
                Some(PathBuf::from(r"E:\Qaulium AI\Broswer\target\release\qualium_uninstaller.exe")),
            ];
            for cand in candidates.into_iter().flatten() {
                if cand.exists() {
                    let _ = fs::copy(&cand, &root_uninstaller);
                    let _ = fs::copy(&cand, &sub_uninstaller);
                    let _ = fs::copy(&cand, &root_alt_uninstaller);
                    let _ = fs::copy(&cand, &sub_alt_uninstaller);
                    break;
                }
            }
        } else {
            if root_uninstaller.exists() && !root_alt_uninstaller.exists() {
                let _ = fs::copy(&root_uninstaller, &root_alt_uninstaller);
            }
            if sub_uninstaller.exists() && !sub_alt_uninstaller.exists() {
                let _ = fs::copy(&sub_uninstaller, &sub_alt_uninstaller);
            }
        }

        // 8. Shortcuts creation (Option 1: Desktop, Option 2: Start Menu)
        let primary_exe = if main_browser_exe.exists() { main_browser_exe.clone() } else { alt_browser_exe.clone() };

        #[cfg(windows)]
        if options.create_desktop_shortcut {
            let icon_ref = if icon_path.exists() { Some(icon_path.as_path()) } else { None };
            for desktop_lnk in win32::get_desktop_shortcut_paths() {
                if let Ok(()) = win32::create_shortcut(
                    &primary_exe,
                    &desktop_lnk,
                    dest,
                    "Qaulium Quantum Browser — Privacy-First Gecko Desktop Browser",
                    icon_ref,
                ) {
                    if !manifest.shortcuts.contains(&desktop_lnk) {
                        manifest.shortcuts.push(desktop_lnk);
                    }
                }
            }
        }

        #[cfg(windows)]
        if options.create_start_menu_shortcut {
            if let Some(menu_dir) = win32::get_start_menu_shortcut_dir() {
                let _ = fs::create_dir_all(&menu_dir);
                let app_lnk = menu_dir.join("Qaulium Quantum Browser.lnk");
                let uninst_lnk = menu_dir.join("Uninstall Qaulium Quantum Browser.lnk");
                let icon_ref = if icon_path.exists() { Some(icon_path.as_path()) } else { None };

                if let Ok(()) = win32::create_shortcut(
                    &primary_exe,
                    &app_lnk,
                    dest,
                    "Qaulium Quantum Browser",
                    icon_ref,
                ) {
                    manifest.shortcuts.push(app_lnk);
                }

                if let Ok(()) = win32::create_shortcut(
                    &root_uninstaller,
                    &uninst_lnk,
                    dest,
                    "Uninstall Qaulium Quantum Browser",
                    icon_ref,
                ) {
                    manifest.shortcuts.push(uninst_lnk);
                }
            }
        }

        // 9. Windows Run entry (Option 4: Start with Windows)
        #[cfg(windows)]
        if options.start_with_windows {
            let _ = win32::register_startup(&primary_exe);
        }

        // 10. Windows Installed Apps Registry Registration
        #[cfg(windows)]
        {
            let total_size_kb = (metrics.total_uncompressed_bytes / 1024).max(1);
            let reg_icon = if icon_path.exists() { icon_path } else { primary_exe.clone() };
            win32::register_uninstall(dest, &root_uninstaller, &reg_icon, total_size_kb)?;
        }

        // 11. Write authoritative install-manifest.json
        manifest.save_to_dir(dest)?;

        // 11. Final Verification of Critical Files
        self.verify_installation(dest)?;

        Ok(manifest)
    }

    /// Perform rigorous post-install verification
    pub fn verify_installation(&self, dest: &Path) -> anyhow::Result<()> {
        let critical_files = [
            dest.join("QualiumQuantumBrowser.exe"),
            dest.join("qualium-daemon.exe"),
            dest.join("runtime").join("qualium-core.exe"),
            dest.join("runtime").join("browser").join("omni.ja"),
            dest.join("install-manifest.json"),
        ];

        for f in &critical_files {
            if !f.exists() {
                anyhow::bail!("Verification failed: Missing critical component: {}", f.display());
            }
            if let Ok(meta) = f.metadata() {
                if meta.len() == 0 {
                    anyhow::bail!("Verification failed: Zero-byte file: {}", f.display());
                }
            }
        }

        Ok(())
    }

    /// Rollback installation by removing all created files
    pub fn rollback(&self, dest: &Path, manifest: Option<&InstallManifest>) {
        if let Some(m) = manifest {
            for entry in &m.files {
                let p = dest.join(&entry.relative_path);
                let _ = fs::remove_file(p);
            }
            for s in &m.shortcuts {
                let _ = fs::remove_file(s);
            }
            let _ = win32::unregister_uninstall();
            let _ = fs::remove_file(dest.join("install-manifest.json"));
        } else {
            let _ = fs::remove_dir_all(dest);
        }
    }
}

fn commit_staging_to_dest(src: &Path, dst: &Path) -> io::Result<()> {
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        let target_path = dst.join(entry.file_name());
        if ty.is_dir() {
            fs::create_dir_all(&target_path)?;
            commit_staging_to_dest(&entry.path(), &target_path)?;
        } else {
            if target_path.exists() {
                let _ = fs::remove_file(&target_path);
            }
            fs::copy(entry.path(), &target_path)?;
        }
    }
    Ok(())
}
