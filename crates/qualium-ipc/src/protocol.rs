//! Strongly-Typed Authenticated Local IPC Protocol

use qualium_core::{CircuitTopology, IdentityContext, PrivacyLevel, QualiumFingerprintProfile, SecurityMetrics};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload")]
pub enum IpcRequest {
    /// Ping/Heartbeat
    Ping { timestamp: u64 },
    /// Authenticate IPC session with token
    Authenticate { auth_token: String },
    /// Request live security & cryptographic metrics for the 🛡 Q toolbar button
    GetSecurityMetrics,
    /// Request active circuit topology
    GetCircuitTopology,
    /// Request identity rotation (New Identity)
    RotateIdentity { old_context_id: Option<Uuid> },
    /// Set browser privacy level (Balanced, Private, Maximum)
    SetPrivacyLevel { level: PrivacyLevel },
    /// Request coherent anti-fingerprinting profile
    GetFingerprintProfile,
    /// Query content filter for a URL
    CheckUrlFilter { url: String, first_party: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload")]
#[allow(clippy::large_enum_variant)]
pub enum IpcResponse {
    Pong { timestamp: u64 },
    Authenticated { success: bool, session_id: Uuid },
    SecurityMetrics(SecurityMetrics),
    CircuitTopology(Option<CircuitTopology>),
    IdentityRotated { new_context: IdentityContext },
    PrivacyLevelChanged { level: PrivacyLevel },
    FingerprintProfile(QualiumFingerprintProfile),
    UrlFilterResult { allowed: bool, category: Option<String> },
    Error { message: String },
}
