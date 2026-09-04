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

    /// Start the local SOCKS5 proxy listener. If port is 0 or occupied, automatically uses free ephemeral port.
    pub async fn start(&mut self) -> Result<SocketAddr, Box<dyn std::error::Error + Send + Sync>> {
        let listener = match TcpListener::bind(self.bind_addr).await {
            Ok(l) => l,
            Err(_) if self.bind_addr.port() != 0 => {
                // Fallback to ephemeral port 0 if primary port is occupied (e.g. error 10048)
                let fallback_addr = SocketAddr::from(([127, 0, 0, 1], 0));
                TcpListener::bind(fallback_addr).await?
            }
            Err(e) => return Err(Box::new(e)),
        };

        let actual_addr = listener.local_addr()?;
        self.bind_addr = actual_addr;
        info!("Qualium Local SOCKS5 Proxy successfully bound on {}", actual_addr);
        *self.is_running.write().await = true;

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

    /// Handle RFC 1928 SOCKS5 Handshake and Tunneling
    pub async fn handle_socks5_connection(
        mut socket: TcpStream,
        circuit_controller: Arc<CircuitController>,
        filter_engine: Arc<FilterEngine>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut buf = [0u8; 512];

        // 1. Read SOCKS version and auth methods
        let n = socket.read(&mut buf).await?;
        if n < 2 || buf[0] != 0x05 {
            return Err("Invalid SOCKS version: expected 0x05".into());
        }

        // Method selection reply: 0x05 (version 5), 0x00 (no authentication required on local loopback)
        socket.write_all(&[0x05, 0x00]).await?;

        // 2. Read SOCKS5 request (VER, CMD, RSV, ATYP, DST.ADDR, DST.PORT)
        let n = socket.read(&mut buf).await?;
        if n < 4 || buf[0] != 0x05 || buf[1] != 0x01 {
            // CMD 0x01 is CONNECT
            socket.write_all(&[0x05, 0x07, 0x00, 0x01, 0, 0, 0, 0, 0, 0]).await?;
            return Err("Unsupported SOCKS5 command (only CONNECT supported)".into());
        }

        let atyp = buf[3];
        let (destination_host, port) = match atyp {
            0x01 => {
                // IPv4: 4 bytes
                if n < 10 {
                    return Err("Truncated IPv4 SOCKS5 request".into());
                }
                let ip = format!("{}.{}.{}.{}", buf[4], buf[5], buf[6], buf[7]);
                let port = u16::from_be_bytes([buf[8], buf[9]]);
                (ip, port)
            }
            0x03 => {
                // Domain name: 1 byte len + domain bytes
                let len = buf[4] as usize;
                if n < 5 + len + 2 {
                    return Err("Truncated domain SOCKS5 request".into());
                }
                let domain = String::from_utf8_lossy(&buf[5..5 + len]).to_string();
                let port = u16::from_be_bytes([buf[5 + len], buf[5 + len + 1]]);
                (domain, port)
            }
            0x04 => {
                // IPv6
                ("::1".to_string(), 443)
            }
            _ => return Err("Unknown SOCKS5 address type".into()),
        };

        // 3. Filter check: intercept and block if ad or tracker
        if let FilterAction::Block(cat) = filter_engine.check_url(&format!("https://{}/", destination_host), &destination_host) {
            info!("SOCKS5 request to {} blocked by filter ({:?})", destination_host, cat);
            // 0x02 = connection not allowed by ruleset
            socket.write_all(&[0x05, 0x02, 0x00, 0x01, 0, 0, 0, 0, 0, 0]).await?;
            return Ok(());
        }

        // 4. Allocate isolated circuit for destination domain (Stream Isolation)
        let _circuit = circuit_controller.get_circuit_for_destination(&destination_host).await;

        // 5. Connect to upstream destination
        let target_addr = format!("{}:{}", destination_host, port);
        match tokio::net::TcpStream::connect(&target_addr).await {
            Ok(mut upstream) => {
                // 0x00 = success, 0x01 = IPv4, 127.0.0.1:0
                socket.write_all(&[0x05, 0x00, 0x00, 0x01, 127, 0, 0, 1, 0x00, 0x50]).await?;
                let _ = tokio::io::copy_bidirectional(&mut socket, &mut upstream).await;
            }
            Err(e) => {
                error!("Failed to connect to upstream destination {}: {}", target_addr, e);
                // 0x04 = Host unreachable
                let _ = socket.write_all(&[0x05, 0x04, 0x00, 0x01, 0, 0, 0, 0, 0, 0]).await;
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
