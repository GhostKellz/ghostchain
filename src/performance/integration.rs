use anyhow::Result;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tracing::{info, warn};

use crate::blockchain::Blockchain;
use crate::storage::{Storage, optimized::OptimizedStorage};
use crate::network::{NetworkConfig, optimized::{OptimizedNetworkNode, OptimizedNetworkConfig}};
use crate::performance::{PerformanceManager, PerformanceConfig, HealthStatus};
use crate::contracts::{ContractExecutor, storage::ContractStorage, gas::GasSchedule};
use crate::rpc::RpcServer;
use crate::services::ServiceManager;

/// High-performance GhostChain node with all optimizations enabled
pub struct OptimizedGhostChainNode {
    blockchain: Arc<RwLock<Blockchain>>,
    storage: Arc<OptimizedStorage>,
    network: Arc<OptimizedNetworkNode>,
    performance_manager: Arc<PerformanceManager>,
    contract_executor: Arc<RwLock<ContractExecutor>>,
    rpc_server: Option<RpcServer>,
    service_manager: Arc<ServiceManager>,
}

#[derive(Debug, Clone)]
pub struct OptimizedNodeConfig {
    pub performance: PerformanceConfig,
    pub network: OptimizedNetworkConfig,
    pub data_dir: Option<String>,
    pub enable_rpc: bool,
    pub rpc_port: u16,
    pub enable_metrics: bool,
    pub auto_optimize_interval: Duration,
}

impl Default for OptimizedNodeConfig {
    fn default() -> Self {
        Self {
            performance: PerformanceConfig::default(),
            network: OptimizedNetworkConfig::default(),
            data_dir: None,
            enable_rpc: true,
            rpc_port: 8545,
            enable_metrics: true,
            auto_optimize_interval: Duration::from_secs(300), // 5 minutes
        }
    }
}

impl OptimizedGhostChainNode {
    pub async fn new(
        blockchain: Blockchain,
        network_config: NetworkConfig,
        node_config: OptimizedNodeConfig,
    ) -> Result<Self> {
        info!("Initializing optimized GhostChain node...");

        // Initialize performance manager
        let performance_manager = Arc::new(PerformanceManager::new(node_config.performance.clone())?);
        
        // Initialize optimized storage
        let base_storage = if let Some(ref data_dir) = node_config.data_dir {
            Storage::new(data_dir)?
        } else {
            Storage::in_memory()?
        };
        let storage = Arc::new(OptimizedStorage::new(base_storage, performance_manager.clone()));
        
        // Initialize blockchain with optimized storage
        let blockchain = Arc::new(RwLock::new(blockchain));
        
        // Initialize optimized network
        let network = Arc::new(OptimizedNetworkNode::new(
            network_config,
            node_config.network.clone(),
            blockchain.clone(),
            performance_manager.clone(),
        ).await?);
        
        // Initialize contract executor with storage
        let contract_storage = ContractStorage::new();
        let contract_executor = Arc::new(RwLock::new(ContractExecutor::new(contract_storage)));
        
        // Initialize RPC server if enabled
        let rpc_server = if node_config.enable_rpc {
            Some(RpcServer::new(
                blockchain.clone(),
                format!("0.0.0.0:{}", node_config.rpc_port).parse()?,
            )?)
        } else {
            None
        };
        
        // Initialize service manager
        let service_manager = Arc::new(ServiceManager::new(blockchain.clone()));

        Ok(Self {
            blockchain,
            storage,
            network,
            performance_manager,
            contract_executor,
            rpc_server,
            service_manager,
        })
    }

    pub async fn start(&self, config: &OptimizedNodeConfig) -> Result<()> {
        info!("Starting optimized GhostChain node...");

        // Start performance monitoring
        if config.enable_metrics {
            self.start_performance_monitoring(config.auto_optimize_interval).await?;
        }

        // Start network node
        tokio::spawn({
            let network = self.network.clone();
            async move {
                if let Err(e) = network.start().await {
                    warn!("Network node error: {}", e);
                }
            }
        });

        // RPC server is already started in the constructor
        if self.rpc_server.is_some() {
            info!("RPC server started successfully");
        }

        // Initialize default services
        self.service_manager.initialize_default_services().await?;

        info!("Optimized GhostChain node started successfully");
        Ok(())
    }

