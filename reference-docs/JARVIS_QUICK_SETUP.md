# 🚀 GhostChain Integration Setup Guide

## 📦 Quick Start - Get Running in 30 Minutes

This guide walks you through setting up a complete GhostChain + Jarvis integration using the existing repositories.

### **Step 1: Clone GhostChain Repositories**

```bash
# Create workspace directory
mkdir ~/ghostchain-workspace
cd ~/ghostchain-workspace

# Clone all GhostChain services
git clone https://github.com/ghostkellz/ghostd
git clone https://github.com/ghostkellz/walletd  
git clone https://github.com/ghostkellz/ghostbridge
git clone https://github.com/ghostkellz/zwallet
git clone https://github.com/ghostkellz/zvm
git clone https://github.com/ghostkellz/zns
git clone https://github.com/ghostkellz/ghostchain

# Your existing Jarvis project
# git clone https://github.com/ghostkellz/jarvis
```

### **Step 2: Create Docker Compose Environment**

Create `docker-compose.ghostchain.yml`:

```yaml
version: '3.8'

services:
  # Core GhostChain blockchain node
  ghostd:
    build: 
      context: ./ghostd
      dockerfile: Dockerfile
    ports:
      - "8545:8545"    # JSON-RPC HTTP
      - "8546:8546"    # JSON-RPC WebSocket  
      - "30303:30303"  # P2P networking
      - "9090:9090"    # gRPC (if supported)
    environment:
      - RUST_LOG=debug
      - GHOSTD_RPC_HOST=0.0.0.0
      - GHOSTD_RPC_PORT=8545
      - GHOSTD_WS_PORT=8546
      - GHOSTD_CHAIN_ID=1337
      - GHOSTD_NETWORK_ID=1337
    volumes:
      - ghostd_data:/app/data
      - ./ghostd/config:/app/config
    restart: unless-stopped
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:8545"]
      interval: 30s
      timeout: 10s
      retries: 5

  # Wallet management service
  walletd:
    build:
      context: ./walletd
      dockerfile: Dockerfile
    ports:
      - "3001:3001"    # Wallet HTTP API
    environment:
      - RUST_LOG=debug
      - WALLETD_HOST=0.0.0.0
      - WALLETD_PORT=3001
      - WALLETD_GHOSTD_URL=http://ghostd:8545
      - WALLETD_DATA_DIR=/app/wallets
    volumes:
      - walletd_data:/app/wallets
      - ./walletd/config:/app/config
    depends_on:
      ghostd:
        condition: service_healthy
    restart: unless-stopped
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:3001/health"]
      interval: 30s
      timeout: 10s
      retries: 3

  # QUIC transport layer (Zig)
  ghostbridge:
    build:
      context: ./ghostbridge
      dockerfile: Dockerfile
    ports:
      - "9000:9000"    # QUIC transport
      - "9001:9001"    # Control API
    environment:
      - GHOSTBRIDGE_QUIC_PORT=9000
      - GHOSTBRIDGE_API_PORT=9001
      - GHOSTBRIDGE_GHOSTD_URL=http://ghostd:8545
      - GHOSTBRIDGE_WALLETD_URL=http://walletd:3001
    depends_on:
      - ghostd
      - walletd
    restart: unless-stopped

  # ZVM smart contract runtime (Zig)
  zvm:
    build:
      context: ./zvm
      dockerfile: Dockerfile
    ports:
      - "8547:8547"    # ZVM API
    environment:
      - ZVM_HOST=0.0.0.0
      - ZVM_PORT=8547
      - ZVM_GHOSTD_URL=http://ghostd:8545
    depends_on:
      ghostd:
        condition: service_healthy
    restart: unless-stopped

  # ZNS name resolution (Zig)
  zns:
    build:
      context: ./zns
      dockerfile: Dockerfile
    ports:
      - "5353:5353"    # DNS resolver
      - "8548:8548"    # HTTP API
    environment:
      - ZNS_DNS_PORT=5353
      - ZNS_API_PORT=8548
      - ZNS_GHOSTD_URL=http://ghostd:8545
    depends_on:
      ghostd:
        condition: service_healthy
    restart: unless-stopped

  # Jarvis AI Assistant
  jarvis:
    build:
      context: ./jarvis
      dockerfile: Dockerfile
    ports:
      - "8080:8080"    # Jarvis API
    environment:
      - RUST_LOG=info
      - JARVIS_GHOSTD_URL=http://ghostd:8545
      - JARVIS_WALLETD_URL=http://walletd:3001
      - JARVIS_GHOSTBRIDGE_URL=http://ghostbridge:9001
      - JARVIS_ZVM_URL=http://zvm:8547
      - JARVIS_ZNS_URL=http://zns:8548
      - JARVIS_CHAIN_ID=1337
    volumes:
      - jarvis_data:/app/data
      - ./jarvis/config:/app/config
    depends_on:
      - ghostd
      - walletd
      - ghostbridge
      - zvm
      - zns
    restart: unless-stopped

volumes:
  ghostd_data:
  walletd_data:
  jarvis_data:

networks:
  default:
    name: ghostchain_network
```

