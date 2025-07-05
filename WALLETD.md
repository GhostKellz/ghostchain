# 💼 WalletD - Secure Wallet Daemon

> **Enterprise-grade wallet daemon** with multi-algorithm support, identity management, and ZQUIC transport.

## 🔧 **Overview**

**WalletD** is the secure wallet service of the GhostChain ecosystem, providing:

- **Multi-Algorithm Wallets**: Ed25519, Secp256k1, Secp256r1 support
- **HD Wallet Management**: Hierarchical deterministic key derivation
- **Identity Services**: RealID decentralized identity integration
- **ZQUIC Transport**: High-performance secure communication
- **Hardware Support**: YubiKey and hardware wallet integration
- **Multi-Signature**: Advanced multi-sig wallet support
- **API Services**: REST API and secure authentication

## 📦 **Installation**

### From Source
```bash
# Clone GhostChain repository
git clone https://github.com/ghostkellz/ghostchain.git
cd ghostchain

# Build walletd
cargo build --release --bin walletd

# Install globally
cargo install --path walletd
```

### Docker
```bash
# Pull image
docker pull ghostchain/walletd:latest

# Or build locally
docker build --target walletd -t ghostchain/walletd .
```

## 🚀 **Quick Start**

### Initialize Wallet Daemon
```bash
# Initialize data directory
walletd init

# Initialize with custom directory
walletd init --data-dir ./my-wallet-data
```

### Start Wallet Service
```bash
# Basic wallet daemon
walletd start

# With QUIC transport
walletd start --enable-quic

# Testnet mode
walletd start --testnet --enable-quic
```

### Create Your First Wallet
```bash
# Create Ed25519 wallet
walletd wallet create main --algorithm ed25519

# Create Secp256k1 wallet
walletd wallet create bitcoin --algorithm secp256k1

# Import from mnemonic
walletd wallet import recovery "abandon abandon abandon..." --algorithm ed25519
```

## 💻 **Command Line Interface**

### Core Commands

#### **`walletd start`** - Start the daemon
```bash
walletd start [OPTIONS]

Options:
  --bind-address <ADDRESS>     Bind address [default: 0.0.0.0:8548]
  --enable-quic               Enable QUIC transport
  --background                Run in background mode
  --testnet                   Run in testnet mode
  --config <FILE>             Configuration file
```

#### **`walletd status`** - Check daemon status
```bash
walletd status

Output:
🔍 WALLETD STATUS:
   Version: 0.3.0
   Daemon: ✅ Running
   Wallets loaded: 3
   Identities: 2
   ZQUIC transport: ✅ Active
```

#### **`walletd init`** - Initialize wallet daemon
```bash
walletd init [OPTIONS]

Options:
  --data-dir <DIR>    Data directory
  --reset             Reset existing data
```

### Wallet Management

#### **`walletd wallet`** - Wallet operations
```bash
# Create new wallet
walletd wallet create <NAME> --algorithm <ALGO>

# List all wallets
walletd wallet list

# Get wallet balance
walletd wallet balance <NAME>

# Generate new address
walletd wallet address <NAME>

# Send tokens
walletd wallet send <FROM> <TO> <AMOUNT> [--token <TOKEN>]

# Import wallet from mnemonic
walletd wallet import <NAME> <MNEMONIC> --algorithm <ALGO>
```

**Example Wallet Operations:**
```bash
# Create main wallet
walletd wallet create main --algorithm ed25519
Output: ✅ Wallet 'main' created successfully
        Algorithm: ed25519
        Mnemonic: abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about
        ⚠️  Store this mnemonic safely - it cannot be recovered!

# List wallets
walletd wallet list
Output: 💼 AVAILABLE WALLETS:
        • main (ed25519) - 1.234 GSPR
        • savings (ed25519) - 10.567 GSPR
        • trading (secp256k1) - 0.891 GSPR

# Send tokens
walletd wallet send main 0x1234567890abcdef 1.5 --token GSPR
Output: 📤 Sending 1.5 GSPR from 'main' to '0x1234567890abcdef'
        Transaction hash: 0xabc123... (pending)
```

### Identity Management

#### **`walletd identity`** - Identity operations
```bash
# Create new identity
walletd identity create <NAME> [--key-algorithm <ALGO>]

# List identities
walletd identity list

# Sign message with identity
walletd identity sign <IDENTITY> <MESSAGE>

# Verify signature
walletd identity verify <IDENTITY> <MESSAGE> <SIGNATURE>
```

