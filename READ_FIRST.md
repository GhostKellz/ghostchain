# 🚦 Ghostchain Project Integration Overview

> How the core Ghostkellz Zig projects integrate with Ghostchain (blockchain/Web5/DLT stack)
>
> All projects are available at: `github.com/ghostkellz/<project>`
> For Zig users: `zig fetch --save https://github.com/ghostkellz/<project>/archive/main.tz`

---

## zledger

[https://github.com/ghostkellz/zledger](https://github.com/ghostkellz/zledger)
`zig fetch --save https://github.com/ghostkellz/zledger/archive/main.tz`

* **What:** Distributed ledger core (block/DAG storage, consensus engine, transaction logic)
* **Why:** The main blockchain state machine and transaction backbone of Ghostchain. All value/state/consensus flows through zledger.

## zsig

[https://github.com/ghostkellz/zsig](https://github.com/ghostkellz/zsig)
`zig fetch --save https://github.com/ghostkellz/zsig/archive/main.tz`

* **What:** Digital signatures & multisig library (Ed25519, Schnorr, threshold/multisig)
* **Why:** Ensures cryptographic authenticity of every transaction, block, message, and peer. Used everywhere Ghostchain needs signatures (wallets, consensus, smart contracts, mesh auth).

## zquic

[https://github.com/ghostkellz/zquic](https://github.com/ghostkellz/zquic)
`zig fetch --save https://github.com/ghostkellz/zquic/archive/main.tz`

* **What:** QUIC protocol library (secure, multiplexed, modern transport)
* **Why:** The foundation for fast, reliable, and NAT-friendly peer-to-peer connections. All Ghostchain node-to-node (and client) transport uses zquic (gossip, block sync, overlay RPC).

## ghostnet

[https://github.com/ghostkellz/ghostnet](https://github.com/ghostkellz/ghostnet)
`zig fetch --save https://github.com/ghostkellz/ghostnet/archive/main.tz`

* **What:** Overlay mesh network stack (peer discovery, NAT traversal, topology mgmt)
* **Why:** Lets Ghostchain nodes auto-discover each other, form overlays, traverse NAT/firewalls, and maintain a dynamic, self-healing mesh.

## zcrypto

[https://github.com/ghostkellz/zcrypto](https://github.com/ghostkellz/zcrypto)
`zig fetch --save https://github.com/ghostkellz/zcrypto/archive/main.tz`

* **What:** Cryptographic primitives library (hashes, curves, advanced ciphers)
* **Why:** Powers both zsig and core protocol crypto. Used for transaction hashes, block hashes, random beacons, proofs, and more.

## zwallet

[https://github.com/ghostkellz/zwallet](https://github.com/ghostkellz/zwallet)
`zig fetch --save https://github.com/ghostkellz/zwallet/archive/main.tz`

* **What:** Hierarchical deterministic (HD) wallet, key management, multisig accounts
* **Why:** The wallet/key engine for Ghostchain. Manages user keys, accounts, multisig, and provides signing support to apps and CLI tools.

## keystone

[https://github.com/ghostkellz/keystone](https://github.com/ghostkellz/keystone)
`zig fetch --save https://github.com/ghostkellz/keystone/archive/main.tz`

* **What:** Core blockchain infra tools (node bootstrap, config, network identity)
* **Why:** Handles network initialization, trusted node onboarding, and the root-of-trust for new Ghostchain deployments.

## zvm

[https://github.com/ghostkellz/zvm](https://github.com/ghostkellz/zvm)
`zig fetch --save https://github.com/ghostkellz/zvm/archive/main.tz`

* **What:** Virtual machine for smart contracts (WASM/EVM-compatible or custom Zig VM)
* **Why:** Executes smart contracts and chain logic. Enables decentralized applications and programmable on-chain state.

## zns

[https://github.com/ghostkellz/zns](https://github.com/ghostkellz/zns)
`zig fetch --save https://github.com/ghostkellz/zns/archive/main.tz`

* **What:** Zig Name System - decentralized, human-friendly names for addresses and resources
* **Why:** Maps human-readable names to Ghostchain accounts, contracts, or resources. Powers dapp UX, wallets, DNS-like services on-chain.

## cns

[https://github.com/ghostkellz/cns](https://github.com/ghostkellz/cns)
`zig fetch --save https://github.com/ghostkellz/cns/archive/main.tz`

* **What:** Chain Name Service (alternative/complement to zns, or legacy support)
* **Why:** Interop layer for non-zig, legacy, or multi-chain name resolution.

## wraith

[https://github.com/ghostkellz/wraith](https://github.com/ghostkellz/wraith)
`zig fetch --save https://github.com/ghostkellz/wraith/archive/main.tz`

* **What:** Programmable reverse proxy, L7 application mesh gateway
* **Why:** Handles secure, policy-driven API access, microservice routing, and app gateway logic for Ghostchain-powered dapps and mesh services.

## shroud

[https://github.com/ghostkellz/shroud](https://github.com/ghostkellz/shroud)
`zig fetch --save https://github.com/ghostkellz/shroud/archive/main.tz`

* **What:** Identity & security framework (DID, SSO, ZKP, privacy controls)
* **Why:** The core identity/auth layer for users, nodes, and apps. Provides decentralized ID (DID), zero-knowledge proofs, authentication, and privacy-preserving controls for everything built on Ghostchain.


## zsync
[https://github.com/ghostkellz/zsync](https://github.com/ghostkellz/zsync)
`zig fetch --save https://github.com/ghostkellz/zsync/archive/main.tz`

- **What:** Async primitives and synchronization library leveraging Zig v0.15+ modern async/await features
- **Why:** Provides efficient, scalable async operations and coordination for Ghostchain nodes, smart contracts, networking, and parallel tasks across your distributed ecosystem.

---

> All code: [https://github.com/ghostkellz/](https://github.com/ghostkellz/)<project>
> Example Zig install: `zig fetch --save https://github.com/ghostkellz/zsig/archive/main.tz`

