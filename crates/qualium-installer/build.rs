use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=../../qualium/chrome");
    println!("cargo:rerun-if-changed=../../runtime/browser/omni.ja");
    println!("cargo:rerun-if-changed=../../dist/QualiumQuantumBrowser.exe");
    println!("cargo:rerun-if-changed=../../dist/qualium-daemon.exe");
    println!("cargo:rerun-if-changed=../../dist/QualiumUninstall.exe");
    println!("cargo:rerun-if-changed=../../target/release/qualium_uninstaller.exe");
    println!("cargo:rerun-if-changed=../../qualium.ico");

    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap_or_else(|_| ".".to_string()));
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".to_string()));
    let repo_root = manifest_dir
        .parent()
        .and_then(|p| p.parent())
        .unwrap_or(&manifest_dir);
    let payload_zip = out_dir.join("payload.zip");

    // Create a staging directory to package into payload.zip
    let stage_dir = out_dir.join("stage");
    let _ = fs::remove_dir_all(&stage_dir);
    let _ = fs::create_dir_all(&stage_dir);

    // 1. Copy QualiumQuantumBrowser.exe as the primary graphical desktop browser
    let browser_candidates = [
        repo_root.join("target").join("release").join("QualiumQuantumBrowser.exe"),
        repo_root.join("dist").join("QualiumQuantumBrowser.exe"),
        repo_root.join("target_build").join("release").join("QualiumQuantumBrowser.exe"),
        repo_root.join("QualiumQuantumBrowser.exe"),
        repo_root.join("QauliumQuantumBrowser.exe"),
    ];
    let target_browser_exe = stage_dir.join("QualiumQuantumBrowser.exe");
    let target_qaulium_exe = stage_dir.join("QauliumQuantumBrowser.exe");
    for cand in &browser_candidates {
        if cand.exists() {
            let _ = fs::copy(cand, &target_browser_exe);
            let _ = fs::copy(cand, &target_qaulium_exe);
            break;
        }
    }

    // 2. Copy qualium-daemon.exe as the separate background service binary
    let daemon_candidates = [
        repo_root.join("target").join("release").join("qualium-daemon.exe"),
        repo_root.join("target_build").join("release").join("qualium-daemon.exe"),
        repo_root.join("dist").join("qualium-daemon.exe"),
        repo_root.join("qualium-daemon.exe"),
    ];
    let target_daemon_exe = stage_dir.join("qualium-daemon.exe");
    for cand in &daemon_candidates {
        if cand.exists() {
            let _ = fs::copy(cand, &target_daemon_exe);
            break;
        }
    }

    // 3. Copy Gecko ESR 140 runtime
    let runtime_src = repo_root.join("runtime");
    let runtime_dst = stage_dir.join("runtime");
    if runtime_src.exists() {
        let _ = copy_dir_all(&runtime_src, &runtime_dst);
    }

    // 4. Copy chrome directory and manifests
    let chrome_src = repo_root.join("qualium").join("chrome");
    let chrome_dst = stage_dir.join("chrome");
    if chrome_src.exists() {
        let _ = copy_dir_all(&chrome_src, &chrome_dst);
        let runtime_chrome_dst = stage_dir.join("runtime").join("chrome");
        let _ = copy_dir_all(&chrome_src, &runtime_chrome_dst);
    }

    let manifest_src = repo_root.join("runtime").join("chrome.manifest");
    if manifest_src.exists() {
        let _ = fs::copy(&manifest_src, stage_dir.join("chrome.manifest"));
        let _ = fs::copy(&manifest_src, stage_dir.join("runtime").join("chrome.manifest"));
    }

    let app_ini_src = repo_root.join("dist").join("application.ini");
    if app_ini_src.exists() {
        let _ = fs::copy(&app_ini_src, stage_dir.join("application.ini"));
    }

    // 5. Copy qualium.ico icon
    let icon_candidates = [
        repo_root.join("qualium.ico"),
        repo_root.join("dist").join("qualium.ico"),
        repo_root.join("qualium").join("chrome").join("content").join("assets").join("icons").join("qualium.ico"),
    ];
    let res_dir = stage_dir.join("resources");
    let _ = fs::create_dir_all(&res_dir);
    for icon_cand in &icon_candidates {
        if icon_cand.exists() {
            let _ = fs::copy(icon_cand, stage_dir.join("qualium.ico"));
            let _ = fs::copy(icon_cand, res_dir.join("qualium.ico"));
            break;
        }
    }

    // 6. Copy QualiumUninstall.exe if available from previous builds or dist
    let uninstaller_candidates = [
        repo_root.join("target").join("release").join("qualium_uninstaller.exe"),
        repo_root.join("dist").join("QualiumUninstall.exe"),
        repo_root.join("target_build").join("release").join("qualium_uninstaller.exe"),
        repo_root.join("QualiumUninstall.exe"),
        repo_root.join("QauliumUninstall.exe"),
    ];
    let uninstall_sub_dir = stage_dir.join("uninstall");
    let _ = fs::create_dir_all(&uninstall_sub_dir);
    for uninst_cand in &uninstaller_candidates {
        if uninst_cand.exists() {
            let _ = fs::copy(uninst_cand, stage_dir.join("QauliumUninstall.exe"));
            let _ = fs::copy(uninst_cand, stage_dir.join("QualiumUninstall.exe"));
            let _ = fs::copy(uninst_cand, uninstall_sub_dir.join("QauliumUninstall.exe"));
            let _ = fs::copy(uninst_cand, uninstall_sub_dir.join("QualiumUninstall.exe"));
            break;
        }
    }

    // 7. Compress staging directory into payload.zip
    let _ = fs::remove_file(&payload_zip);
    let archive_base = out_dir.join("payload");
    let py_cmd = format!(
        "import shutil; shutil.make_archive(r'{}', 'zip', r'{}')",
        archive_base.display(),
        stage_dir.display()
    );

    let mut zip_created = false;
    for py_bin in ["py", "python3", "python"] {
        if let Ok(status) = Command::new(py_bin).args(["-c", &py_cmd]).status() {
            if status.success() && payload_zip.exists() {
                zip_created = true;
                break;
            }
        }
    }

    if !zip_created {
        // Fallback: create a valid standard empty ZIP structure if Python is not present
        let empty_zip: [u8; 22] = [
            0x50, 0x4B, 0x05, 0x06, // End of central directory signature (PK\x05\x06)
            0x00, 0x00, // Number of this disk
            0x00, 0x00, // Disk where central directory starts
            0x00, 0x00, // Number of central directory records on this disk
            0x00, 0x00, // Total number of central directory records
            0x00, 0x00, 0x00, 0x00, // Size of central directory
            0x00, 0x00, 0x00, 0x00, // Offset of start of central directory
            0x00, 0x00, // Comment length
        ];
        let _ = fs::write(&payload_zip, &empty_zip);
    }
}

fn copy_dir_all(src: &Path, dst: &Path) -> std::io::Result<()> {
    fs::create_dir_all(dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        if ty.is_dir() {
            let _ = copy_dir_all(&entry.path(), &dst.join(entry.file_name()));
        } else {
            let _ = fs::copy(entry.path(), dst.join(entry.file_name()));
        }
    }
    Ok(())
}
