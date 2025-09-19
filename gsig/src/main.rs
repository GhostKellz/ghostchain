// GSig Daemon - Ghost Signature Service for GhostChain
//
// Provides cryptographic signing and verification services
// Supports Ed25519, Secp256k1, BLS, and Post-Quantum algorithms

use anyhow::Result;
use clap::{Arg, Command};
use gsig::{GSigDaemon, GSigConfig};
use tracing::{info, error};
use tracing_subscriber;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter("gsig=info,debug")
        .init();

    info!("🔐 Starting GhostChain GSig (Ghost Signature Service)");

    // Parse command line arguments
    let matches = Command::new("gsig")
        .version("0.1.0")
        .about("GhostChain Ghost Signature Service - Cryptographic signing and verification")
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
                .default_value("8554"),
        )
        .arg(
            Arg::new("grpc-port")
                .long("grpc-port")
                .value_name("PORT")
                .help("gRPC server port")
                .default_value("9554"),
        )
        .arg(
            Arg::new("disable-key-generation")
                .long("disable-key-generation")
                .help("Disable key generation functionality")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("disable-key-import")
                .long("disable-key-import")
                .help("Disable key import functionality")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("max-keys")
                .long("max-keys")
                .value_name("COUNT")
                .help("Maximum keys per client")
                .default_value("100"),
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
        GSigConfig {
            bind_address: matches.get_one::<String>("bind").unwrap().clone(),
            rpc_port: matches.get_one::<String>("rpc-port").unwrap().parse()?,
            grpc_port: matches.get_one::<String>("grpc-port").unwrap().parse()?,
            enable_key_generation: !matches.get_flag("disable-key-generation"),
            enable_key_import: !matches.get_flag("disable-key-import"),
            max_keys_per_client: matches.get_one::<String>("max-keys").unwrap().parse()?,
        }
    };

    info!("GSig Configuration: {:?}", config);

    // Create and start GSig daemon
    let daemon = GSigDaemon::new(config);

    if let Err(e) = daemon.start().await {
        error!("Failed to start GSig daemon: {}", e);
        return Err(e);
    }

    info!("GSig daemon started successfully");

    // Keep running
    tokio::signal::ctrl_c().await?;
    info!("Shutting down GSig daemon");

    Ok(())
}
