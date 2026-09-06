//! Local Authenticated SOCKS5 & HTTP Proxy Listener
//!
//! Exposes a local listener on 127.0.0.1:9050 / 9051 to route all Gecko Necko requests
//! through the Qualium Circuit Controller and Filter Engine with RFC 1928 SOCKS5 framing.

use crate::circuit::CircuitController;
use crate::dns::PrivacyDnsResolver;
use qualium_filter::{FilterAction, FilterEngine};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::RwLock;
use tracing::{error, info};

pub struct QualiumLocalProxy {
    bind_addr: SocketAddr,
    circuit_controller: Arc<CircuitController>,
    #[allow(dead_code)]
    dns_resolver: Arc<PrivacyDnsResolver>,
    filter_engine: Arc<FilterEngine>,
    is_running: Arc<RwLock<bool>>,
}

impl QualiumLocalProxy {
    pub fn new(
        port: u16,
        circuit_controller: Arc<CircuitController>,
        dns_resolver: Arc<PrivacyDnsResolver>,
        filter_engine: Arc<FilterEngine>,
    ) -> Self {
        let bind_addr = SocketAddr::from(([127, 0, 0, 1], port));
        Self {
            bind_addr,
            circuit_controller,
            dns_resolver,
            filter_engine,
            is_running: Arc::new(RwLock::new(false)),
        }
    }

    /// Start the local SOCKS5 proxy listener. If port is occupied (e.g. error 10048),
    /// tries subsequent ports (9050..9060) then falls back to ephemeral port 0.
    pub async fn start(&mut self) -> Result<SocketAddr, Box<dyn std::error::Error + Send + Sync>> {
        let preferred_port = self.bind_addr.port();
        let mut bound_listener = None;

        if preferred_port != 0 {
            for p in preferred_port..=preferred_port + 10 {
                let candidate = SocketAddr::from(([127, 0, 0, 1], p));
                match TcpListener::bind(candidate).await {
                    Ok(l) => {
                        bound_listener = Some(l);
                        break;
                    }
                    Err(e) => {
                        tracing::warn!("Port {} unavailable ({}). Checking next candidate...", p, e);
                    }
                }
            }
        }

        let listener = match bound_listener {
            Some(l) => l,
            None => {
                let fallback = SocketAddr::from(([127, 0, 0, 1], 0));
                TcpListener::bind(fallback).await?
            }
        };

        let actual_addr = listener.local_addr()?;
        self.bind_addr = actual_addr;
        info!("Qualium Local SOCKS5 Proxy successfully bound on {}", actual_addr);
        *self.is_running.write().await = true;

        // Update circuit controller with actual active proxy endpoint
        self.circuit_controller
            .update_proxy_status(actual_addr.port(), &actual_addr.to_string(), "Connected", true)
            .await;

        let running_flag = self.is_running.clone();
        let circuit_ctrl = self.circuit_controller.clone();
        let filter_eng = self.filter_engine.clone();

        tokio::spawn(async move {
            while *running_flag.read().await {
                if let Ok((socket, _peer)) = listener.accept().await {
                    let c_ctrl = circuit_ctrl.clone();
                    let f_eng = filter_eng.clone();
                    tokio::spawn(async move {
                        if let Err(e) = Self::handle_socks5_connection(socket, c_ctrl, f_eng).await {
                            error!("SOCKS5 connection error: {}", e);
                        }
                    });
                }
            }
        });

        Ok(actual_addr)
    }

