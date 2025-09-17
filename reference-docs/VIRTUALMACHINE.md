# ⚙️ VIRTUALMACHINE.md – GhostChain VM Strategy

This document outlines the **GhostChain virtual machine architecture**, covering both **Zig-native VMs** and **Rust-native VMs** used across `ghostd`, `walletd`, and smart contract environments.

It serves as a reference to identify which VM is applicable for a given module or feature, and how each integrates with the GhostNet ecosystem.

---

## 🧠 Supported Virtual Machines

| VM     | Language | Type              | Purpose                                          | Target Usage                           |
| ------ | -------- | ----------------- | ------------------------------------------------ | -------------------------------------- |
| `zvm`  | Zig      | WASM-based        | Ghost-native contract engine                     | GhostChain native smart contracts      |
| `zEVM` | Zig      | EVM-like (future) | Lightweight embedded Solidity-compatible runtime | Experimental / lightweight deployments |
| `rvm`  | Rust     | EVM-compatible    | REVM-based Ethereum VM runtime                   | Solidity, ERC20/721 dApp compatibility |
| `rEVM` | Rust     | Ethereum module   | EVM execution layer inside `ghostd`              | ENS, Unstoppable, dApps, bridging      |

---

## 🔧 Runtime Dispatch (ghostd)

```rust
enum VmType {
  Zvm,
  Evm,
}

match tx.vm_type {
  VmType::Zvm => zvm.execute(tx),
  VmType::Evm => revm.execute(tx),
}
```

> `ghostd` dynamically dispatches to the correct VM based on tx metadata.

---

## 🔗 Integration Reference

| Project     | Uses          | Notes                                         |
| ----------- | ------------- | --------------------------------------------- |
| `ghostd`    | `zvm`, `rvm`  | Core runtime for contract execution           |
| `walletd`   | none directly | Signs txs for both VM types                   |
| `ghostsite` | `rEVM`, `zvm` | Web5 dApp loader (EVM + native Zig contracts) |
| `znsd`      | `rEVM`, `zvm` | Domain-based resolution and contract metadata |

---

## 🔁 Cross-Chain Compatibility

| Chain      | Supported Via  | Method                                           |
| ---------- | -------------- | ------------------------------------------------ |
| Ethereum   | `rvm`, `rEVM`  | Full EVM runtime, Solidity contracts             |
| Hedera     | `rvm` (future) | Mirror node or sidecar adapter                   |
| Stellar    | External       | Horizon API + contract bridge                    |
| GhostChain | `zvm`          | Native contracts written in Zig or compiled WASM |

---

## 🚀 VM Development Roadmap

* [x] `zvm` contract runtime integration
* [x] `rvm` + REVM core logic in Rust
* [ ] Unified `VmRuntime` trait for dispatch
* [ ] Gas metering for both engines
* [ ] zkVM plugin support (experimental)

---

## 📂 Include This File In:

* `ghostd/`
* `zvm/`, `zEVM/`
* `rvm/`, `rEVM/`
* `walletd/`

> Helps contributors and tooling identify correct VM context.

---

## 📜 License

MIT © GhostKellz

