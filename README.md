# 👻 GhostChain (Zig Edition)

> **The next-gen pure Zig blockchain platform:** blazing-fast ledger, mesh-native, quantum-secure, full identity stack, and smart contracts—all in Zig.

---

## ⚡ Zig is Now Canonical!

> **Note:** The Zig implementation is now the canonical and actively developed version of GhostChain. The Rust workspace is archived as a legacy reference. **All new features, modules, and security upgrades are now Zig-first.**

---

## 🚦 Core Zig Integrations

GhostChain integrates the entire Ghostkellz Zig ecosystem:

* [zledger](https://github.com/ghostkellz/zledger) — Distributed ledger core
* [zsig](https://github.com/ghostkellz/zsig) — Digital signatures & multisig
* [zquic](https://github.com/ghostkellz/zquic) — QUIC protocol networking
* [ghostnet](https://github.com/ghostkellz/ghostnet) — Mesh overlay networking
* [zcrypto](https://github.com/ghostkellz/zcrypto) — Cryptographic primitives
* [zwallet](https://github.com/ghostkellz/zwallet) — HD wallet & key management
* [keystone](https://github.com/ghostkellz/keystone) — Node bootstrap & network identity
* [zvm](https://github.com/ghostkellz/zvm) — WASM/EVM-compatible smart contract VM
* [zns](https://github.com/ghostkellz/zns) — Zig Name System
* [wraith](https://github.com/ghostkellz/wraith) — Programmable reverse proxy
* [shroud](https://github.com/ghostkellz/shroud) — Identity & privacy (DID, SSO, ZKP)
* [zsync](https://github.com/ghostkellz/zsync) — Async runtime/synchronization
* [cns](https://github.com/ghostkellz/cns) — Chain Name Service (legacy/interoperability)

You can fetch any component with Zig:

```sh
zig fetch --save https://github.com/ghostkellz/<project>/archive/main.tz
```

---

## 🧭 Ghostchain ZNS Domains (arc/warp/gcp v2)

This section defines the namespace TLDs (top-level domains) used in the Ghostchain ZNS (Zig Name System), Ghostchain’s native decentralized naming layer. These domains provide zero-trust identity, smart contract routing, cryptographic key mapping, service resolution, scaling, bridging, and analytics.

### 🧬 Core Identity Domains

| Domain   | Description                                                                                                       |
| -------- | ----------------------------------------------------------------------------------------------------------------- |
| `.ghost` | Root domain of Ghostchain identities and services. Reserved for core system nodes and canonical identity anchors. |
| `.gcc`   | GhostChain Contracts — used for contracts, DAOs, and on-chain logic entities.                                     |
| `.sig`   | Signature authorities and verifiers (maps to public signing keys or validators).                                  |
| `.gpk`   | Ghostchain Public Key registry — generic identity key mapping layer.                                              |
| `.key`   | Public key alias domain (interchangeable with `.gpk` but scoped to manual entries).                               |
| `.pin`   | Persistent Identity Node — stable DID/device/service binding. Sessionless identities or hardware-bound.           |

### 🌐 Arc/Warp/GCP Ecosystem Domains

| Domain  | Description                                                                           |
| ------- | ------------------------------------------------------------------------------------- |
| `.warp` | GhostPlane Layer 2 rollups, batchers, bridges, and L2-native services.                |
| `.arc`  | Cross-domain (L1/L2) bridges, protocol governance, analytics, DAOs, protocol anchors. |
| `.gcp`  | GhostChain Platform: system admin, registry contracts, privileged utilities.          |

### 🔗 Decentralized & Blockchain Infrastructure

| Domain | Description                                                                           |
| ------ | ------------------------------------------------------------------------------------- |
| `.bc`  | General blockchain assets and services, interoperable with other chains.              |
| `.zns` | Root namespace registry (similar to `.eth` for ENS, controls TLDs within Ghostchain). |
| `.ops` | Operational nodes — infrastructure endpoints, gateways, proxies, observability units. |

### 📂 Reserved for Future/Extension Use

| Domain | Description                                                                      |
| ------ | -------------------------------------------------------------------------------- |
| `.sid` | Secure identity domain (may be used for ephemeral tokens or session-based DID).  |
| `.dvm` | Decentralized Virtual Machine domains (ghostVM, zkVM or Wasm runtime instances). |
| `.tmp` | Temporary identity bindings or sandbox test chains.                              |
| `.dbg` | Debug/testnet addresses — useful for ZNS test environments or dummy data.        |
| `.lib` | Shared contract libraries and reusable ghostchain modules.                       |
| `.txo` | Transaction-output indexed namespaces (ideal for financial contracts or flows).  |

**Note:** These domains are managed by the root ZNS registry contract (`registry.gcp` or `zns.ghost`) and enforced via GhostToken signature validation through `realid` and `zsig`.

---

## 🌐 GhostChain Web5 Vision

GhostChain is designed for the next evolution of the internet: **Web5**.

* **Web5 merges Web2 infrastructure (DNS, TLS, HTTP) with Web3 decentralization, identity, and programmable logic.**
* **Ghostchain powers cryptographically secure, real-time, programmable internet applications.**
* **Backwards-compatible with IPv4/HTTP but built for QUIC, HTTP/3, gRPC, and IPv6.**

### Goals

* ✅ Reclaim identity and trust from centralized silos (Google, Meta, CAs)
* ✅ Build a user-first, programmable protocol stack
* ✅ Support smart contracts, payments, messaging, and decentralized apps (dApps)
* ✅ Enable low-latency, privacy-preserving applications
* ✅ Operate on existing hardware and global internet

### Tech Stack Foundations

| Layer           | Technology Used                     | Purpose                                       |
| --------------- | ----------------------------------- | --------------------------------------------- |
| Transport       | QUIC + HTTP/3 + gRPC                | Real-time, secure, efficient communication    |
| Addressing      | IPv6 + DNS + DID + cDNS             | Global addressing with decentralized IDs      |
| Identity        | zkID + DID + Verifiable Credentials | Privacy-aware identity with recoverability    |
| Trust Layer     | GhostChain (PoS + PoC)              | Fast, eco-friendly consensus & record anchor  |
| Auth + Access   | GhostSSO, OIDC bridges              | Seamless auth across dApps and legacy systems |
| Smart Contracts | WASM, Zig VM                        | Gas-efficient, deterministic compute          |
| Data Layer      | GhostVault + zkStorage + IPFS       | Secure storage of user and state data         |

### Unique Web5 Features

* 🧱 Programmable internet stack (smart cDNS, programmable TLS)
* 🪪 GhostID and QID as universal identity anchors
* 🔄 Built-in cryptographic recovery
* 🔍 Auditable but privacy-respecting by design
* 💡 Runs on today’s infrastructure—Linux, nginx, Docker, browsers

### Security Model

* ZK-based proofs for identity & claims
* QUIC + TLS 1.3 for all transport
* cDNS + DNSSEC + DANE-like record verification
* Mesh-aware firewalls and routing via GhostMesh

### Adoption Path

1. GhostChain testnet over GhostMesh and Web2
2. cDNS and GhostDNS for secure domain routing
3. GhostVault as default identity, key, and config provider
4. Web5 SDK for dApp and CLI tool developers
5. Backwards-compatible public gateway for Web2 interaction

**Web5 by GhostChain is the natural evolution of the web—distributed, programmable, encrypted, and user-owned.**

---

## 🚀 Node Features

* **Quantum-Safe Cryptography:** Post-quantum TLS 1.3 via ZQUIC, Ed25519, Schnorr, BLAKE3
* **Mesh-Native Networking:** Peer discovery, relay, NAT traversal via ghostnet
* **Smart Contracts:** WASM/EVM/custom Zig VM (ZVM), contract CLI, and programmable covenants
* **Identity & Privacy:** Full decentralized ID, SSO, and ZKP support via shroud
* **Name Systems:** ZNS and CNS for ENS, UD, Web5, and Ghost domains
* **Wallet:** HD, multisig, and hardware-ready wallet support (zwallet)
* **Async Everything:** Modern, scalable async runtime with zsync
* **Production-Ready:** Docker, Prometheus/Grafana, testnet/mainnet quickstart

---

## 🌐 Token Ecosystem

* **🌟 GSPR (Ghost Spirit):** Main native token (21B max supply)
* **💎 GCC (GhostChain Credits):** Utility token for contracts/operations
* **⚡ GMAN (Ghost Mana):** Governance/staking
* **🔮 SOUL:** Non-transferable identity token

---

## 🔧 Quick Start

### 1. **Clone & Build**

```sh
git clone https://github.com/ghostkellz/ghostchain.git
cd ghostchain
zig build -Drelease-fast
```

Or, **add to any Zig project** via:

```sh
zig fetch --save https://github.com/ghostkellz/ghostchain/archive/main.tz
```

*Or use `zion fetch` if you are using the Zion package manager/TUI.*

### 2. **Run a Node (Testnet)**

```sh
./zig-out/bin/ghostchaind --testnet
```

### 3. **Wallet Operations**

```sh
./zig-out/bin/zwallet create main --algorithm ed25519
./zig-out/bin/zwallet send main 0xabc... 1.5 --token GSPR
```

---

## 🧪 ZEKE: Experimental Zig AI Agent Architecture
 

ZEKE is a pure Zig, async-first agent system designed for Ghostchain, smart contract automation, network security, and programmable DevOps. It is modular, extensible, and built for future mesh-native, blockchain, and device management operations.

---

## Agent Architecture

### Core Framework (`/src/agent/mod.zig`)

* **Agent trait:** Implements agent logic with function pointers for extensibility
* **AgentManager:** Orchestrates multiple concurrent agents
* **AgentType enum:** Defines domains (blockchain, contract, network, security, custom)
* **AgentResult:** Standardized response for all commands/queries

### Specialized Agents

* **blockchain.zig:** Network ops, gas monitoring, health checks
* **smartcontract.zig:** Contract deployment, calls, auditing
* **network.zig:** Network scan, monitoring, traffic analysis
* **security.zig:** Security scanning, hardening, threat detection

---

## Terminal Integration

The terminal app supports a unified CLI for agent control:

```sh
# List all available agents
zeke agent list

# Blockchain operations
zeke agent blockchain status
zeke agent blockchain balance 0x123...
zeke agent blockchain health

# Smart contract interactions
zeke agent smartcontract deploy <bytecode>
zeke agent smartcontract call <address> <method>
zeke agent smartcontract audit <address>

# Network operations
zeke agent network scan 192.168.1.0/24
zeke agent network ping google.com
zeke agent network ports 192.168.1.1

# Security operations
zeke agent security scan
zeke agent security monitor
zeke agent security firewall enable
```

---

## Key Features

1. **Pure Zig Implementation** — No external dependencies, leveraging Zig's speed
2. **Extensible Design** — Easy to add new agent types/domains/commands
3. **Function Pointer Architecture** — Runtime behavior customization
4. **Standardized Interface** — Consistent CLI & TUI command structure
5. **Future-Ready** — Scales to mesh, device, LLM/AI, and blockchain automation

---

---

## 🤝 Contributing

1. **Fork** the repository
2. **Create** a feature branch (`git checkout -b feature/amazing-feature`)
3. **Build** and test (`zig build test`)
4. **Commit** changes (`git commit -m 'Add amazing feature'`)
5. **Push** to branch (`git push origin feature/amazing-feature`)
6. **Open** a Pull Request

---

## 📄 License

Licensed under the **MIT**. See the [LICENSE](LICENSE) file for details.

---

## 👤 Author

Built by [@ghostkellz](https://github.com/ghostkellz) as part of the **GhostMesh** ecosystem.

---

## 🔗 Related Projects

* [zquic](https://github.com/ghostkellz/zquic) — QUIC protocol
* [ghostbridge](https://github.com/ghostkellz/ghostbridge) — gRPC/FFI cross-chain bridge
* [zcrypto](https://github.com/ghostkellz/zcrypto) — Cryptography

**See `docs/` for additional documentation, integration guides, and specs.**

