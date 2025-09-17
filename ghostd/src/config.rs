// GhostD Configuration
use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::net::SocketAddr;
use ghostchain_shared::ffi::shroud_integration::ShroudConfig;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GhostdConfig {
    /// Chain configuration
    pub chain: ChainConfig,
    
    /// Network configuration
    pub network: NetworkConfig,
    
    /// RPC configuration
    pub rpc: RpcConfig,
    
    /// Shroud transport configuration
    pub shroud: ShroudConfig,
    
    /// Performance configuration
    pub performance: PerformanceConfig,
    
    /// Storage configuration
    pub storage: StorageConfig,
    
    /// Logging configuration
    pub logging: LoggingConfig,
    
    /// Enable testnet mode
    pub testnet_mode: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainConfig {
    pub chain_id: String,
    pub block_time_ms: u64,
    pub epoch_length: u64,
    pub genesis_file: Option<PathBuf>,
    pub enable_contracts: bool,
    pub enable_mining: bool,
    pub enable_domains: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    pub bind_address: SocketAddr,
    pub external_address: Option<SocketAddr>,
    pub max_peers: usize,
    pub enable_ipv6: bool,
    pub discovery_port: u16,
    pub ghostbridge_enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcConfig {
    pub enabled: bool,
    pub bind_address: SocketAddr,
    pub max_connections: usize,
    pub auth_required: bool,
    pub cors_enabled: bool,
    pub allowed_origins: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceConfig {
    pub cache_size_mb: usize,
    pub worker_threads: Option<usize>,
    pub enable_metrics: bool,
    pub metrics_port: u16,
    pub enable_tracing: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    pub data_dir: PathBuf,
    pub database_type: DatabaseType,
    pub sync_mode: SyncMode,
    pub enable_compression: bool,
    pub max_db_size_gb: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DatabaseType {
    Sled,
    RocksDB,
    Memory, // For testing
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SyncMode {
    Full,
    Fast,
    Light,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    pub level: String,
    pub output: LogOutput,
    pub enable_json: bool,
    pub log_file: Option<PathBuf>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LogOutput {
    Stdout,
    File,
    Both,
}

impl Default for GhostdConfig {
    fn default() -> Self {
        Self {
            chain: ChainConfig {
                chain_id: "ghostchain-mainnet".to_string(),
                block_time_ms: 6000, // 6 second blocks
                epoch_length: 100,
                genesis_file: None,
                enable_contracts: true,
                enable_mining: true,
                enable_domains: true,
            },
            network: NetworkConfig {
                bind_address: "0.0.0.0:8545".parse().unwrap(),
                external_address: None,
                max_peers: 50,
                enable_ipv6: true,
                discovery_port: 8546,
                ghostbridge_enabled: true,
            },
            rpc: RpcConfig {
                enabled: true,
                bind_address: "127.0.0.1:8547".parse().unwrap(),
                max_connections: 100,
                auth_required: false,
                cors_enabled: true,
                allowed_origins: vec!["*".to_string()],
            },
            shroud: ShroudConfig::default(),
            performance: PerformanceConfig {
                cache_size_mb: 512,
                worker_threads: None, // Use system default
                enable_metrics: true,
                metrics_port: 9090,
                enable_tracing: true,
            },
            storage: StorageConfig {
                data_dir: PathBuf::from("./ghostd_data"),
                database_type: DatabaseType::Sled,
                sync_mode: SyncMode::Full,
                enable_compression: true,
                max_db_size_gb: 100,
            },
            logging: LoggingConfig {
                level: "info".to_string(),
                output: LogOutput::Stdout,
                enable_json: false,
                log_file: None,
            },
            testnet_mode: false,
        }
    }
}

impl GhostdConfig {
    /// Load configuration from file
    pub fn from_file(path: &str) -> Result<Self> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| anyhow!("Failed to read config file {}: {}", path, e))?;
        
        // Try TOML first, then JSON
        if path.ends_with(".toml") {
            toml::from_str(&content)
                .map_err(|e| anyhow!("Failed to parse TOML config: {}", e))
        } else if path.ends_with(".json") {
            serde_json::from_str(&content)
                .map_err(|e| anyhow!("Failed to parse JSON config: {}", e))
        } else {
            // Try both formats
            toml::from_str(&content)
                .or_else(|_| serde_json::from_str(&content))
                .map_err(|e| anyhow!("Failed to parse config file (tried TOML and JSON): {}", e))
        }
    }
    
    /// Save configuration to file
    pub fn save_to_file(&self, path: &str) -> Result<()> {
        let content = if path.ends_with(".json") {
            serde_json::to_string_pretty(self)?
        } else {
            toml::to_string_pretty(self)?
        };
        
        std::fs::write(path, content)
            .map_err(|e| anyhow!("Failed to write config file {}: {}", path, e))?;
        
        Ok(())
    }
    
    /// Create testnet configuration
    pub fn testnet() -> Self {
        let mut config = Self::default();
        config.testnet_mode = true;
        config.chain.chain_id = "ghostchain-testnet".to_string();
        config.chain.block_time_ms = 2000; // 2 second blocks for testing
        config.network.bind_address = "127.0.0.1:18545".parse().unwrap();
        config.rpc.bind_address = "127.0.0.1:18547".parse().unwrap();
        config.storage.data_dir = PathBuf::from("./ghostd_testnet_data");
        config
    }
    
    /// Validate configuration
    pub fn validate(&self) -> Result<()> {
        // Validate chain configuration
        if self.chain.chain_id.is_empty() {
            return Err(anyhow!("Chain ID cannot be empty"));
        }
        
        if self.chain.block_time_ms < 1000 {
            return Err(anyhow!("Block time must be at least 1000ms"));
        }
        
        if self.chain.epoch_length == 0 {
            return Err(anyhow!("Epoch length must be greater than 0"));
        }
        
        // Validate network configuration
        if self.network.max_peers == 0 {
            return Err(anyhow!("Max peers must be greater than 0"));
        }
        
        // Validate storage configuration
        if self.storage.max_db_size_gb == 0 {
            return Err(anyhow!("Max database size must be greater than 0"));
        }
        
        // Validate performance configuration
        if self.performance.cache_size_mb == 0 {
            return Err(anyhow!("Cache size must be greater than 0"));
        }
        
        Ok(())
    }
}