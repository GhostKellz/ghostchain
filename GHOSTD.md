# 👻 GhostD - Blockchain Daemon

> **High-performance blockchain daemon** with ZQUIC transport, consensus mining, and multi-domain support.

## 🔧 **Overview**

**GhostD** is the core blockchain daemon of the GhostChain ecosystem, providing:

- **Blockchain Node**: Full consensus participation and validation
- **Mining Engine**: Automated block production and staking
- **ZQUIC Transport**: Ultra-fast QUIC-based networking
- **Multi-Domain ZNS**: ENS, Unstoppable, Web5, and Ghost domain resolution
- **Smart Contracts**: Native contract execution with gas metering
- **RPC/API Services**: JSON-RPC and REST API endpoints

## 📦 **Installation**

### From Source
```bash
# Clone GhostChain repository
git clone https://github.com/ghostkellz/ghostchain.git
cd ghostchain

# Build ghostd
cargo build --release --bin ghostd

# Install globally
cargo install --path ghostd
```

### Docker
```bash
# Pull image
docker pull ghostchain/ghostd:latest

# Or build locally
docker build --target ghostd -t ghostchain/ghostd .
```

## 🚀 **Quick Start**

### Start Mainnet Node
```bash
# Basic mainnet node
ghostd start

# With QUIC transport and mining
ghostd start --enable-quic --enable-mining

# Custom bind address
ghostd start --bind-address 0.0.0.0:8545
```

### Start Testnet Node
```bash
# Development testnet
ghostd start --testnet

# Testnet with custom configuration
ghostd start --testnet --enable-quic --validator-count 3
```

### Initialize New Blockchain
```bash
# Initialize with custom chain ID
ghostd init --chain-id my-custom-chain

# Reset existing data
ghostd init --chain-id ghostchain-local --reset
```

## 💻 **Command Line Interface**

### Core Commands

#### **`ghostd start`** - Start the daemon
```bash
ghostd start [OPTIONS]

Options:
  --bind-address <ADDRESS>     Bind address [default: 0.0.0.0:8545]
  --enable-quic               Enable QUIC transport
  --enable-mining             Enable block mining
  --validator-count <COUNT>   Number of validators [default: 3]
  --testnet                   Run in testnet mode
  --config <FILE>             Configuration file
```

#### **`ghostd status`** - Check daemon status
```bash
ghostd status

Output:
🔍 GHOSTD STATUS:
   Version: 0.3.0
   Daemon: ✅ Running
   Block height: 1234
   Connected peers: 5
   Tx pool size: 42
```

#### **`ghostd init`** - Initialize blockchain
```bash
ghostd init [OPTIONS]

Options:
  --chain-id <ID>     Chain identifier
  --reset             Reset existing data
```

### Blockchain Operations

#### **`ghostd blockchain`** - Blockchain management
```bash
# Get current height
ghostd blockchain height

# Get block by height
ghostd blockchain get-block 100

# Get transaction
ghostd blockchain get-tx 0xabc123...

# Create test transaction
ghostd blockchain create-test-tx alice bob 1.5

# Start local testnet
ghostd blockchain start-testnet
```

### Performance Monitoring

#### **`ghostd performance`** - Performance tools
```bash
# Show metrics
ghostd performance metrics

# Run benchmark
ghostd performance benchmark

# Cache statistics
ghostd performance cache-stats
```

## ⚙️ **Configuration**

### Configuration File (`ghostd.toml`)

```toml
[chain]
chain_id = "ghostchain-mainnet"
block_time_ms = 6000
epoch_length = 100
enable_contracts = true
enable_mining = true
enable_domains = true

[network]
bind_address = "0.0.0.0:8545"
external_address = "203.0.113.1:8545"
max_peers = 50
enable_ipv6 = true
discovery_port = 8546
ghostbridge_enabled = true

[rpc]
enabled = true
bind_address = "127.0.0.1:8547"
max_connections = 100
auth_required = false
cors_enabled = true
allowed_origins = ["*"]

[zquic]
enabled = true
bind_address = "0.0.0.0:8549"
max_connections = 1000
timeout_ms = 30000
enable_compression = true
ghostbridge_enabled = true

[performance]
cache_size_mb = 512
worker_threads = 8
enable_metrics = true
metrics_port = 9090
enable_tracing = true

[storage]
data_dir = "./ghostd_data"
database_type = "Sled"
sync_mode = "Full"
enable_compression = true
max_db_size_gb = 100

[logging]
level = "info"
output = "Stdout"
enable_json = false
log_file = "/var/log/ghostd/ghostd.log"
```