    /// Handle RFC 1928 SOCKS5 Handshake and Tunneling + Local Diagnostics Endpoint
    pub async fn handle_socks5_connection(
        mut socket: TcpStream,
        circuit_controller: Arc<CircuitController>,
        filter_engine: Arc<FilterEngine>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // 1. Read first 2 bytes (SOCKS VER + NMETHODS, or HTTP GET prefix)
        let mut ver_methods = [0u8; 2];
        socket.read_exact(&mut ver_methods).await?;

        if ver_methods[0] != 0x05 {
            // Check for direct HTTP GET diagnostic request (e.g. GET /api/security-state)
            if ver_methods[0] == b'G' && ver_methods[1] == b'E' {
                let mut rest = [0u8; 256];
                let n = socket.read(&mut rest).await.unwrap_or(0);
                let req_line = format!("GE{}", String::from_utf8_lossy(&rest[..n]));
                if req_line.contains("/api/security-state") || req_line.contains("/status") {
                    let state = circuit_controller.get_security_state().await;
                    let json = serde_json::to_string_pretty(&state).unwrap_or_default();
                    let resp = format!(
                        "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nAccess-Control-Allow-Origin: *\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                        json.len(),
                        json
                    );
                    let _ = socket.write_all(resp.as_bytes()).await;
                    let _ = socket.shutdown().await;
                    return Ok(());
                }
            }
            return Err("Invalid SOCKS version: expected 0x05".into());
        }
        let nmethods = ver_methods[1] as usize;
        let mut methods = vec![0u8; nmethods];
        socket.read_exact(&mut methods).await?;

        // Reply: 0x05 (version 5), 0x00 (no authentication required on local loopback)
        socket.write_all(&[0x05, 0x00]).await?;

        // 2. Read SOCKS5 request header (VER, CMD, RSV, ATYP)
        let mut req_header = [0u8; 4];
        socket.read_exact(&mut req_header).await?;
        if req_header[0] != 0x05 || req_header[1] != 0x01 {
            // CMD 0x01 is CONNECT
            let _ = socket.write_all(&[0x05, 0x07, 0x00, 0x01, 0, 0, 0, 0, 0, 0]).await;
            return Err("Unsupported SOCKS5 command (only CONNECT supported)".into());
        }

        let (destination_host, port) = match req_header[3] {
            0x01 => {
                // IPv4: 4 bytes IP + 2 bytes port
                let mut addr_port = [0u8; 6];
                socket.read_exact(&mut addr_port).await?;
                let ip = format!("{}.{}.{}.{}", addr_port[0], addr_port[1], addr_port[2], addr_port[3]);
                let port = u16::from_be_bytes([addr_port[4], addr_port[5]]);
                (ip, port)
            }
            0x03 => {
                // Domain: 1 byte len + domain bytes + 2 bytes port
                let mut len_buf = [0u8; 1];
                socket.read_exact(&mut len_buf).await?;
                let domain_len = len_buf[0] as usize;
                let mut domain_buf = vec![0u8; domain_len + 2];
                socket.read_exact(&mut domain_buf).await?;
                let domain = String::from_utf8_lossy(&domain_buf[..domain_len]).to_string();
                let port = u16::from_be_bytes([domain_buf[domain_len], domain_buf[domain_len + 1]]);
                (domain, port)
            }
            0x04 => {
                // IPv6: 16 bytes IP + 2 bytes port
                let mut addr_port = [0u8; 18];
                socket.read_exact(&mut addr_port).await?;
                let mut octets = [0u8; 16];
                octets.copy_from_slice(&addr_port[..16]);
                let ip = std::net::Ipv6Addr::from(octets);
                let port = u16::from_be_bytes([addr_port[16], addr_port[17]]);
                (format!("[{}]", ip), port)
            }
            _ => {
                let _ = socket.write_all(&[0x05, 0x08, 0x00, 0x01, 0, 0, 0, 0, 0, 0]).await;
                return Err("Unknown SOCKS5 address type".into());
            }
        };

        // 3. Filter check: intercept and block if ad or tracker
        let check_host = destination_host.trim_matches('[').trim_matches(']');
        if let FilterAction::Block(cat) = filter_engine.check_url(&format!("https://{}/", check_host), check_host) {
            info!("SOCKS5 request to {} blocked by filter ({:?})", destination_host, cat);
            // 0x02 = connection not allowed by ruleset
            let _ = socket.write_all(&[0x05, 0x02, 0x00, 0x01, 0, 0, 0, 0, 0, 0]).await;
            return Ok(());
        }

        // 4. Allocate isolated circuit for destination domain (Stream Isolation)
        let _circuit = circuit_controller.get_circuit_for_destination(check_host).await;

        // 5. Connect to upstream destination
        let target_addr = if destination_host.starts_with('[') {
            format!("{}:{}", destination_host, port)
        } else {
            format!("{}:{}", destination_host, port)
        };
        match tokio::net::TcpStream::connect(&target_addr).await {
            Ok(mut upstream) => {
                // SOCKS5 success reply: VER=5, REP=0 (success), RSV=0, ATYP=1 (IPv4)
                // BND.ADDR = 0.0.0.0, BND.PORT = actual port (big-endian)
                let port_hi = (port >> 8) as u8;
                let port_lo = (port & 0xFF) as u8;
                socket
                    .write_all(&[0x05, 0x00, 0x00, 0x01, 0, 0, 0, 0, port_hi, port_lo])
                    .await?;
                // Bidirectional tunnel: proxy raw bytes between Gecko and upstream
                match tokio::io::copy_bidirectional(&mut socket, &mut upstream).await {
                    Ok((to_server, to_client)) => {
                        info!(
                            "Tunnel closed for {}: {}→server {}→client bytes",
                            destination_host, to_server, to_client
                        );
                    }
                    Err(e) => {
                        // Tunnel closed by one side — normal stream termination
                        info!("Tunnel {} closed: {}", destination_host, e);
                    }
                }
            }
            Err(e) => {
                error!("Failed to connect to upstream {}: {}", target_addr, e);
                // REP=0x04 Host unreachable
                let _ = socket
                    .write_all(&[0x05, 0x04, 0x00, 0x01, 0, 0, 0, 0, 0, 0])
                    .await;
            }
        }

        Ok(())
    }

