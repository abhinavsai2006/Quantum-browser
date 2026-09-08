# Quantum Browser v5 — Comprehensive API Specification

**Document Version:** 5.0.0  
**Classification:** Engineering Interface Specification  
**Author:** Quantum Browser Project Engineering Team  

---

## 1. Browser ↔ Network Security Daemon IPC Protocol

The Browser Engine communicates with `qualium-daemon` over a secured local Inter-Process Communication (IPC) transport:
- **Windows:** Named Pipe (`\\.\pipe\qualium-daemon-ipc`)
- **Linux / macOS:** Unix Domain Socket (`/run/user/<UID>/qualium-daemon.sock` or `~/.qualium/daemon.sock`)

### 1.1 Framing Specification
Messages are framed using a 4-byte big-endian length prefix followed by a UTF-8 encoded JSON payload:

```text
┌───────────────────────────┬────────────────────────────────────────────┐
│ 4-byte Length (u32, BE)  │ Payload (JSON Object, UTF-8)               │
└───────────────────────────┴────────────────────────────────────────────┘
```

### 1.2 IPC Request Messages (`Browser -> Daemon`)

```json
// 1. Query Real-Time Security Status
{
  "type": "StatusQuery",
  "sessionId": "b48f9d0c-83b4-4b95-a24c-9f8bc29df143"
}

// 2. Trigger New Identity & Circuit Flush
{
  "type": "NewIdentity",
  "reason": "user_action",
  "preservePinned": false
}

// 3. Switch Privacy Level
{
  "type": "SetPrivacyLevel",
  "level": "private" // "balanced" | "private" | "maximum"
}

// 4. Force Circuit Rotation for Specific Origin
{
  "type": "RotateCircuit",
  "domain": "example.com"
}
```

### 1.3 IPC Response Messages (`Daemon -> Browser`)

```json
// 1. Authoritative Security State (Feeds Shield Q Popover)
{
  "type": "SecurityStatus",
  "data": {
    "ready": true,
    "pqcSupport": true,
    "pqcState": "Negotiated",
    "pqcAlgorithm": "ML-KEM-768 (NIST FIPS 203)",
    "classicalAlgorithm": "X25519 (RFC 7748)",
    "handshakeState": "Established",
    "sessionId": "b48f9d0c-83b4-4b95-a24c-9f8bc29df143",
    "circuitState": "Active",
    "circuitId": "circ-77a1",
    "guardNode": "relay-alpha.qualium.net (NL)",
    "relayNode": "relay-bravo.qualium.net (SE)",
    "exitNode": "relay-charlie.qualium.net (IS)",
    "proxyState": "Listening",
    "proxyPort": 9150,
    "proxyEndpoint": "127.0.0.1:9150",
    "dnsState": "Protected (Remote DNS in Circuit)",
    "webrtcState": "Protected (ICE Host Filtering)",
    "negotiatedAtEpochMs": 1773099600000
  }
}

// 2. Filter & Blocking Live Metrics
{
  "type": "FilterMetrics",
  "data": {
    "adsBlocked": 24,
    "trackersBlocked": 13,
    "malwareBlocked": 0,
    "fingerprintProbesNeutralized": 7
  }
}
```

---

## 2. Daemon ↔ Relay Onion Routing Protocol

Quantum circuits employ fixed-size **512-byte cells** with multi-layer symmetric AEAD encapsulation:

```text
┌──────────────┬──────────────┬──────────────┬───────────────────────────┐
│ Circuit ID   │ Command Byte │ Length (u16) │ Cell Payload (Variable)   │
│ (4 bytes)    │ (1 byte)     │ (2 bytes)    │ Padding to 512 bytes      │
└──────────────┴──────────────┴──────────────┴───────────────────────────┘
```

### 2.1 Post-Quantum Hybrid Circuit Creation Handshake

```text
Client                                Guard Relay
  │                                        │
  │─── CELL_CREATE_PQC ───────────────────>│
  │    Payload:                            │
  │    - Client X25519 Ephemeral Pubkey    │
  │    - ML-KEM-768 Encapsulation Request  │
  │                                        │
  │<── CELL_CREATED_PQC ───────────────────│
  │    Payload:                            │
  │    - Relay X25519 Ephemeral Pubkey     │
  │    - ML-KEM-768 Ciphertext             │
  │    - Relay Authentication Signature    │
  │                                        │
  ▼                                        ▼
 Derive Shared Secret:
 K_hybrid = HKDF-Extract(salt, K_X25519 || K_ML-KEM)
 Keys: Forward AEAD key, Backward AEAD key, Integrity digest
```

---

## 3. Cryptographic Abstraction Interfaces (`crates/qualium-crypto`)

To satisfy Requirement **P3 (Cryptographic Agility)** and Section 21:

```rust
/// Abstract Key Encapsulation Mechanism interface
pub trait KemEngine: Send + Sync {
    fn algorithm_name(&self) -> &'static str;
    fn public_key_len(&self) -> usize;
    fn ciphertext_len(&self) -> usize;
    fn shared_secret_len(&self) -> usize;

    /// Generate ephemeral public/private keypair
    fn generate_keypair(&self) -> Result<(Vec<u8>, Vec<u8>), CryptoError>;

    /// Encapsulate shared secret against peer's public key
    fn encapsulate(&self, peer_pk: &[u8]) -> Result<(Vec<u8>, Vec<u8>), CryptoError>;

    /// Decapsulate shared secret using client's private key
    fn decapsulate(&self, sk: &[u8], ciphertext: &[u8]) -> Result<Vec<u8>, CryptoError>;
}

/// Hybrid Classical + Post-Quantum Key Exchange Coordinator
pub trait HybridHandshakeCoordinator: Send + Sync {
    /// Initiate a hybrid client handshake payload (X25519 + ML-KEM-768)
    fn create_client_handshake(&self) -> Result<ClientHandshakePayload, CryptoError>;

    /// Complete the handshake with peer's response and derive channel AEAD keys
    fn finalize_client_handshake(
        &self,
        state: HandshakeState,
        server_response: ServerHandshakePayload,
    ) -> Result<HybridSessionKeys, CryptoError>;
}

/// Symmetric Authenticated Encryption with Associated Data (AEAD)
pub trait AeadCipher: Send + Sync {
    fn key_len(&self) -> usize;
    fn nonce_len(&self) -> usize;
    fn tag_len(&self) -> usize;

    fn encrypt(&self, key: &[u8], nonce: &[u8], plaintext: &[u8], aad: &[u8]) -> Result<Vec<u8>, CryptoError>;
    fn decrypt(&self, key: &[u8], nonce: &[u8], ciphertext: &[u8], aad: &[u8]) -> Result<Vec<u8>, CryptoError>;
}
```
