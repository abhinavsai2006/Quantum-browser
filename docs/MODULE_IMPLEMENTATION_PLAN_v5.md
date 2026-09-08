# Qaulium Quantum Browser v5 — Module Implementation Plan

**Document Version:** 5.0.0  
**Classification:** Engineering Implementation Roadmap  
**Author:** Qaulium AI Engineering Team  

---

## 1. Repository Layout Mapping (Section 42 Alignment)

The repository layout defined in Section 42 maps into the codebase across native Rust workspace crates, the browser runtime engine, and the chrome frontend:

```text
qaulium-quantum-browser/
│
├── browser/              --> crates/qualium-browser (Hardened Gecko launcher & pref engine)
├── ui/                   --> qualium/chrome/content (XHTML/HTML5, CSS, Obsidian Glassmorphism)
│
├── privacy/              --> Hardened runtime engine policies & crates/qualium-core
│   ├── fingerprint/      --> Population-based bucketing, letterboxing, canvas/audio noise
│   ├── cookies/          --> First-party storage isolation (privacy.firstparty.isolate)
│   ├── storage/          --> Dynamic partitioning & ephemeral memory context
│   ├── identity/         --> Atomic New Identity session wipe controller
│   └── permissions/      --> Strict origin permission isolation
│
├── blocking/             --> crates/qualium-filter
│   ├── ads/              --> EasyList bloom filters & rule trie
│   ├── trackers/         --> EasyPrivacy tracking pixel & beacon blocker
│   ├── scripts/          --> Aggressive script restriction & AST analysis
│   └── filters/          --> Rule compilation & memory-mapped filter lists
│
├── network/              --> crates/qualium-network & crates/qualium-daemon
│   ├── client/           --> SOCKS5/HTTP local proxy listener
│   ├── circuit/          --> Multi-hop circuit builder & stream isolation
│   ├── relay/            --> Guard, Relay, and Exit node descriptors
│   ├── directory/        --> Signed consensus parser & authority client
│   └── transport/        --> Hybrid post-quantum encrypted stream transport
│
├── crypto/               --> crates/qualium-crypto
│   ├── kem/              --> ML-KEM-512, ML-KEM-768 (primary), ML-KEM-1024 (FIPS 203)
│   ├── signatures/       --> ML-DSA (FIPS 204), Ed25519
│   ├── kdf/              --> HKDF-SHA256, Argon2id
│   └── providers/        --> CryptoProvider trait abstraction (Crypto-agility)
│
├── search/               --> qualium/chrome/content/newtab.js & search gateway integration
│   ├── gateway/          --> Anonymizing search proxy interface
│   ├── ranking/          --> Clean non-profiled result ranking
│   └── privacy/          --> Query sanitizer (zero IP/timestamp/user persistence)
│
├── security/             --> crates/qualium-vault, crates/qualium-ipc
│   ├── sandbox/          --> Multi-process OS sandboxing
│   ├── updater/          --> Cryptographically signed delta updater
│   ├── certificates/     --> Root trust store & post-quantum cert verification
│   └── threat_detection/ --> Heuristic phishing & malware domain screening
│
├── dashboard/            --> qualium/chrome/content/browser.xhtml (Shield Q Popover & Panel)
│
├── tests/                --> tests/
│   ├── unit/             --> Crate-level unit tests (42 tests passing)
│   ├── integration/      --> tests/network, tests/privacy
│   ├── fuzz/             --> tests/fuzz (KEM ciphertext, IPC frames, ABP rules)
│   ├── network/          --> DNS leak tests, circuit isolation tests
│   └── privacy/          --> P01-P14 privacy acceptance test suite
│
├── infrastructure/       --> scripts/ & distribution packaging
│   ├── docker/           --> Linux build containers
│   └── kubernetes/       --> CI/CD build matrix
│
└── docs/                 --> Complete v5 Architecture & SRS Specification Suite
    ├── SRS_v5.md
    ├── SYSTEM_ARCHITECTURE_v5.md
    ├── DATA_FLOW_AND_NON_RETENTION.md
    ├── API_SPECIFICATION_v5.md
    ├── MODULE_IMPLEMENTATION_PLAN_v5.md
    └── CRYPTOGRAPHIC_PROOF_AND_VERIFICATION.md
```

---

## 2. Workspace Rust Crates Summary

| Crate | Version | Role in v5 Architecture |
| :--- | :--- | :--- |
| `qualium-core` | `5.0.0` | Core domain types, `PrivacyLevel`, `PqcState`, live security metrics model |
| `qualium-crypto`| `5.0.0` | NIST FIPS 203 ML-KEM-768/512/1024, X25519 hybrid, ChaCha20-Poly1305, HKDF |
| `qualium-network`| `5.0.0`| Multi-hop circuits, SOCKS5 proxy, remote DNS resolution, stream isolation |
| `qualium-filter`| `5.0.0` | High-throughput ad/tracker blocking engine and bloom filter matching |
| `qualium-vault` | `5.0.0` | Local Argon2id + ChaCha20-Poly1305 encrypted credential and history vault |
| `qualium-ipc`   | `5.0.0` | Length-prefixed framed JSON IPC protocol between Browser and Daemon |
| `qualium-daemon`| `5.0.0` | Standalone background controller managing circuits, proxy, and telemetry |
| `qualium-browser`| `5.0.0`| Browser launcher, profile hardening, Firefox ESR prefs and flag injector |
| `qualium-installer`| `5.0.0`| Native Windows installer and uninstaller generator |

---

## 3. Engineering Acceptance Verification Gates

Each release build must pass all 4 verification gates:

1. **Gate 1 — Cryptographic Verification:**
   - ML-KEM-512, ML-KEM-768, and ML-KEM-1024 roundtrip key encapsulation tests.
   - Hybrid X25519 + ML-KEM handshake execution with forward secrecy verification.
   - Ciphertext tampering rejection and downgrade resistance checks.
2. **Gate 2 — Privacy & Leak Prevention:**
   - Test 01: URL telemetry elimination.
   - Test 02: Search query non-reconstruction.
   - Test 03: Zero DNS leaks (Remote DNS over circuit enforced).
   - Test 04: WebRTC candidate isolation (No local/public IP leak).
   - Test 05: Fingerprint entropy reduction (Normalized buckets).
   - Test 06: Zero default history retention on close.
3. **Gate 3 — Network Isolation & Fuzzing:**
   - Distinct domain circuit isolation (`bank.com` != `news.com`).
   - Fuzz testing of IPC frame ingestion and network cell parsing.
4. **Gate 4 — UI & Operational Reliability:**
   - Browser launches to single clean New Tab with zero modals.
   - Live Shield Q inspector renders all 12 metrics accurately.
   - Tab strip decoupling and single-tab closure stability.
