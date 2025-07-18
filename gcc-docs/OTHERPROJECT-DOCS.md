# 📄 GhostBridge Implementation Guide (DOCS.md)

> This living document provides integration and usage documentation for projects interfacing with **GhostBridge**, including `zwallet`, `zns`, and other components within the **GhostMesh** and **GhostChain** ecosystems.

---

## 🔍 Overview

**GhostBridge** is a high-performance gRPC interoperability layer that links Zig-based DNS and identity services to Rust-based blockchain nodes, enabling seamless Web2 ↔ Web3 communication for decentralized identity, resolution, and ownership.

### Key Features:

* ✅ Zig-native gRPC server for ultra-low latency
* ✅ Rust clients using `tonic` for typed, async access
* ✅ Shared protobufs and consistent schema
* ✅ Supports domain resolution, token queries, and identity fetch

---

## 🛠️ Using GhostBridge in Projects

### 📦 zwallet (Zig Wallet Client)

`zwallet` must link against the protobuf definitions under `ghostbridge/proto` and use the gRPC interface to query:

* Account balances (SPR, MNA, RLUSD, SOUL)
* Transaction status
* Domain ownership verification (linked to address)

**Zig integration tip:** Use the `protobuf.zig` module from `ghostbridge/zig-server/src` to deserialize `AccountResponse` and `BalanceResponse` with zero-copy.

**Example call:**

```zig
const account = try ghostbridge.get_account(allocator, my_address);
std.debug.print("Balance: {} SPR\n", .{account.balance});
```

### 📦 zns (Zig Name System)

`zns` integrates with `GhostBridge` to perform:

* Domain → Identity resolution
* Identity signature validation (Ed25519)
* TTL-aware DNS record caching

It uses:

* `ResolveDomain(DomainQuery)`
* `SubscribeDomainChanges(DomainSubscription)` for real-time updates

**Example use-case:**

```zig
const domain_result = try ghostbridge.resolve_domain("cktechx.io");
assert(domain_result.owner_id != null);
```

---

## 📡 Protocol Schemas (proto/\*.proto)

All projects must consume protobuf files:

* `ghostchain.proto` – Core blockchain + resolution APIs
* `ghostdns.proto` – Cache management, stats, zone update
* `ghostid.proto` – (Coming soon) DID and identity queries
* `common.proto` – Shared types

Keep these in sync across all services. They are the single source of truth for cross-language compatibility.

---

## 🧪 Testing & Debugging

### Development Mode:

```bash
zig build run -- --bind 127.0.0.1:9090  # GhostBridge
cargo run -- node                      # GhostChain
zig build run --bridge-endpoint http://127.0.0.1:9090  # GhostDNS or zns
```

### Example Test Query:

```rust
let res = client.resolve_domain("ghostkellz.sh").await?;
println!("Resolved records: {:?}", res.records);
```

---

## 🤝 FFI Notes

* Zig ↔ Rust integration is done over gRPC, NOT raw FFI
* All gRPC messages must be encoded/decoded using shared `.proto` logic
* Use C headers (under `bindings/c/`) only for lightweight interop; prefer gRPC client/server

---

## 🗺️ Future Integrations

* `ghostctl`: Admin CLI using `ghostbridge-client`
* `phantomlink`: Overlay resolver using `ghostdns` + gRPC
* `ghostvault`: Secure key storage tied to GhostChain accounts
* `jarvis`: AI assistant interfacing with DNS/chain for automated resolution

---

## 🧾 License

MIT License
Copyright (c) 2025 CK Technology LLC

