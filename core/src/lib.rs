// GhostChain Core - Blockchain implementation
//
// This crate contains the core blockchain logic including:
// - Blockchain data structures and consensus
// - Transaction processing and state management
// - Contract execution and storage
// - Performance monitoring and caching
// - RPC interfaces

pub mod blockchain;
pub mod contracts;
pub mod performance;
pub mod rpc;
pub mod storage;
pub mod token;

// Re-export commonly used types from shared
pub use ghostchain_shared::{
    types::*,
    crypto::*,
    domains::*,
    ffi::*,
};

use anyhow::Result;
use tracing::info;

/// Initialize the GhostChain core with default configuration
pub async fn init_ghostchain_core() -> Result<()> {
    info!("🚀 Initializing GhostChain Core");
    
    // Initialize performance monitoring
    let perf_config = performance::PerformanceConfig::default();
    let _perf_manager = performance::PerformanceManager::new(perf_config).await?;
    
    info!("✅ GhostChain Core initialized successfully");
    Ok(())
}

/// Get version information for the core
pub fn get_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

/// Get build information
pub fn get_build_info() -> BuildInfo {
    BuildInfo {
        version: get_version().to_string(),
        git_hash: option_env!("GIT_HASH").unwrap_or("unknown").to_string(),
        build_time: option_env!("BUILD_TIME").unwrap_or("unknown").to_string(),
        rust_version: option_env!("RUST_VERSION").unwrap_or("unknown").to_string(),
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BuildInfo {
    pub version: String,
    pub git_hash: String,
    pub build_time: String,
    pub rust_version: String,
}