### Environment Variables

```bash
# Logging
export RUST_LOG=info
export GHOSTCHAIN_LOG_LEVEL=info

# Chain configuration
export GHOSTCHAIN_CHAIN_ID=ghostchain-docker
export GHOSTCHAIN_DATA_DIR=/opt/ghostchain/data

# Network
export GHOSTCHAIN_BIND_ADDRESS=0.0.0.0:8545
```

## 🌐 **API Endpoints**

### REST API (Port 8547)

#### Health Check
```bash
curl http://localhost:8547/api/v1/health
```

#### Status
```bash
curl http://localhost:8547/api/v1/status

Response:
{
  "success": true,
  "data": {
    "version": "0.3.0",
    "chain_id": "ghostchain-mainnet",
    "current_height": 12345,
    "peer_count": 8,
    "testnet_mode": false,
    "services": {
      "zquic_enabled": true,
      "rpc_enabled": true,
      "api_enabled": true,
      "mining_enabled": true
    }
  }
}
```

#### Blockchain Height
```bash
curl http://localhost:8547/api/v1/blockchain/height

Response:
{
  "success": true,
  "data": 12345
}
```

#### Get Block
```bash
curl http://localhost:8547/api/v1/blockchain/block/100

Response:
{
  "success": true,
  "data": {
    "height": 100,
    "hash": "0xabc123...",
    "previous_hash": "0xdef456...",
    "timestamp": "2024-01-01T00:00:00Z",
    "transactions": [...],
    "validator": "ghost_validator_1"
  }
}
```

#### Performance Metrics
```bash
curl http://localhost:8547/api/v1/performance/metrics

Response:
{
  "success": true,
  "data": {
    "transactions_per_second": 1234,
    "memory_usage_mb": 512,
    "cache_hit_rate": 0.945,
    "active_connections": 42
  }
}
```

### JSON-RPC API (Port 8545)

#### Get Chain Info
```bash
curl -X POST http://localhost:8545 \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "method": "chain_getInfo",
    "params": [],
    "id": 1
  }'
```

#### Get Block
```bash
curl -X POST http://localhost:8545 \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "method": "chain_getBlock",
    "params": [100],
    "id": 1
  }'
```

#### Send Transaction
```bash
curl -X POST http://localhost:8545 \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "method": "chain_sendTransaction",
    "params": [{
      "from": "0xabc123...",
      "to": "0xdef456...",
      "amount": "1000000000000000000",
      "token": "GSPR"
    }],
    "id": 1
  }'
```

## 🐳 **Docker Deployment**

### Standalone Container
```bash
# Run mainnet node
docker run -d \
  --name ghostd \
  -p 8545:8545 \
  -p 8547:8547 \
  -p 8549:8549 \
  -v ghostd-data:/opt/ghostchain/data \
  ghostchain/ghostd:latest

# Run testnet node
docker run -d \
  --name ghostd-testnet \
  -p 18545:8545 \
  -p 18547:8547 \
  -p 18549:8549 \
  -v ghostd-testnet-data:/opt/ghostchain/data \
  ghostchain/ghostd:latest \
  ghostd start --testnet --enable-quic --enable-mining
```

### Docker Compose
```yaml
# docker-compose.yml
services:
  ghostd:
    image: ghostchain/ghostd:latest
    ports:
      - "8545:8545"
      - "8547:8547" 
      - "8549:8549"
    volumes:
      - ghostd-data:/opt/ghostchain/data
      - ./config/ghostd.toml:/opt/ghostchain/config/ghostd.toml
    environment:
      - RUST_LOG=info
      - GHOSTCHAIN_CHAIN_ID=ghostchain-docker
    command: [
      "ghostd", "start",
      "--enable-quic",
      "--enable-mining"
    ]

volumes:
  ghostd-data:
```

## 🔧 **Integration with Other Services**

