# 👻 GhostChain

[![Rust](https://img.shields.io/badge/Rust-2024-informational?logo=rust)](https://www.rust-lang.org/)
[![Workspace](https://img.shields.io/badge/Architecture-Monorepo%20Workspace-blue)](https://doc.rust-lang.org/book/ch14-03-cargo-workspaces.html)
[![Shroud](https://img.shields.io/badge/Transport-Shroud%20%7C%20QUIC%20%7C%20HTTP3-0a8fdc)](https://github.com/ghostkellz/shroud)
[![Docker](https://img.shields.io/badge/Deployment-Docker%20%7C%20Compose-2496ED?logo=docker)](https://www.docker.com/)
[![License](https://img.shields.io/badge/License-Apache%202.0-green)](LICENSE)

> **High-performance blockchain platform** with integrated wallet services, built on Rust workspace architecture with Shroud transport and native Zig cryptography.

---
<p align="center">
  <img src="assets/gcc-logo.png" alt="GhostChain Logo" width="240"/>
</p>

---

## 🏗️ **Workspace Architecture**

GhostChain uses a **modern Rust workspace** for unified development across multiple services:

```
ghostchain/
├── 📦 Cargo.toml (workspace root)
├── 🔧 core/           # Blockchain implementation (ghostchain-core)
├── 🔗 shared/         # Common types, crypto, FFI (ghostchain-shared)
├── 👻 ghostd/         # Blockchain daemon with Shroud
├── 💼 walletd/        # Secure wallet daemon with identity
├── 🧪 integration-tests/  # Cross-service testing
├── 🐳 docker/         # Container deployment
├── 📋 scripts/        # Build and development tools
└── 📚 reference-docs/ # Archive and reference materials
```

## 🚀 **Core Services**

### 👻 **GhostD** - Blockchain Daemon
High-performance blockchain node with consensus and mining capabilities.

**Features:**
- **Shroud Transport**: Ultra-fast QUIC/HTTP3-based networking via ghostwire
- **Mining & Consensus**: Automated block production and validation
- **Multi-Domain ZNS**: ENS, Unstoppable, Web5, and native Ghost domains
- **Smart Contracts**: Full contract execution with gas metering
- **Performance Monitoring**: Real-time metrics and optimization

```bash
# Start mainnet node
ghostd start --enable-quic --enable-mining

# Start testnet for development
ghostd start --testnet --bind-address 0.0.0.0:8545

# Get blockchain status
ghostd status
```

### 💼 **WalletD** - Secure Wallet Daemon
Advanced wallet management with multi-algorithm support and identity services.

**Features:**
- **Multi-Algorithm**: Ed25519, Secp256k1, Secp256r1 support
- **HD Wallets**: Hierarchical deterministic key management
- **Identity (RealID)**: Decentralized identity management
- **Hardware Support**: Ready for hardware wallet integration
- **Shroud Integration**: High-performance transport and cryptography

```bash
# Start wallet daemon
walletd start --enable-quic

# Create new wallet
walletd wallet create main --algorithm ed25519

# Create identity
walletd identity create alice --key-algorithm ed25519

# Send tokens
walletd wallet send main 0xabc... 1.5 --token GSPR
```

## 🌐 **Token Ecosystem**

- **🌟 GSPR (Ghost Spirit)**: Primary native token (21B max supply)
- **💎 GCC (GhostChain Credits)**: Utility token for contracts and operations
- **⚡ GMAN (Ghost Mana)**: Governance and staking rewards (earned through participation)
- **🔮 SOUL**: Non-transferable identity tokens

## 🔧 **Quick Start**

### Option 1: Docker Deployment (Recommended)
```bash
# Development environment
./scripts/start-dev.sh

# Full production stack
docker-compose up --build

# Testnet only
docker-compose -f docker-compose.dev.yml up
```

### Option 2: Native Build
```bash
# Build entire workspace
cargo build --release --workspace

# Run specific service
cargo run --bin ghostd -- start --testnet
cargo run --bin walletd -- start --testnet
```

### Option 3: Individual Services
```bash
# Install and run ghostd
cargo install --path ghostd
ghostd start --testnet

# Install and run walletd  
cargo install --path walletd
walletd start --testnet
```

## 🐳 **Docker Services**

The docker-compose setup includes:

- **ghostd/walletd**: Main blockchain and wallet services
- **ghostd-testnet/walletd-testnet**: Development testnet
- **Redis**: Caching and session storage
- **PostgreSQL**: Analytics and indexing
- **Nginx**: Reverse proxy and load balancing
- **Prometheus/Grafana**: Monitoring and visualization

**Service Ports:**
- GhostD RPC: `8545` (mainnet), `18545` (testnet)
- GhostD API: `8547` (mainnet), `18547` (testnet)
- WalletD API: `8548` (mainnet), `18548` (testnet)
- Grafana: `3000` (admin: `ghostchain_admin`)
- Prometheus: `9090`

## 🔐 **Security & Features**

### Cryptography
- **ZCrypto Integration**: Ed25519, Secp256k1, Blake3, SHA256
- **Quantum-Ready**: Post-quantum cryptography support planned
- **Hardware Integration**: YubiKey and hardware wallet support

### Transport
- **ZQUIC**: High-performance QUIC implementation in Zig
- **GhostBridge**: gRPC over QUIC for service communication
- **IPv6 First**: Native IPv6 support with dual-stack fallback

### Performance
- **Async Runtime**: Full Tokio async/await implementation
- **Multi-Level Caching**: Advanced caching with LRU and TTL
- **Connection Pooling**: Optimized service communication
- **Batch Processing**: High-throughput transaction processing

## 📚 **Documentation**

### Core Documentation
- **[AUTH.md](AUTH.md)**: Authentication and authorization
- **[SMARTCONTRACT.md](SMARTCONTRACT.md)**: Smart contract development
- **[PROTOCOLS.md](PROTOCOLS.md)**: Network protocols and standards
- **[DOMAINS.md](DOMAINS.md)**: Multi-domain name system (ZNS)
- **[WEB5.md](WEB5.md)**: Web5 and DID integration
- **[TOKEN.md](TOKEN.md)**: Token economics and management

### Development
- **[CLAUDE.md](CLAUDE.md)**: Architecture and development notes
- **[CONTRACT.md](CONTRACT.md)**: Contract deployment and management
- **[IDENTITY.md](IDENTITY.md)**: Identity and RealID integration
- **[WALLET.md](WALLET.md)**: Wallet development and API

### Reference
- **[reference-docs/](reference-docs/)**: Archive and historical documents
- **[legacy-archive/](legacy-archive/)**: Previous implementations

## 🛠️ **Development**

### Building from Source
```bash
# Clone repository
git clone https://github.com/ghostkellz/ghostchain.git
cd ghostchain

# Build workspace
cargo build --workspace

# Run tests
cargo test --workspace

# Run integration tests
cargo test --package ghostchain-integration-tests
```

### Development Environment
```bash
# Start development stack
./scripts/start-dev.sh

# Build Docker images
./scripts/docker-build.sh

# Check services
curl http://localhost:8547/api/v1/status
curl http://localhost:8548/health
```

## 🗺️ **Roadmap**

### ✅ **Completed (v0.3.0)**
- Monorepo workspace architecture
- GhostD blockchain daemon with ZQUIC
- WalletD secure wallet daemon
- Multi-service Docker deployment
- Smart contract execution engine
- Multi-domain name resolution (ENS, UD, Web5, Ghost)
- Performance monitoring and optimization

### 🚧 **In Progress**
- ZQUIC FFI integration completion
- GhostBridge gRPC relay implementation
- Hardware wallet integration
- Enhanced security audit

### 📋 **Planned**
- Web5 DID full implementation
- Zero-knowledge proof integration
- Cross-chain interoperability
- Mobile wallet applications
- Decentralized exchange (DEX)

## 🤝 **Contributing**

1. **Fork** the repository
2. **Create** a feature branch (`git checkout -b feature/amazing-feature`)
3. **Build** and test (`cargo test --workspace`)
4. **Commit** changes (`git commit -m 'Add amazing feature'`)
5. **Push** to branch (`git push origin feature/amazing-feature`)
6. **Open** a Pull Request

## 📄 **License**

This project is licensed under the **Apache License 2.0** - see the [LICENSE](LICENSE) file for details.

## 👤 **Author**

Built by [@ghostkellz](https://github.com/ghostkellz) as part of the **GhostMesh** ecosystem.

---

**🔗 Related Projects:**
- [ZQUIC](https://github.com/ghostkellz/zquic) - High-performance QUIC implementation
- [GhostBridge](https://github.com/ghostkellz/ghostbridge) - Cross-service communication
- [ZCrypto](https://github.com/ghostkellz/zcrypto) - Cryptographic library

*For additional documentation and references, see the [reference-docs/](reference-docs/) directory.*