# 🚀 GHOSTD (GhostChain Blockchain Daemon)

> **High-performance blockchain node daemon with QUIC transport and consensus participation**

[![Rust](https://img.shields.io/badge/rust-1.75+-blue.svg)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-MIT-green.svg)](../LICENSE)
[![Port](https://img.shields.io/badge/port-8545-red.svg)](http://localhost:8545)

---

## 🚀 **Overview**

GHOSTD is GhostChain's core blockchain daemon that provides full node functionality with high-performance QUIC transport, consensus participation, and comprehensive blockchain operations. It serves as the backbone of the GhostChain network.

### **Key Features**
- **🏗️ Full Blockchain Node** - Complete blockchain validation and storage
- **⚡ QUIC Transport** - High-performance networking via GhostLink
- **🏛️ Consensus Participation** - Validator and mining capabilities
- **🌐 Multi-Domain Resolution** - ENS, ZNS, Unstoppable Domains, Web5
- **📊 Performance Monitoring** - Built-in metrics and optimization
- **🔗 Smart Contract Execution** - EVM-compatible contract support

---

## 🏗️ **Architecture**

```
┌─────────────────────────────────────────────────────────────────┐
│                      GHOSTD Core Node                          │
├─────────────────┬─────────────────┬─────────────────────────────┤
│   Blockchain    │   Consensus     │        Networking           │
│     Engine      │    Engine       │                             │
│                 │                 │                             │
│ • Block Proc    │ • Validator     │ • QUIC Transport            │
│ • State Mgmt    │ • Mining        │ • Peer Discovery            │
│ • Tx Pool       │ • Finality      │ • Block Sync                │
└─────────────────┴─────────────────┴─────────────────────────────┘
           │                │                        │
           ▼                ▼                        ▼
┌─────────────────┐ ┌─────────────────┐ ┌─────────────────────┐
│ Storage Engine  │ │ Contract Engine │ │ Performance Monitor │
│                 │ │                 │ │                     │
│ • Block Store   │ • EVM Execution  │ • Metrics Collection    │
│ • State DB      │ • Contract State │ • Cache Management      │
│ • Index Store   │ • Gas Metering   │ • Auto Optimization     │
└─────────────────┘ └─────────────────┘ └─────────────────────┘
```

### **Core Components**

| Component | Purpose | Features |
|-----------|---------|----------|
| **Blockchain Engine** | Block processing and validation | Transaction execution, state management |
| **Consensus Engine** | Network consensus participation | PoS validation, mining support |
| **QUIC Transport** | High-performance networking | Low-latency, multiplexed connections |
| **Contract Engine** | Smart contract execution | EVM compatibility, gas metering |

---

## 🔧 **Usage**

### **Start GHOSTD Node**
```bash
# Start full node
ghostd start --bind-address 0.0.0.0:8545 --enable-quic --enable-mining

# Start validator node
ghostd start --enable-mining --validator-count 5

# Start in testnet mode
ghostd --testnet start --bind-address 0.0.0.0:8545
```

### **Node Operations**
```bash
# Initialize new blockchain
ghostd init --chain-id ghostchain-mainnet

# Check node status
ghostd status

# View blockchain info
ghostd blockchain height
ghostd blockchain get-block 12345

# Performance monitoring
ghostd performance metrics
ghostd performance benchmark
```

### **Configuration**
```toml
# ghostd.toml
[node]
chain_id = "ghostchain-mainnet"
data_dir = "./ghostd_data"
bind_address = "0.0.0.0:8545"
enable_quic = true
enable_mining = false

[consensus]
validator_count = 100
block_time = 6000  # 6 seconds
epoch_length = 100

[networking]
max_peers = 50
peer_discovery = true
bootstrap_nodes = [
    "quic://bootstrap1.ghostchain.org:8545",
    "quic://bootstrap2.ghostchain.org:8545"
]

[performance]
cache_size = 512  # MB
worker_threads = 8
enable_metrics = true

[storage]
enable_pruning = true
pruning_interval = 1000  # blocks
archive_mode = false
```

---

## 🏛️ **Blockchain Features**

### **Block Structure**
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Block {
    pub header: BlockHeader,
    pub transactions: Vec<Transaction>,
    pub receipts: Vec<TransactionReceipt>,
    pub validators: Vec<ValidatorInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockHeader {
    pub height: u64,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub parent_hash: String,
    pub state_root: String,
    pub tx_root: String,
    pub validator_root: String,
    pub hash: String,
}
```

### **Transaction Processing**
```rust
// Transaction lifecycle in GHOSTD
let transaction_flow = TransactionFlow {
    // 1. Receive transaction
    mempool_entry: receive_transaction(raw_tx).await?,

    // 2. Validate transaction
    validation: validate_transaction(&tx, &current_state).await?,

    // 3. Execute transaction
    execution: execute_transaction(&tx, &mut state).await?,

    // 4. Include in block
    block_inclusion: include_in_next_block(&tx).await?,

    // 5. Finalize on chain
    finalization: finalize_block(&block).await?,
};
```

### **State Management**
```rust
// GhostChain state structure
#[derive(Debug)]
pub struct ChainState {
    pub accounts: HashMap<Address, Account>,
    pub contract_storage: HashMap<Address, ContractStorage>,
    pub token_balances: HashMap<(Address, TokenType), TokenAmount>,
    pub domain_registry: HashMap<String, DomainInfo>,
    pub validator_set: ValidatorSet,
}
```

---

## ⚡ **Performance Features**

### **QUIC Transport Integration**
```rust
// High-performance QUIC networking
use ghostlink::GhostLinkTransport;

let transport_config = QuicTransportConfig {
    max_concurrent_streams: 1000,
    keep_alive_interval: Duration::from_secs(30),
    max_idle_timeout: Duration::from_secs(300),
    initial_window_size: 1024 * 1024, // 1MB
};

let transport = GhostLinkTransport::new(transport_config).await?;
```

### **Performance Monitoring**
```rust
#[derive(Debug, Serialize)]
pub struct NodeMetrics {
    pub block_height: u64,
    pub transactions_per_second: f64,
    pub average_block_time: f64,
    pub peer_count: usize,
    pub memory_usage_mb: u64,
    pub cache_hit_rate: f64,
    pub sync_status: SyncStatus,
}

// Real-time performance tracking
let metrics = ghostd.get_performance_metrics().await?;
println!("TPS: {}, Block Time: {}s", metrics.transactions_per_second, metrics.average_block_time);
```

### **Optimization Features**
- **Parallel Transaction Processing** - Multiple transaction execution threads
- **State Caching** - LRU cache for frequently accessed state
- **Block Pruning** - Automatic old block cleanup
- **Connection Pooling** - Efficient peer connection management

---

## 🗳️ **Consensus & Validation**

### **Proof of Stake Consensus**
```rust
// Validator participation
#[derive(Debug, Clone)]
pub struct ValidatorInfo {
    pub address: Address,
    pub stake_amount: TokenAmount,
    pub commission_rate: f64,
    pub voting_power: u64,
    pub status: ValidatorStatus,
}

// Consensus participation
let validator_config = ValidatorConfig {
    stake_amount: parse_token_amount("100000")?, // 100k SPIRIT
    commission_rate: 0.05, // 5% commission
    slashing_protection: true,
    auto_delegation: false,
};
```

### **Block Production**
```rust
// Block creation process
async fn produce_block(&self) -> Result<Block> {
    // 1. Collect pending transactions
    let pending_txs = self.mempool.get_pending_transactions(1000).await?;

    // 2. Execute transactions and update state
    let (executed_txs, receipts, new_state_root) =
        self.execute_transactions(pending_txs).await?;

    // 3. Create block
    let block = Block {
        header: BlockHeader {
            height: self.current_height + 1,
            timestamp: chrono::Utc::now(),
            parent_hash: self.latest_block_hash.clone(),
            state_root: new_state_root,
            tx_root: calculate_merkle_root(&executed_txs),
            validator_root: self.validator_set.merkle_root(),
            hash: String::new(), // Calculated after creation
        },
        transactions: executed_txs,
        receipts,
        validators: self.validator_set.active_validators(),
    };

    // 4. Sign and broadcast block
    let signed_block = self.sign_block(block).await?;
    self.broadcast_block(&signed_block).await?;

    Ok(signed_block)
}
```

---

## 🌐 **Multi-Domain Resolution**

### **Domain Integration**
```rust
// Multi-domain resolver integration
#[derive(Debug)]
pub struct DomainResolver {
    pub ens_resolver: ENSResolver,
    pub zns_resolver: ZNSResolver,
    pub unstoppable_resolver: UnstoppableResolver,
    pub web5_resolver: Web5Resolver,
}

// Domain resolution examples
let resolution_results = vec![
    resolver.resolve_ens("vitalik.eth").await?,
    resolver.resolve_zns("alice.zil").await?,
    resolver.resolve_unstoppable("alice.crypto").await?,
    resolver.resolve_web5("did:web:example.com").await?,
];
```

### **Contract Integration**
```rust
// Smart contract domain operations
let domain_contract = DomainContract {
    register_domain: "register(string,address,uint256)",
    resolve_domain: "resolve(string) returns (address)",
    transfer_domain: "transfer(string,address)",
    set_records: "setRecords(string,bytes[])",
};
```

---

## 📊 **API Reference**

### **Blockchain Operations**

#### **Get Block Information**
```bash
curl -X POST http://localhost:8545 \
  -H "Content-Type: application/json" \
  -d '{
    "method": "ghost_getBlock",
    "params": {
      "height": 12345,
      "include_transactions": true
    },
    "id": 1
  }'
```

#### **Submit Transaction**
```bash
curl -X POST http://localhost:8545 \
  -H "Content-Type: application/json" \
  -d '{
    "method": "ghost_sendTransaction",
    "params": {
      "from": "0x1234567890abcdef1234567890abcdef12345678",
      "to": "0xabcdefabcdefabcdefabcdefabcdefabcdefabcd",
      "value": "1000000000000000000",
      "gas": 21000,
      "gasPrice": "20000000000",
      "data": "0x"
    },
    "id": 1
  }'
```

### **Node Management**

#### **Get Node Status**
```bash
curl -X POST http://localhost:8545 \
  -H "Content-Type: application/json" \
  -d '{
    "method": "ghost_nodeStatus",
    "params": {},
    "id": 1
  }'
```

#### **Performance Metrics**
```bash
curl -X POST http://localhost:8545 \
  -H "Content-Type: application/json" \
  -d '{
    "method": "ghost_getMetrics",
    "params": {
      "include_cache_stats": true,
      "include_peer_info": true
    },
    "id": 1
  }'
```

---

## 🧪 **Local Testnet**

### **Testnet Configuration**
```rust
// Local testnet setup
let testnet_config = TestnetConfig {
    chain_id: "ghostchain-testnet".to_string(),
    block_time: 2000, // 2 second blocks
    epoch_length: 10,
    initial_validators: 3,
    test_accounts: 10,
    enable_mining: true,
    enable_contracts: true,
    enable_domains: true,
};
```

### **Running Integration Tests**
```bash
# Start local testnet with integration tests
ghostd blockchain start-testnet

# Run specific test scenarios
ghostd test --scenario contract-deployment
ghostd test --scenario domain-registration
ghostd test --scenario high-throughput
```

---

## 🔒 **Security Features**

### **Network Security**
- **TLS 1.3 Encryption** - All QUIC connections encrypted
- **Peer Authentication** - Cryptographic peer verification
- **DDoS Protection** - Rate limiting and connection throttling
- **Slashing Protection** - Validator misbehavior penalties

### **Consensus Security**
```rust
// Slashing conditions for validators
#[derive(Debug)]
pub enum SlashingCondition {
    DoubleSign { height: u64, evidence: SigningEvidence },
    Downtime { missed_blocks: u64, threshold: u64 },
    InvalidBlock { block_hash: String, reason: String },
    Byzantine { behavior: ByzantineBehavior },
}

// Automatic slashing protection
let slashing_protection = SlashingProtection {
    enable_double_sign_protection: true,
    enable_downtime_protection: true,
    max_missed_blocks: 100,
    slash_percentage: 5.0, // 5% of stake
};
```

---

## 🎛️ **Advanced Features**

### **State Snapshots**
```rust
// Blockchain state snapshots for fast sync
let snapshot_config = SnapshotConfig {
    enable_snapshots: true,
    snapshot_interval: 1000, // Every 1000 blocks
    compression: CompressionType::LZ4,
    verify_integrity: true,
};

// Create and restore snapshots
let snapshot = ghostd.create_state_snapshot(height).await?;
ghostd.restore_from_snapshot(&snapshot_path).await?;
```

### **Cross-Chain Bridges**
```rust
// Cross-chain integration
let bridge_config = CrossChainConfig {
    ethereum_rpc: "https://mainnet.infura.io/v3/YOUR_KEY",
    polygon_rpc: "https://polygon-rpc.com",
    arbitrum_rpc: "https://arb1.arbitrum.io/rpc",
    enable_bridge_validation: true,
};
```

### **Contract Debugging**
```rust
// Smart contract debugging features
let debug_config = ContractDebugConfig {
    enable_tracing: true,
    capture_state_changes: true,
    gas_profiling: true,
    execution_replay: true,
};
```

---

## 📈 **Performance Benchmarks**

### **Throughput Metrics**
- **Transaction Processing**: 2,500+ TPS
- **Block Production**: 6-second target, 4-second average
- **State Updates**: 10,000+ updates/second
- **Network Latency**: <100ms peer-to-peer

### **Resource Usage**
- **Memory**: 512MB baseline, 2GB under load
- **CPU**: 2-4 cores recommended, auto-scaling
- **Storage**: 100GB+ for full node, 10GB for light
- **Network**: 100Mbps recommended bandwidth

---

## 🔗 **Integration Examples**

### **With Service Mesh**
```rust
use ghostd::GhostdClient;

// Connect to other GCC services
let gid_client = GIDClient::connect("http://localhost:8552").await?;
let cns_client = CNSClient::connect("http://localhost:8553").await?;
let gledger_client = GLEDGERClient::connect("http://localhost:8555").await?;

// Cross-service operations
let identity = gid_client.resolve("did:ghost:alice").await?;
let domain = cns_client.resolve("alice.ghost").await?;
let balance = gledger_client.get_balance(&identity).await?;
```

### **Custom Applications**
```rust
// Building applications on GHOSTD
let app_config = ApplicationConfig {
    node_endpoint: "http://localhost:8545",
    enable_websockets: true,
    auto_reconnect: true,
    event_filters: vec!["block", "transaction", "contract"],
};

let ghostd_client = GhostdClient::new(app_config).await?;

// Subscribe to blockchain events
let mut block_stream = ghostd_client.subscribe_blocks().await?;
while let Some(block) = block_stream.next().await {
    println!("New block: {} with {} transactions",
             block.header.height, block.transactions.len());
}
```

---

## 🛠️ **Development Tools**

### **CLI Commands**
```bash
# Development utilities
ghostd dev generate-keypair --algorithm ed25519
ghostd dev create-test-transaction --from alice --to bob --amount 100
ghostd dev benchmark --duration 60s --tps-target 1000

# Network tools
ghostd network list-peers
ghostd network connect-peer quic://peer.example.com:8545
ghostd network broadcast-transaction <tx_hex>

# Storage tools
ghostd storage compact
ghostd storage backup --output ./backup.tar.gz
ghostd storage restore --input ./backup.tar.gz
```

---

## 🔗 **Related Services**

- **[GID](../gid/README.md)** - Identity verification for validators
- **[CNS](../cns/README.md)** - Domain resolution integration
- **[GSIG](../gsig/README.md)** - Block and transaction signing
- **[GLEDGER](../gledger/README.md)** - Token balance verification

---

## 📚 **Resources**

- **[Node Setup Guide](../gcc-docs/node-setup.md)**
- **[Validator Guide](../gcc-docs/validator-guide.md)**
- **[API Documentation](../gcc-docs/ghostd-api.md)**
- **[Performance Tuning](../gcc-docs/performance-tuning.md)**
- **[Troubleshooting](../gcc-docs/ghostd-troubleshooting.md)**

---

*🚀 Powering the GhostChain network with high-performance blockchain infrastructure*