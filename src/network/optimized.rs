use anyhow::Result;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{mpsc, RwLock, Mutex};
use tokio::time::interval;
use tracing::{debug, info, warn, error};

use crate::network::{NetworkMessage, NetworkNode, NetworkConfig, PeerInfo};
use crate::performance::{PerformanceManager, ServiceConnectionManager, BatchOperation};
use crate::blockchain::Blockchain;
use crate::types::*;

/// High-performance network node with connection pooling and batch processing
pub struct OptimizedNetworkNode {
    inner: Arc<NetworkNode>,
    performance_manager: Arc<PerformanceManager>,
    connection_manager: Arc<ServiceConnectionManager>,
    message_queue: Arc<Mutex<mpsc::Receiver<QueuedMessage>>>,
    message_sender: mpsc::Sender<QueuedMessage>,
    peer_stats: Arc<RwLock<HashMap<String, PeerStats>>>,
    config: OptimizedNetworkConfig,
}

#[derive(Debug, Clone)]
pub struct OptimizedNetworkConfig {
    pub max_connections_per_peer: usize,
    pub message_batch_size: usize,
    pub message_batch_timeout: Duration,
    pub peer_timeout: Duration,
    pub enable_compression: bool,
    pub enable_encryption: bool,
    pub connection_retry_attempts: u32,
    pub connection_retry_delay: Duration,
}

impl Default for OptimizedNetworkConfig {
    fn default() -> Self {
        Self {
            max_connections_per_peer: 10,
            message_batch_size: 100,
            message_batch_timeout: Duration::from_millis(50),
            peer_timeout: Duration::from_secs(30),
            enable_compression: true,
            enable_encryption: true,
            connection_retry_attempts: 3,
            connection_retry_delay: Duration::from_secs(1),
        }
    }
}

#[derive(Debug, Clone)]
struct QueuedMessage {
    peer_id: String,
    message: NetworkMessage,
    priority: MessagePriority,
    timestamp: Instant,
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
enum MessagePriority {
    Low = 0,
    Normal = 1,
    High = 2,
    Critical = 3,
}

#[derive(Debug, Clone)]
struct PeerStats {
    messages_sent: u64,
    messages_received: u64,
    bytes_sent: u64,
    bytes_received: u64,
    last_seen: Instant,
    connection_count: usize,
    latency_ms: f64,
    error_count: u64,
}

impl OptimizedNetworkNode {
    pub async fn new(
        config: NetworkConfig,
        optimized_config: OptimizedNetworkConfig,
        blockchain: Arc<RwLock<Blockchain>>,
        performance_manager: Arc<PerformanceManager>,
    ) -> Result<Self> {
        let inner = Arc::new(NetworkNode::new(config, blockchain).await?);
        let connection_manager = Arc::new(ServiceConnectionManager::new(
            optimized_config.max_connections_per_peer,
        ));
        
        let (message_sender, message_receiver) = mpsc::channel(10000);
        
        Ok(Self {
            inner,
            performance_manager,
            connection_manager,
            message_queue: Arc::new(Mutex::new(message_receiver)),
            message_sender,
            peer_stats: Arc::new(RwLock::new(HashMap::new())),
            config: optimized_config,
        })
    }

    pub async fn start(&self) -> Result<()> {
        info!("Starting optimized network node...");
        
        // Start message processing
        self.start_message_processor().await?;
        
        // Start peer management
        self.start_peer_manager().await?;
        
        // Start performance monitoring
        self.start_performance_monitor().await?;
        
        // Start the underlying network node
        self.inner.start().await
    }

    async fn start_message_processor(&self) -> Result<()> {
        let message_queue = self.message_queue.clone();
        let performance_manager = self.performance_manager.clone();
        let peer_stats = self.peer_stats.clone();
        let batch_size = self.config.message_batch_size;
        let batch_timeout = self.config.message_batch_timeout;

        tokio::spawn(async move {
            let mut interval = interval(batch_timeout);
            let mut message_batch = Vec::new();
            
            loop {
                tokio::select! {
                    // Process batch on timeout
                    _ = interval.tick() => {
                        if !message_batch.is_empty() {
                            Self::process_message_batch(&message_batch, &performance_manager, &peer_stats).await;
                            message_batch.clear();
                        }
                    }
                    
                    // Receive new messages
                    result = async {
                        let mut queue = message_queue.lock().await;
                        queue.recv().await
                    } => {
                        if let Some(queued_message) = result {
                            message_batch.push(queued_message);
                            
                            // Process batch if it's full
                            if message_batch.len() >= batch_size {
                                Self::process_message_batch(&message_batch, &performance_manager, &peer_stats).await;
                                message_batch.clear();
                            }
                        }
                    }
                }
            }
        });

        Ok(())
    }