**Example Identity Operations:**
```bash
# Create identity
walletd identity create alice --key-algorithm ed25519
Output: 🆔 Creating identity 'alice' with ed25519
        ✅ Identity created with ID: ghost1234567890abcdef

# List identities
walletd identity list
Output: 🆔 AVAILABLE IDENTITIES:
        • alice (ed25519) - ghost1abc...
        • bob (secp256k1) - ghost2def...

# Sign message
walletd identity sign alice "Hello GhostChain"
Output: ✍️  Signing message with identity 'alice'
        Message: Hello GhostChain
        Signature: 0x3d4017c3e843895a19caa4d3a58fe4334fd4f0ca2b1b...
```

### Cryptographic Operations

#### **`walletd crypto`** - Crypto utilities
```bash
# Generate keypair
walletd crypto generate --algorithm <ALGO>

# Sign message
walletd crypto sign <MESSAGE> <PRIVATE_KEY> --algorithm <ALGO>

# Verify signature
walletd crypto verify <MESSAGE> <SIGNATURE> <PUBLIC_KEY> --algorithm <ALGO>

# Hash data
walletd crypto hash <DATA> --algorithm <ALGO>
```

**Example Crypto Operations:**
```bash
# Generate Ed25519 keypair
walletd crypto generate --algorithm ed25519
Output: 🔑 Generating ed25519 keypair:
        Private key: 0x1234567890abcdef... (keep secure!)
        Public key: 0xabcdef1234567890...

# Hash data with Blake3
walletd crypto hash "Hello World" --algorithm blake3
Output: 🔢 Hashing with blake3 algorithm:
        Data: Hello World
        Hash: 0x3a985da74fe225b2...
```

## ⚙️ **Configuration**

### Configuration File (`walletd.toml`)

```toml
[wallet]
default_algorithm = "ed25519"
enable_hd_wallets = true
derivation_path = "m/44'/60'/0'/0"
auto_save = true
backup_enabled = true
backup_interval_hours = 24

[network]
ghostd_endpoint = "http://127.0.0.1:8547"
timeout_seconds = 30
retry_attempts = 3
enable_ssl = false

[api]
enabled = true
bind_address = "127.0.0.1:8548"
enable_cors = true
allowed_origins = ["*"]
auth_required = true
max_connections = 100
request_timeout_seconds = 30

[zquic]
enabled = true
bind_address = "0.0.0.0:8550"
max_connections = 1000
timeout_ms = 30000
enable_compression = true
ghostbridge_enabled = true

[identity]
enabled = true
default_algorithm = "ed25519"
auto_backup = true
enable_social_recovery = false

[security]
encryption_enabled = true
password_required = true
session_timeout_minutes = 60
max_failed_attempts = 5
hardware_wallet_support = false
yubikey_support = false

[storage]
data_dir = "./walletd_data"
database_type = "Sqlite"
encryption_at_rest = true
backup_retention_days = 90
max_wallet_count = 100

[logging]
level = "info"
output = "Stdout"
enable_json = false
audit_enabled = true
```

### Environment Variables

```bash
# Logging
export RUST_LOG=info

# Network
export GHOSTCHAIN_GHOSTD_ENDPOINT=http://ghostd:8547
export GHOSTCHAIN_DATA_DIR=/opt/ghostchain

# Security
export WALLETD_MASTER_PASSWORD=your_secure_password
export WALLETD_ENCRYPTION_KEY=your_encryption_key
```

## 🌐 **API Endpoints**

### REST API (Port 8548)

#### Health Check
```bash
curl http://localhost:8548/health

Response:
{
  "status": "healthy",
  "version": "0.3.0",
  "timestamp": "2024-01-01T00:00:00Z"
}
```

#### Authentication
```bash
# Login
curl -X POST http://localhost:8548/api/v1/auth/login \
  -H "Content-Type: application/json" \
  -d '{
    "password": "your_password"
  }'

Response:
{
  "success": true,
  "data": {
    "token": "jwt_token_here",
    "expires_in": 3600
  }
}
```

