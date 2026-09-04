//! Qualium Core Domain Types & System Abstractions
//!
//! Provides the primary types for privacy levels, live security metrics,
//! cryptographic status, anonymity circuits, and anti-fingerprint profiles.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Privacy operational levels supported by Qualium Quantum Browser.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum PrivacyLevel {
    /// Level 1: Balanced mode. Native ad/tracker blocking, HTTPS enforcement, basic fingerprint shielding.
    Balanced,
    /// Level 2: Private mode (Default). Anonymous circuit routing, zero history, zero telemetry, strict storage partitioning, enhanced fingerprint bucket normalization.
    #[default]
    Private,
    /// Level 3: Maximum mode. Aggressive script restrictions, strict first-party isolation, maximum circuit isolation, strict fingerprint normalization.
    Maximum,
}

/// Verification state for a security or cryptographic subsystem.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VerificationState {
    Protected,
    Negotiated,
    Active,
    Disabled,
    Unavailable,
    Degraded,
    NotVerified,
}

/// Live Cryptographic Status reported by the Post-Quantum engine.
/// Strictly distinguishes between local library capability ("Available"),
/// privacy network relay transport ("Negotiated"), and origin website TLS ("Host-Dependent").
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CryptoStatus {
    pub classical_kex: String,
    pub classical_state: VerificationState,
    pub pq_capability: String,
    pub pq_capability_state: VerificationState,
    pub pq_transport: String,
    pub pq_transport_state: VerificationState,
    pub website_tls: String,
    pub website_tls_state: VerificationState,
    pub hybrid_mode: String,
    pub hybrid_state: VerificationState,
    pub aead: String,
    pub aead_state: VerificationState,
    pub protocol_version: String,
}

impl Default for CryptoStatus {
    fn default() -> Self {
        Self {
            classical_kex: "X25519 (RFC 7748)".to_string(),
            classical_state: VerificationState::Negotiated,
            pq_capability: "Available (ML-KEM-768 / NIST FIPS 203)".to_string(),
            pq_capability_state: VerificationState::Active,
            pq_transport: "Negotiated (Relay Tunnel)".to_string(),
            pq_transport_state: VerificationState::Negotiated,
            website_tls: "Classical (ECDHE) / Hybrid / PQ (Host-Dependent)".to_string(),
            website_tls_state: VerificationState::Protected,
            hybrid_mode: "X25519+ML-KEM-768-HKDF-SHA384".to_string(),
            hybrid_state: VerificationState::Active,
            aead: "ChaCha20-Poly1305 (RFC 8439)".to_string(),
            aead_state: VerificationState::Active,
            protocol_version: "Qualium-PQ-v5.0".to_string(),
        }
    }
}

/// Single hop node in an anonymity circuit.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircuitNode {
    pub nickname: String,
    pub fingerprint: String,
    pub country_code: String,
    pub ip_redacted: String,
    pub rtt_ms: u32,
}

/// Active circuit topology.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircuitTopology {
    pub circuit_id: Uuid,
    pub guard: CircuitNode,
    pub relay: CircuitNode,
    pub exit: CircuitNode,
    pub state: VerificationState,
    pub established_at_epoch_ms: u64,
    pub streams_count: u32,
}

impl Default for CircuitTopology {
    fn default() -> Self {
        Self {
            circuit_id: Uuid::new_v4(),
            guard: CircuitNode {
                nickname: "qualium-guard-01".to_string(),
                fingerprint: "9A4F3B2C...".to_string(),
                country_code: "IS".to_string(),
                ip_redacted: "185.220.xxx.12".to_string(),
                rtt_ms: 28,
            },
            relay: CircuitNode {
                nickname: "qualium-relay-09".to_string(),
                fingerprint: "7E1C8D4A...".to_string(),
                country_code: "CH".to_string(),
                ip_redacted: "179.43.xxx.88".to_string(),
                rtt_ms: 45,
            },
            exit: CircuitNode {
                nickname: "qualium-exit-04".to_string(),
                fingerprint: "3F8A9B1D...".to_string(),
                country_code: "SE".to_string(),
                ip_redacted: "193.187.xxx.201".to_string(),
                rtt_ms: 62,
            },
            state: VerificationState::Active,
            established_at_epoch_ms: 1724600000000,
            streams_count: 3,
        }
    }
}

/// Real-time live metrics displayed in the 🛡 Q-Security Hub and Security Dashboard.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityMetrics {
    pub anonymous_routing: VerificationState,
    pub circuit_status: VerificationState,
    pub dns_protection: VerificationState,
    pub webrtc_protection: VerificationState,
    pub ads_blocked_count: u64,
    pub trackers_blocked_count: u64,
    pub fingerprint_defense: VerificationState,
    pub history_retention: String,
    pub telemetry_status: String,
    pub phishing_shield: VerificationState,
    pub malware_shield: VerificationState,
    pub downloads_shield: VerificationState,
    pub crypto: CryptoStatus,
    pub circuit: Option<CircuitTopology>,
}

impl Default for SecurityMetrics {
    fn default() -> Self {
        Self {
            anonymous_routing: VerificationState::Protected,
            circuit_status: VerificationState::Active,
            dns_protection: VerificationState::Protected,
            webrtc_protection: VerificationState::Protected,
            ads_blocked_count: 0,
            trackers_blocked_count: 0,
            fingerprint_defense: VerificationState::Active,
            history_retention: "OFF (Zero Retention)".to_string(),
            telemetry_status: "OFF (Zero Telemetry)".to_string(),
            phishing_shield: VerificationState::Protected,
            malware_shield: VerificationState::Protected,
            downloads_shield: VerificationState::Protected,
            crypto: CryptoStatus::default(),
            circuit: Some(CircuitTopology::default()),
        }
    }
}

/// Coherent population-based Anti-Fingerprinting profile.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualiumFingerprintProfile {
    pub viewport_width: u32,
    pub viewport_height: u32,
    pub screen_width: u32,
    pub screen_height: u32,
    pub device_pixel_ratio: f32,
    pub user_agent: String,
    pub platform: String,
    pub language: String,
    pub timezone: String,
    pub hardware_concurrency: u32,
    pub device_memory_gb: u32,
    pub canvas_noise_seed: u64,
    pub audio_noise_seed: u64,
    pub webgl_vendor: String,
    pub webgl_renderer: String,
}

impl Default for QualiumFingerprintProfile {
    fn default() -> Self {
        Self {
            viewport_width: 1280,
            viewport_height: 720,
            screen_width: 1920,
            screen_height: 1080,
            device_pixel_ratio: 1.0,
            user_agent: "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:140.0) Gecko/20100101 Firefox/140.0".to_string(),
            platform: "Win32".to_string(),
            language: "en-US".to_string(),
            timezone: "UTC".to_string(),
            hardware_concurrency: 4,
            device_memory_gb: 8,
            canvas_noise_seed: 0x4155_414C_4955_4D35,
            audio_noise_seed: 0x5155_414E_5455_4D35,
            webgl_vendor: "Qualium Privacy Normalized".to_string(),
            webgl_renderer: "Gecko WebRender (Standardized)".to_string(),
        }
    }
}

/// Identity context representing isolated session state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdentityContext {
    pub context_id: Uuid,
    pub created_at_epoch_ms: u64,
    pub partition_key: String,
    pub privacy_level: PrivacyLevel,
    pub circuit_id: Uuid,
}
