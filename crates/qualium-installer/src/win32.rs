//! Windows Native API Bindings & Helpers for Qualium Installer/Uninstaller
//! Provides disk queries, registry registration, process detection, and shortcut creation without WinForms/.NET.
//!
//! This module is Windows-only and must not be compiled on Linux or macOS.
#![cfg(windows)]

use std::ffi::OsStr;
use std::fs;
use std::mem::size_of;
use std::os::windows::ffi::OsStrExt;
use std::path::{Path, PathBuf};
use std::process::Command;

#[cfg(windows)]
pub mod sys {
    pub type HWND = *mut std::ffi::c_void;
    pub type HKEY = *mut std::ffi::c_void;
    pub type BOOL = i32;
    pub type DWORD = u32;
    pub type LSTATUS = i32;
    pub type LPCWSTR = *const u16;
    pub type LPWSTR = *mut u16;

    pub const HKEY_CURRENT_USER: HKEY = 0x80000001usize as HKEY;
    pub const KEY_ALL_ACCESS: DWORD = 0xF003F;
    pub const KEY_WRITE: DWORD = 0x20006;
    pub const REG_SZ: DWORD = 1;
    pub const REG_DWORD: DWORD = 4;
    pub const CREATE_NO_WINDOW: u32 = 0x08000000;

    #[repr(C)]
    pub struct PROCESSENTRY32W {
        pub dw_size: u32,
        pub cnt_usage: u32,
        pub th32_process_id: u32,
        pub th32_default_heap_id: usize,
        pub th32_module_id: u32,
        pub cnt_threads: u32,
        pub th32_parent_process_id: u32,
        pub pc_pri_class_base: i32,
        pub dw_flags: u32,
        pub sz_exe_file: [u16; 260],
    }

    #[link(name = "kernel32")]
    extern "system" {
        pub fn GetDiskFreeSpaceExW(
            lpDirectoryName: LPCWSTR,
            lpFreeBytesAvailableToCaller: *mut u64,
            lpTotalNumberOfBytes: *mut u64,
            lpTotalNumberOfFreeBytes: *mut u64,
        ) -> BOOL;
        pub fn CreateToolhelp32Snapshot(dwFlags: u32, th32ProcessID: u32) -> *mut std::ffi::c_void;
        pub fn Process32FirstW(hSnapshot: *mut std::ffi::c_void, lppe: *mut PROCESSENTRY32W) -> BOOL;
        pub fn Process32NextW(hSnapshot: *mut std::ffi::c_void, lppe: *mut PROCESSENTRY32W) -> BOOL;
        pub fn CloseHandle(hObject: *mut std::ffi::c_void) -> BOOL;
    }

    #[link(name = "advapi32")]
    extern "system" {
        pub fn RegCreateKeyExW(
            hKey: HKEY,
            lpSubKey: LPCWSTR,
            Reserved: DWORD,
            lpClass: LPWSTR,
            dwOptions: DWORD,
            samDesired: DWORD,
            lpSecurityAttributes: *mut std::ffi::c_void,
            phkResult: *mut HKEY,
            lpdwDisposition: *mut DWORD,
        ) -> LSTATUS;
        pub fn RegSetValueExW(
            hKey: HKEY,
            lpValueName: LPCWSTR,
            Reserved: DWORD,
            dwType: DWORD,
            lpData: *const u8,
            cbData: DWORD,
        ) -> LSTATUS;
        pub fn RegDeleteKeyW(hKey: HKEY, lpSubKey: LPCWSTR) -> LSTATUS;
        pub fn RegDeleteTreeW(hKey: HKEY, lpSubKey: LPCWSTR) -> LSTATUS;
        pub fn RegCloseKey(hKey: HKEY) -> LSTATUS;
    }

    #[link(name = "shell32")]
    extern "system" {
        pub fn SHGetSpecialFolderPathW(
            hwndOwner: HWND,
            lpszPath: LPWSTR,
            nFolder: i32,
            fCreate: BOOL,
        ) -> BOOL;
    }
}

pub fn to_wide_null(s: &str) -> Vec<u16> {
    OsStr::new(s).encode_wide().chain(std::iter::once(0)).collect()
}

