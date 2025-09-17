use quinn::{Endpoint, ServerConfig, Connection};
use anyhow::Result;
use std::net::SocketAddr;
use tokio::sync::mpsc;
use crate::types::*;

pub struct QuicTransport {
    endpoint: Endpoint,
    connections: Vec<Connection>,
}

impl QuicTransport {
    pub async fn new(bind_addr: SocketAddr) -> Result<Self> {
        // Create QUIC endpoint with TLS config
        let server_config = ServerConfig::with_single_cert(
            vec![], // Add your certificate chain
            rustls::PrivateKey(vec![]), // Add your private key
        )?;
        
        let endpoint = Endpoint::server(server_config, bind_addr)?;
        
        Ok(Self {
            endpoint,
            connections: Vec::new(),
        })
    }
    
    pub async fn connect_to_peer(&mut self, addr: SocketAddr) -> Result<Connection> {
        let connection = self.endpoint.connect(addr, "ghostchain")?.await?;
        self.connections.push(connection.clone());
        Ok(connection)
    }
    
    pub async fn broadcast_message(&self, message: &NetworkMessage) -> Result<()> {
        let data = bincode::serialize(message)?;
        
        for conn in &self.connections {
            let mut send_stream = conn.open_uni().await?;
            send_stream.write_all(&data).await?;
            send_stream.finish().await?;
        }
        Ok(())
    }
}