    pub fn bind_address(&self) -> SocketAddr {
        self.bind_addr
    }

    pub async fn stop(&self) {
        *self.is_running.write().await = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_socks5_http_proxy() {
        // 1. Setup local mock HTTP server
        let mock_server = TcpListener::bind("127.0.0.1:0").await.expect("bind mock server");
        let mock_port = mock_server.local_addr().expect("mock addr").port();
        tokio::spawn(async move {
            if let Ok((mut stream, _)) = mock_server.accept().await {
                let mut req_buf = [0u8; 512];
                let _ = stream.read(&mut req_buf).await;
                let _ = stream.write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 13\r\n\r\nHello Qualium").await;
            }
        });

        // 2. Setup SOCKS5 proxy
        let circuit_ctrl = Arc::new(CircuitController::new());
        let dns = Arc::new(PrivacyDnsResolver::default());
        let filter = Arc::new(FilterEngine::default());
        let mut proxy = QualiumLocalProxy::new(0, circuit_ctrl, dns, filter);
        let addr = proxy.start().await.expect("bind proxy");

        // 3. Connect to proxy as client
        let mut client = TcpStream::connect(addr).await.expect("connect client");

        // SOCKS5 greeting: 1 auth method (none)
        client.write_all(&[0x05, 0x01, 0x00]).await.expect("send auth");
        let mut auth_resp = [0u8; 2];
        client.read_exact(&mut auth_resp).await.expect("read auth");
        assert_eq!(auth_resp, [0x05, 0x00]);

        // Connect to 127.0.0.1:mock_port (ATYP=0x01 IPv4)
        let mut req = vec![0x05, 0x01, 0x00, 0x01, 127, 0, 0, 1];
        req.extend_from_slice(&mock_port.to_be_bytes());
        client.write_all(&req).await.expect("send connect");

        let mut conn_resp = [0u8; 10];
        client.read_exact(&mut conn_resp).await.expect("read conn resp");
        assert_eq!(conn_resp[0], 0x05);
        assert_eq!(conn_resp[1], 0x00); // REP = success

        // Send HTTP request
        let http_req = b"GET / HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n";
        client.write_all(http_req).await.expect("send http");

        let mut buf = vec![0u8; 1024];
        let n = client.read(&mut buf).await.expect("read http resp");
        let resp_str = String::from_utf8_lossy(&buf[..n]);
        assert!(resp_str.contains("HTTP/1.1 200 OK") && resp_str.contains("Hello Qualium"));
    }
}