### **Step 3: Update Jarvis Configuration**

Update `jarvis-core/src/config.rs` to include blockchain config:

```rust
// Add to existing Config struct
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub llm: LLMConfig,
    pub system: SystemConfig,
    pub blockchain: BlockchainConfig,  // Add this line
    pub database_path: String,
    pub plugin_paths: Vec<String>,
}

// Add new blockchain configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockchainConfig {
    pub enabled_networks: Vec<String>,
    pub ghostchain: GhostChainConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GhostChainConfig {
    pub ghostd_url: String,
    pub walletd_url: Option<String>,
    pub ghostbridge_url: Option<String>,
    pub zvm_url: Option<String>,
    pub zns_url: Option<String>,
    pub chain_id: u64,
    pub explorer_url: String,
    pub gas_optimization: bool,
    pub security_monitoring: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            llm: LLMConfig::default(),
            system: SystemConfig::default(),
            blockchain: BlockchainConfig::default(),  // Add this line
            database_path: "~/.config/jarvis/jarvis.db".to_string(),
            plugin_paths: vec![],
        }
    }
}

impl Default for BlockchainConfig {
    fn default() -> Self {
        Self {
            enabled_networks: vec!["ghostchain".to_string()],
            ghostchain: GhostChainConfig {
                ghostd_url: std::env::var("JARVIS_GHOSTD_URL")
                    .unwrap_or_else(|_| "http://localhost:8545".to_string()),
                walletd_url: std::env::var("JARVIS_WALLETD_URL").ok()
                    .or_else(|| Some("http://localhost:3001".to_string())),
                ghostbridge_url: std::env::var("JARVIS_GHOSTBRIDGE_URL").ok(),
                zvm_url: std::env::var("JARVIS_ZVM_URL").ok(),
                zns_url: std::env::var("JARVIS_ZNS_URL").ok(),
                chain_id: std::env::var("JARVIS_CHAIN_ID")
                    .unwrap_or_else(|_| "1337".to_string())
                    .parse().unwrap_or(1337),
                explorer_url: "https://ghostscan.io".to_string(),
                gas_optimization: true,
                security_monitoring: true,
            },
        }
    }
}
```

### **Step 4: Update Jarvis Main CLI**

Add GhostChain commands to `src/main.rs`:

