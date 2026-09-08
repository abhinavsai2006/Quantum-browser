# Qaulium Quantum Browser v5 — Data-Flow & Non-Retention Specification

**Document Version:** 5.0.0  
**Classification:** Privacy Architecture & Data Policy  
**Author:** Qaulium AI Engineering Team  

---

## 1. Core Principle: "The Database We Don't Build"

The foundational invariant of Qaulium Quantum Browser v5 is:
> **The most important database is the one we refuse to build.**

Mainstream commercial browsers maintain centralized telemetry, sync databases, and behavioral tracking stores that link user identities to browsing actions. Qaulium's architecture is mathematically and structurally designed to make the creation or reconstruction of such databases impossible.

### Explicitly Prohibited Centralized Schema
Qaulium servers and infrastructure shall **never** define, host, or persist tables matching or equivalent to:

```text
PROHIBITED SCHEMA:
┌────────────────────────────────────────────────────────────────────────┐
│ user_id │ ip_address │ timestamp │ search_query │ visited_url │ target │
└────────────────────────────────────────────────────────────────────────┘
```

---

## 2. Ephemeral Session Lifecycle

By default, browsing data lives strictly in ephemeral process memory:

```text
Browser Launch
      │
      ▼
 Create Ephemeral Memory Context
 (RAM-only SQLite, partitioned session cookies)
      │
      ├── Active Browsing Session
      │
      ▼
 Browser Termination or "New Identity"
      │
      ▼
 Atomic Memory Wipe (zeroize_memory)
 (Zero traces written to permanent disk storage)
```

### Data Category Defaults

| Data Category | Default Storage Location | Persistence Policy | Cloud Synchronization |
| :--- | :--- | :--- | :--- |
| **Browsing History** | Memory only (None on disk) | `DISABLED` (Purged immediately) | **NEVER** |
| **Search Queries** | Ephemeral RAM | Discarded after result render | **NEVER** |
| **HTTP Cookies** | Session memory | Destroyed on tab/browser close | **NEVER** |
| **Local Storage / Cache** | First-Party Isolated Sandbox | Cleared on session exit | **NEVER** |
| **Saved Passwords** | Local Encrypted Vault | User Master Password required | **OFF (Opt-in only)** |
| **Bookmarks** | Local Disk (JSON/SQLite) | Persistent local device only | **OFF (Opt-in only)** |
| **Telemetry / Analytics**| Stripped / Non-existent | Zero collection | **NEVER** |

---

## 3. Private Search Gateway Architecture

When a user executes a search through Qaulium Search, the query path decouples identity from search intent:

```text
User Device
     │
     │ 1. Search Query ("quantum cryptography")
     ▼
Qaulium Anonymous Network (Guard -> Relay -> Exit)
     │
     │ 2. Exit Node IP (User IP completely stripped)
     ▼
Qaulium Search Gateway
     │
     ├── Inbound Sanitizer (Strips headers, cookies, User-Agent fingerprints)
     ├── Query Anonymizer (Randomizes order, batches queries)
     │
     ├── Multi-Provider Upstream Querying
     │     ├── Provider A (Encrypted REST API)
     │     ├── Provider B (Encrypted REST API)
     │     └── Qaulium Index (Local index)
     │
     ├── Result Aggregator & Tracker Cleaner
     │     (Strips redirect tracking parameters, affiliate tokens, click beacons)
     │
     │ 3. Clean Result Payload
     ▼
Qaulium Anonymous Network
     │
     │ 4. Return via encrypted multi-hop circuit
     ▼
User Device
```

### Gateway Non-Persistence Guarantee
1. **No IP Logging:** The Search Gateway only sees the IP address of the circuit Exit Node, never the user's origin IP.
2. **Zero Query Persistence:** Queries are processed in volatile memory and discarded immediately once the result stream completes.
3. **No Long-Lived Session IDs:** Search requests contain no persistent cookies, user tokens, or device identifiers.

---

## 4. Local Encrypted Vault Specification (`crates/qualium-vault`)

If a user explicitly chooses to save credentials or enable optional local history:

```text
Master Password
     │
     ▼
Argon2id Key Derivation
 (Memory: 64 MB, Iterations: 3, Parallelism: 4)
     │
     ▼
256-bit Vault Key (K_vault)
     │
     ▼
ChaCha20-Poly1305 AEAD Engine
     │
 ┌───┴────────────────────────┐
 ▼                            ▼
Encrypted Password Vault    Encrypted Local History Vault
(local device disk only)    (local device disk only)
```

- **Zero Knowledge:** Qaulium AI holds no recovery keys, escrow servers, or master backdoors.
- **Zero Cloud Leak:** Vault files (`qualium-vault.db`) are never transmitted across the network unless the user explicitly configures self-hosted sync.
