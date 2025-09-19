# 🔐 GSIG (Ghost Signature Service)

> **Multi-algorithm signature verification and validation service for GhostChain**

[![Rust](https://img.shields.io/badge/rust-1.75+-blue.svg)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-MIT-green.svg)](../LICENSE)
[![Port](https://img.shields.io/badge/port-8554-green.svg)](http://localhost:8554)

---

## 🚀 **Overview**

GSIG is GhostChain's comprehensive signature service that provides cryptographic signature operations across multiple algorithms. It supports both traditional and post-quantum cryptography with Guardian framework integration for zero-trust verification.

### **Supported Algorithms**
- **Ed25519** - Fast elliptic curve signatures
- **Secp256k1** - Bitcoin/Ethereum compatibility
- **BLS** - Aggregate signatures for efficiency
- **ML-DSA** - Post-quantum signatures (via gcrypt)
- **Dilithium** - Post-quantum alternative

### **Key Features**
- **🔒 Multi-Algorithm Support** - Ed25519, Secp256k1, BLS, Post-Quantum
- **⚡ High Performance** - 15,000+ verifications/second
- **🕶️ Guardian Integration** - Zero-trust policy enforcement
- **🌐 Cross-Chain** - Ethereum, Bitcoin signature compatibility
- **🛡️ Post-Quantum Ready** - Future-proof cryptography

---

## 🏗️ **Architecture**

```
┌─────────────────┐    ┌──────────────────┐    ┌─────────────────┐
│  GSIG Client    │───▶│  GSIG Service    │───▶│ Guardian Policy │
│                 │    │   Port: 8554     │    │                 │
└─────────────────┘    └──────────────────┘    └─────────────────┘
                                │                        │
                                ▼                        ▼
                       ┌──────────────────┐    ┌─────────────────┐
                       │ Algorithm Engine │    │ Signature Cache │
                       │ Ed25519│BLS│PQ   │    │                 │
                       └──────────────────┘    └─────────────────┘
                                │                        │
                                ▼                        ▼
                       ┌──────────────────┐    ┌─────────────────┐
                       │   GCrypt FFI     │    │  Verification   │
                       │                  │    │     Store       │
                       └──────────────────┘    └─────────────────┘
```

### **Core Components**

| Component | Purpose | Guardian Features |
|-----------|---------|-------------------|
| **Algorithm Engine** | Multi-signature support | Policy-based algorithm selection |
| **Verification Store** | Signature caching | Zero-trust verification logs |
| **GCrypt Integration** | Post-quantum crypto | Quantum-resistant signatures |
| **Cross-Chain Bridge** | External blockchain support | Multi-chain signature validation |

---

## 🔧 **Usage**

### **Start GSIG Service**
```bash
cargo run --bin gsig -- \
  --rpc-port 8554 \
  --grpc-port 9554 \
  --enable-guardian \
  --enable-post-quantum
```

### **Configuration**
```toml
# gsig.toml
[server]
rpc_port = 8554
grpc_port = 9554
enable_guardian = true

[algorithms]
enable_ed25519 = true
enable_secp256k1 = true
enable_bls = true
enable_post_quantum = true

[cache]
max_signatures = 100000
verification_ttl = 3600  # 1 hour

[guardian]
require_policy_check = true
signature_audit_log = true
```

---

## 🔐 **Signature Algorithms**

### **Ed25519 - Default Algorithm**
```rust
// Fast, secure elliptic curve signatures
let signature = gsig.ed25519_sign(&private_key, message).await?;
let is_valid = gsig.ed25519_verify(&public_key, message, &signature).await?;
```

### **Secp256k1 - Blockchain Compatibility**
```rust
// Bitcoin/Ethereum compatible signatures
let signature = gsig.secp256k1_sign(&private_key, message).await?;
let is_valid = gsig.secp256k1_verify(&public_key, message, &signature).await?;
```

### **BLS - Aggregate Signatures**
```rust
// Efficient aggregate signatures for multiple signers
let signatures = vec![sig1, sig2, sig3];
let public_keys = vec![pk1, pk2, pk3];
let aggregate_sig = gsig.bls_aggregate(&signatures).await?;
let is_valid = gsig.bls_verify_aggregate(&public_keys, message, &aggregate_sig).await?;
```

### **Post-Quantum Signatures**
```rust
// Future-proof quantum-resistant signatures via gcrypt
let signature = gsig.ml_dsa_sign(&private_key, message).await?;
let is_valid = gsig.ml_dsa_verify(&public_key, message, &signature).await?;
```

---

## 🕶️ **Guardian Integration**

### **Policy-Based Signature Verification**

GSIG integrates with Guardian for zero-trust signature operations:

```rust
// Guardian-enforced signature verification
let verification_context = GuardianContext::new()
    .with_identity("did:ghost:alice")
    .with_algorithm_preference(SignatureAlgorithm::Ed25519)
    .with_required_permissions(vec![Permission::SignTransaction]);

let result = gsig.guardian_verify_signature(
    &signature,
    &public_key,
    message,
    verification_context
).await?;

match result {
    GuardianVerification::Approved => {
        // Signature valid and policy compliant
    }
    GuardianVerification::Denied(reason) => {
        // Policy violation or invalid signature
    }
    GuardianVerification::RequireMultiSig => {
        // Additional signatures required
    }
}
```

### **Signature Audit Trail**

All signature operations are logged for security auditing:

```rust
// Comprehensive audit logging
#[derive(Debug)]
pub struct SignatureAuditLog {
    pub signature_id: String,
    pub signer_identity: String,
    pub algorithm: SignatureAlgorithm,
    pub message_hash: Vec<u8>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub verification_result: bool,
    pub guardian_policy: String,
    pub ip_address: Option<String>,
}
```

---

## 📊 **API Reference**

### **Signature Operations**

#### **Sign Message**
```bash
curl -X POST http://localhost:8554 \
  -H "Content-Type: application/json" \
  -d '{
    "method": "gsig_sign",
    "params": {
      "algorithm": "ed25519",
      "private_key": "5dab087e624a8a4b79e17f8b83800ee66f3bb1292618b6fd1c2f8b27ff88e0eb",
      "message": "Hello, GhostChain!",
      "identity": "did:ghost:alice"
    },
    "id": 1
  }'
```

#### **Verify Signature**
```bash
curl -X POST http://localhost:8554 \
  -H "Content-Type: application/json" \
  -d '{
    "method": "gsig_verify",
    "params": {
      "algorithm": "ed25519",
      "public_key": "d75a980182b10ab7d54bfed3c964073a0ee172f3daa62325af021a68f707511a",
      "message": "Hello, GhostChain!",
      "signature": "92a009a9f0d4cab8720e820b5f642540a2b27b5416503f8fb3762223ebdb69da085ac1e43e15996e458f3613d0f11d8c387b2eaeb4302aeeb00d291612bb0c00",
      "identity": "did:ghost:alice"
    },
    "id": 1
  }'
```

### **Batch Operations**

#### **Batch Verification**
```bash
curl -X POST http://localhost:8554 \
  -H "Content-Type: application/json" \
  -d '{
    "method": "gsig_batch_verify",
    "params": {
      "verifications": [
        {
          "algorithm": "ed25519",
          "public_key": "d75a980182b10ab7d54bfed3c964073a0ee172f3daa62325af021a68f707511a",
          "message": "Message 1",
          "signature": "signature1..."
        },
        {
          "algorithm": "secp256k1",
          "public_key": "03d75a980182b10ab7d54bfed3c964073a0ee172f3daa62325af021a68f707511a",
          "message": "Message 2",
          "signature": "signature2..."
        }
      ]
    },
    "id": 1
  }'
```

---

## 💰 **Token Integration**

GSIG operations integrate with the 4-token economy:

| Operation | Cost | Token |
|-----------|------|-------|
| **Standard Verification** | 1 GCC | ⚡ |
| **Post-Quantum Signature** | 5 GCC | ⚡ |
| **BLS Aggregate Operation** | 3 GCC | ⚡ |
| **Audit Log Query** | 2 GCC | ⚡ |

```rust
// Token-gated signature operations
let signature_result = gsig.sign_with_payment(
    SignatureRequest {
        algorithm: SignatureAlgorithm::Ed25519,
        private_key: private_key_bytes,
        message: message_bytes,
        identity: "did:ghost:alice".to_string(),
    },
    PaymentToken::GCC(1)
).await?;
```

---

## 🌐 **Cross-Chain Integration**

### **Ethereum Compatibility**
```rust
// Ethereum-compatible signature generation
let eth_signature = gsig.ethereum_sign(
    &private_key,
    &transaction_hash,
    Some(chain_id)
).await?;

// Verify Ethereum signatures
let is_valid = gsig.ethereum_verify(
    &public_key,
    &transaction_hash,
    &eth_signature,
    Some(chain_id)
).await?;
```

### **Bitcoin Compatibility**
```rust
// Bitcoin-compatible signature operations
let btc_signature = gsig.bitcoin_sign(
    &private_key,
    &transaction_hash,
    SignatureHashType::All
).await?;

let is_valid = gsig.bitcoin_verify(
    &public_key,
    &transaction_hash,
    &btc_signature
).await?;
```

---

## 🎛️ **Advanced Features**

### **Threshold Signatures**
```rust
// Multi-party threshold signature scheme
let threshold_config = ThresholdConfig {
    threshold: 3,
    total_parties: 5,
    algorithm: SignatureAlgorithm::BLS,
};

let partial_signature = gsig.threshold_sign_partial(
    &private_share,
    message,
    &threshold_config
).await?;

let complete_signature = gsig.threshold_combine_signatures(
    partial_signatures,
    &threshold_config
).await?;
```

### **Hardware Security Module (HSM) Integration**
```rust
// HSM-backed signature operations
let hsm_config = HSMConfig {
    provider: "aws-cloudhsm",
    key_label: "ghost-signing-key",
    pin: secure_pin,
};

let hsm_signature = gsig.hsm_sign(
    &hsm_config,
    message,
    SignatureAlgorithm::Ed25519
).await?;
```

### **Signature Delegation**
```rust
// Delegate signing authority with Guardian policies
let delegation = gsig.create_signing_delegation(
    &delegator_identity,
    &delegate_identity,
    vec![Permission::SignTransaction],
    chrono::Duration::hours(24)
).await?;

// Delegate can sign on behalf of delegator
let delegated_signature = gsig.delegated_sign(
    &delegation_token,
    message,
    SignatureAlgorithm::Ed25519
).await?;
```

---

## 📈 **Performance Metrics**

### **Benchmarks**
- **Ed25519 Signing**: 25,000+ ops/second
- **Ed25519 Verification**: 15,000+ ops/second
- **Secp256k1 Signing**: 8,000+ ops/second
- **BLS Aggregate**: 1,000+ aggregations/second
- **Post-Quantum**: 500+ ops/second

### **Monitoring**
```bash
# Service health
curl http://localhost:8554/health

# Performance metrics
curl http://localhost:8554/metrics

# Signature statistics
curl http://localhost:8554/stats
```

---

## 🔒 **Security Features**

### **Zero-Trust Verification**
- **Policy Enforcement** - Every signature checked against Guardian policies
- **Identity Verification** - Signatures linked to verified GID identities
- **Audit Logging** - Complete signature operation trail
- **Rate Limiting** - DDoS protection and abuse prevention

### **Post-Quantum Readiness**
- **ML-DSA Support** - NIST-standardized post-quantum signatures
- **Dilithium Integration** - Alternative post-quantum algorithm
- **Hybrid Mode** - Traditional + post-quantum signature combinations

---

## 🧪 **Testing**

```bash
# Run unit tests
cargo test -p gsig

# Run signature algorithm tests
cargo test -p gsig --test signature_algorithms

# Performance benchmarking
cargo run --bin gsig-benchmark -- --algorithm ed25519 --operations 10000

# Security audit
cargo audit && cargo test security_tests
```

---

## 🔗 **Integration Examples**

### **With GID Service**
```rust
use gsig::GSIGService;
use gid::GIDService;

// Identity-verified signature operations
let gid = GIDService::new();
let gsig = GSIGService::new();

let identity = gid.resolve("did:ghost:alice").await?;
let signature = gsig.sign_with_identity_verification(
    message,
    &identity,
    SignatureAlgorithm::Ed25519
).await?;
```

### **With CNS Service**
```rust
// Domain-based signature verification
let domain_owner = cns.resolve_owner("alice.ghost").await?;
let signature_valid = gsig.verify_domain_signature(
    "alice.ghost",
    message,
    &signature,
    &domain_owner
).await?;
```

---

## 🔗 **Related Services**

- **[GID](../gid/README.md)** - Identity verification integration
- **[CNS](../cns/README.md)** - Domain-based signatures
- **[GLEDGER](../gledger/README.md)** - Payment processing for operations

---

## 📚 **Resources**

- **[Signature Algorithm Guide](../gcc-docs/signature-algorithms.md)**
- **[Guardian Signature Policies](../gcc-docs/guardian-signatures.md)**
- **[Cross-Chain Integration](../gcc-docs/cross-chain-signatures.md)**
- **[Post-Quantum Cryptography](../gcc-docs/post-quantum.md)**
- **[API Reference](../gcc-docs/gsig-api.md)**

---

*🔐 Secure signatures for the multi-chain future*