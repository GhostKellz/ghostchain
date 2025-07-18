# 👻 GhostChain

[![Rust](https://img.shields.io/badge/Rust-2024-informational?logo=rust)](https://www.rust-lang.org/)
[![clap CLI](https://img.shields.io/badge/CLI-clap-blue?logo=command-line)](https://github.com/clap-rs/clap)
[![Crypto](https://img.shields.io/badge/Crypto-Ed25519%20%7C%20Blake3%20%7C%20X25519-purple?logo=cryptpad)]()
[![QUIC](https://img.shields.io/badge/Networking-QUIC%20%7C%20HTTP3-0a8fdc?logo=quic)](https://github.com/quinn-rs/quinn)
[![Blockchain](https://img.shields.io/badge/Blockchain-PoS%20%7C%20Multi--Token%20%7C%20zkVM-green?logo=ethereum)]()
[![Async](https://img.shields.io/badge/Async-Tokio%20%7C%20Rust%20Futures-orange?logo=tokio)]()

> A modular, privacy-respecting Layer-1 blockchain written in **Rust**, designed for secure messaging, anonymous transactions, and ultra-fast consensus.

---
<p align="center">
  <img src="https://github.com/ghostkellz/ghostchain/raw/main/assets/gcc-logo.png" alt="GhostChain Logo" width="240"/>
</p>

---
## 🌐 Project Overview

**GhostChain** is a next-generation blockchain framework focused on:

* 🔐 End-to-end encryption by default
* 🌍 QUIC/HTTP3 transport via `quinn`
* ⚡ Fully async runtime using `tokio`
* 🧠 Modular architecture for consensus, accounts, state, and contracts
* 🧩 Flexible VM support (e.g., WASM, interpreted, ZKVM)
* 🕵️ Zero-trust identity via Ed25519/X25519

---

## 📁 Project Structure

```
GhostChain
├── crates/
│   ├── ghostchain     # Core blockchain logic (blocks, ledger, tx)
│   ├── ghostnet      # Networking over QUIC
│   ├── ghostcrypto   # Cryptographic primitives
│   ├── ghostvm       # Optional WASM/ZK VM
│   └── gcc           # CLI node and tools
├── examples/
│   ├── minimal_node.rs
│   └── quic_echo.rs
└── Cargo.toml
```

---

## 🚀 Features Implemented

✅ **Core Blockchain**

* Advanced block structure with cryptographic hash linking
* Genesis block creation with configurable parameters
* Complete transaction system with multiple types
* Comprehensive state management and validation
* Block validation and state transitions

✅ **Proof of Stake Consensus**

* Weighted validator selection based on stake
* Epoch-based validator rotation
* Performance tracking and slashing mechanics
* Minimum stake requirements (100k SPIRIT)
* Active validator management (up to 100 validators)

✅ **Multi-Token System**

* **SPIRIT (SPR):** Gas/utility token (1B supply, 18 decimals)
* **MANA (MNA):** Contribution rewards (dynamic supply)
* **RLUSD:** Stablecoin (100M supply, 18 decimals)
* **SOUL:** Soulbound identity tokens (0 decimals)

✅ **Transaction Types**

* Token transfers
* Account creation (Ed25519 keys)
* Staking/unstaking, validator registration
* Soul token minting, contribution rewards

✅ **P2P Networking Infrastructure**

* QUIC-based networking (via `quinn`)
* Peer discovery and management
* Async message handling, NAT traversal

✅ **Persistent Storage**

* Sled-based key-value storage engine
* Block and account state persistence
* Transaction and validator history

✅ **Enhanced CLI Interface**

* Account management (create, balance, info)
* Token operations (transfer, stake, list)
* Chain, node, and peer info
* Node/network startup & RPC server management

---

## 🔧 Quick Start

```bash
cargo build --release
```

**Create a new account:**

```bash
cargo run -- account new
```

**Check chain info:**

```bash
cargo run -- chain info
```

**List available tokens:**

```bash
cargo run -- token list
```

**Start a node with networking:**

```bash
cargo run -- node --bind 0.0.0.0:7777 --chain-id ghostchain-devnet
```

**Start with persistent storage:**

```bash
cargo run -- node --bind 0.0.0.0:7777 --data-dir ./ghostchain-data
```

**Connect to peers:**

```bash
cargo run -- node --bind 0.0.0.0:7778 --peer 127.0.0.1:7777
```

**Start RPC server:**

```bash
cargo run -- rpc --bind 0.0.0.0:8545
```

---

## 🧩 Architecture

* Written in Rust (2024 edition)
* Async/await with Tokio runtime
* Modular design (blockchain, consensus, crypto, token, network, storage, CLI)
* Pluggable consensus and VM engines
* QUIC networking and async message relay

---

## 🏗️ Advanced Features

### Cryptography

* Ed25519 digital signatures, Blake3 hashing
* Secure random number generation
* Address derivation from pubkeys

### Consensus Engine

* Pluggable PoS architecture, slashing, epoch rotation

### Storage Layer

* Sled embedded DB, atomic ops, block & state indexing

### Networking

* QUIC P2P with peer discovery
* Async message passing and NAT traversal

---

## 📊 Performance & Specifications

* Proof of Stake: 6s block times, up to 100 validators
* Embedded Sled DB
* Ed25519 + Blake3 cryptography
* Efficient async runtime

---

## 🎯 Recent Updates (v0.2.0)

### Smart Contract Platform ✅
* Native contract execution engine with gas metering
* Contract storage layer with isolation
* Domain registry and token manager contracts
* Full blockchain integration

### ZNS (Zig Name Service) ✅
* On-chain domain registration system
* DNS record management (A, AAAA, CNAME, MX, TXT)
* Domain ownership and transfers
* Smart contract integration

### RPC/API Layer ✅
* JSON-RPC server with Ethereum compatibility
* WebSocket support for real-time updates
* API authentication with key management
* Contract and domain management endpoints

### Service Integration ✅
* Service manager framework
* Clients for ghostd, walletd, zvm, ghostbridge, zquic
* Health monitoring and connection pooling
* CLI commands for service management

### Performance Optimizations ✅
* Advanced multi-level caching system
* Connection pooling for all services
* Batch processing for operations
* Comprehensive metrics and monitoring
* Optimized storage and network layers

## 🛣️ Roadmap / What's Next

See [WHATSNEEDEDNEXT.md](WHATSNEEDEDNEXT.md) for detailed implementation roadmap.

### Immediate Priorities:
1. Fix compilation issues (add JSON-RPC dependencies)
2. ZQUIC FFI integration for Rust services
3. GhostBridge gRPC implementation
4. Complete ghostd and walletd services
5. ZVM enhancements for EVM compatibility

## 📚 Documentation

* [CHANGELOG.md](CHANGELOG.md) - Detailed change history
* [WHATSNEEDEDNEXT.md](WHATSNEEDEDNEXT.md) - Implementation roadmap and priorities
* [CLAUDE.md](CLAUDE.md) - Development context and architecture notes

---

## 👤 Author

Built by [@ghostkellz](https://github.com/ghostkellz) as part of the GhostMesh ecosystem.

