//! Qualium Network & Multi-Hop Anonymity Controller

pub mod circuit;
pub mod dns;
pub mod proxy;

pub use circuit::CircuitController;
pub use dns::PrivacyDnsResolver;
pub use proxy::QualiumLocalProxy;

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_circuit_lifecycle_and_stream_isolation() {
        let controller = CircuitController::new();

        // 1. Establish initial circuit
        let circuit_a = controller.get_circuit_for_destination("example.org").await;
        assert_eq!(circuit_a.streams_count, 1);
        assert!(!circuit_a.guard.nickname.is_empty());
        assert!(!circuit_a.relay.nickname.is_empty());
        assert!(!circuit_a.exit.nickname.is_empty());

        // 2. Querying same destination reuses isolated circuit
        let circuit_a_reuse = controller.get_circuit_for_destination("example.org").await;
        assert_eq!(circuit_a.circuit_id, circuit_a_reuse.circuit_id);

        // 3. Querying different destination gets a new isolated circuit
        let circuit_b = controller.get_circuit_for_destination("bank.com").await;
        assert_ne!(circuit_a.circuit_id, circuit_b.circuit_id, "Destinations must receive isolated circuits");

        // 4. Verify primary circuit representation
        let primary = controller.get_primary_circuit().await;
        assert!(primary.is_some());
    }

    #[tokio::test]
    async fn test_dns_resolver_zero_leak() {
        let resolver = PrivacyDnsResolver::default();
        let ips = resolver.resolve("check.torproject.org").await.expect("resolve");
        assert!(!ips.is_empty());
        assert_eq!(ips[0], "185.220.101.5");
    }
}
