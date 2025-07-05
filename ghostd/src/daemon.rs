// GhostD Daemon Implementation
use anyhow::{Result, anyhow};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, error, warn};

use ghostchain_core::{
    blockchain::{Blockchain, local_testnet::LocalTestnet},
    performance::PerformanceManager,
    rpc::RPCServer,
    contracts::ContractExecutor,
};
use ghostchain_shared::{
    types::*,
    ffi::{zquic::*, ghostbridge::*},
};

use crate::config::GhostdConfig;
use crate::api::ApiServer;
use crate::zquic_integration::ZQuicDaemonServer;

pub struct GhostDaemon {
    config: GhostdConfig,
    blockchain: Arc<RwLock<Blockchain>>,
    performance_manager: PerformanceManager,
    zquic_server: Option<ZQuicDaemonServer>,
    rpc_server: Option<RPCServer>,
    api_server: Option<ApiServer>,
    testnet: Option<LocalTestnet>,
}

impl GhostDaemon {
    /// Create a new GhostDaemon instance
    pub async fn new(config: GhostdConfig) -> Result<Self> {
        info!("🔧 Initializing GhostDaemon");
        
        // Validate configuration
        config.validate()?;
        
        // Initialize blockchain
        let blockchain = if config.testnet_mode {
            info!("🧪 Initializing testnet blockchain");
            Arc::new(RwLock::new(Blockchain::new_default()))
        } else {
            info!("🌐 Initializing mainnet blockchain");
            // Load genesis configuration if specified
            let genesis_config = if let Some(ref genesis_file) = config.chain.genesis_file {
                GenesisConfig::from_file(genesis_file)?
            } else {
                GenesisConfig::default()
            };
            Arc::new(RwLock::new(Blockchain::new(genesis_config)?))
        };
        
        // Initialize performance manager
        let perf_config = ghostchain_core::performance::PerformanceConfig {
            cache_size: config.performance.cache_size_mb * 1024 * 1024,
            enable_metrics: config.performance.enable_metrics,
            worker_threads: config.performance.worker_threads.unwrap_or_else(|| num_cpus::get()),
            enable_connection_pooling: true,
            batch_size: 1000,
            optimization_level: ghostchain_core::performance::OptimizationLevel::High,
        };
        let performance_manager = PerformanceManager::new(perf_config).await?;
        
        info!("✅ GhostDaemon core initialized");
        
        Ok(Self {
            config,
            blockchain,
            performance_manager,
            zquic_server: None,
            rpc_server: None,
            api_server: None,
            testnet: None,
        })
    }
    
    /// Start the daemon services
    pub async fn start(
        &mut self,
        bind_address: String,
        enable_quic: bool,
        enable_mining: bool,
        validator_count: usize,
    ) -> Result<()> {
        info!("🚀 Starting GhostDaemon services");
        
        // Initialize testnet if in testnet mode
        if self.config.testnet_mode {
            let testnet_config = ghostchain_core::blockchain::local_testnet::TestnetConfig {
                chain_id: self.config.chain.chain_id.clone(),
                block_time: self.config.chain.block_time_ms,
                epoch_length: self.config.chain.epoch_length,
                initial_validators: validator_count,
                test_accounts: 10,
                enable_mining,
                enable_contracts: self.config.chain.enable_contracts,
                enable_domains: self.config.chain.enable_domains,
            };
            
            let testnet = LocalTestnet::new(testnet_config).await?;
            self.testnet = Some(testnet);
            info!("🧪 Local testnet initialized");
        }
        
        // Start ZQUIC server if enabled
        if enable_quic && self.config.zquic.enabled {
            let zquic_server = ZQuicDaemonServer::new(
                self.config.zquic.clone(),
                self.blockchain.clone(),
                self.performance_manager.clone(),
            ).await?;
            
            self.zquic_server = Some(zquic_server);
            info!("⚡ ZQUIC server initialized");
        }
        
        // Start RPC server if enabled
        if self.config.rpc.enabled {
            let rpc_server = RPCServer::new(
                self.config.rpc.bind_address,
                self.blockchain.clone(),
            ).await?;
            
            self.rpc_server = Some(rpc_server);
            info!("📡 RPC server initialized on {}", self.config.rpc.bind_address);
        }
        
        // Start API server
        let api_server = ApiServer::new(
            self.config.clone(),
            self.blockchain.clone(),
            self.performance_manager.clone(),
        ).await?;
        
        self.api_server = Some(api_server);
        info!("🌐 API server initialized on {}", bind_address);
        
        info!("✅ All GhostDaemon services started successfully");
        Ok(())
    }
    
