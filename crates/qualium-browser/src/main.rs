#![windows_subsystem = "windows"]

//! Qualium Quantum Browser v1 — Native Gecko Desktop Browser Host
//!
//! Architecture:
//! QualiumQuantumBrowser.exe
//!         |
//!         +--> native Gecko/Firefox ESR 140 runtime
//!         |        |
//!         |        +--> native browser chrome (chrome://qualium/content/browser.xhtml)
//!         |        +--> native browsing context & docshell
//!         |        +--> Gecko multi-process content processes (plugin-container.exe)
//!         |        +--> Necko network stack (routed to local SOCKS5 proxy)
//!         |        +--> actual web destinations (Google, YouTube, GitHub, etc.)
//!         |
//!         +--> qualium-daemon.exe (Post-Quantum ML-KEM-768 Security Proxy on 127.0.0.1:9050)

use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Child, Command};

#[cfg(windows)]
use std::os::windows::process::CommandExt;

const CREATE_NO_WINDOW: u32 = 0x08000000;
const EMBEDDED_USER_CHROME: &str = include_str!("../../../qualium/chrome/userChrome.css");

fn append_boot_log(msg: &str) {
    use std::io::Write;
    let log_file = env::temp_dir().join("qualium_boot.log");
    if let Ok(mut f) = fs::OpenOptions::new().create(true).append(true).open(log_file) {
        let _ = writeln!(f, "[pid={}] {}", std::process::id(), msg);
    }
}

fn get_app_dir() -> PathBuf {
    if let Ok(current_exe) = env::current_exe() {
        if let Some(parent) = current_exe.parent() {
            // Check macOS bundle structure: Qualium Quantum Browser.app/Contents/MacOS/../Resources
            if parent.ends_with("MacOS") {
                let resources = parent.parent().map(|p| p.join("Resources")).unwrap_or_default();
                if resources.exists() {
                    return resources;
                }
            }
            if parent.join("runtime").exists() || parent.join("qualium-core.exe").exists() || parent.join("qualium-core").exists() {
                return parent.to_path_buf();
            }
        }
    }
    #[cfg(windows)]
    {
        if let Ok(local_appdata) = env::var("LOCALAPPDATA") {
            let p_qua = PathBuf::from(&local_appdata).join("Programs").join("Qualium");
            if p_qua.exists() {
                return p_qua;
            }
            let p_qau = PathBuf::from(&local_appdata).join("Programs").join("Qaulium");
            if p_qau.exists() {
                return p_qau;
            }
        }
    }
    #[cfg(target_os = "macos")]
    {
        let mac_app = PathBuf::from("/Applications/Qualium Quantum Browser.app/Contents/Resources");
        if mac_app.exists() {
            return mac_app;
        }
    }
    #[cfg(target_os = "linux")]
    {
        let opt_app = PathBuf::from("/opt/qualium-quantum-browser");
        if opt_app.exists() {
            return opt_app;
        }
    }
    if let Ok(current_exe) = env::current_exe() {
        if let Some(parent) = current_exe.parent() {
            return parent.to_path_buf();
        }
    }
    PathBuf::from(".")
}

fn get_profile_dir() -> PathBuf {
    #[cfg(windows)]
    {
        if let Ok(local_appdata) = env::var("LOCALAPPDATA") {
            let p_qau = PathBuf::from(&local_appdata).join("Qaulium").join("Profile");
            let p_qua = PathBuf::from(&local_appdata).join("Qualium").join("Profile");
            if p_qau.exists() {
                return p_qau;
            } else if p_qua.exists() {
                return p_qua;
            }
            let _ = fs::create_dir_all(&p_qua);
            return p_qua;
        }
    }
    #[cfg(target_os = "macos")]
    {
        if let Ok(home) = env::var("HOME") {
            let p_mac = PathBuf::from(&home).join("Library").join("Application Support").join("Qualium").join("Profile");
            let _ = fs::create_dir_all(&p_mac);
            return p_mac;
        }
    }
    #[cfg(target_os = "linux")]
    {
        if let Ok(xdg_config) = env::var("XDG_CONFIG_HOME") {
            let p_xdg = PathBuf::from(&xdg_config).join("qualium").join("profile");
            let _ = fs::create_dir_all(&p_xdg);
            return p_xdg;
        } else if let Ok(home) = env::var("HOME") {
            let p_lin = PathBuf::from(&home).join(".config").join("qualium").join("profile");
            let _ = fs::create_dir_all(&p_lin);
            return p_lin;
        }
    }
    let p = env::temp_dir().join("qualium_profile");
    let _ = fs::create_dir_all(&p);
    p
}