```rust
// Add to existing Commands enum
#[derive(Subcommand)]
enum Commands {
    // ...existing commands...
    
    /// GhostChain blockchain operations
    Ghost {
        #[command(subcommand)]
        action: GhostChainCommands,
    },
}

#[derive(Subcommand)]
enum GhostChainCommands {
    /// Monitor GhostChain network status
    Monitor,
    /// Check account balance (supports .ghost names)
    Balance {
        /// Address or .ghost name
        address: String,
    },
    /// Get latest block information
    Block,
    /// Audit smart contract for security issues
    Audit {
        /// Contract address to audit
        contract: String,
    },
    /// Get current gas prices
    Gas,
    /// Check network health
    Health,
    /// Resolve .ghost name to address
    Resolve {
        /// .ghost domain name
        name: String,
    },
    /// Deploy smart contract via ZVM
    Deploy {
        /// Contract file path
        contract_path: String,
        /// Constructor arguments (JSON)
        #[arg(short, long)]
        args: Option<String>,
    },
}

// Add to main() function after loading config
Commands::Ghost { action } => {
    info!("👻 GhostChain operation starting...");
    
    // Initialize GhostChain network
    let ghostchain = GhostChainNetwork::new(
        config.blockchain.ghostchain.ghostd_url.clone(),
        config.blockchain.ghostchain.chain_id,
        config.blockchain.ghostchain.walletd_url.clone(),
        config.blockchain.ghostchain.ghostbridge_url.clone(),
        config.blockchain.ghostchain.zvm_url.clone(),
        config.blockchain.ghostchain.zns_url.clone(),
    ).await?;
    
    match action {
        GhostChainCommands::Monitor => {
            info!("📊 Monitoring GhostChain network...");
            let health = ghostchain.get_network_health().await?;
            println!("Network Health: {:.1}%", health.overall_health);
            println!("Block Height: {}", health.block_height);
            println!("Sync Status: {:?}", health.sync_status);
            if !health.issues.is_empty() {
                println!("Issues: {}", health.issues.join(", "));
            }
        }
        
        GhostChainCommands::Balance { address } => {
            info!("💰 Checking balance for: {}", address);
            let balance = ghostchain.get_balance(&address).await?;
            let balance_eth = u64::from_str_radix(balance.trim_start_matches("0x"), 16)? as f64 / 1e18;
            println!("Balance: {} GHOST ({} wei)", balance_eth, balance);
        }
        
        GhostChainCommands::Block => {
            info!("🧱 Getting latest block...");
            let block = ghostchain.get_latest_block().await?;
            println!("Block #{}: {}", block.number, block.hash);
            println!("Timestamp: {}", block.timestamp);
            println!("Transactions: {}", block.transaction_count);
            println!("Gas Used: {}/{}", block.gas_used, block.gas_limit);
        }
        
        GhostChainCommands::Audit { contract } => {
            info!("🔍 Auditing contract: {}", contract);
            let report = ghostchain.audit_contract(&contract).await?;
            println!("Security Report for {}", contract);
            println!("Risk Level: {:?}", report.risk_level);
            println!("Compliance Score: {:.1}%", report.compliance_score);
            println!("Vulnerabilities: {}", report.vulnerabilities.len());
            for vuln in &report.vulnerabilities {
                println!("  - {:?}: {} ({})", vuln.severity, vuln.description, vuln.vulnerability_type);
            }
        }
        
        GhostChainCommands::Gas => {
            info!("⛽ Getting gas information...");
            let gas = ghostchain.get_gas_info().await?;
            println!("Gas Price: {} gwei", gas.gas_price as f64 / 1e9);
            println!("Base Fee: {} gwei", gas.base_fee as f64 / 1e9);
            println!("Network Congestion: {:?}", gas.network_congestion);
            println!("Est. Confirmation Time: {:?}", gas.estimated_confirmation_time);
        }
        
        GhostChainCommands::Health => {
            info!("🏥 Checking network health...");
            let health = ghostchain.get_network_health().await?;
            println!("Overall Health: {:.1}%", health.overall_health);
            println!("Block Height: {}", health.block_height);
            println!("Peer Count: {}", health.peer_count);
            println!("Sync Status: {:?}", health.sync_status);
            if let Some(last_block) = health.last_block_time {
                println!("Last Block: {}", last_block);
            }
            if !health.issues.is_empty() {
                println!("\nIssues:");
                for issue in &health.issues {
                    println!("  - {}", issue);
                }
            }
        }
        
        GhostChainCommands::Resolve { name } => {
            info!("🔍 Resolving .ghost name: {}", name);
            let address = ghostchain.resolve_ghost_name(&name).await?;
            println!("{} -> {}", name, address);
        }
        
        GhostChainCommands::Deploy { contract_path, args } => {
            info!("🚀 Deploying contract: {}", contract_path);
            // TODO: Implement contract deployment via ZVM
            println!("Contract deployment via ZVM not yet implemented");
        }
    }
}
```

