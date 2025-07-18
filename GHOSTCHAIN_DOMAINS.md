# 🧭 Ghostchain ZNS Domains (arc/warp/gcp v2)

This document defines the namespace TLDs (top-level domains) used in the Ghostchain ZNS (Zig Name System), Ghostchain’s native decentralized naming layer. These domains provide zero-trust identity, smart contract routing, cryptographic key mapping, service resolution, scaling, bridging, and analytics.

---

## 🧬 Core Identity Domains

| Domain   | Description                                                                                                       |
| -------- | ----------------------------------------------------------------------------------------------------------------- |
| `.ghost` | Root domain of Ghostchain identities and services. Reserved for core system nodes and canonical identity anchors. |
| `.gcc`   | GhostChain Contracts — used for contracts, DAOs, and on-chain logic entities.                                     |
| `.sig`   | Signature authorities and verifiers (maps to public signing keys or validators).                                  |
| `.gpk`   | Ghostchain Public Key registry — generic identity key mapping layer.                                              |
| `.key`   | Public key alias domain (interchangeable with `.gpk` but scoped to manual entries).                               |
| `.pin`   | Persistent Identity Node — stable DID/device/service binding. Sessionless identities or hardware-bound.           |

---

## 🌐 Arc/Warp/GCP Ecosystem Domains

| Domain  | Description                                                                           |
| ------- | ------------------------------------------------------------------------------------- |
| `.warp` | GhostPlane Layer 2 rollups, batchers, bridges, and L2-native services.                |
| `.arc`  | Cross-domain (L1/L2) bridges, protocol governance, analytics, DAOs, protocol anchors. |
| `.gcp`  | GhostChain Platform: system admin, registry contracts, privileged utilities.          |

---

## 🔗 Decentralized & Blockchain Infrastructure

| Domain | Description                                                                           |
| ------ | ------------------------------------------------------------------------------------- |
| `.bc`  | General blockchain assets and services, interoperable with other chains.              |
| `.zns` | Root namespace registry (similar to `.eth` for ENS, controls TLDs within Ghostchain). |
| `.ops` | Operational nodes — infrastructure endpoints, gateways, proxies, observability units. |

---

## 📂 Reserved for Future/Extension Use

| Domain | Description                                                                      |
| ------ | -------------------------------------------------------------------------------- |
| `.sid` | Secure identity domain (may be used for ephemeral tokens or session-based DID).  |
| `.dvm` | Decentralized Virtual Machine domains (ghostVM, zkVM or Wasm runtime instances). |
| `.tmp` | Temporary identity bindings or sandbox test chains.                              |
| `.dbg` | Debug/testnet addresses — useful for ZNS test environments or dummy data.        |
| `.lib` | Shared contract libraries and reusable ghostchain modules.                       |
| `.txo` | Transaction-output indexed namespaces (ideal for financial contracts or flows).  |

---

## ✅ Summary

Total Active ZNS Domains: **15**

* Identity / Auth: `.ghost`, `.sig`, `.gpk`, `.key`, `.pin`, `.sid`
* Infra / Ops: `.gcc`, `.ops`, `.zns`, `.bc`, `.gcp`
* Scaling / Bridges / Analytics: `.warp`, `.arc`
* Experimental / Future: `.dvm`, `.tmp`, `.dbg`, `.lib`, `.txo`

---

> **Note:** These domains are managed by the root ZNS registry contract (`registry.gcp` or `zns.ghost`) and enforced via GhostToken signature validation through `realid` and `zsig`.

---

## 🚀 Examples and Usage Patterns

### GhostPlane L2

* L2 batchers and rollups: `batcher.warp`, `mainnet.warp`, `bridge.warp`

### L1–L2 Bridges / Governance / Analytics

* Canonical bridge: `rollup-bridge.arc`
* Governance root: `governance.arc`
* Protocol analytics: `analytics.arc`

### System Registry / Admin / Utilities

* Registry contract root: `registry.gcp`
* Admin utilities: `admin.gcp`
* Native RPC: `rpc.gcp`

---

### Reserved & Future Domains

* Testnets: `alpha.dbg`, `sandbox.tmp`
* Experimental contracts: `zk.dvm`, `math.lib`
* Transaction-indexed: `tx12345.txo`

---

## 🔒 Registration Policy

* Only the ZNS root contract (`registry.gcp` or `zns.ghost`) may authorize new top-level domains (TLDs).
* All reserved domains (`.warp`, `.arc`, `.gcp`, etc.) require governance or admin approval for subdomain issuance.
* User-level registration is available for select namespaces (`.ghost`, `.gcc`, `.pin`, etc.) after on-chain validation.

---

**This version introduces explicit scaling/bridge/system domains (`.warp`, `.arc`, `.gcp`) for future-proof, modular, and enterprise-grade Ghostchain architecture.**