fn wait_for_port_ready(port: u16, max_wait_ms: u64) -> bool {
    if port == 0 {
        return false;
    }
    let start = std::time::Instant::now();
    let addr = std::net::SocketAddr::from(([127, 0, 0, 1], port));
    while start.elapsed().as_millis() < max_wait_ms as u128 {
        if std::net::TcpStream::connect_timeout(&addr, std::time::Duration::from_millis(50)).is_ok() {
            return true;
        }
        std::thread::sleep(std::time::Duration::from_millis(50));
    }
    false
}

fn discover_active_proxy(profile_dir: &Path, max_wait_ms: u64) -> Option<u16> {
    let start = std::time::Instant::now();
    let state_file = profile_dir.join("qualium_security_state.json");
    let fallback_state = env::temp_dir().join("qualium_security_state.json");

    while start.elapsed().as_millis() < max_wait_ms as u128 {
        for path in [&state_file, &fallback_state] {
            if let Ok(content) = fs::read_to_string(path) {
                if let Ok(val) = serde_json::from_str::<serde_json::Value>(&content) {
                    if val.get("ready").and_then(|v| v.as_bool()).unwrap_or(false) {
                        if let Some(port) = val.get("proxyPort").and_then(|v| v.as_u64()) {
                            let p = port as u16;
                            if wait_for_port_ready(p, 150) {
                                return Some(p);
                            }
                        }
                    }
                }
            }
        }
        std::thread::sleep(std::time::Duration::from_millis(50));
    }
    None
}

fn configure_profile(app_dir: &Path, profile_dir: &Path, proxy_port: Option<u16>) {
    let proxy_config = if let Some(port) = proxy_port {
        format!(
            r#"user_pref("network.proxy.type", 1);
user_pref("network.proxy.socks", "127.0.0.1");
user_pref("network.proxy.socks_port", {});
user_pref("network.proxy.socks_version", 5);
user_pref("network.proxy.socks_remote_dns", true);
user_pref("network.proxy.no_proxies_on", "");
user_pref("network.trr.mode", 5);"#,
            port
        )
    } else {
        r#"user_pref("network.proxy.type", 0);"#.to_string()
    };

    let user_js_content = format!(
        r#"// Qualium Quantum Browser v1 — Profile Configuration
{}
user_pref("app.update.enabled", false);
user_pref("app.update.auto", false);
user_pref("datareporting.policy.dataSubmissionEnabled", false);
user_pref("datareporting.healthreport.uploadEnabled", false);
user_pref("toolkit.telemetry.enabled", false);
user_pref("toolkit.telemetry.unified", false);
user_pref("browser.startup.page", 1);
user_pref("browser.startup.homepage", "chrome://qualium/content/newtab.xhtml");
user_pref("browser.newtabpage.enabled", false);
user_pref("browser.tabs.firefox-view", false);
user_pref("browser.theme.toolbar-theme", 0);
user_pref("extensions.activeThemeID", "firefox-compact-dark@mozilla.org");
user_pref("toolkit.legacyUserProfileCustomizations.stylesheets", true);
user_pref("identity.fxaccounts.enabled", false);
user_pref("browser.onboarding.enabled", false);
user_pref("browser.uitour.enabled", false);
user_pref("browser.aboutwelcome.enabled", false);
user_pref("browser.startup.homepage_override.mstone", "ignore");
user_pref("trailhead.firstrun.branches", "nofirstrun-empty");
user_pref("browser.sessionstore.resume_session_once", false);
user_pref("browser.sessionstore.resume_from_crash", false);
user_pref("browser.shell.checkDefaultBrowser", false);
user_pref("privacy.trackingprotection.enabled", true);
user_pref("privacy.trackingprotection.pbmode.enabled", true);
user_pref("privacy.resistFingerprinting", true);
user_pref("media.peerconnection.ice.default_address_only", true);
user_pref("media.peerconnection.ice.no_host", true);
user_pref("gfx.webrender.software", true);
user_pref("layers.acceleration.disabled", true);
"#,
        proxy_config
    );

    let user_js_path = profile_dir.join("user.js");
    let _ = fs::write(&user_js_path, user_js_content);

    // Write userChrome.css to profile
    let chrome_dir = profile_dir.join("chrome");
    let _ = fs::create_dir_all(&chrome_dir);
    let _ = fs::write(chrome_dir.join("userChrome.css"), EMBEDDED_USER_CHROME);

    let src_user_content = app_dir.join("chrome").join("userContent.css");
    if src_user_content.exists() {
        let _ = fs::copy(&src_user_content, chrome_dir.join("userContent.css"));
    }
}

