# 🌐 CNS (Crypto Name Service)

> **Multi-domain resolution and registration service for the GhostChain ecosystem**

[![Rust](https://img.shields.io/badge/rust-1.75+-blue.svg)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-MIT-green.svg)](../LICENSE)
[![Port](https://img.shields.io/badge/port-8553-orange.svg)](http://localhost:8553)

---

## 🚀 **Overview**

CNS is GhostChain's comprehensive domain name service that bridges Web2, Web3, and Web5 naming systems. It provides unified resolution for multiple TLDs and decentralized identity systems.

### **Supported Domains**
- **Native**: `.ghost`, `.gcc`, `.warp`, `.arc`, `.gcp`
- **Bridge**: `.eth` (ENS), `.crypto`/`.nft` (Unstoppable), `did:*` (Web5)
- **Performance**: 10,000+ RPS with <5ms latency

---

## 🏗️ **Architecture**

```
┌─────────────────┐    ┌──────────────────┐    ┌─────────────────┐
│   CNS Client    │───▶│   CNS Service    │───▶│  Domain Cache   │
│                 │    │   Port: 8553     │    │                 │
└─────────────────┘    └──────────────────┘    └─────────────────┘
                                │
                                ▼
                       ┌──────────────────┐
                       │   Bridge Layer   │
                       │  ENS │ UD │ Web5 │
                       └──────────────────┘
```

### **Core Components**

| Component | Purpose | Features |
|-----------|---------|----------|
| **Native Resolver** | `.ghost` domains | Registration, updates, transfers |
| **Bridge Resolvers** | External domains | ENS, Unstoppable, Web5 DIDs |
| **Cache Manager** | Performance | LRU cache with TTL expiration |
| **Record Manager** | DNS records | A, AAAA, CNAME, TXT, MX |

---

## 🔧 **Usage**

### **Start CNS Service**
```bash
cargo run --bin cns -- \
  --rpc-port 8553 \
  --dns-port 53 \
  --enable-ens \
  --enable-web5
```

### **Configuration**
```toml
# cns.toml
[server]
rpc_port = 8553
dns_port = 53
enable_cache = true

[bridges]
enable_ens = true
enable_unstoppable = true
enable_web5 = true

[cache]
max_entries = 10000
ttl_seconds = 300
```

### **API Examples**

#### **Resolve Domain**
```bash
curl -X POST http://localhost:8553 \
  -H "Content-Type: application/json" \
  -d '{
    "method": "cns_resolve",
    "params": {
      "domain": "alice.ghost"
    },
    "id": 1
  }'
```

#### **Register Domain**
```bash
curl -X POST http://localhost:8553 \
  -H "Content-Type: application/json" \
  -d '{
    "method": "cns_register",
    "params": {
      "domain": "mysite.ghost",
      "owner": "did:ghost:alice",
      "records": [
        {
          "type": "A",
          "name": "mysite.ghost",
          "value": "192.168.1.100",
          "ttl": 300
        }
      ]
    },
    "id": 1
  }'
```

---

## 📊 **Domain Types & Features**

### **🏠 Native Domains (.ghost, .gcc, .warp, .arc, .gcp)**

| Feature | Description | Supported |
|---------|-------------|-----------|
| **Registration** | On-chain domain registration | ✅ |
| **Updates** | Record modification | ✅ |
| **Transfers** | Domain ownership transfer | ✅ |
| **Expiration** | Automatic renewal system | ✅ |
| **Subdomains** | Unlimited subdomain creation | ✅ |

### **🌉 Bridge Domains**

#### **ENS (.eth)**
```rust
// Bridge to Ethereum Name Service
let result = cns.resolve_ens("vitalik.eth").await?;
```

#### **Unstoppable Domains (.crypto, .nft)**
```rust
// Bridge to Unstoppable Domains
let result = cns.resolve_unstoppable("alice.crypto").await?;
```

#### **Web5 DIDs (did:*)**
```rust
// Bridge to Web5 Decentralized Identifiers
let result = cns.resolve_web5("did:web:example.com").await?;
```

---

## 🔐 **Integration with GID**

CNS integrates seamlessly with the Ghost Identity system:

```rust
use cns::CNSService;
use gid::GIDService;

// Link domain to Ghost Identity
let cns = CNSService::new();
let gid = GIDService::new();

// Register domain with GID ownership
cns.register_with_gid(
    "alice.ghost",
    "did:ghost:alice",
    records
).await?;

// Resolve domain through GID
let owner = gid.resolve_domain_owner("alice.ghost").await?;
```

---

## 💰 **Token Integration**

CNS operations integrate with the 4-token economy:

| Operation | Cost | Token |
|-----------|------|-------|
| **Domain Registration** | 100 GHOST | 💻 |
| **Domain Renewal** | 50 GHOST | 💻 |
| **Record Updates** | 10 GCC | ⚡ |
| **Bridge Queries** | 1 GCC | ⚡ |

```rust
// Pay with GHOST tokens for domain registration
cns.register_domain_with_payment(
    "mysite.ghost",
    owner_gid,
    records,
    PaymentToken::GHOST(100)
).await?;
```

---

## 🎛️ **Advanced Features**

### **Wildcard Domains**
```rust
// Register wildcard domain
cns.register_wildcard("*.api.ghost", records).await?;

// Resolves: user1.api.ghost, user2.api.ghost, etc.
```

### **DNSSEC Support**
```rust
// Enable DNSSEC for enhanced security
let config = CNSConfig {
    enable_dnssec: true,
    dnssec_keys: load_dnssec_keys()?,
    ..Default::default()
};
```

### **Content Addressing**
```rust
// IPFS integration for decentralized websites
let ipfs_record = DomainRecord {
    record_type: "TXT".to_string(),
    name: "mysite.ghost".to_string(),
    value: "ipfs=QmXYZ...".to_string(),
    ttl: 300,
    priority: None,
};
```

---

## 📈 **Performance Metrics**

### **Benchmarks**
- **Throughput**: 10,000+ queries/second
- **Latency**: <5ms average response time
- **Cache Hit Rate**: 95%+ for popular domains
- **Memory Usage**: <500MB with 100k cached entries

### **Monitoring**
```bash
# Health check
curl http://localhost:8553/health

# Metrics
curl http://localhost:8553/metrics

# Cache stats
curl http://localhost:8553/cache/stats
```

---

## 🧪 **Testing**

```bash
# Run unit tests
cargo test -p cns

# Run integration tests
cargo test -p cns --test integration

# Load testing
cargo run --bin cns-load-test -- --queries 10000
```

---

## 🔗 **Related Services**

- **[GID](../gid/README.md)** - Ghost Identity integration
- **[GSIG](../gsig/README.md)** - Signature verification
- **[GLEDGER](../gledger/README.md)** - Payment processing

---

## 📚 **Resources**

- **[CNS Protocol Specification](../gcc-docs/cns-protocol.md)**
- **[Domain Registration Guide](../gcc-docs/domain-registration.md)**
- **[API Reference](../gcc-docs/cns-api.md)**
- **[Bridge Integration](../gcc-docs/cns-bridges.md)**

---

*Built with ❤️ for the GhostChain ecosystem*