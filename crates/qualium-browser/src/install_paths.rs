//! Install paths helper for Quantum Browser
//! Determines appropriate install and data directories, supporting
//! portable mode and fallback to per‑user locations.

use std::env;
use std::fs;
use std::path::PathBuf;

/// Returns the default installation directory.
/// On Windows this is `%LOCALAPPDATA%\Programs\Qaulium`.
/// Falls back to `C:\Program Files\Qaulium` if the environment variable is missing.
pub fn default_install_dir() -> PathBuf {
    if let Ok(local_appdata) = env::var("LOCALAPPDATA") {
        PathBuf::from(local_appdata).join("Programs").join("Qaulium")
    } else {
        PathBuf::from(r"C:\Program Files\Qaulium")
    }
}

/// Returns true if running in portable mode
pub fn is_portable_mode() -> bool {
    if let Ok(current_exe) = env::current_exe() {
        if let Some(parent) = current_exe.parent() {
            if parent.join("portable.dat").exists() || parent.join("Data").exists() {
                return true;
            }
        }
    }
    false
}

/// Returns a writable data directory.
/// In portable mode, returns `<install_dir>\Data`.
/// Otherwise defaults to `%LOCALAPPDATA%\Programs\Qaulium\Data`.
pub fn data_dir() -> PathBuf {
    // 1. Portable mode detection
    if let Ok(current_exe) = env::current_exe() {
        if let Some(parent) = current_exe.parent() {
            if parent.join("portable.dat").exists() || parent.join("Data").exists() {
                let local_data = parent.join("Data");
                let _ = fs::create_dir_all(&local_data);
                return local_data;
            }
        }
    }

    // 2. Default standard path: %LOCALAPPDATA%\Programs\Qaulium\Data
    if let Ok(local_appdata) = env::var("LOCALAPPDATA") {
        let p = PathBuf::from(local_appdata).join("Programs").join("Qaulium").join("Data");
        let _ = fs::create_dir_all(&p);
        return p;
    }

    // 3. Fallback
    let fallback = default_install_dir().join("Data");
    let _ = fs::create_dir_all(&fallback);
    fallback
}
