//! Qualium Authenticated Local IPC Bridge & Multi-Layer Security Architecture

pub mod protocol;
pub mod security;

pub use protocol::{IpcRequest, IpcResponse};
pub use security::{
    AuthenticatedSession, CallerPrivilegeLevel, IpcRateLimiter, IpcSecurityManager, PeerCredentials,
    SecureIpcFrame, MAX_IPC_FRAME_BYTES,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ipc_serialization_roundtrip() {
        let req = IpcRequest::GetSecurityMetrics;
        let json = serde_json::to_string(&req).expect("serialize");
        let deserialized: IpcRequest = serde_json::from_str(&json).expect("deserialize");
        assert!(matches!(deserialized, IpcRequest::GetSecurityMetrics));
    }

    #[tokio::test]
    async fn test_ipc_defense_in_depth_lifecycle() {
        let manager = IpcSecurityManager::new(1000);

        // 1. Unauthorized peer rejected by OS UID check
        let rogue_peer = PeerCredentials::new(4567, 9999, CallerPrivilegeLevel::BrowserParentChrome, 1000);
        assert!(manager.establish_session(rogue_peer).await.is_err());

        // 2. Sandboxed Renderer process is strictly forbidden from privileged IPC
        let renderer_peer = PeerCredentials::new(2345, 1000, CallerPrivilegeLevel::ContentRenderer, 1000);
        let renderer_err = manager.establish_session(renderer_peer).await;
        assert!(renderer_err.is_err(), "Renderer process must be denied connection to privileged IPC");

        // 3. Authorized BrowserParentChrome peer establishes ephemeral session
        let authorized_peer = PeerCredentials::new(1234, 1000, CallerPrivilegeLevel::BrowserParentChrome, 1000);
        let (session_id, session_key) = manager.establish_session(authorized_peer).await.expect("handshake");

        // 4. Create valid signed frame (seq: 1)
        let frame1 = SecureIpcFrame::create_and_sign(
            session_id,
            1,
            r#"{"type":"GetSecurityMetrics"}"#.to_string(),
            &session_key,
        ).expect("sign frame1");

        let payload1 = manager.process_incoming_frame(&frame1).await.expect("verify frame1");
        assert_eq!(payload1, r#"{"type":"GetSecurityMetrics"}"#);

        // 5. Replay attack with same sequence number (seq: 1) MUST FAIL
        let replay_result = manager.process_incoming_frame(&frame1).await;
        assert!(replay_result.is_err(), "Replay frame must be rejected");

        // 6. Out-of-order sequence attack (seq: 0) MUST FAIL
        let old_seq_frame = SecureIpcFrame::create_and_sign(
            session_id,
            0,
            r#"{"type":"GetCircuitTopology"}"#.to_string(),
            &session_key,
        ).expect("sign old seq");
        assert!(manager.process_incoming_frame(&old_seq_frame).await.is_err());

        // 7. Tampered frame MUST FAIL HMAC check
        let mut tampered_frame = SecureIpcFrame::create_and_sign(
            session_id,
            2,
            r#"{"type":"GetCircuitTopology"}"#.to_string(),
            &session_key,
        ).expect("sign frame2");
        tampered_frame.payload_json = r#"{"type":"TamperedPayload"}"#.to_string();
        assert!(manager.process_incoming_frame(&tampered_frame).await.is_err());

        // 8. Oversized payload (> 64KB) MUST FAIL creation / verification
        let huge_payload = "A".repeat(MAX_IPC_FRAME_BYTES + 1);
        let oversized_result = SecureIpcFrame::create_and_sign(session_id, 3, huge_payload, &session_key);
        assert!(oversized_result.is_err(), "Oversized payload must be rejected");

        // 9. Legitimate next sequence (seq: 2) SUCCEEDS
        let frame2 = SecureIpcFrame::create_and_sign(
            session_id,
            2,
            r#"{"type":"GetCircuitTopology"}"#.to_string(),
            &session_key,
        ).expect("sign frame2 legitimate");
        assert!(manager.process_incoming_frame(&frame2).await.is_ok());
    }
}
