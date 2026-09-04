//! Network Integration & Circuit Isolation Verification

#[cfg(test)]
mod tests {
    use qualium_network::{CircuitController, PrivacyDnsResolver};

    #[tokio::test]
    async fn test_circuit_isolation_between_distinct_domains() {
        let controller = CircuitController::new();

        let domain_1 = "financial-institution.com";
        let domain_2 = "news-portal.org";
        let domain_3 = "search-engine.privacy";

        let c1 = controller.get_circuit_for_destination(domain_1).await;
        let c2 = controller.get_circuit_for_destination(domain_2).await;
        let c3 = controller.get_circuit_for_destination(domain_3).await;

        assert_ne!(c1.circuit_id, c2.circuit_id);
        assert_ne!(c2.circuit_id, c3.circuit_id);
        assert_ne!(c1.circuit_id, c3.circuit_id);

        // Verification of 3-hop structure
        assert!(c1.guard.nickname.contains("guard"));
        assert!(c1.relay.nickname.contains("relay"));
        assert!(c1.exit.nickname.contains("exit"));
    }

    #[tokio::test]
    async fn test_dns_no_external_os_leak() {
        let resolver = PrivacyDnsResolver::default();
        let ips = resolver.resolve("example.com").await.expect("dns resolve");
        assert_eq!(ips[0], "93.184.216.34");
        assert!(resolver.is_ecs_stripped(), "EDNS Client Subnet must be stripped");
    }

    #[tokio::test]
    async fn test_circuit_rotation_isolation() {
        let controller = CircuitController::new();
        let c1 = controller.get_circuit_for_destination("example.org").await;
        
        // Trigger manual rotation / identity reset
        controller.rotate_all_circuits().await;

        let c2 = controller.get_circuit_for_destination("example.org").await;
        assert_ne!(c1.circuit_id, c2.circuit_id, "Rotated circuit must have a distinct circuit ID");
    }
}
