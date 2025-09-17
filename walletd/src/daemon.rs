// WalletD Daemon Implementation
use anyhow::Result;
use std::sync::Arc;
use tracing::{info, error};

use ghostchain_shared::{
    types::*,
    crypto::*,
    ffi::shroud_integration::*,
    transport::{GhostwireTransport, TransportOperations},
};

use crate::config::WalletdConfig;
use crate::shroud_wallet::GhostWallet;

pub struct WalletDaemon {
    config: WalletdConfig,
    ghost_wallet: Option<GhostWallet>,
}

impl WalletDaemon {
    pub async fn new(config: WalletdConfig) -> Result<Self> {
        info!("🔧 Initializing WalletDaemon");
        config.validate()?;
        
        Ok(Self { 
            config,
            ghost_wallet: None,
        })
    }
    
    pub async fn start(&mut self, bind_address: String, enable_quic: bool) -> Result<()> {
        info!("🚀 Starting WalletDaemon services");
        info!("   Bind address: {}", bind_address);
        info!("   QUIC enabled: {}", enable_quic);
        
        // Initialize Shroud wallet if QUIC is enabled
        if enable_quic {
            let port = bind_address
                .split(':')
                .last()
                .and_then(|p| p.parse::<u16>().ok())
                .unwrap_or(8548);
            
            let ghost_wallet = GhostWallet::new(port).await?;
            ghost_wallet.start().await?;
            self.ghost_wallet = Some(ghost_wallet);
            info!("⚡ Shroud wallet initialized with Ghostwire transport");
        }
        
        Ok(())
    }
    
    pub async fn run(&mut self) -> Result<()> {
        info!("🏃 WalletDaemon main loop started");
        
        // If we have a ghost wallet, it's already running in background tasks
        // Here we just keep the daemon alive
        if self.ghost_wallet.is_some() {
            // The wallet services are running in background tasks
            // We'll just wait here until shutdown is requested
            tokio::time::sleep(tokio::time::Duration::from_secs(u64::MAX)).await;
        }
        
        Ok(())
    }
    
    pub async fn stop(&mut self) -> Result<()> {
        info!("🛑 Stopping WalletDaemon services");
        
        // Shroud wallet will stop when dropped
        if let Some(wallet) = &self.ghost_wallet {
            info!("⚡ Shroud wallet stopped");
        }
        
        Ok(())
    }
}