#### Wallet Operations
```bash
# List wallets
curl -H "Authorization: Bearer <token>" \
  http://localhost:8548/api/v1/wallets

# Create wallet
curl -X POST http://localhost:8548/api/v1/wallets \
  -H "Authorization: Bearer <token>" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "main",
    "algorithm": "ed25519"
  }'

# Get wallet balance
curl -H "Authorization: Bearer <token>" \
  http://localhost:8548/api/v1/wallets/main/balance

# Send transaction
curl -X POST http://localhost:8548/api/v1/wallets/main/send \
  -H "Authorization: Bearer <token>" \
  -H "Content-Type: application/json" \
  -d '{
    "to": "0x1234567890abcdef",
    "amount": "1500000000000000000",
    "token": "GSPR"
  }'
```

#### Identity Operations
```bash
# List identities
curl -H "Authorization: Bearer <token>" \
  http://localhost:8548/api/v1/identities

# Create identity
curl -X POST http://localhost:8548/api/v1/identities \
  -H "Authorization: Bearer <token>" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "alice",
    "algorithm": "ed25519"
  }'

# Sign message
curl -X POST http://localhost:8548/api/v1/identities/alice/sign \
  -H "Authorization: Bearer <token>" \
  -H "Content-Type: application/json" \
  -d '{
    "message": "Hello GhostChain"
  }'
```

## 🔐 **Security Features**

### Encryption at Rest
```bash
# Enable encryption for new wallet
walletd wallet create secure --algorithm ed25519 --encrypt

# Configure encryption in config
[security]
encryption_enabled = true
password_required = true
```

### Multi-Signature Wallets
```bash
# Create 2-of-3 multisig wallet
walletd wallet create multisig \
  --algorithm ed25519 \
  --multisig \
  --required-signatures 2 \
  --total-signers 3

# Add signers
walletd wallet multisig add-signer multisig 0xpubkey1
walletd wallet multisig add-signer multisig 0xpubkey2
walletd wallet multisig add-signer multisig 0xpubkey3
```

### Hardware Wallet Integration
```bash
# Enable hardware wallet support
[security]
hardware_wallet_support = true
yubikey_support = true

# Create hardware-backed wallet
walletd wallet create hardware --hardware-device ledger
```

### Session Management
```bash
# Configure session timeout
[security]
session_timeout_minutes = 30
max_failed_attempts = 3

# Lock wallet after inactivity
walletd wallet lock main

# Unlock with password
walletd wallet unlock main
```

## 🐳 **Docker Deployment**

### Standalone Container
```bash
# Run wallet daemon
docker run -d \
  --name walletd \
  -p 8548:8548 \
  -p 8550:8550 \
  -v walletd-data:/opt/ghostchain/wallets \
  -v walletd-keys:/opt/ghostchain/keys \
  -e GHOSTCHAIN_GHOSTD_ENDPOINT=http://ghostd:8547 \
  ghostchain/walletd:latest

# Run testnet wallet
docker run -d \
  --name walletd-testnet \
  -p 18548:8548 \
  -p 18550:8550 \
  -v walletd-testnet-data:/opt/ghostchain/wallets \
  ghostchain/walletd:latest \
  walletd start --testnet --enable-quic
```

### Docker Compose
```yaml
# docker-compose.yml
services:
  walletd:
    image: ghostchain/walletd:latest
    ports:
      - "8548:8548"
      - "8550:8550"
    volumes:
      - walletd-data:/opt/ghostchain/wallets
      - walletd-keys:/opt/ghostchain/keys
      - walletd-config:/opt/ghostchain/config
      - ./config/walletd.toml:/opt/ghostchain/config/walletd.toml
    environment:
      - RUST_LOG=info
      - GHOSTCHAIN_GHOSTD_ENDPOINT=http://ghostd:8547
    command: [
      "walletd", "start",
      "--enable-quic"
    ]
    depends_on:
      - ghostd

volumes:
  walletd-data:
  walletd-keys:
  walletd-config:
```

## 🔧 **Integration Examples**

### With GhostD
```bash
# Ensure ghostd is running
curl http://localhost:8547/api/v1/health

# Configure walletd to connect
export GHOSTCHAIN_GHOSTD_ENDPOINT=http://localhost:8547
walletd start --enable-quic

# Create wallet and check balance
walletd wallet create main --algorithm ed25519
walletd wallet balance main
```

