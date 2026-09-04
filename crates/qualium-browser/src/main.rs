#![windows_subsystem = "windows"]

//! Qualium Quantum Browser v5 — Native Gecko Desktop Browser Host
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

#[cfg(target_os = "windows")]
mod win_branding {
    type HWND = *mut std::ffi::c_void;
    type BOOL = i32;
    type LPARAM = isize;
    type WNDENUMPROC = unsafe extern "system" fn(HWND, LPARAM) -> BOOL;

    #[link(name = "user32")]
    extern "system" {
        fn EnumWindows(lpEnumFunc: WNDENUMPROC, lParam: LPARAM) -> BOOL;
        fn IsWindowVisible(hWnd: HWND) -> BOOL;
        fn GetClassNameW(hWnd: HWND, lpClassName: *mut u16, nMaxCount: i32) -> i32;
        fn SetWindowTextW(hWnd: HWND, lpString: *const u16) -> BOOL;
    }

    unsafe extern "system" fn enum_proc(hwnd: HWND, _lparam: LPARAM) -> BOOL {
        if IsWindowVisible(hwnd) != 0 {
            let mut class_buf = [0u16; 256];
            let len = GetClassNameW(hwnd, class_buf.as_mut_ptr(), 256);
            if len > 0 {
                let class_str = String::from_utf16_lossy(&class_buf[..len as usize]);
                if class_str == "MozillaWindowClass" {
                    let title: Vec<u16> = "Qaulium Quantum Browser\0".encode_utf16().collect();
                    SetWindowTextW(hwnd, title.as_ptr());
                }
            }
        }
        1
    }

    pub fn rebrand_all() {
        unsafe {
            EnumWindows(enum_proc, 0);
        }
    }
}

fn get_app_dir() -> PathBuf {
    if let Ok(current_exe) = env::current_exe() {
        if let Some(parent) = current_exe.parent() {
            return parent.to_path_buf();
        }
    }
    if let Ok(local_appdata) = env::var("LOCALAPPDATA") {
        let p_qau = PathBuf::from(&local_appdata).join("Programs").join("Qaulium");
        if p_qau.exists() {
            return p_qau;
        }
        let p_qua = PathBuf::from(&local_appdata).join("Programs").join("Qualium");
        if p_qua.exists() {
            return p_qua;
        }
    }
    PathBuf::from("C:\\Qaulium")
}

fn get_profile_dir() -> PathBuf {
    if let Ok(local_appdata) = env::var("LOCALAPPDATA") {
        let p_qau = PathBuf::from(&local_appdata).join("Qaulium").join("Profile");
        let p_qua = PathBuf::from(&local_appdata).join("Qualium").join("Profile");
        if p_qau.exists() {
            return p_qau;
        } else if p_qua.exists() {
            return p_qua;
        }
        let _ = fs::create_dir_all(&p_qau);
        return p_qau;
    }
    let p = env::temp_dir().join("qaulium_profile");
    let _ = fs::create_dir_all(&p);
    p
}