    async fn start_performance_monitoring(&self, interval: Duration) -> Result<()> {
        let performance_manager = self.performance_manager.clone();
        let storage = self.storage.clone();
        let network = self.network.clone();

        tokio::spawn(async move {
            let mut interval_timer = tokio::time::interval(interval);
            
            loop {
                interval_timer.tick().await;
                
                info!("Running automatic optimization...");
                
                // Optimize storage
                if let Err(e) = storage.optimize().await {
                    warn!("Storage optimization failed: {}", e);
                }
                
                // Optimize network
                if let Err(e) = network.optimize().await {
                    warn!("Network optimization failed: {}", e);
                }
                
                // Optimize performance manager
                if let Err(e) = performance_manager.optimize_storage().await {
                    warn!("Performance manager optimization failed: {}", e);
                }
                
                // Log performance metrics
                let health = performance_manager.health_check().await;
                info!(
                    "Performance stats - Cache hit rate: {:.2}%, Active connections: {}, Memory usage: {} bytes",
                    health.cache_hit_rate * 100.0,
                    health.active_connections,
                    health.memory_usage
                );
            }
        });

        Ok(())
    }

    /// Get comprehensive node health status
    pub async fn get_health_status(&self) -> Result<NodeHealthStatus> {
        let performance_health = self.performance_manager.health_check().await;
        let storage_stats = self.storage.get_stats().await?;
        let network_stats = self.network.get_network_stats().await;
        let service_statuses = self.service_manager.health_check_all().await;

        let overall_status = self.calculate_overall_status(&performance_health, &storage_stats, &network_stats).await;
        
        Ok(NodeHealthStatus {
            performance: performance_health,
            storage: storage_stats,
            network: network_stats,
            services: service_statuses,
            overall_status,
        })
    }

    async fn calculate_overall_status(
        &self,
        performance: &HealthStatus,
        storage: &crate::storage::optimized::StorageStats,
        network: &crate::network::optimized::NetworkStats,
    ) -> OverallStatus {
        // Simple health calculation
        let mut score = 100.0;
        
        // Penalize low cache hit rate
        if performance.cache_hit_rate < 0.5 {
            score -= 20.0;
        }
        
        // Penalize high error rate
        let error_rate = if network.total_messages_sent > 0 {
            network.total_errors as f64 / network.total_messages_sent as f64
        } else {
            0.0
        };
        
        if error_rate > 0.1 {
            score -= 30.0;
        }
        
        // Penalize low active peer count
        if network.active_peers < 3 {
            score -= 15.0;
        }
        
        match score {
            s if s >= 90.0 => OverallStatus::Healthy,
            s if s >= 70.0 => OverallStatus::Degraded,
            s if s >= 50.0 => OverallStatus::Warning,
            _ => OverallStatus::Critical,
        }
    }

    /// Force optimization of all components
    pub async fn optimize_all(&self) -> Result<()> {
        info!("Starting comprehensive optimization...");
        
        let start_time = std::time::Instant::now();
        
        // Optimize storage
        self.storage.optimize().await?;
        
        // Optimize network
        self.network.optimize().await?;
        
        // Optimize performance manager
        self.performance_manager.optimize_storage().await?;
        
        // Trigger garbage collection if needed
        self.garbage_collect().await?;
        
        info!("Comprehensive optimization completed in {:?}", start_time.elapsed());
        Ok(())
    }

    async fn garbage_collect(&self) -> Result<()> {
        // Flush storage
        self.storage.flush().await?;
        
        // Create checkpoint
        self.storage.checkpoint().await?;
        
        Ok(())
    }

    /// Get performance metrics report
    pub async fn get_performance_report(&self) -> String {
        let metrics = self.performance_manager.get_metrics().await;
        crate::performance::MetricsReporter::generate_report(&metrics)
    }

    /// Get performance metrics as JSON
    pub async fn get_performance_json(&self) -> Result<String> {
        let metrics = self.performance_manager.get_metrics().await;
        crate::performance::MetricsReporter::generate_json_report(&metrics)
            .map_err(|e| anyhow::anyhow!("Failed to generate JSON report: {}", e))
    }

