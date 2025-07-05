mod types;
mod crypto;
mod blockchain;
mod token;
mod cli;
mod consensus;
mod network;
mod storage;
mod contracts; // Smart contract execution engine
mod zns_integration; // Add ZNS integration module
mod zvm_integration; // Add ZVM integration module
mod rpc; // Add RPC module
mod services; // Service integration framework
mod performance; // Performance optimization framework
mod domains; // Multi-domain resolver (ENS, UD, Web5, Ghost)
mod ffi; // FFI bindings for ZQUIC and GhostBridge

use clap::Parser;
use cli::{Cli, CliHandler};
use anyhow::Result;
use tracing_subscriber;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    
    let cli = Cli::parse();
    let mut handler = CliHandler::new()?;
    
    handler.handle_command(cli).await?;
    
    Ok(())
}