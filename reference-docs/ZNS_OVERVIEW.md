# 📚 GHOSTCHAIN: CRYPTO OVERVIEW (CRYPTO.md)

> A living knowledgebase of the cryptographic + wallet + resolution architecture inside **GhostChain** and its Zig/Rust dual implementation. This includes `zwallet`, `zsig`, `zns`, and the `GhostBridge` gRPC interoperability layer.

---

## 🌐 GhostChain Architecture

GhostChain is a hybrid-layer blockchain system built in **Rust**, with a prototype variant in **Zig** for edge experimentation. Both chains expose a common gRPC interface via **GhostBridge**, allowing resolution, token operations, and smart contract interactions across Zig/Rust boundaries.

### 🧱 Core Components

* 🔗 **GhostChain (Rust)** – Main L1 blockchain runtime with smart contract VM, ledger, and RPC
* 🌀 **GhostChain-Z (Zig)** – Lightweight experimental VM with direct `zvm` support
* 🌉 **GhostBridge** – gRPC bridge to expose contract/DNS/token operations across Zig ↔ Rust

## 🔐 Cryptographic Primitives

* **zsig** → Zig-native cryptographic signing module

  * Supports Ed25519, Schnorr, and keccak256 hashing
  * Used by `zwallet`, `zvm`, and `zns`

* **zvm** → Zig Virtual Machine

  * Smart contract layer to process on-chain logic for:

    * Tokens (MANA, SPIRIT, SOUL, RLUSD)
    * DNS identity
    * Domain signature verification (via `zsig`)

## 💸 Wallets and Identity

### 🔑 `zwallet`

* Premier wallet CLI + lib written in Zig
* Supports:

  * SPR / MNA balance checks
  * Domain ownership
  * `zsig` key generation and signing
  * Recovery phrase derivation

```sh
zwallet balance ck.kz
zwallet sign --msg "verify" --key ~/.ghost/keys/primary.zk
```

### 👤 GhostID

* Identity system (linked to `zns`) for user-based name resolution and DID-style verification
* Exposed via `ghostid.proto`

## 🌐 zns (Zig Name Service)

On-chain naming protocol for `.ghost`, `.zkellz`, `.kz`, etc., domains.

* Replaces ENS/Unstoppable Domains
* Fully backed by GhostChain smart contracts
* Supports:

  * `ResolveDomain()` via GhostBridge
  * `RegisterDomain()` (Zig CLI coming)
  * `SubscribeDomainChanges()`

#### zns Record Example

```json
{
  "domain": "ghostkellz.zkellz",
  "records": [
    {"type": "A", "value": "10.6.0.3", "ttl": 600},
    {"type": "TXT", "value": "v=ghost1", "ttl": 600}
  ],
  "owner_id": "ghostid:ckel@zns",
  "signature": "ed25519:..."
}
```

## 🌉 GhostBridge in Crypto Context

* Facilitates:

  * Wallet <-> Blockchain balance/tokens
  * DNS ↔ Chain domain resolution
  * gRPC signature validation
* All crypto is verified at the chain level, exposed through bridge APIs

## 🔮 Development Roadmap

| Phase | Milestone                                                      |
| ----- | -------------------------------------------------------------- |
| ✅     | `zsig`, `zledger` complete                                     |
| 🔧    | `zwallet` gRPC + offline support                               |
| 🔧    | `zns` registration logic                                       |
| 🔜    | `zvm` contract runtime finalize                                |
| 🔜    | GhostBridge schema hardening (domain, identity, token modules) |

## 🧪 Testing Workflow

```sh
# Run full stack locally
zig build run  # zwallet/zns test
cargo run --bin ghostchain
zig build run --bridge-endpoint http://localhost:9090
```

```zig
const domain = try ghostbridge.resolve_domain("ghostkellz.zkellz");
std.debug.print("Resolved A: {}\n", .{domain.records[0].value});
```

## 📦 Integration Summary

| Module      | Language | Purpose                    |
| ----------- | -------- | -------------------------- |
| zwallet     | Zig      | Wallet / Identity Ops      |
| zsig        | Zig      | Keygen / Signing / Hashing |
| zns         | Zig      | Name Service / DNS         |
| zvm         | Zig      | Contract Runtime           |
| GhostChain  | Rust     | Mainnet Chain + RPC        |
| GhostBridge | Zig+Rust | gRPC Bridge                |

## 📝 License

MIT License — 2025 CK Technology LLC