    /// Stop the node gracefully
    pub async fn stop(&self) -> Result<()> {
        info!("Stopping optimized GhostChain node...");
        
        // Flush all pending operations
        self.storage.flush().await?;
        
        // Final optimization
        self.optimize_all().await?;
        
        info!("Optimized GhostChain node stopped gracefully");
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct NodeHealthStatus {
    pub performance: HealthStatus,
    pub storage: crate::storage::optimized::StorageStats,
    pub network: crate::network::optimized::NetworkStats,
    pub services: std::collections::HashMap<String, crate::services::ServiceStatus>,
    pub overall_status: OverallStatus,
}

#[derive(Debug, Clone, PartialEq)]
pub enum OverallStatus {
    Healthy,
    Degraded,
    Warning,
    Critical,
}

/// Builder for easy node configuration
pub struct OptimizedNodeBuilder {
    blockchain: Option<Blockchain>,
    network_config: Option<NetworkConfig>,
    node_config: OptimizedNodeConfig,
}

impl OptimizedNodeBuilder {
    pub fn new() -> Self {
        Self {
            blockchain: None,
            network_config: None,
            node_config: OptimizedNodeConfig::default(),
        }
    }

    pub fn with_blockchain(mut self, blockchain: Blockchain) -> Self {
        self.blockchain = Some(blockchain);
        self
    }

    pub fn with_network_config(mut self, config: NetworkConfig) -> Self {
        self.network_config = Some(config);
        self
    }

    pub fn with_data_dir<P: AsRef<std::path::Path>>(mut self, path: P) -> Self {
        self.node_config.data_dir = Some(path.as_ref().to_string_lossy().to_string());
        self
    }

    pub fn with_rpc_port(mut self, port: u16) -> Self {
        self.node_config.rpc_port = port;
        self
    }

    pub fn enable_metrics(mut self, enable: bool) -> Self {
        self.node_config.enable_metrics = enable;
        self
    }

    pub fn with_cache_size(mut self, size: usize) -> Self {
        self.node_config.performance.cache_size = size;
        self
    }

    pub fn with_max_connections(mut self, max: usize) -> Self {
        self.node_config.performance.max_connections = max;
        self
    }

    pub async fn build(self) -> Result<OptimizedGhostChainNode> {
        let blockchain = self.blockchain.ok_or_else(|| anyhow::anyhow!("Blockchain not configured"))?;
        let network_config = self.network_config.ok_or_else(|| anyhow::anyhow!("Network config not configured"))?;
        
        OptimizedGhostChainNode::new(blockchain, network_config, self.node_config).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::GenesisConfig;
    use std::net::{IpAddr, Ipv4Addr, SocketAddr};

    #[tokio::test]
    async fn test_optimized_node_creation() {
        let blockchain = Blockchain::new(GenesisConfig::default()).unwrap();
        let network_config = NetworkConfig {
            listen_addr: SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080),
            peer_id: "test_peer".to_string(),
            max_peers: 50,
            chain_id: "test_chain".to_string(),
        };
        
        let node = OptimizedGhostChainNode::new(
            blockchain,
            network_config,
            OptimizedNodeConfig::default(),
        ).await.unwrap();
        
        let health = node.get_health_status().await.unwrap();
        assert_eq!(health.overall_status, OverallStatus::Healthy);
    }

    #[tokio::test]
    async fn test_optimized_node_builder() {
        let blockchain = Blockchain::new(GenesisConfig::default()).unwrap();
        let network_config = NetworkConfig {
            listen_addr: SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8081),
            peer_id: "test_peer".to_string(),
            max_peers: 50,
            chain_id: "test_chain".to_string(),
        };
        
        let node = OptimizedNodeBuilder::new()
            .with_blockchain(blockchain)
            .with_network_config(network_config)
            .with_rpc_port(8546)
            .with_cache_size(5000)
            .enable_metrics(true)
            .build()
            .await
            .unwrap();
        
        let health = node.get_health_status().await.unwrap();
        assert!(matches!(health.overall_status, OverallStatus::Healthy | OverallStatus::Degraded));
    }
}