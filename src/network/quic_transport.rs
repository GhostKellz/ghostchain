use quinn::{Endpoint, ServerConfig, ClientConfig, Connection, TransportConfig};
use rustls::{Certificate, PrivateKey, ServerConfig as RustlsServerConfig};
use rcgen::{generate_simple_self_signed, CertifiedKey};
use anyhow::Result;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use std::collections::HashMap;
use tokio::sync::RwLock;
use crate::types::*;

pub struct QuicTransport {
    endpoint: Endpoint,
    connections: Arc<RwLock<HashMap<String, Connection>>>,
    max_connections: usize,
}

impl QuicTransport {
    pub async fn new(bind_addr: SocketAddr) -> Result<Self> {
        // Generate self-signed certificate for development
        let cert = generate_simple_self_signed(vec!["localhost".into()])?;
        let cert_der = cert.serialize_der()?;
        let priv_key = cert.serialize_private_key_der();
        
        // Create rustls server config
        let mut server_config = RustlsServerConfig::builder()
            .with_safe_defaults()
            .with_no_client_auth()
            .with_single_cert(
                vec![Certificate(cert_der)],
                PrivateKey(priv_key),
            )?;
        
        // Configure transport for UDP multiplexing
        let mut transport_config = TransportConfig::default();
        transport_config.max_concurrent_uni_streams(1000.into());
        transport_config.max_concurrent_bidi_streams(100.into());
        transport_config.max_idle_timeout(Some(Duration::from_secs(30).try_into()?));
        transport_config.keep_alive_interval(Some(Duration::from_secs(5)));
        
        let mut server_config = ServerConfig::with_crypto(Arc::new(server_config));
        server_config.transport_config(Arc::new(transport_config));
        
        let endpoint = Endpoint::server(server_config, bind_addr)?;
        
        Ok(Self {
            endpoint,
            connections: Arc::new(RwLock::new(HashMap::new())),
            max_connections: 1000,
        })
    }
    
    pub async fn connect_to_peer(&self, addr: SocketAddr) -> Result<Connection> {
        let addr_str = addr.to_string();
        
        // Check if connection already exists
        {
            let connections = self.connections.read().await;
            if let Some(conn) = connections.get(&addr_str) {
                return Ok(conn.clone());
            }
        }
        
        // Create new connection
        let connection = self.endpoint.connect(addr, "ghostchain")?.await?;
        
        // Store connection with limit check
        {
            let mut connections = self.connections.write().await;
            if connections.len() >= self.max_connections {
                // Remove oldest connection
                if let Some(key) = connections.keys().next().cloned() {
                    connections.remove(&key);
                }
            }
            connections.insert(addr_str, connection.clone());
        }
        
        Ok(connection)
    }
    
    pub async fn broadcast_message(&self, message: &NetworkMessage) -> Result<()> {
        let data = bincode::serialize(message)?;
        let connections = self.connections.read().await;
        
        for conn in connections.values() {
            if let Ok(mut send_stream) = conn.open_uni().await {
                let _ = send_stream.write_all(&data).await;
                let _ = send_stream.finish().await;
            }
        }
        Ok(())
    }
    
    pub async fn send_to_peer(&self, addr: SocketAddr, message: &NetworkMessage) -> Result<()> {
        let connection = self.connect_to_peer(addr).await?;
        let data = bincode::serialize(message)?;
        
        let mut send_stream = connection.open_uni().await?;
        send_stream.write_all(&data).await?;
        send_stream.finish().await?;
        
        Ok(())
    }
    
    pub async fn handle_incoming_connections(&self) -> Result<()> {
        while let Some(conn) = self.endpoint.accept().await {
            let conn = conn.await?;
            
            // Handle incoming streams
            tokio::spawn(async move {
                while let Ok((mut recv_stream, _)) = conn.accept_uni().await {
                    let mut buffer = Vec::new();
                    if recv_stream.read_to_end(&mut buffer).await.is_ok() {
                        if let Ok(message) = bincode::deserialize::<NetworkMessage>(&buffer) {
                            // Process message
                            println!("Received message: {:?}", message);
                        }
                    }
                }
            });
        }
        Ok(())
    }
}
