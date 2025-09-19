# WALLET-NOTES.md

## 🔐 Core Principles for a Better Wallet

* Use strong, deterministic cryptography

  * Ed25519 / secp256k1 support (default)
  * HKDF, Argon2id, HMAC-based authentication
  * AES-GCM and ChaCha20-Poly1305 for symmetric encryption
  * Built on `zcrypto` (✅ implemented)

---

## 🌐 Real Internet Infrastructure Integration

### IPv6 + DNS Integration

* Each wallet has a unique, globally addressable IPv6 (prefix derived from key hash)
* Use DNSSEC + DANE / OPENPGPKEY records to anchor identity
* Example DNS Records:

  ```
  alice.ckelley.dev. IN OPENPGPKEY ...
  wallet.alice.id. IN AAAA 2001:db8::cafe:ed25
  ```

---

## 🧬 Identity Binding & Recovery

### Optional Real-World ID

* OIDC support via Google / Entra ID / Apple ID
* Derive keys using user-known secrets:

  ```
  Argon2id(name + birthdate + email) + HMAC(seed)
  ```

---

## 📦 Backup & Recovery Strategy

### Redundant, Encrypted Splits

* Use Shamir's Secret Sharing or threshold encryption
* Store encrypted shares:

  * Locally (disk)
  * DNS TXT record
  * IPFS blob
  * Encrypted cloud (Google Drive / OneDrive)
* CLI:

  ```sh
  zwallet split-backup --to ~/.local, dns, drive
  zwallet recover --email alice@gmail.com --device myphone
  ```

---

## 🔐 Self-Custody with Recovery Support

* Social recovery (2-of-3 trust scheme)
* Combine with:

  * Time-locked seed
  * OTP TOTP (Google Authenticator)
  * OIDC/email challenge

---

## 💎 Human-Centric UX

### CLI Example:

```sh
zwallet new --easy --name "Alice"
zwallet backup --cloud --dns --qr
zwallet link-id --google alice@gmail.com
zwallet send --to ghostkellz.sh --amount 0.05 BTC
```

### Mobile App:

* Secure Enclave-based local key protection
* QR/NFC support for cold backups or transactions

---

## 🌐 Global Addressing Layer (Wallet URI)

* Examples:

  * `alice.ckelley.dev`
  * `wallet://ghostkellz.sh/alice`
* Resolves to:

  * Current pubkey/address
  * Signed ownership proof
  * Transaction push endpoint (Nostr / QUIC)

---

## 🚀 Project Idea: **PhantomID**

> Human-friendly wallet ID and recovery layer built on:
>
> * IPv6 / DNS / QUIC
> * zcrypto security
> * OIDC Identity binding
> * CLI + mobile parity

---

## 🔭 Complementary Spec: GhostWallet

GhostWallet is the official digital identity and asset manager of GhostChain. More than a wallet, it’s your secure, recoverable, cross-device vault for interacting with Web5, GhostNet, and the decentralized web.

### 🎯 Core Goals

* Zero-friction onboarding (passkey-style)
* Secure, recoverable, and decentralized key management
* Support for fungible, non-fungible, and soulbound tokens
* Programmable identity and wallet policies
* Interoperability with Web2, Web3, and cDNS domains

### 🔐 Key Features

* DID-based identity and GhostID integration
* Smart key management (MPC, SSS, Secure Enclave)
* zkLogin, biometric support
* GhostMesh agents and peer-based key recovery
* Soulbound assets, programmable badges
* WebAuthn, WalletConnect, OIDC bridges
* zkProof-based recovery metadata

### Transaction Layer

* QUIC + gRPC + WebSockets
* zkValidated txs
* Multi-party approval for enterprise/team wallets

### Developer Tools

* Rust/Zig/TS SDKs
* ghostctl CLI wallet module
* Built-in signer, gRPC/GraphQL APIs

### Next Steps

* Finalize GhostVault
* Build recovery agent prototype
* UI/UX for mobile + terminal
* PhantomID + GhostMesh encrypted tx support

GhostWallet is your Web5 passport: identity, payment, voting, access — all self-sovereign.

---

Ready for Ghostchain, `zwallet`, `zsig`, and beyond.

