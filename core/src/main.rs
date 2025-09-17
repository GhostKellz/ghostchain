// GhostChain Core - Main binary entry point
// This will be moved to a separate CLI binary in the workspace structure

mod blockchain;
mod token;
mod cli;
mod consensus;
mod network;
mod storage;
mod contracts;
mod zns_integration;
mod zvm_integration;
mod rpc;
mod services;
mod performance;

use ghostchain_shared::*;
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