fn resolve_internal_route(input: &str) -> String {
    let trimmed = input.trim();
    let normalized = trimmed.replace("qaulium://", "qualium://");
    if normalized.starts_with("qualium://") {
        return normalized;
    }
    if trimmed.is_empty() {
        return "qualium://newtab".to_string();
    }

    if trimmed.starts_with("chrome://") || trimmed.starts_with("about:") || trimmed.starts_with("resource://") {
        return trimmed.to_string();
    }

    if trimmed.starts_with("http://") || trimmed.starts_with("https://") {
        return trimmed.to_string();
    }

    // Domain pattern
    if !trimmed.contains(' ') && trimmed.contains('.') {
        return format!("https://{}", trimmed);
    }

    // Default search
    format!("https://www.google.com/search?q={}", urlencoding_simple(trimmed))
}

fn urlencoding_simple(input: &str) -> String {
    let mut out = String::new();
    for b in input.bytes() {
        if b.is_ascii_alphanumeric() || b == b'-' || b == b'_' || b == b'.' || b == b'~' {
            out.push(b as char);
        } else if b == b' ' {
            out.push('+');
        } else {
            out.push_str(&format!("%{:02X}", b));
        }
    }
    out
}

fn get_target_url_from_args() -> Option<String> {
    let args: Vec<String> = env::args().collect();
    let mut idx = 1;
    while idx < args.len() {
        if (args[idx] == "-url" || args[idx] == "--url") && idx + 1 < args.len() {
            return Some(resolve_internal_route(&args[idx + 1]));
        }
        if !args[idx].starts_with('-') {
            return Some(resolve_internal_route(&args[idx]));
        }
        idx += 1;
    }
    None
}

fn spawn_daemon(app_dir: &Path, profile_dir: &Path) -> Option<Child> {
    let mut candidate_paths = vec![
        app_dir.join("qualium-daemon.exe"),
        app_dir.join("qualium-daemon"),
        app_dir.join("bin").join("qualium-daemon"),
        PathBuf::from("target").join("release").join("qualium-daemon.exe"),
        PathBuf::from("target").join("release").join("qualium-daemon"),
        PathBuf::from("target").join("debug").join("qualium-daemon.exe"),
        PathBuf::from("target").join("debug").join("qualium-daemon"),
    ];

    #[cfg(windows)]
    if let Ok(local_appdata) = env::var("LOCALAPPDATA") {
        candidate_paths.push(PathBuf::from(&local_appdata).join("Programs").join("Qualium").join("qualium-daemon.exe"));
        candidate_paths.push(PathBuf::from(&local_appdata).join("Programs").join("Qaulium").join("qualium-daemon.exe"));
    }

    #[cfg(target_os = "macos")]
    {
        candidate_paths.push(PathBuf::from("/Applications/Qualium Quantum Browser.app/Contents/Resources/bin/qualium-daemon"));
    }

    #[cfg(target_os = "linux")]
    {
        candidate_paths.push(PathBuf::from("/opt/qualium-quantum-browser/bin/qualium-daemon"));
        candidate_paths.push(PathBuf::from("/usr/bin/qualium-daemon"));
    }

    for daemon_exe in candidate_paths {
        if daemon_exe.exists() {
            let cwd = daemon_exe.parent().unwrap_or(app_dir);
            let mut cmd = Command::new(&daemon_exe);
            cmd.current_dir(cwd);
            cmd.arg("--profile");
            cmd.arg(profile_dir.to_string_lossy().as_ref());
            #[cfg(windows)]
            cmd.creation_flags(CREATE_NO_WINDOW);
            if let Ok(child) = cmd.spawn() {
                return Some(child);
            }
        }
    }
    None
}

