// CNS Daemon - Crypto Name Service for GhostChain
//
// Provides multi-domain resolution for .ghost, .gcc, .warp, .arc, .gcp
// and bridges to external systems like ENS and Unstoppable Domains

use anyhow::Result;
use clap::{Arg, Command};
use cns::{CNSDaemon, CNSConfig};
use tracing::{info, error};
use tracing_subscriber;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter("cns=info,debug")
        .init();

    info!("🧭 Starting GhostChain CNS (Crypto Name Service)");

    // Parse command line arguments
    let matches = Command::new("cns")
        .version("0.1.0")
        .about("GhostChain Crypto Name Service - Multi-domain resolution")
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
                .default_value("8553"),
        )
        .arg(
            Arg::new("dns-port")
                .long("dns-port")
                .value_name("PORT")
                .help("DNS server port")
                .default_value("53"),
        )
        .arg(
            Arg::new("cache-ttl")
                .long("cache-ttl")
                .value_name("SECONDS")
                .help("Cache TTL in seconds")
                .default_value("300"),
        )
        .arg(
            Arg::new("enable-ens")
                .long("enable-ens")
                .help("Enable ENS bridge for .eth domains")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("enable-unstoppable")
                .long("enable-unstoppable")
                .help("Enable Unstoppable Domains bridge")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("enable-web5")
                .long("enable-web5")
                .help("Enable Web5 DID resolution")
                .action(clap::ArgAction::SetTrue),
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
        CNSConfig {
            bind_address: matches.get_one::<String>("bind").unwrap().clone(),
            rpc_port: matches.get_one::<String>("rpc-port").unwrap().parse()?,
            dns_port: matches.get_one::<String>("dns-port").unwrap().parse()?,
            cache_ttl_seconds: matches.get_one::<String>("cache-ttl").unwrap().parse()?,
            enable_ens_bridge: matches.get_flag("enable-ens"),
            enable_unstoppable_bridge: matches.get_flag("enable-unstoppable"),
            enable_web5_resolver: matches.get_flag("enable-web5"),
        }
    };

    info!("CNS Configuration: {:?}", config);

    // Create and start CNS daemon
    let daemon = CNSDaemon::new(config);

    if let Err(e) = daemon.start().await {
        error!("Failed to start CNS daemon: {}", e);
        return Err(e);
    }

    info!("CNS daemon started successfully");

    // Keep running
    tokio::signal::ctrl_c().await?;
    info!("Shutting down CNS daemon");

    Ok(())
}