### Web Application Integration
```javascript
// JavaScript example using fetch
async function createWallet(name, algorithm) {
  const response = await fetch('http://localhost:8548/api/v1/wallets', {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
      'Authorization': `Bearer ${authToken}`
    },
    body: JSON.stringify({
      name: name,
      algorithm: algorithm
    })
  });
  
  return await response.json();
}

async function sendTransaction(from, to, amount, token = 'GSPR') {
  const response = await fetch(`http://localhost:8548/api/v1/wallets/${from}/send`, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
      'Authorization': `Bearer ${authToken}`
    },
    body: JSON.stringify({
      to: to,
      amount: amount,
      token: token
    })
  });
  
  return await response.json();
}
```

### Python Integration
```python
import requests
import json

class WalletdClient:
    def __init__(self, base_url="http://localhost:8548", auth_token=None):
        self.base_url = base_url
        self.auth_token = auth_token
        self.headers = {
            'Content-Type': 'application/json',
            'Authorization': f'Bearer {auth_token}' if auth_token else None
        }
    
    def create_wallet(self, name, algorithm="ed25519"):
        response = requests.post(
            f"{self.base_url}/api/v1/wallets",
            headers=self.headers,
            json={"name": name, "algorithm": algorithm}
        )
        return response.json()
    
    def get_balance(self, wallet_name):
        response = requests.get(
            f"{self.base_url}/api/v1/wallets/{wallet_name}/balance",
            headers=self.headers
        )
        return response.json()
    
    def send_transaction(self, from_wallet, to_address, amount, token="GSPR"):
        response = requests.post(
            f"{self.base_url}/api/v1/wallets/{from_wallet}/send",
            headers=self.headers,
            json={"to": to_address, "amount": amount, "token": token}
        )
        return response.json()

# Usage
client = WalletdClient(auth_token="your_jwt_token")
wallet = client.create_wallet("main", "ed25519")
balance = client.get_balance("main")
tx = client.send_transaction("main", "0x1234...", "1000000000000000000")
```

## 📊 **Monitoring and Maintenance**

### Backup Management
```bash
# Manual backup
walletd backup create --output ./wallet-backup-$(date +%Y%m%d).zip

# Restore from backup
walletd backup restore ./wallet-backup-20240101.zip

# Configure automatic backups
[wallet]
backup_enabled = true
backup_interval_hours = 24
```

### Health Monitoring
```bash
# Check daemon health
curl -f http://localhost:8548/health || echo "Service down"

# Monitor wallet operations
curl -H "Authorization: Bearer <token>" \
  http://localhost:8548/api/v1/stats

# View audit logs
tail -f ./walletd_data/audit.log
```

### Performance Tuning
```toml
[performance]
# Increase cache for better performance
cache_size_mb = 512

# Optimize database settings
[storage]
database_type = "Sqlite"
enable_wal_mode = true
cache_size_kb = 10000
```

## 🐛 **Troubleshooting**

### Common Issues

**Cannot Connect to GhostD**
```bash
# Check ghostd is running
curl http://localhost:8547/api/v1/health

# Update connection settings
export GHOSTCHAIN_GHOSTD_ENDPOINT=http://localhost:8547
walletd start
```

**Wallet Locked/Encrypted**
```bash
# Unlock wallet
walletd wallet unlock main

# Reset password (requires mnemonic)
walletd wallet recover main --mnemonic "your twelve word phrase"
```

**Permission Denied**
```bash
# Fix data directory permissions
sudo chown -R $USER:$USER ./walletd_data
chmod 700 ./walletd_data
```

### Debug Information
```bash
# Enable debug logging
RUST_LOG=debug walletd start

# View wallet state
walletd wallet list --verbose

# Check configuration
walletd config show
```

## 📚 **Additional Resources**

- **[GhostChain README](README.md)**: Main project documentation
- **[GhostD Documentation](GHOSTD.md)**: Blockchain daemon guide
- **[Authentication Guide](AUTH.md)**: Security and authentication
- **[Identity Documentation](IDENTITY.md)**: RealID integration
- **[Smart Contracts](SMARTCONTRACT.md)**: Contract interaction

## 🤝 **Support**

- **GitHub Issues**: [ghostkellz/ghostchain/issues](https://github.com/ghostkellz/ghostchain/issues)
- **Documentation**: [GhostChain Docs](README.md)
- **Discord**: Join the GhostMesh community

---

*WalletD is part of the GhostChain ecosystem. For the complete platform overview, see the [main README](README.md).*