fn find_gecko_runtime(app_dir: &Path) -> Option<(PathBuf, PathBuf)> {
    // 1. Check runtime/qualium-core or root binaries
    for name in &["qualium-core.exe", "qualium-core", "firefox.exe", "firefox"] {
        let p1 = app_dir.join("runtime").join(name);
        if p1.exists() {
            return Some((p1, app_dir.join("runtime")));
        }
        let p2 = app_dir.join(name);
        if p2.exists() {
            return Some((p2, app_dir.to_path_buf()));
        }
    }

    // 2. Windows LocalAppData
    #[cfg(windows)]
    if let Ok(local_appdata) = env::var("LOCALAPPDATA") {
        let p3 = PathBuf::from(&local_appdata).join("Programs").join("Qualium").join("runtime").join("qualium-core.exe");
        if p3.exists() {
            let cwd = p3.parent().unwrap().to_path_buf();
            return Some((p3, cwd));
        }
        let p3b = PathBuf::from(&local_appdata).join("Programs").join("Qaulium").join("runtime").join("qualium-core.exe");
        if p3b.exists() {
            let cwd = p3b.parent().unwrap().to_path_buf();
            return Some((p3b, cwd));
        }
    }

    // 3. macOS system Firefox or embedded runtime
    #[cfg(target_os = "macos")]
    {
        let mac_paths = [
            PathBuf::from("/Applications/Qualium Quantum Browser.app/Contents/Resources/runtime/qualium-core"),
            PathBuf::from("/Applications/Firefox.app/Contents/MacOS/firefox"),
            PathBuf::from("/Applications/Firefox Developer Edition.app/Contents/MacOS/firefox"),
            PathBuf::from("/Applications/Firefox Nightly.app/Contents/MacOS/firefox"),
        ];
        for mp in &mac_paths {
            if mp.exists() {
                let cwd = mp.parent().unwrap().to_path_buf();
                return Some((mp.clone(), cwd));
            }
        }
    }

    // 4. Linux system Firefox or /opt runtime
    #[cfg(target_os = "linux")]
    {
        let linux_paths = [
            PathBuf::from("/opt/qualium-quantum-browser/runtime/qualium-core"),
            PathBuf::from("/usr/lib/firefox/firefox"),
            PathBuf::from("/usr/lib64/firefox/firefox"),
            PathBuf::from("/usr/bin/firefox"),
        ];
        for lp in &linux_paths {
            if lp.exists() {
                let cwd = lp.parent().unwrap().to_path_buf();
                return Some((lp.clone(), cwd));
            }
        }
    }

    // 5. Check current working directory runtime
    for name in &["qualium-core.exe", "qualium-core", "firefox"] {
        let p_cwd = PathBuf::from("runtime").join(name);
        if p_cwd.exists() {
            if let Ok(full) = p_cwd.canonicalize() {
                let cwd = full.parent().unwrap().to_path_buf();
                return Some((full, cwd));
            }
        }
    }

    // 6. Check repo root runtime during development
    if let Ok(manifest_dir) = env::var("CARGO_MANIFEST_DIR") {
        for name in &["qualium-core.exe", "qualium-core", "firefox"] {
            let p4 = PathBuf::from(&manifest_dir).join("..").join("..").join("runtime").join(name);
            if p4.exists() {
                let cwd = p4.parent().unwrap().to_path_buf();
                return Some((p4, cwd));
            }
        }
    }

    None
}

