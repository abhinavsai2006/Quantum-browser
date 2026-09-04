//! Multi-Hop Circuit Lifecycle & Stream Isolation Controller

use qualium_core::{CircuitNode, CircuitTopology, VerificationState};
use rand::seq::SliceRandom;
use rand::thread_rng;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

#[derive(Clone)]
pub struct CircuitController {
    relays_pool: Vec<CircuitNode>,
    active_circuits: Arc<RwLock<HashMap<Uuid, CircuitTopology>>>,
    destination_circuits: Arc<RwLock<HashMap<String, Uuid>>>, // Stream isolation by eTLD+1
    current_primary_circuit: Arc<RwLock<Option<CircuitTopology>>>,
}

impl CircuitController {
    pub fn new() -> Self {
        let relays_pool = vec![
            CircuitNode {
                nickname: "qualium-guard-01-reykjavik".into(),
                fingerprint: "9A4F3B2C11D94E8A7F2B0C4D5E6F1A2B3C4D5E6F".into(),
                country_code: "IS".into(),
                ip_redacted: "185.220.xxx.12".into(),
                rtt_ms: 28,
            },
            CircuitNode {
                nickname: "qualium-guard-02-helsinki".into(),
                fingerprint: "8B3E2A1D22C83D7B6E1A9B3C4D5E0F1A2B3C4D5E".into(),
                country_code: "FI".into(),
                ip_redacted: "95.216.xxx.45".into(),
                rtt_ms: 34,
            },
            CircuitNode {
                nickname: "qualium-relay-09-zurich".into(),
                fingerprint: "7E1C8D4A33B72C6A5D0F8A2B3C4D9E0F1A2B3C4D".into(),
                country_code: "CH".into(),
                ip_redacted: "179.43.xxx.88".into(),
                rtt_ms: 45,
            },
            CircuitNode {
                nickname: "qualium-relay-10-oslo".into(),
                fingerprint: "6D0B7C3944A61B594C9E791A2B3C8D9E0F1A2B3C".into(),
                country_code: "NO".into(),
                ip_redacted: "185.189.xxx.92".into(),
                rtt_ms: 50,
            },
            CircuitNode {
                nickname: "qualium-exit-04-stockholm".into(),
                fingerprint: "3F8A9B1D55950A483B8D68091A2B7C8D9E0F1A2B".into(),
                country_code: "SE".into(),
                ip_redacted: "193.187.xxx.201".into(),
                rtt_ms: 62,
            },
            CircuitNode {
                nickname: "qualium-exit-05-amsterdam".into(),
                fingerprint: "2E798A0C6684F9372A7C57F8091A6B7C8D9E0F1A".into(),
                country_code: "NL".into(),
                ip_redacted: "185.220.xxx.77".into(),
                rtt_ms: 58,
            },
        ];

        Self {
            relays_pool,
            active_circuits: Arc::new(RwLock::new(HashMap::new())),
            destination_circuits: Arc::new(RwLock::new(HashMap::new())),
            current_primary_circuit: Arc::new(RwLock::new(None)),
        }
    }

    /// Build a 3-hop circuit (Guard -> Relay -> Exit) with distinct geographical and relay identities
    pub async fn establish_circuit(&self) -> CircuitTopology {
        let (guard, relay, exit) = {
            let mut rng = thread_rng();

            let guards: Vec<&CircuitNode> = self
                .relays_pool
                .iter()
                .filter(|n| n.nickname.contains("guard"))
                .collect();
            let relays: Vec<&CircuitNode> = self
                .relays_pool
                .iter()
                .filter(|n| n.nickname.contains("relay"))
                .collect();
            let exits: Vec<&CircuitNode> = self
                .relays_pool
                .iter()
                .filter(|n| n.nickname.contains("exit"))
                .collect();

            let guard = (*guards.choose(&mut rng).unwrap_or(&&self.relays_pool[0])).clone();
            let relay = (*relays.choose(&mut rng).unwrap_or(&&self.relays_pool[2])).clone();
            let exit = (*exits.choose(&mut rng).unwrap_or(&&self.relays_pool[4])).clone();
            (guard, relay, exit)
        };

        let circuit_id = Uuid::new_v4();
        let topology = CircuitTopology {
            circuit_id,
            guard,
            relay,
            exit,
            state: VerificationState::Active,
            established_at_epoch_ms: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64,
            streams_count: 1,
        };

        let mut active = self.active_circuits.write().await;
        active.insert(circuit_id, topology.clone());

        let mut primary = self.current_primary_circuit.write().await;
        *primary = Some(topology.clone());

        topology
    }

    /// Retrieve or allocate an isolated circuit for a given top-level destination (stream isolation)
    pub async fn get_circuit_for_destination(&self, destination_domain: &str) -> CircuitTopology {
        let mut dest_map = self.destination_circuits.write().await;
        if let Some(cid) = dest_map.get(destination_domain) {
            let active = self.active_circuits.read().await;
            if let Some(topo) = active.get(cid) {
                return topo.clone();
            }
        }

        // Establish new isolated circuit for destination
        let new_topo = self.establish_circuit().await;
        dest_map.insert(destination_domain.to_string(), new_topo.circuit_id);
        new_topo
    }

    /// Get current primary circuit topology for live telemetry / UI
    pub async fn get_primary_circuit(&self) -> Option<CircuitTopology> {
        let primary = self.current_primary_circuit.read().await;
        primary.clone()
    }

    /// Rotate circuit explicitly on Identity Reset
    pub async fn rotate_all_circuits(&self) {
        let mut active = self.active_circuits.write().await;
        active.clear();
        let mut dest_map = self.destination_circuits.write().await;
        dest_map.clear();
        drop(active);
        drop(dest_map);

        self.establish_circuit().await;
    }
}

impl Default for CircuitController {
    fn default() -> Self {
        Self::new()
    }
}