### **Step 5: Update Blockchain Module**

Update `jarvis-core/src/blockchain.rs` to include GhostChain:

```rust
// Add to existing imports
pub mod ghostchain;
pub use ghostchain::GhostChainNetwork;

// Update BlockchainManager implementation
impl BlockchainManager {
    pub async fn new(config: &crate::config::BlockchainConfig) -> Result<Self> {
        let mut networks: HashMap<String, Box<dyn BlockchainNetwork>> = HashMap::new();
        
        // Add GhostChain if enabled
        if config.enabled_networks.contains(&"ghostchain".to_string()) {
            let ghostchain = GhostChainNetwork::new(
                config.ghostchain.ghostd_url.clone(),
                config.ghostchain.chain_id,
                config.ghostchain.walletd_url.clone(),
                config.ghostchain.ghostbridge_url.clone(),
                config.ghostchain.zvm_url.clone(),
                config.ghostchain.zns_url.clone(),
            ).await?;
            
            networks.insert("ghostchain".to_string(), Box::new(ghostchain));
        }
        
        Ok(Self {
            networks,
            gas_monitor: GasMonitor::new(),
            security_auditor: SecurityAuditor::new(),
            metrics: BlockchainMetrics::new(),
        })
    }
}
```

### **Step 6: Build and Test**

```bash
# Build services
docker-compose -f docker-compose.ghostchain.yml build

# Start GhostChain ecosystem
docker-compose -f docker-compose.ghostchain.yml up -d

# Wait for services to be healthy
docker-compose -f docker-compose.ghostchain.yml ps

# Test basic connectivity
curl -X POST http://localhost:8545 \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"net_version","params":[],"id":1}'

curl http://localhost:3001/health

# Build and test Jarvis with GhostChain
cd jarvis
cargo build --release

# Test GhostChain integration
./target/release/jarvis ghost health
./target/release/jarvis ghost block
./target/release/jarvis ghost gas
```

### **Step 7: Example Usage**

```bash
# Monitor network
jarvis ghost monitor

# Check balance
jarvis ghost balance 0x1234567890abcdef1234567890abcdef12345678
jarvis ghost balance alice.ghost

# Get latest block
jarvis ghost block

# Check gas prices
jarvis ghost gas

# Audit a contract
jarvis ghost audit 0xcontractaddress1234567890abcdef

# Resolve .ghost names
jarvis ghost resolve alice.ghost

# Check overall health
jarvis ghost health
```

## 🎯 **Verification Checklist**

After setup, verify everything works:

- [ ] `docker-compose ps` shows all services as healthy
- [ ] `curl http://localhost:8545` returns JSON-RPC response
- [ ] `curl http://localhost:3001/health` returns 200 OK
- [ ] `jarvis ghost health` shows network health > 80%
- [ ] `jarvis ghost block` returns latest block info
- [ ] `jarvis ghost gas` shows current gas prices

## 🔧 **Troubleshooting**

### **Common Issues:**

1. **Services not starting:** Check logs with `docker-compose logs <service>`
2. **Connection refused:** Ensure ports aren't conflicting
3. **Build failures:** Make sure Rust/Zig toolchains are installed in Docker images
4. **Health checks failing:** Increase timeout values in docker-compose.yml

### **Debug Commands:**

```bash
# Check service logs
docker-compose -f docker-compose.ghostchain.yml logs ghostd
docker-compose -f docker-compose.ghostchain.yml logs walletd
docker-compose -f docker-compose.ghostchain.yml logs jarvis

# Test individual services
curl -v http://localhost:8545
curl -v http://localhost:3001/health
curl -v http://localhost:8547/status  # ZVM
curl -v http://localhost:8548/status  # ZNS

# Rebuild specific service
docker-compose -f docker-compose.ghostchain.yml build --no-cache ghostd
```

This setup gives you a complete, integrated GhostChain + Jarvis environment ready for development and testing!
