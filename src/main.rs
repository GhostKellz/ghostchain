mod types;
mod crypto;
mod blockchain;
mod token;
mod cli;
mod consensus;
mod network;
mod storage;
mod zns_integration; // Add ZNS integration module
mod zvm_integration; // Add ZVM integration module

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