#[cfg(windows)]
fn count_processes_named(target_name: &str) -> usize {
    use std::mem::size_of;
    #[repr(C)]
    struct PROCESSENTRY32W {
        dw_size: u32,
        cnt_usage: u32,
        th32_process_id: u32,
        th32_default_heap_id: usize,
        th32_module_id: u32,
        cnt_threads: u32,
        th32_parent_process_id: u32,
        pc_pri_class_base: i32,
        dw_flags: u32,
        sz_exe_file: [u16; 260],
    }
    extern "system" {
        fn CreateToolhelp32Snapshot(dw_flags: u32, th32_process_id: u32) -> *mut std::ffi::c_void;
        fn Process32FirstW(h_snapshot: *mut std::ffi::c_void, lppe: *mut PROCESSENTRY32W) -> i32;
        fn Process32NextW(h_snapshot: *mut std::ffi::c_void, lppe: *mut PROCESSENTRY32W) -> i32;
        fn CloseHandle(h_object: *mut std::ffi::c_void) -> i32;
    }

    unsafe {
        let snapshot = CreateToolhelp32Snapshot(0x00000002, 0); // TH32CS_SNAPPROCESS
        if snapshot.is_null() || snapshot as isize == -1 {
            return 0;
        }
        let mut entry = PROCESSENTRY32W {
            dw_size: size_of::<PROCESSENTRY32W>() as u32,
            cnt_usage: 0,
            th32_process_id: 0,
            th32_default_heap_id: 0,
            th32_module_id: 0,
            cnt_threads: 0,
            th32_parent_process_id: 0,
            pc_pri_class_base: 0,
            dw_flags: 0,
            sz_exe_file: [0; 260],
        };
        let mut count = 0;
        if Process32FirstW(snapshot, &mut entry) != 0 {
            loop {
                let len = entry.sz_exe_file.iter().position(|&c| c == 0).unwrap_or(260);
                let name = String::from_utf16_lossy(&entry.sz_exe_file[..len]);
                if name.eq_ignore_ascii_case(target_name) {
                    count += 1;
                }
                if Process32NextW(snapshot, &mut entry) == 0 {
                    break;
                }
            }
        }
        CloseHandle(snapshot);
        count
    }
}

fn is_gecko_running() -> bool {
    #[cfg(windows)]
    {
        count_processes_named("qualium-core.exe") > 0
    }
    #[cfg(not(windows))]
    {
        Command::new("pgrep")
            .args(["-f", "qualium-core"])
            .output()
            .map(|o| o.status.success() && !o.stdout.is_empty())
            .unwrap_or(false)
    }
}

fn is_primary_instance() -> bool {
    #[cfg(windows)]
    {
        let current_exe_name = env::current_exe()
            .ok()
            .and_then(|p| p.file_name().map(|n| n.to_string_lossy().to_string()))
            .unwrap_or_else(|| "QualiumQuantumBrowser.exe".to_string());

        let browser_count = count_processes_named(&current_exe_name);
        let default_host_count = count_processes_named("QualiumQuantumBrowser.exe");
        let qualium_browser_count = count_processes_named("qualium-browser.exe");
        let gecko_count = count_processes_named("qualium-core.exe");

        let total_host_count = browser_count.max(default_host_count).max(qualium_browser_count);
        total_host_count <= 1 || gecko_count == 0
    }
    #[cfg(not(windows))]
    {
        let output = Command::new("pgrep")
            .args(["-f", "qualium-browser"])
            .output();
        if let Ok(out) = output {
            let count = String::from_utf8_lossy(&out.stdout).lines().count();
            count <= 1
        } else {
            true
        }
    }
}

