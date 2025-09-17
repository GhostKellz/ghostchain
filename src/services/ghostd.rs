use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use crate::types::*;
use crate::blockchain::Blockchain;

#[derive(Debug, Clone)]
pub struct GhostdClient {
    endpoint: String,
    port: u16,
    connected: bool,
    blockchain: Arc<RwLock<Blockchain>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GhostdConfig {
    pub node_id: String,
    pub network_id: String,
    pub consensus_config: ConsensusSettings,
    pub p2p_config: P2pSettings,
    pub rpc_config: RpcSettings,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsensusSettings {
    pub validator_key: Option<String>,
    pub enable_mining: bool,
    pub min_peers: u32,
    pub block_time_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct P2pSettings {
    pub listen_addr: String,
    pub listen_port: u16,
    pub bootstrap_peers: Vec<String>,
    pub max_peers: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcSettings {
    pub enabled: bool,
    pub listen_addr: String,
    pub listen_port: u16,
    pub cors_enabled: bool,
    pub allowed_origins: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeStatus {
    pub node_id: String,
    pub version: String,
    pub network: String,
    pub peer_count: u32,
    pub block_height: u64,
    pub syncing: bool,
    pub validator: bool,
    pub uptime_seconds: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerInfo {
    pub peer_id: String,
    pub address: String,
    pub version: String,
    pub direction: String, // inbound/outbound
    pub connected_since: u64,
    pub last_seen: u64,
}

impl GhostdClient {
    pub fn new(endpoint: String, port: u16, blockchain: Arc<RwLock<Blockchain>>) -> Self {
        Self {
            endpoint,
            port,
            connected: false,
            blockchain,
        }
    }
    
    pub async fn connect(&mut self) -> Result<()> {
        println!("Connecting to ghostd at {}:{}", self.endpoint, self.port);
        
        // In a real implementation, this would establish a gRPC connection
        // For now, we'll simulate the connection
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        
        self.connected = true;
        println!("Successfully connected to ghostd");
        
        // Initialize the daemon with blockchain state
        self.sync_blockchain_state().await?;
        
        Ok(())
    }
    
    pub async fn disconnect(&mut self) -> Result<()> {
        self.connected = false;
        println!("Disconnected from ghostd");
        Ok(())
    }
    
    pub fn is_connected(&self) -> bool {
        self.connected
    }
    
    /// Sync the current blockchain state with ghostd
    pub async fn sync_blockchain_state(&self) -> Result<()> {
        if !self.connected {
            return Err(anyhow!("Not connected to ghostd"));
        }
        
        let blockchain = self.blockchain.read().await;
        
        println!("Syncing blockchain state with ghostd:");
        println!("  - Chain height: {}", blockchain.chain.len() - 1);
        println!("  - Current epoch: {}", blockchain.state.current_epoch);
        println!("  - Validator count: {}", blockchain.state.validators.len());
        println!("  - Account count: {}", blockchain.state.accounts.len());
        println!("  - Contract count: {}", blockchain.state.contracts.len());
        
        // In a real implementation, this would:
        // 1. Send current blockchain state to ghostd
        // 2. Receive any missing blocks/transactions
        // 3. Update local state accordingly
        
        Ok(())
    }
    
    /// Start the daemon with specified configuration
    pub async fn start_daemon(&self, config: GhostdConfig) -> Result<()> {
        if !self.connected {
            return Err(anyhow!("Not connected to ghostd"));
        }
        
        println!("Starting ghostd with configuration:");
        println!("  - Node ID: {}", config.node_id);
        println!("  - Network: {}", config.network_id);
        println!("  - Validator: {}", config.consensus_config.enable_mining);
        println!("  - P2P Listen: {}:{}", config.p2p_config.listen_addr, config.p2p_config.listen_port);
        
        // In a real implementation, this would send the configuration to ghostd
        // and start the daemon processes
        
        Ok(())
    }
    
    /// Stop the daemon
    pub async fn stop_daemon(&self) -> Result<()> {
        if !self.connected {
            return Err(anyhow!("Not connected to ghostd"));
        }
        
        println!("Stopping ghostd daemon");
        
        // In a real implementation, this would gracefully shutdown ghostd
        
        Ok(())
    }
    
    /// Get current node status
    pub async fn get_node_status(&self) -> Result<NodeStatus> {
        if !self.connected {
            return Err(anyhow!("Not connected to ghostd"));
        }
        
        let blockchain = self.blockchain.read().await;
        
        // Simulate getting status from ghostd
        Ok(NodeStatus {
            node_id: "ghost_node_001".to_string(),
            version: "1.0.0".to_string(),
            network: "ghostchain-mainnet".to_string(),
            peer_count: 5, // Simulated
            block_height: blockchain.chain.len() as u64 - 1,
            syncing: false,
            validator: true,
            uptime_seconds: 3600, // 1 hour uptime
        })
    }
    
    /// Get connected peers
    pub async fn get_peers(&self) -> Result<Vec<PeerInfo>> {
        if !self.connected {
            return Err(anyhow!("Not connected to ghostd"));
        }
        
        // Simulate peer information
        Ok(vec![
            PeerInfo {
                peer_id: "peer_001".to_string(),
                address: "192.168.1.100:8545".to_string(),
                version: "1.0.0".to_string(),
                direction: "outbound".to_string(),
                connected_since: chrono::Utc::now().timestamp() as u64 - 3600,
                last_seen: chrono::Utc::now().timestamp() as u64,
            },
            PeerInfo {
                peer_id: "peer_002".to_string(),
                address: "10.0.0.50:8545".to_string(),
                version: "1.0.0".to_string(),
                direction: "inbound".to_string(),
                connected_since: chrono::Utc::now().timestamp() as u64 - 1800,
                last_seen: chrono::Utc::now().timestamp() as u64,
            },
        ])
    }
    
    /// Add a peer to the network
    pub async fn add_peer(&self, peer_address: &str) -> Result<()> {
        if !self.connected {
            return Err(anyhow!("Not connected to ghostd"));
        }
        
        println!("Adding peer: {}", peer_address);
        
        // In a real implementation, this would:
        // 1. Validate the peer address
        // 2. Attempt to connect to the peer
        // 3. Exchange handshake information
        // 4. Add to peer list if successful
        
        Ok(())
    }
    
    /// Remove a peer from the network
    pub async fn remove_peer(&self, peer_id: &str) -> Result<()> {
        if !self.connected {
            return Err(anyhow!("Not connected to ghostd"));
        }
        
        println!("Removing peer: {}", peer_id);
        
        // In a real implementation, this would disconnect from the peer
        
        Ok(())
    }
    
    /// Submit a transaction to the network
    pub async fn submit_transaction(&self, transaction: &Transaction) -> Result<String> {
        if !self.connected {
            return Err(anyhow!("Not connected to ghostd"));
        }
        
        println!("Submitting transaction: {}", transaction.id);
        
        // Add transaction to our local blockchain (simulation)
        {
            let mut blockchain = self.blockchain.write().await;
            blockchain.add_transaction(transaction.clone())?;
        }
        
        // In a real implementation, this would:
        // 1. Validate the transaction
        // 2. Add to mempool
        // 3. Broadcast to network
        // 4. Return transaction hash
        
        Ok(transaction.id.to_string())
    }
    
    /// Get transaction by ID
    pub async fn get_transaction(&self, tx_id: &str) -> Result<Option<Transaction>> {
        if !self.connected {
            return Err(anyhow!("Not connected to ghostd"));
        }
        
        let blockchain = self.blockchain.read().await;
        
        // Search through all blocks for the transaction
        for block in &blockchain.chain {
            for tx in &block.transactions {
                if tx.id.to_string() == tx_id {
                    return Ok(Some(tx.clone()));
                }
            }
        }
        
        Ok(None)
    }
    
    /// Start block production (if this node is a validator)
    pub async fn start_validator(&self, validator_key: &str) -> Result<()> {
        if !self.connected {
            return Err(anyhow!("Not connected to ghostd"));
        }
        
        println!("Starting validator with key: {}...", &validator_key[..8]);
        
        // In a real implementation, this would:
        // 1. Validate the validator key
        // 2. Check if this node is eligible to be a validator
        // 3. Start the consensus engine
        // 4. Begin participating in block production
        
        Ok(())
    }
    
    /// Stop block production
    pub async fn stop_validator(&self) -> Result<()> {
        if !self.connected {
            return Err(anyhow!("Not connected to ghostd"));
        }
        
        println!("Stopping validator");
        
        // In a real implementation, this would stop consensus participation
        
        Ok(())
    }
    
    /// Get pending transactions in mempool
    pub async fn get_mempool(&self) -> Result<Vec<Transaction>> {
        if !self.connected {
            return Err(anyhow!("Not connected to ghostd"));
        }
        
        let blockchain = self.blockchain.read().await;
        
        // Return pending transactions
        Ok(blockchain.pending_transactions.clone())
    }
    
    /// Clear the mempool
    pub async fn clear_mempool(&self) -> Result<()> {
        if !self.connected {
            return Err(anyhow!("Not connected to ghostd"));
        }
        
        let mut blockchain = self.blockchain.write().await;
        blockchain.pending_transactions.clear();
        
        println!("Cleared mempool");
        Ok(())
    }
}

impl Default for GhostdConfig {
    fn default() -> Self {
        Self {
            node_id: uuid::Uuid::new_v4().to_string(),
            network_id: "ghostchain-mainnet".to_string(),
            consensus_config: ConsensusSettings {
                validator_key: None,
                enable_mining: false,
                min_peers: 3,
                block_time_ms: 6000,
            },
            p2p_config: P2pSettings {
                listen_addr: "0.0.0.0".to_string(),
                listen_port: 8545,
                bootstrap_peers: vec![
                    "bootstrap1.ghostchain.io:8545".to_string(),
                    "bootstrap2.ghostchain.io:8545".to_string(),
                ],
                max_peers: 50,
            },
            rpc_config: RpcSettings {
                enabled: true,
                listen_addr: "127.0.0.1".to_string(),
                listen_port: 8546,
                cors_enabled: true,
                allowed_origins: vec!["*".to_string()],
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::GenesisConfig;
    
    #[tokio::test]
    async fn test_ghostd_client_connection() {
        let config = GenesisConfig::default();
        let blockchain = Arc::new(RwLock::new(
            crate::blockchain::Blockchain::new(config).unwrap()
        ));
        
        let mut client = GhostdClient::new(
            "localhost".to_string(),
            8545,
            blockchain
        );
        
        assert!(!client.is_connected());
        
        client.connect().await.unwrap();
        assert!(client.is_connected());
        
        let status = client.get_node_status().await.unwrap();
        assert_eq!(status.network, "ghostchain-mainnet");
        
        client.disconnect().await.unwrap();
        assert!(!client.is_connected());
    }
}