    async fn start_peer_manager(&self) -> Result<()> {
        let peer_stats = self.peer_stats.clone();
        let peer_timeout = self.config.peer_timeout;

        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(30));
            
            loop {
                interval.tick().await;
                
                // Clean up inactive peers
                let mut stats = peer_stats.write().await;
                let now = Instant::now();
                
                stats.retain(|peer_id, stat| {
                    let keep = now.duration_since(stat.last_seen) < peer_timeout;
                    if !keep {
                        debug!("Removing inactive peer: {}", peer_id);
                    }
                    keep
                });
            }
        });

        Ok(())
    }

    async fn start_performance_monitor(&self) -> Result<()> {
        let performance_manager = self.performance_manager.clone();
        let peer_stats = self.peer_stats.clone();

        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(60));
            
            loop {
                interval.tick().await;
                
                // Record network metrics
                let stats = peer_stats.read().await;
                let mut total_sent = 0;
                let mut total_received = 0;
                let mut total_bytes_sent = 0;
                let mut total_bytes_received = 0;
                let mut total_errors = 0;
                
                for stat in stats.values() {
                    total_sent += stat.messages_sent;
                    total_received += stat.messages_received;
                    total_bytes_sent += stat.bytes_sent;
                    total_bytes_received += stat.bytes_received;
                    total_errors += stat.error_count;
                }
                
                info!(
                    "Network stats: {} peers, {} sent, {} received, {} bytes sent, {} bytes received, {} errors",
                    stats.len(), total_sent, total_received, total_bytes_sent, total_bytes_received, total_errors
                );
            }
        });

        Ok(())
    }

    async fn process_message_batch(
        batch: &[QueuedMessage],
        performance_manager: &Arc<PerformanceManager>,
        peer_stats: &Arc<RwLock<HashMap<String, PeerStats>>>,
    ) {
        let start_time = Instant::now();
        
        // Sort by priority
        let mut sorted_batch = batch.to_vec();
        sorted_batch.sort_by(|a, b| b.priority.partial_cmp(&a.priority).unwrap());
        
        for queued_message in sorted_batch {
            if let Err(e) = Self::process_single_message(&queued_message, peer_stats).await {
                warn!("Failed to process message for peer {}: {}", queued_message.peer_id, e);
                
                // Update error stats
                let mut stats = peer_stats.write().await;
                if let Some(peer_stat) = stats.get_mut(&queued_message.peer_id) {
                    peer_stat.error_count += 1;
                }
            }
        }
        
        performance_manager.record_operation("process_message_batch", start_time.elapsed()).await;
    }

    async fn process_single_message(
        queued_message: &QueuedMessage,
        peer_stats: &Arc<RwLock<HashMap<String, PeerStats>>>,
    ) -> Result<()> {
        // Simulate message processing
        let processing_time = match queued_message.message {
            NetworkMessage::NewBlock(_) => Duration::from_millis(10),
            NetworkMessage::NewTransaction(_) => Duration::from_millis(1),
            NetworkMessage::RequestBlock { .. } => Duration::from_millis(5),
            _ => Duration::from_millis(2),
        };
        
        tokio::time::sleep(processing_time).await;
        
        // Update peer stats
        let mut stats = peer_stats.write().await;
        let peer_stat = stats.entry(queued_message.peer_id.clone()).or_insert(PeerStats {
            messages_sent: 0,
            messages_received: 0,
            bytes_sent: 0,
            bytes_received: 0,
            last_seen: Instant::now(),
            connection_count: 1,
            latency_ms: 0.0,
            error_count: 0,
        });
        
        peer_stat.messages_sent += 1;
        peer_stat.bytes_sent += 100; // Estimate
        peer_stat.last_seen = Instant::now();
        
        Ok(())
    }

    /// Send message with priority and optimization
    pub async fn send_message_optimized(
        &self,
        peer_id: String,
        message: NetworkMessage,
        priority: MessagePriority,
    ) -> Result<()> {
        let start_time = Instant::now();
        
        let queued_message = QueuedMessage {
            peer_id: peer_id.clone(),
            message,
            priority,
            timestamp: Instant::now(),
        };
        
        // Add to processing queue
        if let Err(e) = self.message_sender.send(queued_message).await {
            warn!("Failed to queue message for peer {}: {}", peer_id, e);
            return Err(e.into());
        }
        
        self.performance_manager.record_operation("send_message_optimized", start_time.elapsed()).await;
        Ok(())
    }

    /// Broadcast message to all peers with optimization
    pub async fn broadcast_optimized(
        &self,
        message: NetworkMessage,
        priority: MessagePriority,
    ) -> Result<()> {
        let start_time = Instant::now();
        let peer_ids = self.get_active_peer_ids().await;
        
        // Create batch operations for broadcasting
        let batch_ops: Vec<BatchOperation> = peer_ids
            .into_iter()
            .map(|peer_id| BatchOperation::NetworkMessage {
                peer_id,
                message: bincode::serialize(&message).unwrap_or_default(),
            })
            .collect();
        
        // Process batch
        self.performance_manager.batch_process(batch_ops).await?;
        
        self.performance_manager.record_operation("broadcast_optimized", start_time.elapsed()).await;
        Ok(())
    }

    async fn get_active_peer_ids(&self) -> Vec<String> {
        let stats = self.peer_stats.read().await;
        let now = Instant::now();
        
        stats
            .iter()
            .filter(|(_, stat)| now.duration_since(stat.last_seen) < self.config.peer_timeout)
            .map(|(peer_id, _)| peer_id.clone())
            .collect()
    }

    /// Connect to peer with retry logic and connection pooling
    pub async fn connect_to_peer_optimized(&self, addr: SocketAddr) -> Result<()> {
        let start_time = Instant::now();
        let peer_id = format!("peer_{}", addr);
        
        // Try to get existing connection
        if let Ok(connection) = self.connection_manager.get_quinn_connection(&addr.to_string()).await {
            debug!("Reusing existing connection to {}", addr);
            self.performance_manager.record_operation("connect_to_peer_reused", start_time.elapsed()).await;
            return Ok(());
        }
        
        // Attempt connection with retries
        let mut attempts = 0;
        while attempts < self.config.connection_retry_attempts {
            match self.inner.connect_to_peer(addr).await {
                Ok(_) => {
                    info!("Successfully connected to peer {}", addr);
                    
                    // Initialize peer stats
                    let mut stats = self.peer_stats.write().await;
                    stats.insert(peer_id, PeerStats {
                        messages_sent: 0,
                        messages_received: 0,
                        bytes_sent: 0,
                        bytes_received: 0,
                        last_seen: Instant::now(),
                        connection_count: 1,
                        latency_ms: 0.0,
                        error_count: 0,
                    });
                    
                    self.performance_manager.record_operation("connect_to_peer_success", start_time.elapsed()).await;
                    return Ok(());
                }
                Err(e) => {
                    attempts += 1;
                    warn!("Failed to connect to {} (attempt {}): {}", addr, attempts, e);
                    
                    if attempts < self.config.connection_retry_attempts {
                        tokio::time::sleep(self.config.connection_retry_delay).await;
                    }
                }
            }
        }
        
        error!("Failed to connect to {} after {} attempts", addr, self.config.connection_retry_attempts);
        self.performance_manager.record_operation("connect_to_peer_failed", start_time.elapsed()).await;
        Err(anyhow::anyhow!("Connection failed after {} attempts", self.config.connection_retry_attempts))
    }

    /// Get network statistics
    pub async fn get_network_stats(&self) -> NetworkStats {
        let stats = self.peer_stats.read().await;
        let health = self.performance_manager.health_check().await;
        
        let total_peers = stats.len();
        let active_peers = stats
            .values()
            .filter(|stat| Instant::now().duration_since(stat.last_seen) < self.config.peer_timeout)
            .count();
        
        let total_messages_sent: u64 = stats.values().map(|s| s.messages_sent).sum();
        let total_messages_received: u64 = stats.values().map(|s| s.messages_received).sum();
        let total_bytes_sent: u64 = stats.values().map(|s| s.bytes_sent).sum();
        let total_bytes_received: u64 = stats.values().map(|s| s.bytes_received).sum();
        let total_errors: u64 = stats.values().map(|s| s.error_count).sum();
        
        let avg_latency = if !stats.is_empty() {
            stats.values().map(|s| s.latency_ms).sum::<f64>() / stats.len() as f64
        } else {
            0.0
        };

        NetworkStats {
            total_peers,
            active_peers,
            total_messages_sent,
            total_messages_received,
            total_bytes_sent,
            total_bytes_received,
            total_errors,
            avg_latency_ms: avg_latency,
            uptime: health.uptime,
            active_connections: health.active_connections,
        }
    }

    /// Optimize network performance
    pub async fn optimize(&self) -> Result<()> {
        let start_time = Instant::now();
        
        info!("Starting network optimization...");
        
        // Optimize connection manager
        self.connection_manager.optimize_all().await?;
        
        // Clean up old peer stats
        let mut stats = self.peer_stats.write().await;
        let now = Instant::now();
        let timeout = self.config.peer_timeout * 2; // Keep stats a bit longer
        
        stats.retain(|peer_id, stat| {
            let keep = now.duration_since(stat.last_seen) < timeout;
            if !keep {
                debug!("Cleaning up stats for peer: {}", peer_id);
            }
            keep
        });
        
        // Optimize performance manager
        self.performance_manager.optimize_storage().await?;
        
        info!("Network optimization completed in {:?}", start_time.elapsed());
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct NetworkStats {
    pub total_peers: usize,
    pub active_peers: usize,
    pub total_messages_sent: u64,
    pub total_messages_received: u64,
    pub total_bytes_sent: u64,
    pub total_bytes_received: u64,
    pub total_errors: u64,
    pub avg_latency_ms: f64,
    pub uptime: Duration,
    pub active_connections: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::performance::PerformanceConfig;
    use std::net::{IpAddr, Ipv4Addr};

    #[tokio::test]
    async fn test_optimized_network_node() {
        let config = NetworkConfig {
            listen_addr: SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080),
            peer_id: "test_peer".to_string(),
            max_peers: 50,
            chain_id: "test_chain".to_string(),
        };
        
        let optimized_config = OptimizedNetworkConfig::default();
        let blockchain = Arc::new(RwLock::new(crate::blockchain::Blockchain::new(crate::types::GenesisConfig::default()).unwrap()));
        let performance_config = PerformanceConfig::default();
        let performance_manager = Arc::new(PerformanceManager::new(performance_config).unwrap());
        
        let network = OptimizedNetworkNode::new(
            config,
            optimized_config,
            blockchain,
            performance_manager,
        ).await.unwrap();
        
        // Test message sending
        let result = network.send_message_optimized(
            "test_peer".to_string(),
            NetworkMessage::Hello {
                version: "1.0".to_string(),
                chain_id: "test".to_string(),
                height: 0,
                peer_id: "test".to_string(),
            },
            MessagePriority::Normal,
        ).await;
        
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_network_stats() {
        let config = NetworkConfig {
            listen_addr: SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8081),
            peer_id: "test_peer".to_string(),
            max_peers: 50,
            chain_id: "test_chain".to_string(),
        };
        
        let optimized_config = OptimizedNetworkConfig::default();
        let blockchain = Arc::new(RwLock::new(crate::blockchain::Blockchain::new(crate::types::GenesisConfig::default()).unwrap()));
        let performance_config = PerformanceConfig::default();
        let performance_manager = Arc::new(PerformanceManager::new(performance_config).unwrap());
        
        let network = OptimizedNetworkNode::new(
            config,
            optimized_config,
            blockchain,
            performance_manager,
        ).await.unwrap();
        
        let stats = network.get_network_stats().await;
        assert_eq!(stats.total_peers, 0);
        assert_eq!(stats.active_peers, 0);
    }
}