fn main() -> anyhow::Result<()> {
    let app_dir = get_app_dir();
    let profile_dir = get_profile_dir();

    let is_primary = is_primary_instance();

    append_boot_log(&format!(
        "Qualium Boot: is_primary={}, app_dir={}, profile_dir={}",
        is_primary,
        app_dir.display(),
        profile_dir.display()
    ));

    if is_primary {
        // 1. Primary instance: purge all stale background zombies from previous sessions/crashes
        #[cfg(windows)]
        {
            let _ = Command::new("taskkill")
                .args(["/F", "/IM", "qualium-core.exe", "/T"])
                .creation_flags(CREATE_NO_WINDOW)
                .status();

            let _ = Command::new("taskkill")
                .args(["/F", "/IM", "qualium-daemon.exe", "/T"])
                .creation_flags(CREATE_NO_WINDOW)
                .status();
        }
        #[cfg(not(windows))]
        {
            let _ = Command::new("pkill")
                .args(["-f", "qualium-daemon"])
                .status();
        }

        for _ in 0..5 {
            let lock1 = profile_dir.join("parent.lock");
            if lock1.exists() {
                let _ = fs::remove_file(&lock1);
            }
            let lock2 = profile_dir.join(".parentlock");
            if lock2.exists() {
                let _ = fs::remove_file(&lock2);
            }
            if !profile_dir.join("parent.lock").exists() && !profile_dir.join(".parentlock").exists() {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(100));
        }

        append_boot_log(&format!("Primary supervisor: app_dir={}", app_dir.display()));

        // 2. Start Post-Quantum Security Daemon with active profile context
        let daemon_child = spawn_daemon(&app_dir, &profile_dir);
        let active_proxy_port = discover_active_proxy(&profile_dir, 4000);
        let proxy_enabled = active_proxy_port.is_some();
        append_boot_log(&format!(
            "Daemon spawned: {}, Active SOCKS5 port: {:?}, Proxy enabled: {}",
            daemon_child.is_some(),
            active_proxy_port,
            proxy_enabled
        ));

        // 3. Configure profile with dynamic proxy port
        configure_profile(&app_dir, &profile_dir, active_proxy_port);
        append_boot_log(&format!("Profile dir configured: {}, proxy_enabled: {}", profile_dir.display(), proxy_enabled));

        // 4. Locate Gecko runtime
        let (gecko_exe, gecko_cwd) = match find_gecko_runtime(&app_dir) {
            Some(pair) => pair,
            None => {
                append_boot_log("Gecko runtime not found!");
                return Ok(());
            }
        };
        append_boot_log(&format!("Gecko runtime located: {}", gecko_exe.display()));

        // 5. Target URL handling
        let target_url = get_target_url_from_args();
        if let Some(ref u) = target_url {
            append_boot_log(&format!("Target url: {}", u));
            let _ = fs::write(env::temp_dir().join("qualium_pending_nav.txt"), u);
        } else {
            append_boot_log("Target url: None (loading default homepage)");
        }

        // 6. Launch native Gecko browser engine with Qualium branding & profile
        let mut gecko_cmd = Command::new(&gecko_exe);
        gecko_cmd.current_dir(&gecko_cwd);
        gecko_cmd.arg("-profile");
        gecko_cmd.arg(profile_dir.to_string_lossy().as_ref());
        gecko_cmd.arg("-no-remote");
        if let Some(ref url) = target_url {
            gecko_cmd.arg("-url");
            gecko_cmd.arg(url);
        }

        let mut gecko_child = gecko_cmd.spawn()?;
        append_boot_log("Gecko process spawned successfully. Supervising browser session...");

        // 7. Supervise browser lifecycle until Gecko terminates
        let _ = gecko_child.wait();

        while is_gecko_running() {
            std::thread::sleep(std::time::Duration::from_millis(500));
        }

        append_boot_log("Gecko browser closed. Terminating background daemon...");

        // 8. Browser closed: kill daemon cleanly
        if let Some(mut child) = daemon_child {
            let _ = child.kill();
            append_boot_log("Daemon killed cleanly.");
        }
    } else {
        // Secondary instance: Browser is ALREADY running!
        append_boot_log("Secondary instance: delegating URL to running Gecko instance");
        let (gecko_exe, gecko_cwd) = match find_gecko_runtime(&app_dir) {
            Some(pair) => pair,
            None => return Ok(()),
        };

        let target_url = get_target_url_from_args();

        let mut gecko_cmd = Command::new(&gecko_exe);
        gecko_cmd.current_dir(&gecko_cwd);
        gecko_cmd.arg("-profile");
        gecko_cmd.arg(profile_dir.to_string_lossy().as_ref());
        if let Some(ref url) = target_url {
            gecko_cmd.arg("-url");
            gecko_cmd.arg(url);
        }

        let mut proc = gecko_cmd.spawn()?;
        let _ = proc.wait();
    }

    Ok(())
}