fn wait_for_port_ready(port: u16, max_wait_ms: u64) -> bool {
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

fn configure_profile(app_dir: &Path, profile_dir: &Path, use_proxy: bool) {
    let proxy_config = if use_proxy {
        r#"user_pref("network.proxy.type", 1);
user_pref("network.proxy.socks", "127.0.0.1");
user_pref("network.proxy.socks_port", 9050);
user_pref("network.proxy.socks_version", 5);
user_pref("network.proxy.socks_remote_dns", true);"#
    } else {
        r#"user_pref("network.proxy.type", 0);"#
    };

    let user_js_content = format!(
        r#"// Qualium Quantum Browser v5 — Profile Configuration
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
"#,
        proxy_config
    );

    let user_js_path = profile_dir.join("user.js");
    let _ = fs::write(&user_js_path, user_js_content);

    // Copy userChrome.css and userContent.css to profile
    let chrome_dir = profile_dir.join("chrome");
    let _ = fs::create_dir_all(&chrome_dir);
    let src_user_chrome = app_dir.join("chrome").join("userChrome.css");
    if src_user_chrome.exists() {
        let _ = fs::copy(&src_user_chrome, chrome_dir.join("userChrome.css"));
    }
    let src_user_content = app_dir.join("chrome").join("userContent.css");
    if src_user_content.exists() {
        let _ = fs::copy(&src_user_content, chrome_dir.join("userContent.css"));
    }
}

fn resolve_internal_route(input: &str) -> String {
    let trimmed = input.trim();
    let normalized = trimmed.replace("qaulium://", "qualium://");
    if normalized.is_empty() || normalized == "qualium://newtab" {
        return "about:home".to_string();
    }
    if normalized == "qualium://settings" || normalized == "qualium://about" {
        return "about:preferences".to_string();
    }
    if normalized == "qualium://privacy" || normalized == "qualium://security" {
        return "about:protections".to_string();
    }
    if normalized == "qualium://downloads" {
        return "about:downloads".to_string();
    }
    if normalized == "qualium://bookmarks" {
        return "about:bookmarks".to_string();
    }
    if normalized == "qualium://history" {
        return "about:history".to_string();
    }
    if normalized == "qualium://passwords" {
        return "about:logins".to_string();
    }
    if normalized == "qualium://extensions" {
        return "about:addons".to_string();
    }
    if normalized == "qualium://onboarding" || normalized == "qualium://welcome" {
        return "about:welcome".to_string();
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

fn spawn_daemon(app_dir: &Path) -> Option<Child> {
    let daemon_exe = app_dir.join("qualium-daemon.exe");
    if daemon_exe.exists() {
        let mut cmd = Command::new(&daemon_exe);
        cmd.current_dir(app_dir);
        #[cfg(windows)]
        cmd.creation_flags(CREATE_NO_WINDOW);
        cmd.spawn().ok()
    } else {
        None
    }
}

fn find_gecko_runtime(app_dir: &Path) -> Option<(PathBuf, PathBuf)> {
    // 1. Check app_dir/runtime/qualium-core.exe
    let p1 = app_dir.join("runtime").join("qualium-core.exe");
    if p1.exists() {
        return Some((p1, app_dir.join("runtime")));
    }

    // 2. Check app_dir/qualium-core.exe
    let p2 = app_dir.join("qualium-core.exe");
    if p2.exists() {
        return Some((p2, app_dir.to_path_buf()));
    }

    // 3. Check repo root runtime/qualium-core.exe during development
    if let Ok(manifest_dir) = env::var("CARGO_MANIFEST_DIR") {
        let p4 = PathBuf::from(manifest_dir).join("..").join("..").join("runtime").join("qualium-core.exe");
        if p4.exists() {
            let cwd = p4.parent().unwrap().to_path_buf();
            return Some((p4, cwd));
        }
    }

    None
}

#[cfg(windows)]
fn is_gecko_running() -> bool {
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
            return false;
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
        let mut found = false;
        if Process32FirstW(snapshot, &mut entry) != 0 {
            loop {
                let len = entry.sz_exe_file.iter().position(|&c| c == 0).unwrap_or(260);
                let name = String::from_utf16_lossy(&entry.sz_exe_file[..len]);
                if name.eq_ignore_ascii_case("qualium-core.exe") {
                    found = true;
                    break;
                }
                if Process32NextW(snapshot, &mut entry) == 0 {
                    break;
                }
            }
        }
        CloseHandle(snapshot);
        found
    }
}

#[cfg(not(windows))]
fn is_gecko_running() -> bool {
    false
}



extern "system" {
    fn CreateMutexW(lp_mutex_attributes: *mut std::ffi::c_void, b_initial_owner: i32, lp_name: *const u16) -> *mut std::ffi::c_void;
    fn GetLastError() -> u32;
}

fn main() -> anyhow::Result<()> {
    let app_dir = get_app_dir();
    let profile_dir = get_profile_dir();

    // Check if this is the primary supervisor instance
    let (is_primary, _supervisor_mutex_handle) = {
        #[cfg(windows)]
        {
            use std::ffi::OsStr;
            use std::os::windows::ffi::OsStrExt;
            let name: Vec<u16> = OsStr::new("Local\\QualiumQuantumBrowser_Supervisor_Mutex\0").encode_wide().collect();
            let handle = unsafe { CreateMutexW(std::ptr::null_mut(), 0, name.as_ptr()) };
            let is_first = !handle.is_null() && unsafe { GetLastError() } != 183; // 183 = ERROR_ALREADY_EXISTS
            (is_first, handle)
        }
        #[cfg(not(windows))]
        (true, std::ptr::null_mut())
    };

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

        let lock1 = profile_dir.join("parent.lock");
        if lock1.exists() {
            let _ = fs::remove_file(&lock1);
        }
        let lock2 = profile_dir.join(".parentlock");
        if lock2.exists() {
            let _ = fs::remove_file(&lock2);
        }

        let log_file = env::temp_dir().join("qualium_boot.log");
        let mut log = format!("Qualium Boot (Primary): app_dir={}\n", app_dir.display());

        // 2. Start Post-Quantum Security Daemon
        let daemon_child = spawn_daemon(&app_dir);
        let daemon_ready = if daemon_child.is_some() {
            wait_for_port_ready(9050, 2500)
        } else {
            wait_for_port_ready(9050, 500)
        };
        log.push_str(&format!("Daemon spawned: {}, SOCKS5 port 9050 ready: {}\n", daemon_child.is_some(), daemon_ready));

        // 3. Configure profile
        configure_profile(&app_dir, &profile_dir, daemon_ready);
        log.push_str(&format!("Profile dir: {}, proxy_enabled: {}\n", profile_dir.display(), daemon_ready));

        // 4. Locate Gecko runtime
        let (gecko_exe, gecko_cwd) = match find_gecko_runtime(&app_dir) {
            Some(pair) => pair,
            None => {
                log.push_str("Gecko runtime not found!\n");
                let _ = fs::write(&log_file, log);
                return Ok(());
            }
        };
        log.push_str(&format!("Gecko exe: {}\n", gecko_exe.display()));

        // 5. Target URL handling
        let target_url = get_target_url_from_args();
        if let Some(ref u) = target_url {
            log.push_str(&format!("Target url: {}\n", u));
        } else {
            log.push_str("Target url: None (loading default homepage from user.js)\n");
        }

        // 6. Launch native Gecko browser engine with Qualium branding & profile
        let mut gecko_cmd = Command::new(&gecko_exe);
        gecko_cmd.current_dir(&gecko_cwd);
        gecko_cmd.arg("-profile");
        gecko_cmd.arg(profile_dir.to_string_lossy().as_ref());
        gecko_cmd.arg("-no-remote");
        if let Some(ref url) = target_url {
            if url.starts_with("chrome://") {
                gecko_cmd.arg("--chrome");
                gecko_cmd.arg(url);
            } else {
                gecko_cmd.arg("-url");
                gecko_cmd.arg(url);
            }
        }

        let _ = gecko_cmd.spawn()?;
        log.push_str("Gecko proc spawned successfully. Beginning supervision loop...\n");
        let _ = fs::write(&log_file, &log);

        // 7. Supervise browser lifecycle & rebrand window title
        let mut started = false;
        for _ in 0..20 {
            if is_gecko_running() {
                started = true;
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(500));
        }

        if started {
            let mut missing_count = 0;
            while missing_count < 4 {
                if is_gecko_running() {
                    missing_count = 0;
                    #[cfg(target_os = "windows")]
                    win_branding::rebrand_all();
                } else {
                    missing_count += 1;
                }
                std::thread::sleep(std::time::Duration::from_millis(500));
            }
        }

        log.push_str("Gecko browser closed. Terminating background daemon...\n");
        let _ = fs::write(&log_file, &log);

        // 8. Browser closed: kill daemon cleanly
        if let Some(mut child) = daemon_child {
            let _ = child.kill();
            log.push_str("Daemon killed cleanly.\n");
            let _ = fs::write(&log_file, &log);
        }
    } else {
        // Secondary instance: Browser is ALREADY running!
        // Delegate cleanly via Gecko native remoting without killing anything or touching locks
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
