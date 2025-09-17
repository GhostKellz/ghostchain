// WalletD Configuration
use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::net::SocketAddr;
use ghostchain_shared::ffi::shroud_integration::ShroudConfig;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletdConfig {
    /// Wallet configuration
    pub wallet: WalletConfig,
    
    /// Network configuration
    pub network: NetworkConfig,
    
    /// API configuration
    pub api: ApiConfig,
    
    /// Shroud transport configuration
    pub shroud: ShroudConfig,
    
    /// Identity configuration
    pub identity: IdentityConfig,
    
    /// Security configuration
    pub security: SecurityConfig,
    
    /// Storage configuration
    pub storage: StorageConfig,
    
    /// Logging configuration
    pub logging: LoggingConfig,
    
    /// Enable testnet mode
    pub testnet_mode: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletConfig {
    pub default_algorithm: String,
    pub enable_hd_wallets: bool,
    pub derivation_path: String,
    pub auto_save: bool,
    pub backup_enabled: bool,
    pub backup_interval_hours: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    pub ghostd_endpoint: String,
    pub timeout_seconds: u64,
    pub retry_attempts: u32,
    pub enable_ssl: bool,
    pub ca_cert_path: Option<PathBuf>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiConfig {
    pub enabled: bool,
    pub bind_address: SocketAddr,
    pub enable_cors: bool,
    pub allowed_origins: Vec<String>,
    pub auth_required: bool,
    pub max_connections: usize,
    pub request_timeout_seconds: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdentityConfig {
    pub enabled: bool,
    pub default_algorithm: String,
    pub auto_backup: bool,
    pub identity_server_endpoint: Option<String>,
    pub enable_social_recovery: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    pub encryption_enabled: bool,
    pub password_required: bool,
    pub session_timeout_minutes: u64,
    pub max_failed_attempts: u32,
    pub hardware_wallet_support: bool,
    pub yubikey_support: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    pub data_dir: PathBuf,
    pub database_type: DatabaseType,
    pub encryption_at_rest: bool,
    pub backup_retention_days: u32,
    pub max_wallet_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DatabaseType {
    Sqlite,
    Sled,
    Memory, // For testing
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    pub level: String,
    pub output: LogOutput,
    pub enable_json: bool,
    pub log_file: Option<PathBuf>,
    pub audit_enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LogOutput {
    Stdout,
    File,
    Both,
}

impl Default for WalletdConfig {
    fn default() -> Self {
        Self {
            wallet: WalletConfig {
                default_algorithm: "ed25519".to_string(),
                enable_hd_wallets: true,
                derivation_path: "m/44'/60'/0'/0".to_string(),
                auto_save: true,
                backup_enabled: true,
                backup_interval_hours: 24,
            },
            network: NetworkConfig {
                ghostd_endpoint: "http://127.0.0.1:8547".to_string(),
                timeout_seconds: 30,
                retry_attempts: 3,
                enable_ssl: false,
                ca_cert_path: None,
            },
            api: ApiConfig {
                enabled: true,
                bind_address: "127.0.0.1:8548".parse().unwrap(),
                enable_cors: true,
                allowed_origins: vec!["*".to_string()],
                auth_required: true,
                max_connections: 100,
                request_timeout_seconds: 30,
            },
            shroud: ShroudConfig::default(),
            identity: IdentityConfig {
                enabled: true,
                default_algorithm: "ed25519".to_string(),
                auto_backup: true,
                identity_server_endpoint: None,
                enable_social_recovery: false,
            },
            security: SecurityConfig {
                encryption_enabled: true,
                password_required: true,
                session_timeout_minutes: 60,
                max_failed_attempts: 5,
                hardware_wallet_support: false,
                yubikey_support: false,
            },
            storage: StorageConfig {
                data_dir: PathBuf::from("./walletd_data"),
                database_type: DatabaseType::Sqlite,
                encryption_at_rest: true,
                backup_retention_days: 90,
                max_wallet_count: 100,
            },
            logging: LoggingConfig {
                level: "info".to_string(),
                output: LogOutput::Stdout,
                enable_json: false,
                log_file: None,
                audit_enabled: true,
            },
            testnet_mode: false,
        }
    }
}

impl WalletdConfig {
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
        config.network.ghostd_endpoint = "http://127.0.0.1:18547".to_string();
        config.api.bind_address = "127.0.0.1:18548".parse().unwrap();
        config.storage.data_dir = PathBuf::from("./walletd_testnet_data");
        config
    }
    
    /// Validate configuration
    pub fn validate(&self) -> Result<()> {
        // Validate wallet configuration
        if self.wallet.default_algorithm.is_empty() {
            return Err(anyhow!("Default algorithm cannot be empty"));
        }
        
        if !["ed25519", "secp256k1", "secp256r1"].contains(&self.wallet.default_algorithm.as_str()) {
            return Err(anyhow!("Unsupported default algorithm: {}", self.wallet.default_algorithm));
        }
        
        // Validate network configuration
        if self.network.ghostd_endpoint.is_empty() {
            return Err(anyhow!("GhostD endpoint cannot be empty"));
        }
        
        if self.network.timeout_seconds == 0 {
            return Err(anyhow!("Network timeout must be greater than 0"));
        }
        
        // Validate security configuration
        if self.security.session_timeout_minutes == 0 {
            return Err(anyhow!("Session timeout must be greater than 0"));
        }
        
        // Validate storage configuration
        if self.storage.max_wallet_count == 0 {
            return Err(anyhow!("Max wallet count must be greater than 0"));
        }
        
        Ok(())
    }
}