    /// Run the daemon (main event loop)
    pub async fn run(&mut self) -> Result<()> {
        info!("🏃 GhostDaemon main loop started");
        
        // Start all service tasks
        let mut handles = Vec::new();
        
        // ZQUIC server task
        if let Some(zquic_server) = &mut self.zquic_server {
            let zquic_handle = {
                let server = zquic_server.clone();
                tokio::spawn(async move {
                    if let Err(e) = server.run().await {
                        error!("ZQUIC server error: {}", e);
                    }
                })
            };
            handles.push(zquic_handle);
        }
        
        // RPC server task
        if let Some(rpc_server) = &mut self.rpc_server {
            let rpc_handle = {
                let server = rpc_server.clone();
                tokio::spawn(async move {
                    if let Err(e) = server.run().await {
                        error!("RPC server error: {}", e);
                    }
                })
            };
            handles.push(rpc_handle);
        }
        
        // API server task
        if let Some(api_server) = &mut self.api_server {
            let api_handle = {
                let server = api_server.clone();
                tokio::spawn(async move {
                    if let Err(e) = server.run().await {
                        error!("API server error: {}", e);
                    }
                })
            };
            handles.push(api_handle);
        }
        
        // Block processing task (if mining enabled)
        if self.config.chain.enable_mining {
            let blockchain = self.blockchain.clone();
            let block_time = self.config.chain.block_time_ms;
            let mining_handle = tokio::spawn(async move {
                Self::mining_loop(blockchain, block_time).await;
            });
            handles.push(mining_handle);
        }
        
        // Performance monitoring task
        if self.config.performance.enable_metrics {
            let perf_manager = self.performance_manager.clone();
            let metrics_handle = tokio::spawn(async move {
                Self::metrics_loop(perf_manager).await;
            });
            handles.push(metrics_handle);
        }
        
        // Wait for all tasks to complete
        for handle in handles {
            if let Err(e) = handle.await {
                error!("Task error: {}", e);
            }
        }
        
        Ok(())
    }
    
    /// Stop the daemon gracefully
    pub async fn stop(&mut self) -> Result<()> {
        info!("🛑 Stopping GhostDaemon services");
        
        // Stop ZQUIC server
        if let Some(zquic_server) = &mut self.zquic_server {
            zquic_server.stop().await?;
            info!("⚡ ZQUIC server stopped");
        }
        
        // Stop RPC server
        if let Some(rpc_server) = &mut self.rpc_server {
            rpc_server.stop().await?;
            info!("📡 RPC server stopped");
        }
        
        // Stop API server
        if let Some(api_server) = &mut self.api_server {
            api_server.stop().await?;
            info!("🌐 API server stopped");
        }
        
        // Flush performance data
        self.performance_manager.flush().await?;
        info!("📊 Performance data flushed");
        
        info!("✅ GhostDaemon stopped gracefully");
        Ok(())
    }
    
    /// Mining loop for block production
    async fn mining_loop(blockchain: Arc<RwLock<Blockchain>>, block_time_ms: u64) {
        let mut interval = tokio::time::interval(
            std::time::Duration::from_millis(block_time_ms)
        );
        
        loop {
            interval.tick().await;
            
            // Create new block
            match Self::create_block(blockchain.clone()).await {
                Ok(block) => {
                    info!("⛏️  Mined block #{} with hash {}", block.height, block.hash);
                },
                Err(e) => {
                    warn!("Mining error: {}", e);
                }
            }
        }
    }
    
    /// Create a new block
    async fn create_block(blockchain: Arc<RwLock<Blockchain>>) -> Result<Block> {
        let mut chain = blockchain.write().await;
        
        // Simple mining - in production this would involve proper consensus
        let validator_address = "ghost_miner".to_string();
        let validator_signature = vec![0; 64]; // Mock signature
        
        let block = chain.create_block(validator_address, validator_signature)?;
        chain.add_block(block.clone())?;
        
        Ok(block)
    }
    
    /// Performance metrics collection loop
    async fn metrics_loop(perf_manager: PerformanceManager) {
        let mut interval = tokio::time::interval(
            std::time::Duration::from_secs(30)
        );
        
        loop {
            interval.tick().await;
            
            // Collect and log metrics
            match perf_manager.collect_metrics().await {
                Ok(metrics) => {
                    info!("📊 Performance: TPS: {}, Memory: {} MB, Cache hit: {:.1}%", 
                          metrics.transactions_per_second,
                          metrics.memory_usage_mb,
                          metrics.cache_hit_rate * 100.0);
                },
                Err(e) => {
                    warn!("Metrics collection error: {}", e);
                }
            }
        }
    }
    
    /// Get daemon status
    pub async fn get_status(&self) -> DaemonStatus {
        let blockchain = self.blockchain.read().await;
        
        DaemonStatus {
            version: env!("CARGO_PKG_VERSION").to_string(),
            chain_id: self.config.chain.chain_id.clone(),
            current_height: blockchain.current_height(),
            peer_count: 0, // TODO: Implement peer management
            testnet_mode: self.config.testnet_mode,
            services: ServiceStatus {
                zquic_enabled: self.zquic_server.is_some(),
                rpc_enabled: self.rpc_server.is_some(),
                api_enabled: self.api_server.is_some(),
                mining_enabled: self.config.chain.enable_mining,
            },
        }
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct DaemonStatus {
    pub version: String,
    pub chain_id: String,
    pub current_height: u64,
    pub peer_count: usize,
    pub testnet_mode: bool,
    pub services: ServiceStatus,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct ServiceStatus {
    pub zquic_enabled: bool,
    pub rpc_enabled: bool,
    pub api_enabled: bool,
    pub mining_enabled: bool,
}