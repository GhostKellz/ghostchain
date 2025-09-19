// GLedger Daemon - Ghost Ledger Service for GhostChain
//
// Provides transaction processing and state management
// Double-entry accounting with audit trails

use anyhow::Result;
use clap::{Arg, Command};
use gledger::{GLedgerDaemon, GLedgerConfig};
use tracing::{info, error};
use tracing_subscriber;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter("gledger=info,debug")
        .init();

    info!("📒 Starting GhostChain GLedger (Ghost Ledger Service)");

    // Parse command line arguments
    let matches = Command::new("gledger")
        .version("0.1.0")
        .about("GhostChain Ghost Ledger Service - Transaction processing and state management")
        .arg(
            Arg::new("bind")
                .long("bind")
                .value_name("ADDRESS")
                .help("Bind address for RPC server")
                .default_value("127.0.0.1"),
        )
        .arg(
            Arg::new("rpc-port")
                .long("rpc-port")
                .value_name("PORT")
                .help("RPC server port")
                .default_value("8555"),
        )
        .arg(
            Arg::new("grpc-port")
                .long("grpc-port")
                .value_name("PORT")
                .help("gRPC server port")
                .default_value("9555"),
        )
        .arg(
            Arg::new("disable-audit")
                .long("disable-audit")
                .help("Disable audit trail functionality")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("max-journal-entries")
                .long("max-journal-entries")
                .value_name("COUNT")
                .help("Maximum journal entries")
                .default_value("1000000"),
        )
        .arg(
            Arg::new("config")
                .short('c')
                .long("config")
                .value_name("FILE")
                .help("Configuration file path"),
        )
        .get_matches();

    // Create configuration
    let config = if let Some(config_path) = matches.get_one::<String>("config") {
        // Load from file
        let config_content = tokio::fs::read_to_string(config_path).await?;
        serde_json::from_str(&config_content)?
    } else {
        // Create from command line arguments
        GLedgerConfig {
            bind_address: matches.get_one::<String>("bind").unwrap().clone(),
            rpc_port: matches.get_one::<String>("rpc-port").unwrap().parse()?,
            grpc_port: matches.get_one::<String>("grpc-port").unwrap().parse()?,
            enable_audit: !matches.get_flag("disable-audit"),
            max_journal_entries: matches.get_one::<String>("max-journal-entries").unwrap().parse()?,
        }
    };

    info!("GLedger Configuration: {:?}", config);

    // Create and start GLedger daemon
    let daemon = GLedgerDaemon::new(config);

    if let Err(e) = daemon.start().await {
        error!("Failed to start GLedger daemon: {}", e);
        return Err(e);
    }

    info!("GLedger daemon started successfully");

    // Keep running
    tokio::signal::ctrl_c().await?;
    info!("Shutting down GLedger daemon");

    Ok(())
}