/// Dynamic Disk Free Space Query via GetDiskFreeSpaceExW
pub fn get_disk_free_space(path: &Path) -> Option<(u64, u64)> {
    let mut root = path.to_path_buf();
    while !root.exists() {
        if let Some(parent) = root.parent() {
            root = parent.to_path_buf();
        } else {
            break;
        }
    }
    let wide = to_wide_null(&root.to_string_lossy());
    let mut free_avail: u64 = 0;
    let mut total_bytes: u64 = 0;
    let mut total_free: u64 = 0;

    let res = unsafe {
        sys::GetDiskFreeSpaceExW(
            wide.as_ptr(),
            &mut free_avail,
            &mut total_bytes,
            &mut total_free,
        )
    };

    if res != 0 {
        Some((free_avail, total_bytes))
    } else {
        None
    }
}

/// Detect Running Qualium Processes (QualiumQuantumBrowser, qualium-core, qualium-daemon)
pub fn get_running_qualium_processes() -> Vec<String> {
    let targets = [
        "qualiumquantumbrowser.exe",
        "qauliumquantumbrowser.exe",
        "qualium-core.exe",
        "qualium-daemon.exe",
    ];

    let mut running = Vec::new();
    unsafe {
        let snapshot = sys::CreateToolhelp32Snapshot(0x00000002, 0); // TH32CS_SNAPPROCESS
        if snapshot.is_null() || snapshot as isize == -1 {
            return running;
        }
        let mut entry = sys::PROCESSENTRY32W {
            dw_size: size_of::<sys::PROCESSENTRY32W>() as u32,
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

        if sys::Process32FirstW(snapshot, &mut entry) != 0 {
            loop {
                let len = entry.sz_exe_file.iter().position(|&c| c == 0).unwrap_or(260);
                let name = String::from_utf16_lossy(&entry.sz_exe_file[..len]);
                let lower = name.to_lowercase();
                for t in &targets {
                    if lower == *t && !running.contains(&name) {
                        running.push(name.clone());
                    }
                }
                if sys::Process32NextW(snapshot, &mut entry) == 0 {
                    break;
                }
            }
        }
        sys::CloseHandle(snapshot);
    }
    running
}

/// Native Windows Shortcut Creation using WScript.Shell via COM or Windows Script Host
pub fn create_shortcut(
    target_exe: &Path,
    shortcut_path: &Path,
    working_dir: &Path,
    description: &str,
    icon_path: Option<&Path>,
) -> anyhow::Result<()> {
    if let Some(parent) = shortcut_path.parent() {
        fs::create_dir_all(parent)?;
    }

    let icon_arg = if let Some(ic) = icon_path {
        format!("$s.IconLocation = '{}';", ic.display())
    } else {
        String::new()
    };

    let ps_script = format!(
        r#"$ws = New-Object -ComObject WScript.Shell; $s = $ws.CreateShortcut('{}'); $s.TargetPath = '{}'; $s.WorkingDirectory = '{}'; $s.Description = '{}'; {}$s.Save()"#,
        shortcut_path.display(),
        target_exe.display(),
        working_dir.display(),
        description,
        icon_arg
    );

    let status = Command::new("powershell")
        .args(["-NoProfile", "-NonInteractive", "-Command", &ps_script])
        .status()?;

    if !status.success() {
        anyhow::bail!("Failed to create Windows shortcut at {}", shortcut_path.display());
    }

    Ok(())
}

/// Register Windows Installed Apps entry in HKCU\Software\Microsoft\Windows\CurrentVersion\Uninstall\QualiumQuantumBrowser
pub fn register_uninstall(
    install_dir: &Path,
    uninstaller_exe: &Path,
    icon_path: &Path,
    total_size_kb: u64,
) -> anyhow::Result<()> {
    let subkey_str = r"Software\Microsoft\Windows\CurrentVersion\Uninstall\QualiumQuantumBrowser";
    let subkey_wide = to_wide_null(subkey_str);

    let mut hkey: sys::HKEY = std::ptr::null_mut();
    let mut disp: sys::DWORD = 0;

    let res = unsafe {
        sys::RegCreateKeyExW(
            sys::HKEY_CURRENT_USER,
            subkey_wide.as_ptr(),
            0,
            std::ptr::null_mut(),
            0,
            sys::KEY_ALL_ACCESS,
            std::ptr::null_mut(),
            &mut hkey,
            &mut disp,
        )
    };

    if res != 0 {
        anyhow::bail!("Failed to create Windows registry key: error code {}", res);
    }

    let set_str = |key: &str, val: &str| unsafe {
        let k_wide = to_wide_null(key);
        let val_bytes: Vec<u8> = to_wide_null(val).iter().flat_map(|w| w.to_le_bytes()).collect();
        sys::RegSetValueExW(hkey, k_wide.as_ptr(), 0, sys::REG_SZ, val_bytes.as_ptr(), val_bytes.len() as u32);
    };

    let set_dword = |key: &str, val: u32| unsafe {
        let k_wide = to_wide_null(key);
        let val_bytes = val.to_le_bytes();
        sys::RegSetValueExW(hkey, k_wide.as_ptr(), 0, sys::REG_DWORD, val_bytes.as_ptr(), 4);
    };

    set_str("DisplayName", "Qualium Quantum Browser");
    set_str("DisplayVersion", "1.0.0");
    set_str("Publisher", "Qualium AI");
    set_str("InstallLocation", &install_dir.to_string_lossy());
    set_str("DisplayIcon", &format!("{},0", icon_path.display()));
    set_str("UninstallString", &format!("\"{}\"", uninstaller_exe.display()));
    set_str("QuietUninstallString", &format!("\"{}\" /silent", uninstaller_exe.display()));
    set_str("HelpLink", "https://qualium.ai/support");
    set_str("URLInfoAbout", "https://qualium.ai");
    set_dword("EstimatedSize", total_size_kb as u32);
    set_dword("NoModify", 1);
    set_dword("NoRepair", 1);

    unsafe {
        sys::RegCloseKey(hkey);
    }

    Ok(())
}

/// Unregister Windows Installed Apps entry
pub fn unregister_uninstall() -> anyhow::Result<()> {
    let subkey_str = r"Software\Microsoft\Windows\CurrentVersion\Uninstall\QualiumQuantumBrowser";
    let subkey_wide = to_wide_null(subkey_str);

    unsafe {
        let _ = sys::RegDeleteTreeW(sys::HKEY_CURRENT_USER, subkey_wide.as_ptr());
        let _ = sys::RegDeleteKeyW(sys::HKEY_CURRENT_USER, subkey_wide.as_ptr());
    }

    Ok(())
}

/// Standard paths helper
pub fn get_default_install_dir() -> PathBuf {
    if let Ok(local_appdata) = std::env::var("LOCALAPPDATA") {
        PathBuf::from(local_appdata).join("Programs").join("Qualium")
    } else {
        PathBuf::from(r"C:\Program Files\Qualium")
    }
}

pub fn get_user_data_dir() -> PathBuf {
    if let Ok(local_appdata) = std::env::var("LOCALAPPDATA") {
        PathBuf::from(local_appdata).join("Qualium")
    } else {
        PathBuf::from(r"C:\QualiumProfile")
    }
}

pub fn get_desktop_shortcut_paths() -> Vec<PathBuf> {
    let mut paths = Vec::new();
    let mut buf = [0u16; 260];
    unsafe {
        if sys::SHGetSpecialFolderPathW(std::ptr::null_mut(), buf.as_mut_ptr(), 0x0010, 0) != 0 {
            let len = buf.iter().position(|&c| c == 0).unwrap_or(260);
            let dir = PathBuf::from(String::from_utf16_lossy(&buf[..len]));
            if dir.exists() {
                paths.push(dir.join("Qualium Quantum Browser.lnk"));
            }
        }
    }
    if let Ok(userprofile) = std::env::var("USERPROFILE") {
        let p1 = PathBuf::from(&userprofile).join("Desktop").join("Qualium Quantum Browser.lnk");
        let p2 = PathBuf::from(&userprofile).join("OneDrive").join("Desktop").join("Qualium Quantum Browser.lnk");
        if !paths.contains(&p1) && p1.parent().map(|d| d.exists()).unwrap_or(false) {
            paths.push(p1);
        }
        if !paths.contains(&p2) && p2.parent().map(|d| d.exists()).unwrap_or(false) {
            paths.push(p2);
        }
    }
    paths
}

pub fn get_desktop_shortcut_path() -> Option<PathBuf> {
    get_desktop_shortcut_paths().into_iter().next()
}

pub fn get_start_menu_shortcut_dir() -> Option<PathBuf> {
    if let Ok(appdata) = std::env::var("APPDATA") {
        Some(PathBuf::from(appdata).join("Microsoft").join("Windows").join("Start Menu").join("Programs").join("Qualium"))
    } else {
        None
    }
}
