# API.md — GhostAPI Specification (gRPC over QUIC)

## 🔧 Purpose

GhostAPI is a high-performance, type-safe, multiplexed API layer that replaces legacy REST APIs. Built on gRPC over QUIC, it powers every system in GhostChain including smart contracts, wallets, dApps, identity resolution, and node coordination.

---

## 🧱 Architecture Stack

| Layer     | Role                                       |
| --------- | ------------------------------------------ |
| gRPC      | Typed service definitions (protobuf)       |
| QUIC      | Encrypted, low-latency transport layer     |
| HTTP/3    | Public gateway for web-compatible clients  |
| WireGuard | Secure fallback tunnel for mesh/core nodes |

---

## 🧩 Key Features

* **Strong Typing**: Enforced via `.proto` contracts, no loose JSON
* **Streaming Support**: Real-time push/pull capabilities
* **Multiplexed Calls**: Simultaneous methods per connection
* **Bi-Directional**: Clients and servers can both initiate streams
* **Efficient Compression**: Small payloads, low overhead

---

## 🚀 Replacing REST

| REST Equivalent     | GhostAPI Replacement          |
| ------------------- | ----------------------------- |
| `GET /wallets/:id`  | `WalletService.GetWallet()`   |
| `POST /tx`          | `TransactionService.Submit()` |
| `GET /chain/status` | `ChainService.GetStatus()`    |
| `PUT /identity`     | `IdentityService.Update()`    |

GhostAPI removes boilerplate and standardizes all interfaces through proto schemas.

---

## 📚 Service Domains

* `AuthService` — Token issuance, login, OIDC + GhostID
* `WalletService` — Balances, transfers, stake management
* `NodeService` — Peer sync, block propagation, relay
* `ContractService` — Deploy, call, audit contracts
* `DNSService` — GhostDNS & cDNS records
* `VaultService` — Key storage, backup, sealed secrets

---

## 🔐 Security Layer

* **TLS 1.3 + QUIC**: Encrypted by default
* **Identity Verification**: Bound to GhostID + signed auth headers
* **Tokenized Access**: Using scoped OIDC or JWT
* **Rate Limiting & Isolation**: Smart contract-level access control

---

## 🌐 HTTP/3 Gateway

GhostAPI exposes HTTP/3 endpoints for:

* Web dashboards
* Browser-based wallets
* Public smart contract viewing

Non-gRPC clients are auto-translated via API gateway.

---

## 🛠 Developer Tooling

* `ghost-api-cli` for testing and introspection
* Code generation for Rust, Zig, Go, Python, JS
* Built-in versioning + contract linting

---

## 🌍 Interoperability

* Compatible with REST via translation proxy
* Works with existing OAuth/OIDC flows
* Integrated with IPv6, DNS-over-QUIC

---

## 🧪 Future Extensions

* GraphQL-to-gRPC schema converter
* AI-assisted contract gateway (Jarvis-agent linked)
* Edge-to-core QUIC relays for mesh dApp hosting

---

## Summary

GhostAPI is the universal interface for the GhostChain ecosystem, merging the best of modern APIs (gRPC, QUIC, HTTP/3) with decentralized, identity-driven architecture. Fast, extensible, and secure by default.