### With WalletD
```bash
# Start ghostd first
ghostd start --testnet

# Configure walletd to connect
export GHOSTCHAIN_GHOSTD_ENDPOINT=http://localhost:8547
walletd start --testnet
```

### With Monitoring
```bash
# Start with metrics enabled
ghostd start --enable-metrics

# Prometheus configuration
# Add to prometheus.yml:
scrape_configs:
  - job_name: 'ghostd'
    static_configs:
      - targets: ['localhost:9090']
```

### With Load Balancer (Nginx)
```nginx
upstream ghostd_api {
    server ghostd-1:8547;
    server ghostd-2:8547;
    server ghostd-3:8547;
}

server {
    listen 80;
    server_name api.ghostchain.local;
    
    location /api/ {
        proxy_pass http://ghostd_api;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
    }
}
```

## 📊 **Performance Tuning**

### Hardware Requirements

**Minimum:**
- CPU: 2 cores
- RAM: 4 GB
- Storage: 50 GB SSD
- Network: 10 Mbps

**Recommended:**
- CPU: 8+ cores
- RAM: 16+ GB
- Storage: 500+ GB NVMe SSD
- Network: 100+ Mbps

### Optimization Settings

```toml
[performance]
cache_size_mb = 2048        # Increase for more memory
worker_threads = 16         # Match CPU cores
enable_metrics = true       # Monitor performance

[storage]
database_type = "RocksDB"   # Better for high throughput
enable_compression = true   # Reduce disk usage
sync_mode = "Fast"         # Faster sync (less secure)

[zquic]
max_connections = 10000     # Scale for high load
enable_compression = true   # Reduce bandwidth
```

## 🔐 **Security Considerations**

### Network Security
```bash
# Bind to localhost only for RPC
ghostd start --rpc-bind 127.0.0.1:8545

# Use firewall rules
ufw allow 8545/tcp  # RPC
ufw allow 8547/tcp  # API
ufw allow 8549/tcp  # QUIC
```

### Data Protection
```bash
# Set proper permissions
chmod 700 ./ghostd_data
chown ghostd:ghostd ./ghostd_data

# Use encrypted storage
# Configure disk encryption for data directory
```

### Authentication
```toml
[rpc]
auth_required = true
api_key = "your-secure-api-key"

[api]
cors_enabled = false
allowed_origins = ["https://yourdapp.com"]
```

## 🐛 **Troubleshooting**

### Common Issues

**Port Already in Use**
```bash
# Find process using port
netstat -tulpn | grep 8545
kill <pid>

# Or use different port
ghostd start --bind-address 0.0.0.0:8555
```

**Database Corruption**
```bash
# Reset blockchain data
ghostd init --reset

# Or specify new data directory
ghostd start --data-dir ./new_ghostd_data
```

**High Memory Usage**
```bash
# Reduce cache size
ghostd start --config custom-config.toml

# In config file:
[performance]
cache_size_mb = 256
```

### Logs and Debugging

```bash
# Enable debug logging
RUST_LOG=debug ghostd start

# View logs in Docker
docker logs ghostd -f

# Log to file
ghostd start --log-file ./ghostd.log
```

### Health Checks

```bash
# Check if daemon is responsive
curl -f http://localhost:8547/api/v1/health || echo "Service down"

# Check blockchain sync status
curl http://localhost:8547/api/v1/status | jq '.data.current_height'

# Monitor resource usage
docker stats ghostd
```

## 📚 **Additional Resources**

- **[GhostChain README](README.md)**: Main project documentation
- **[WalletD Documentation](WALLETD.md)**: Wallet daemon guide
- **[Smart Contracts](SMARTCONTRACT.md)**: Contract development
- **[Authentication](AUTH.md)**: Security and auth setup
- **[Protocols](PROTOCOLS.md)**: Network protocol details

## 🤝 **Support**

- **GitHub Issues**: [ghostkellz/ghostchain/issues](https://github.com/ghostkellz/ghostchain/issues)
- **Documentation**: [GhostChain Docs](README.md)
- **Discord**: Join the GhostMesh community

---

*GhostD is part of the GhostChain ecosystem. For the complete platform overview, see the [main README](README.md).*