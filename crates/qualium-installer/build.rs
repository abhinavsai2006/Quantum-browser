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

    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let repo_root = manifest_dir.parent().unwrap().parent().unwrap();
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
        PathBuf::from(r"C:\Users\mndab\AppData\Local\Temp\qualium_target\release\QualiumQuantumBrowser.exe"),
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
    copy_dir_all(&chrome_src, &chrome_dst).unwrap();
    let runtime_chrome_dst = stage_dir.join("runtime").join("chrome");
    copy_dir_all(&chrome_src, &runtime_chrome_dst).unwrap();

    let manifest_src = repo_root.join("runtime").join("chrome.manifest");
    if manifest_src.exists() {
        fs::copy(&manifest_src, stage_dir.join("chrome.manifest")).unwrap();
        fs::copy(&manifest_src, stage_dir.join("runtime").join("chrome.manifest")).unwrap();
    }

    let app_ini_src = repo_root.join("dist").join("application.ini");
    if app_ini_src.exists() {
        fs::copy(&app_ini_src, stage_dir.join("application.ini")).unwrap();
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
        PathBuf::from(r"C:\Users\mndab\AppData\Local\Temp\qualium_target\release\qualium_uninstaller.exe"),
    ];
    let uninstall_sub_dir = stage_dir.join("uninstall");
    let _ = fs::create_dir_all(&uninstall_sub_dir);
    for uninst_cand in &uninstaller_candidates {
        if uninst_cand.exists() {
            let _ = fs::copy(uninst_cand, stage_dir.join("QualiumUninstall.exe"));
            let _ = fs::copy(uninst_cand, uninstall_sub_dir.join("QualiumUninstall.exe"));
            break;
        }
    }

    // 3. Compress staging directory into payload.zip using Python shutil.make_archive
    let _ = fs::remove_file(&payload_zip);
    let archive_base = out_dir.join("payload");
    let py_cmd = format!(
        "import shutil; shutil.make_archive(r'{}', 'zip', r'{}')",
        archive_base.display(),
        stage_dir.display()
    );

    let python = if cfg!(windows) { "py" } else { "python3" };
    let status = Command::new(python)
        .args(["-c", &py_cmd])
        .status()
        .unwrap();

    if !status.success() || !payload_zip.exists() {
        panic!("Failed to generate payload.zip in build.rs via Python shutil.make_archive");
    }
}

fn copy_dir_all(src: &Path, dst: &Path) -> std::io::Result<()> {
    fs::create_dir_all(dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        if ty.is_dir() {
            copy_dir_all(&entry.path(), &dst.join(entry.file_name()))?;
        } else {
            fs::copy(entry.path(), dst.join(entry.file_name()))?;
        }
    }
    Ok(())
}
