# 💰 GLEDGER (Ghost Ledger Service)

> **Multi-token accounting and transaction ledger for the GhostChain 4-token economy**

[![Rust](https://img.shields.io/badge/rust-1.75+-blue.svg)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-MIT-green.svg)](../LICENSE)
[![Port](https://img.shields.io/badge/port-8555-blue.svg)](http://localhost:8555)

---

## 🚀 **Overview**

GLEDGER is GhostChain's comprehensive accounting and transaction ledger service that manages the 4-token economy with double-entry accounting, Guardian policy enforcement, and real-time balance tracking.

### **4-Token Economy**
- **GCC** ⚡ - Gas & Computation
- **SPIRIT** 🗳️ - Governance & Staking
- **MANA** ✨ - AI & Smart Contracts
- **GHOST** 👻 - Identity & Domains

### **Key Features**
- **📚 Double-Entry Accounting** - Traditional accounting principles
- **⚡ Real-Time Balances** - Instant transaction processing
- **🕶️ Guardian Integration** - Policy-based transaction approval
- **🔗 Multi-Chain Support** - Bridge to external blockchains
- **📊 Advanced Analytics** - Transaction patterns and insights

---

## 🏗️ **Architecture**

```
┌─────────────────┐    ┌──────────────────┐    ┌─────────────────┐
│ GLEDGER Client  │───▶│ GLEDGER Service  │───▶│ Guardian Policy │
│                 │    │   Port: 8555     │    │                 │
└─────────────────┘    └──────────────────┘    └─────────────────┘
                                │                        │
                                ▼                        ▼
                       ┌──────────────────┐    ┌─────────────────┐
                       │ Double-Entry     │    │ Balance Cache   │
                       │ Ledger Engine    │    │                 │
                       └──────────────────┘    └─────────────────┘
                                │                        │
                                ▼                        ▼
                       ┌──────────────────┐    ┌─────────────────┐
                       │   ZQLITE Store   │    │ Transaction     │
                       │                  │    │   Analytics     │
                       └──────────────────┘    └─────────────────┘
```

### **Core Components**

| Component | Purpose | Features |
|-----------|---------|----------|
| **Ledger Engine** | Double-entry accounting | Debits, credits, balance validation |
| **Guardian Integration** | Transaction policies | Approval workflows, spending limits |
| **Balance Cache** | Real-time tracking | Instant balance queries, updates |
| **Analytics Engine** | Transaction insights | Patterns, trends, reporting |

---

## 🔧 **Usage**

### **Start GLEDGER Service**
```bash
cargo run --bin gledger -- \
  --rpc-port 8555 \
  --grpc-port 9555 \
  --enable-guardian \
  --enable-analytics
```

### **Configuration**
```toml
# gledger.toml
[server]
rpc_port = 8555
grpc_port = 9555
enable_guardian = true

[accounting]
enforce_double_entry = true
precision_decimals = 18
enable_negative_balances = false

[tokens]
gcc_decimals = 18
spirit_decimals = 18
mana_decimals = 18
ghost_decimals = 18

[guardian]
require_approval_threshold = 10000  # Large transactions
spending_velocity_limit = 100       # Tx per hour
multi_sig_threshold = 50000         # Requires multiple signatures

[cache]
balance_cache_ttl = 60      # 1 minute
transaction_batch_size = 1000
```

---

## 💰 **4-Token System**

### **Token Specifications**

| Token | Symbol | Decimals | Primary Use | Supply Model |
|-------|--------|----------|-------------|--------------|
| **GCC** | ⚡ | 18 | Gas & computation fees | Deflationary |
| **SPIRIT** | 🗳️ | 18 | Governance & staking rewards | Fixed supply |
| **MANA** | ✨ | 18 | AI operations & smart contracts | Inflationary |
| **GHOST** | 👻 | 18 | Identity & domain registration | Burn-to-mint |

### **Token Operations**

#### **Transfer Tokens**
```rust
// Transfer tokens between accounts
let transfer = TokenTransfer {
    from: "did:ghost:alice".to_string(),
    to: "did:ghost:bob".to_string(),
    token_type: TokenType::GCC,
    amount: parse_token_amount("100.5")?,
    memo: Some("Payment for services".to_string()),
};

let result = gledger.transfer_tokens(transfer).await?;
```

#### **Mint/Burn Operations**
```rust
// Mint new tokens (authorized operations only)
let mint_result = gledger.mint_tokens(
    TokenType::MANA,
    "did:ghost:alice",
    parse_token_amount("1000")?,
    "AI operation reward"
).await?;

// Burn tokens for GHOST mint mechanism
let burn_result = gledger.burn_for_ghost_mint(
    "did:ghost:alice",
    TokenType::GCC,
    parse_token_amount("500")?,
    "Domain registration payment"
).await?;
```

---

## 📚 **Double-Entry Accounting**

### **Account Structure**

GLEDGER implements traditional double-entry bookkeeping:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LedgerAccount {
    pub account_id: String,
    pub account_type: AccountType,
    pub token_type: TokenType,
    pub balance: TokenAmount,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AccountType {
    Asset,          // User token balances
    Liability,      // Outstanding obligations
    Revenue,        // Protocol income
    Expense,        // Protocol costs
    Equity,         // Protocol reserves
}
```

### **Transaction Entries**

Every transaction creates balanced double-entry records:

```rust
// Transfer: Alice sends 100 GCC to Bob
// Entry 1: Debit Bob's GCC account (+100)
// Entry 2: Credit Alice's GCC account (-100)

let entries = vec![
    LedgerEntry {
        account_id: "did:ghost:bob:gcc".to_string(),
        entry_type: EntryType::Debit,
        amount: parse_token_amount("100")?,
        transaction_id: tx_id.clone(),
    },
    LedgerEntry {
        account_id: "did:ghost:alice:gcc".to_string(),
        entry_type: EntryType::Credit,
        amount: parse_token_amount("100")?,
        transaction_id: tx_id.clone(),
    },
];
```

---

## 🕶️ **Guardian Integration**

### **Transaction Policies**

GLEDGER integrates with Guardian for policy-based transaction control:

```rust
// Guardian policy enforcement for large transfers
let transfer_context = GuardianContext::new()
    .with_identity("did:ghost:alice")
    .with_token_balances(current_balances)
    .with_transaction_amount(transfer_amount)
    .with_recipient("did:ghost:bob");

let approval = gledger.guardian_approve_transfer(
    &transfer_request,
    transfer_context
).await?;

match approval {
    GuardianApproval::Approved => {
        // Execute transfer immediately
    }
    GuardianApproval::RequireMultiSig => {
        // Initiate multi-signature workflow
    }
    GuardianApproval::Denied(reason) => {
        // Transaction blocked by policy
    }
    GuardianApproval::Delayed(until) => {
        // Transfer scheduled for later execution
    }
}
```

### **Spending Limits & Velocity Controls**

```rust
// Configure per-identity spending limits
let spending_policy = SpendingPolicy {
    identity: "did:ghost:alice".to_string(),
    daily_limit: hashmap! {
        TokenType::GCC => parse_token_amount("1000")?,
        TokenType::SPIRIT => parse_token_amount("100")?,
        TokenType::MANA => parse_token_amount("5000")?,
        TokenType::GHOST => parse_token_amount("50")?,
    },
    velocity_limit: 20, // Max 20 transactions per hour
    requires_multi_sig_above: parse_token_amount("10000")?,
};
```

---

## 📊 **API Reference**

### **Balance Operations**

#### **Get Account Balance**
```bash
curl -X POST http://localhost:8555 \
  -H "Content-Type: application/json" \
  -d '{
    "method": "gledger_get_balance",
    "params": {
      "identity": "did:ghost:alice",
      "token_type": "gcc"
    },
    "id": 1
  }'
```

#### **Get All Balances**
```bash
curl -X POST http://localhost:8555 \
  -H "Content-Type: application/json" \
  -d '{
    "method": "gledger_get_all_balances",
    "params": {
      "identity": "did:ghost:alice"
    },
    "id": 1
  }'
```

### **Transaction Operations**

#### **Transfer Tokens**
```bash
curl -X POST http://localhost:8555 \
  -H "Content-Type: application/json" \
  -d '{
    "method": "gledger_transfer",
    "params": {
      "from": "did:ghost:alice",
      "to": "did:ghost:bob",
      "token_type": "gcc",
      "amount": "100.5",
      "memo": "Payment for services"
    },
    "id": 1
  }'
```

#### **Transaction History**
```bash
curl -X POST http://localhost:8555 \
  -H "Content-Type: application/json" \
  -d '{
    "method": "gledger_transaction_history",
    "params": {
      "identity": "did:ghost:alice",
      "token_type": "gcc",
      "limit": 50,
      "offset": 0,
      "start_date": "2024-01-01T00:00:00Z",
      "end_date": "2024-12-31T23:59:59Z"
    },
    "id": 1
  }'
```

---

## 🎛️ **Advanced Features**

### **Staking Operations**
```rust
// SPIRIT token staking for governance participation
let staking_result = gledger.stake_tokens(
    "did:ghost:alice",
    TokenType::SPIRIT,
    parse_token_amount("1000")?,
    StakingDuration::Days(30),
    "Governance participation"
).await?;

// Calculate staking rewards
let rewards = gledger.calculate_staking_rewards(
    "did:ghost:alice",
    TokenType::SPIRIT
).await?;
```

### **Multi-Token Swaps**
```rust
// Atomic token swaps within the 4-token economy
let swap_request = TokenSwap {
    identity: "did:ghost:alice".to_string(),
    from_token: TokenType::GCC,
    from_amount: parse_token_amount("100")?,
    to_token: TokenType::MANA,
    minimum_received: parse_token_amount("95")?,  // 5% slippage tolerance
    expires_at: chrono::Utc::now() + chrono::Duration::minutes(10),
};

let swap_result = gledger.execute_swap(swap_request).await?;
```

### **Payment Streams**
```rust
// Continuous payment streams for recurring services
let payment_stream = PaymentStream {
    from: "did:ghost:company".to_string(),
    to: "did:ghost:alice".to_string(),
    token_type: TokenType::GCC,
    rate_per_second: parse_token_amount("0.001")?,  // 0.001 GCC/second
    total_amount: parse_token_amount("86.4")?,      // 24 hours worth
    start_time: chrono::Utc::now(),
    end_time: chrono::Utc::now() + chrono::Duration::hours(24),
};

let stream_id = gledger.create_payment_stream(payment_stream).await?;
```

---

## 📈 **Analytics & Reporting**

### **Transaction Analytics**
```rust
// Comprehensive transaction analytics
let analytics = gledger.get_transaction_analytics(
    "did:ghost:alice",
    AnalyticsTimeframe::Last30Days
).await?;

#[derive(Debug, Serialize)]
pub struct TransactionAnalytics {
    pub total_volume: HashMap<TokenType, TokenAmount>,
    pub transaction_count: u64,
    pub average_transaction_size: HashMap<TokenType, TokenAmount>,
    pub top_recipients: Vec<(String, TokenAmount)>,
    pub spending_patterns: Vec<SpendingPattern>,
    pub balance_trends: Vec<BalanceSnapshot>,
}
```

### **Protocol Metrics**
```bash
# Protocol-wide statistics
curl http://localhost:8555/analytics/protocol

# Token supply metrics
curl http://localhost:8555/analytics/token-supply

# Top accounts by balance
curl http://localhost:8555/analytics/top-accounts?token=gcc&limit=100
```

---

## 🔒 **Security Features**

### **Transaction Validation**
- **Double-Entry Verification** - All transactions must balance
- **Guardian Policy Check** - Policy approval before execution
- **Balance Validation** - Prevent overdrafts and negative balances
- **Rate Limiting** - Velocity controls and spending limits

### **Audit Trail**
```rust
// Complete audit trail for all operations
#[derive(Debug, Serialize)]
pub struct AuditLogEntry {
    pub transaction_id: String,
    pub operation_type: OperationType,
    pub initiator: String,
    pub affected_accounts: Vec<String>,
    pub amounts: HashMap<TokenType, TokenAmount>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub guardian_approval: Option<String>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
}
```

---

## 📊 **Performance Metrics**

### **Benchmarks**
- **Balance Queries**: 50,000+ ops/second
- **Simple Transfers**: 10,000+ TPS
- **Complex Transactions**: 5,000+ TPS
- **Analytics Queries**: 1,000+ ops/second

### **Monitoring**
```bash
# Service health
curl http://localhost:8555/health

# Performance metrics
curl http://localhost:8555/metrics

# Database statistics
curl http://localhost:8555/stats/database
```

---

## 🧪 **Testing**

```bash
# Run unit tests
cargo test -p gledger

# Run accounting tests
cargo test -p gledger --test double_entry_accounting

# Load testing
cargo run --bin gledger-load-test -- --transactions 10000

# Audit tests
cargo test -p gledger --test audit_compliance
```

---

## 🔗 **Integration Examples**

### **With GID Service**
```rust
use gledger::GLEDGERService;
use gid::GIDService;

// Identity-verified transfers
let gid = GIDService::new();
let gledger = GLEDGERService::new();

let sender_identity = gid.resolve("did:ghost:alice").await?;
let transfer_result = gledger.transfer_with_identity_verification(
    &sender_identity,
    transfer_request
).await?;
```

### **With CNS Service**
```rust
// Domain-based payments
let domain_owner = cns.resolve_owner("alice.ghost").await?;
let payment_result = gledger.pay_domain_owner(
    "alice.ghost",
    TokenType::GHOST,
    parse_token_amount("100")?,
    "Domain renewal payment"
).await?;
```

---

## 🔗 **Related Services**

- **[GID](../gid/README.md)** - Identity verification for transactions
- **[CNS](../cns/README.md)** - Domain-based payments
- **[GSIG](../gsig/README.md)** - Transaction signature verification

---

## 📚 **Resources**

- **[4-Token Economy Guide](../gcc-docs/token-economy.md)**
- **[Double-Entry Accounting](../gcc-docs/accounting-principles.md)**
- **[Guardian Transaction Policies](../gcc-docs/guardian-transactions.md)**
- **[Analytics & Reporting](../gcc-docs/transaction-analytics.md)**
- **[API Reference](../gcc-docs/gledger-api.md)**

---

*💰 Powering the GhostChain economy with precision and trust*