//! Qualium Quantum Browser v5 — Installer & Uninstaller Library
//! Provides Win32 native integration, installation engine, manifest tracking, and uninstaller engine.

pub mod win32;
pub mod manifest;
pub mod engine;
pub mod uninstaller_engine;
pub mod win32_gui;

pub use manifest::{InstallManifest, ManifestFileEntry};
pub use engine::{InstallEngine, InstallOptions, PayloadMetrics};
pub use uninstaller_engine::UninstallerEngine;
