# 🏛️ Keystone

> Core ledger and transaction engine powering the Ghost ecosystem.

**Keystone** is a modular, identity-aware, and audit-friendly execution layer designed for use in distributed systems, Web3 runtimes, and secure transactional infrastructure. It integrates tightly with `zledger`, and optionally supports `shroud`, `zsig`, and `zwallet`.
## Rust Based 
- Now rust-based in ghostchain repo 
- Archive-zig has a zig variation from past prototpyes.
---

## ✨ Features

* 📒 Account abstraction via `account.rs`
* 💰 Double-entry transactions and balances (`tx.rs`)
* 📜 Journaled state changes with audit trail (`journal.rs`, `audit.rs`)
* 🔍 Signature and identity-aware validation (optional)
* 🧱 Designed for ZVM and Ghostchain compatibility
* ⚖️ Zero external dependencies by default


## 🧪 Example Usage



---

## 🚧 Roadmap

* [x] Keystone v0.1.0 — Core ledger API
* [ ] Keystone v0.2.0 — Journal replay + audit layer
* [ ] Keystone v0.3.0 — identity hooks (pluggable)
* [ ] Keystone v0.4.0 — CLI + RVM execution gateway

---

## 📜 License

MIT

---

