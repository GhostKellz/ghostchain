# 🌐 Ethereum Integration: GhostChain + GhostPlane + ZNS

This document outlines how to fully integrate **GhostChain (L1)** and **GhostPlane (L2)** into the Ethereum ecosystem, with support for Ethereum RPC, EVM compatibility, and Web3 domain resolution via ZNS.

---

## 🔗 L1: Ethereum-Compatible RPC & Smart Contracts

### ✅ Ethereum JSON-RPC Compatibility

To integrate GhostChain with Ethereum tooling:

* Implement Ethereum JSON-RPC methods:

  * `eth_chainId`
  * `eth_blockNumber`
  * `eth_getTransactionByHash`
  * `eth_call`
  * `eth_sendRawTransaction`
* Serve over QUIC/HTTP3 using `ghostwire`
* Use `ghostbridge` (gRPC or HTTP FFI) to forward requests to Zig modules

### ✅ EVM-Compatible Execution

Use `zvm` (Zig VM / zEVM) as the EVM runtime:

* Support Solidity bytecode execution
* Interpret Wasm modules if using solang backend
* Maintain deterministic gas model and storage model

### ✅ Wallet Compatibility

* Use `ghostcipher` for:

  * `secp256k1` signing
  * `keccak256` hashing
* Generate Ethereum-compatible addresses
* Support `eth_sign`, `eth_accounts`

---

## ☁️ GhostPlane L2 Integration

* Runs parallel to GhostChain as an L2
* Batches transactions, optionally ZK-rollup ready
* Settlement on GhostChain or Ethereum mainnet
* Uses `ghostbridge` for syncing with L1
* Token bridging support for GCC / ZMAN

---

## 🧠 ZNS: Web3 Domain Resolution

### Supported Domains:

| TLD       | Source                | Strategy                         |
| --------- | --------------------- | -------------------------------- |
| `.zns`    | Ghostchain-native ZNS | Local resolver                   |
| `.ghost`  | Ghostchain            | Local resolver                   |
| `.gsig`   | Sigil-based identity  | Local resolver                   |
| `.eth`    | ENS on Ethereum       | `eth_call` to public RPC         |
| `.crypto` | Unstoppable Domains   | HTTP API or EVM contract via RPC |
| `.x`      | Unstoppable Domains   | HTTP API or EVM contract via RPC |

### ZNS Resolver Logic (in `zns`):

```zig
if (domain.endsWith(".zns") or ".ghost") resolveZNS();
else if (domain.endsWith(".eth")) resolveENS();
else if (domain.endsWith(".crypto")) resolveUD();
```

---

## ⚙️ Smart Contract Support

* Compile Solidity contracts with `solc`
* Deploy to GhostChain (via RPC)
* Optional: Build Zig-native DSL ("zillidum") or use `solang` to emit Wasm
* Execute in `zvm`

---

## 🧩 GhostBridge Roles

| Layer      | Tool        | Role                                     |
| ---------- | ----------- | ---------------------------------------- |
| RPC Bridge | ghostbridge | Bridge JSON-RPC to Zig via gRPC/FFI      |
| L1/L2 Sync | ghostplane  | Batch relay, rollup coordination         |
| Wallet API | ghostwallet | Address/key management, MetaMask support |

---

## ✅ Implementation Checklist

### GhostChain Core:

* [ ] Add `eth_*` RPC handlers
* [ ] Connect `zvm` to JSON-RPC
* [ ] Expose over QUIC/HTTP3

### ZNS Resolver:

* [ ] Add dispatch logic for `.eth`, `.crypto`, etc.
* [ ] Use namehash + `eth_call` for ENS
* [ ] Use UD resolution API (HTTP or RPC)

### GhostPlane:

* [ ] Implement batching / settlement hooks
* [ ] Sync rollups back to GhostChain
* [ ] Support bridging ZMAN / GCC

---

> This integration brings GhostChain to parity with Ethereum infrastructure — bridging Web2, Web3, and Web5 with zero-trust identity and native Zig execution.

