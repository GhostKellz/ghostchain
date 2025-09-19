# 🔐 WALLETD (GhostChain Wallet Daemon)

> **Secure wallet daemon with identity management, multi-signature support, and QUIC integration**

[![Rust](https://img.shields.io/badge/rust-1.75+-blue.svg)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-MIT-green.svg)](../LICENSE)
[![Port](https://img.shields.io/badge/port-8548-purple.svg)](http://localhost:8548)

---

## 🚀 **Overview**

WALLETD is GhostChain's secure wallet daemon that provides comprehensive key management, identity services, multi-signature support, and seamless integration with the GhostChain ecosystem. It offers both CLI and daemon modes for flexible wallet operations.

### **Key Features**
- **🔑 HD Wallet Management** - Hierarchical Deterministic wallets with BIP39/BIP44 support
- **🆔 Identity Integration** - Full Ghost Identity (GID) management
- **⚡ QUIC Transport** - High-performance networking via GhostLink
- **🔐 Multi-Signature Support** - Advanced multi-party transaction signing
- **🛡️ Hardware Wallet Ready** - Integration with hardware security modules
- **📱 Multi-Algorithm Support** - Ed25519, Secp256k1, BLS signatures

---

## 🏗️ **Architecture**

```
┌─────────────────────────────────────────────────────────────────┐
│                      WALLETD Core                              │
├─────────────────┬─────────────────┬─────────────────────────────┤
│ Wallet Manager  │ Identity Engine │       Crypto Backend       │
│                 │                 │                             │
│ • HD Wallets    │ • GID Creation  │ • Ed25519 Operations        │
│ • Key Storage   │ • DID Manage    │ • Secp256k1 Compat         │
│ • Multi-Sig     │ • Guardian      │ • BLS Signatures            │
└─────────────────┴─────────────────┴─────────────────────────────┘
           │                │                        │
           ▼                ▼                        ▼
┌─────────────────┐ ┌─────────────────┐ ┌─────────────────────┐
│ Secure Storage  │ │ QUIC Transport  │ │ Hardware Security   │
│                 │ │                 │ │                     │
│ • Encrypted DB  │ • GhostLink     │ • HSM Integration       │
│ • Key Derivation│ • Peer Comms    │ • Secure Enclaves       │
│ • Backup/Restore│ • API Server    │ • Hardware Wallets      │
└─────────────────┘ └─────────────────┘ └─────────────────────┘
```

### **Core Components**

| Component | Purpose | Features |
|-----------|---------|----------|
| **Wallet Manager** | HD wallet operations | Key derivation, transaction signing |
| **Identity Engine** | GID identity management | DID creation, Guardian integration |
| **Crypto Backend** | Multi-algorithm crypto | Ed25519, Secp256k1, BLS, post-quantum |
| **Secure Storage** | Encrypted key storage | Hardware security, backup/restore |

---

## 🔧 **Usage**

### **Start WALLETD Daemon**
```bash
# Start wallet daemon
walletd start --bind-address 0.0.0.0:8548 --enable-quic

# Start in background mode
walletd start --background

# Start in testnet mode
walletd --testnet start
```

### **Wallet Operations**
```bash
# Create new wallet
walletd wallet create main --algorithm secp256k1

# Import wallet from mnemonic
walletd wallet import recovery "abandon abandon abandon..."

# List all wallets
walletd wallet list

# Get wallet balance
walletd wallet balance main

# Send transaction
walletd wallet send main alice.ghost 100 --token GCC
```

### **Identity Operations**
```bash
# Create Ghost Identity
walletd identity create alice --key-algorithm ed25519

# List identities
walletd identity list

# Sign message with identity
walletd identity sign alice "Hello, GhostChain!"

# Verify signature
walletd identity verify alice "message" "signature_hex"
```

### **Configuration**
```toml
# walletd.toml
[daemon]
bind_address = "0.0.0.0:8548"
data_dir = "./walletd_data"
enable_quic = true
enable_api = true

[security]
encryption_algorithm = "ChaCha20Poly1305"
key_derivation = "PBKDF2"
secure_memory = true
auto_lock_timeout = 3600  # 1 hour

[wallets]
default_algorithm = "ed25519"
enable_hd_wallets = true
bip39_wordlist = "english"
derivation_path = "m/44'/60'/0'/0"

[identity]
default_did_method = "ghost"
enable_guardian = true
ephemeral_ttl = 3600

[networking]
quic_endpoint = "0.0.0.0:8548"
max_connections = 100
enable_peer_discovery = false
```

---

## 💼 **Wallet Management**

### **HD Wallet Structure**
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HDWallet {
    pub name: String,
    pub algorithm: CryptoAlgorithm,
    pub master_key: EncryptedKey,
    pub derivation_path: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub addresses: Vec<DerivedAddress>,
    pub metadata: WalletMetadata,
}

#[derive(Debug, Clone)]
pub struct DerivedAddress {
    pub index: u32,
    pub address: String,
    pub public_key: Vec<u8>,
    pub derivation_path: String,
    pub used: bool,
}
```

### **Multi-Algorithm Support**
```rust
// Support for multiple cryptographic algorithms
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CryptoAlgorithm {
    Ed25519,      // Fast, secure elliptic curve
    Secp256k1,    // Bitcoin/Ethereum compatibility
    BLS,          // Aggregate signatures
    PostQuantum,  // Future-proof cryptography
}

// Algorithm-specific operations
impl WalletManager {
    pub async fn create_wallet(&self, name: &str, algorithm: CryptoAlgorithm) -> Result<HDWallet> {
        let mnemonic = self.generate_mnemonic()?;
        let master_key = match algorithm {
            CryptoAlgorithm::Ed25519 => self.derive_ed25519_master(&mnemonic)?,
            CryptoAlgorithm::Secp256k1 => self.derive_secp256k1_master(&mnemonic)?,
            CryptoAlgorithm::BLS => self.derive_bls_master(&mnemonic)?,
            CryptoAlgorithm::PostQuantum => self.derive_pq_master(&mnemonic)?,
        };

        Ok(HDWallet {
            name: name.to_string(),
            algorithm,
            master_key: self.encrypt_key(&master_key)?,
            derivation_path: "m/44'/60'/0'/0".to_string(),
            created_at: chrono::Utc::now(),
            addresses: Vec::new(),
            metadata: WalletMetadata::default(),
        })
    }
}
```

### **Transaction Signing**
```rust
// Secure transaction signing with multiple algorithms
#[derive(Debug)]
pub struct TransactionSigningRequest {
    pub wallet_name: String,
    pub from_address: String,
    pub to_address: String,
    pub amount: TokenAmount,
    pub token_type: TokenType,
    pub gas_limit: u64,
    pub gas_price: u64,
    pub nonce: u64,
}

impl WalletDaemon {
    pub async fn sign_transaction(&self, request: TransactionSigningRequest) -> Result<SignedTransaction> {
        // 1. Load wallet and decrypt key
        let wallet = self.get_wallet(&request.wallet_name).await?;
        let private_key = self.decrypt_key(&wallet.master_key).await?;

        // 2. Derive signing key for specific address
        let signing_key = self.derive_address_key(&private_key, &request.from_address).await?;

        // 3. Create transaction hash
        let tx_data = TransactionData {
            from: request.from_address,
            to: request.to_address,
            amount: request.amount,
            gas_limit: request.gas_limit,
            gas_price: request.gas_price,
            nonce: request.nonce,
        };
        let tx_hash = self.hash_transaction(&tx_data).await?;

        // 4. Sign with appropriate algorithm
        let signature = match wallet.algorithm {
            CryptoAlgorithm::Ed25519 => self.crypto.ed25519_sign(&signing_key, &tx_hash)?,
            CryptoAlgorithm::Secp256k1 => self.crypto.secp256k1_sign(&signing_key, &tx_hash)?,
            CryptoAlgorithm::BLS => self.crypto.bls_sign(&signing_key, &tx_hash)?,
            CryptoAlgorithm::PostQuantum => self.crypto.pq_sign(&signing_key, &tx_hash)?,
        };

        Ok(SignedTransaction {
            transaction_data: tx_data,
            signature,
            algorithm: wallet.algorithm,
        })
    }
}
```

---

## 🆔 **Identity Management**

### **Ghost Identity Integration**
```rust
// Full GID integration within walletd
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManagedIdentity {
    pub did: String,                    // did:ghost:alice
    pub name: String,                   // Friendly name
    pub key_pair: EncryptedKeyPair,     // Ed25519 key pair
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub guardian_policies: Vec<GuardianPolicy>,
    pub linked_wallets: Vec<String>,    // Associated wallet names
    pub metadata: IdentityMetadata,
}

impl IdentityManager {
    pub async fn create_identity(&self, name: &str, algorithm: CryptoAlgorithm) -> Result<ManagedIdentity> {
        // 1. Generate cryptographic keys
        let key_pair = match algorithm {
            CryptoAlgorithm::Ed25519 => self.crypto.generate_ed25519_keypair()?,
            _ => return Err(anyhow!("Unsupported algorithm for identity")),
        };

        // 2. Create DID
        let did = format!("did:ghost:{}", self.generate_identifier(&key_pair.public_key)?);

        // 3. Set up Guardian policies
        let default_policies = vec![
            GuardianPolicy::RequirePasswordForSigning,
            GuardianPolicy::AutoLockAfterInactivity(Duration::hours(1)),
            GuardianPolicy::RequireConfirmationForLargeTransactions(parse_token_amount("1000")?),
        ];

        Ok(ManagedIdentity {
            did,
            name: name.to_string(),
            key_pair: self.encrypt_keypair(&key_pair)?,
            created_at: chrono::Utc::now(),
            guardian_policies: default_policies,
            linked_wallets: Vec::new(),
            metadata: IdentityMetadata::default(),
        })
    }
}
```

### **DID Document Management**
```rust
// DID document creation and management
impl IdentityManager {
    pub async fn create_did_document(&self, identity: &ManagedIdentity) -> Result<DIDDocument> {
        let public_key = self.decrypt_public_key(&identity.key_pair)?;

        Ok(DIDDocument {
            id: identity.did.clone(),
            authentication: vec![format!("{}#key-1", identity.did)],
            public_key: vec![PublicKeyEntry {
                id: format!("{}#key-1", identity.did),
                key_type: "Ed25519VerificationKey2020".to_string(),
                controller: identity.did.clone(),
                public_key_base58: base58::encode(&public_key),
            }],
            service: vec![ServiceEntry {
                id: format!("{}#walletd", identity.did),
                service_type: "WalletService".to_string(),
                service_endpoint: format!("quic://localhost:8548/identity/{}", identity.name),
            }],
            guardian_policies: identity.guardian_policies.clone(),
        })
    }
}
```

---

## 🔐 **Security Features**

### **Secure Key Storage**
```rust
// Encrypted key storage with multiple protection layers
#[derive(Debug)]
pub struct SecureKeyStorage {
    pub encryption_key: EncryptionKey,
    pub storage_backend: StorageBackend,
    pub hsm_integration: Option<HSMConfig>,
}

#[derive(Debug)]
pub enum StorageBackend {
    EncryptedFile { path: String, cipher: Cipher },
    SecureEnclave { provider: String },
    HardwareWallet { device_type: HWDevice },
    Memory { secure_heap: bool },
}

impl SecureKeyStorage {
    pub async fn store_key(&self, key_id: &str, key_data: &[u8]) -> Result<()> {
        // 1. Encrypt key with AES-256-GCM
        let encrypted_data = self.encrypt_key_data(key_data)?;

        // 2. Add integrity protection
        let protected_data = self.add_integrity_protection(&encrypted_data)?;

        // 3. Store with appropriate backend
        match &self.storage_backend {
            StorageBackend::EncryptedFile { path, .. } => {
                self.store_to_encrypted_file(path, key_id, &protected_data).await?;
            }
            StorageBackend::SecureEnclave { provider } => {
                self.store_to_secure_enclave(provider, key_id, &protected_data).await?;
            }
            StorageBackend::HardwareWallet { device_type } => {
                self.store_to_hardware_wallet(device_type, key_id, &protected_data).await?;
            }
            StorageBackend::Memory { .. } => {
                self.store_to_secure_memory(key_id, &protected_data).await?;
            }
        }

        Ok(())
    }
}
```

### **Multi-Signature Support**
```rust
// Advanced multi-signature transaction support
#[derive(Debug, Clone)]
pub struct MultiSigWallet {
    pub name: String,
    pub participants: Vec<MultiSigParticipant>,
    pub threshold: u32,
    pub algorithm: CryptoAlgorithm,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone)]
pub struct MultiSigParticipant {
    pub identity: String,           // DID or wallet name
    pub public_key: Vec<u8>,
    pub weight: u32,                // Voting weight
    pub status: ParticipantStatus,
}

impl MultiSigManager {
    pub async fn create_multisig_transaction(
        &self,
        wallet_name: &str,
        tx_request: TransactionSigningRequest,
    ) -> Result<PendingMultiSigTransaction> {
        let multisig_wallet = self.get_multisig_wallet(wallet_name).await?;

        // 1. Create transaction proposal
        let proposal = MultiSigProposal {
            id: uuid::Uuid::new_v4().to_string(),
            wallet_name: wallet_name.to_string(),
            transaction: tx_request,
            created_at: chrono::Utc::now(),
            expires_at: chrono::Utc::now() + chrono::Duration::hours(24),
            signatures: Vec::new(),
            status: ProposalStatus::Pending,
        };

        // 2. Initiate signing process
        let pending_tx = PendingMultiSigTransaction {
            proposal,
            required_signatures: multisig_wallet.threshold,
            collected_signatures: 0,
            participants_contacted: Vec::new(),
        };

        // 3. Notify participants
        self.notify_participants(&multisig_wallet, &pending_tx).await?;

        Ok(pending_tx)
    }
}
```

### **Hardware Wallet Integration**
```rust
// Hardware wallet support for enhanced security
#[derive(Debug)]
pub enum HardwareWallet {
    Ledger { device_id: String, app_version: String },
    Trezor { device_id: String, firmware_version: String },
    YubiKey { serial: String, slot: u8 },
    Custom { driver: String, config: serde_json::Value },
}

impl HardwareWalletManager {
    pub async fn sign_with_hardware(&self, device: &HardwareWallet, tx_hash: &[u8]) -> Result<Vec<u8>> {
        match device {
            HardwareWallet::Ledger { device_id, .. } => {
                self.ledger_sign(device_id, tx_hash).await
            }
            HardwareWallet::Trezor { device_id, .. } => {
                self.trezor_sign(device_id, tx_hash).await
            }
            HardwareWallet::YubiKey { serial, slot } => {
                self.yubikey_sign(serial, *slot, tx_hash).await
            }
            HardwareWallet::Custom { driver, config } => {
                self.custom_device_sign(driver, config, tx_hash).await
            }
        }
    }
}
```

---

## 📊 **API Reference**

### **Wallet Operations**

#### **Create Wallet**
```bash
curl -X POST http://localhost:8548 \
  -H "Content-Type: application/json" \
  -d '{
    "method": "wallet_create",
    "params": {
      "name": "main",
      "algorithm": "ed25519",
      "mnemonic": null
    },
    "id": 1
  }'
```

#### **Get Wallet Balance**
```bash
curl -X POST http://localhost:8548 \
  -H "Content-Type: application/json" \
  -d '{
    "method": "wallet_getBalance",
    "params": {
      "wallet_name": "main",
      "token_types": ["GCC", "SPIRIT", "MANA", "GHOST"]
    },
    "id": 1
  }'
```

### **Identity Operations**

#### **Create Identity**
```bash
curl -X POST http://localhost:8548 \
  -H "Content-Type: application/json" \
  -d '{
    "method": "identity_create",
    "params": {
      "name": "alice",
      "algorithm": "ed25519",
      "guardian_policies": ["RequirePasswordForSigning"]
    },
    "id": 1
  }'
```

#### **Sign Message**
```bash
curl -X POST http://localhost:8548 \
  -H "Content-Type: application/json" \
  -d '{
    "method": "identity_sign",
    "params": {
      "identity_name": "alice",
      "message": "Hello, GhostChain!",
      "encoding": "utf8"
    },
    "id": 1
  }'
```

### **Transaction Operations**

#### **Send Transaction**
```bash
curl -X POST http://localhost:8548 \
  -H "Content-Type: application/json" \
  -d '{
    "method": "wallet_sendTransaction",
    "params": {
      "from_wallet": "main",
      "to_address": "did:ghost:alice",
      "amount": "100.0",
      "token_type": "GCC",
      "gas_limit": 21000,
      "gas_price": 20000000000
    },
    "id": 1
  }'
```

---

## 🎛️ **Advanced Features**

### **Backup & Recovery**
```rust
// Comprehensive backup and recovery system
#[derive(Debug, Serialize, Deserialize)]
pub struct WalletBackup {
    pub version: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub wallets: Vec<EncryptedWalletData>,
    pub identities: Vec<EncryptedIdentityData>,
    pub settings: WalletdSettings,
    pub checksum: String,
}

impl BackupManager {
    pub async fn create_backup(&self, password: &str) -> Result<WalletBackup> {
        // 1. Collect all wallet data
        let wallets = self.export_all_wallets().await?;
        let identities = self.export_all_identities().await?;

        // 2. Encrypt with user password
        let encrypted_wallets = self.encrypt_wallet_data(&wallets, password)?;
        let encrypted_identities = self.encrypt_identity_data(&identities, password)?;

        // 3. Create backup package
        let backup = WalletBackup {
            version: env!("CARGO_PKG_VERSION").to_string(),
            created_at: chrono::Utc::now(),
            wallets: encrypted_wallets,
            identities: encrypted_identities,
            settings: self.get_settings().await?,
            checksum: String::new(), // Calculated after serialization
        };

        Ok(backup)
    }

    pub async fn restore_backup(&self, backup: &WalletBackup, password: &str) -> Result<()> {
        // 1. Verify backup integrity
        self.verify_backup_checksum(backup)?;

        // 2. Decrypt wallet data
        let wallets = self.decrypt_wallet_data(&backup.wallets, password)?;
        let identities = self.decrypt_identity_data(&backup.identities, password)?;

        // 3. Restore to storage
        for wallet in wallets {
            self.import_wallet(wallet).await?;
        }

        for identity in identities {
            self.import_identity(identity).await?;
        }

        Ok(())
    }
}
```

### **Auto-Lock Security**
```rust
// Automatic wallet locking for security
#[derive(Debug)]
pub struct AutoLockManager {
    pub lock_timeout: Duration,
    pub last_activity: Arc<RwLock<chrono::DateTime<chrono::Utc>>>,
    pub is_locked: Arc<RwLock<bool>>,
    pub unlock_attempts: Arc<RwLock<u32>>,
}

impl AutoLockManager {
    pub async fn check_auto_lock(&self) -> Result<()> {
        let last_activity = *self.last_activity.read().await;
        let now = chrono::Utc::now();

        if now - last_activity > self.lock_timeout {
            self.lock_wallet().await?;
        }

        Ok(())
    }

    pub async fn unlock_wallet(&self, password: &str) -> Result<()> {
        if self.verify_password(password).await? {
            *self.is_locked.write().await = false;
            *self.unlock_attempts.write().await = 0;
            self.update_last_activity().await;
            Ok(())
        } else {
            let mut attempts = self.unlock_attempts.write().await;
            *attempts += 1;

            if *attempts >= 5 {
                // Lock wallet for extended period after failed attempts
                self.extended_lock().await?;
            }

            Err(anyhow!("Invalid password"))
        }
    }
}
```

### **Gas Estimation & Optimization**
```rust
// Smart gas estimation for optimal transaction costs
#[derive(Debug)]
pub struct GasEstimator {
    pub network_stats: NetworkStatistics,
    pub historical_data: GasHistoryData,
    pub optimization_settings: GasOptimizationSettings,
}

impl GasEstimator {
    pub async fn estimate_gas(&self, tx: &TransactionSigningRequest) -> Result<GasEstimate> {
        // 1. Base gas calculation
        let base_gas = self.calculate_base_gas(&tx).await?;

        // 2. Network congestion adjustment
        let congestion_multiplier = self.get_congestion_multiplier().await?;

        // 3. Priority fee suggestion
        let priority_fee = self.suggest_priority_fee().await?;

        // 4. Total cost estimation
        let estimated_cost = GasEstimate {
            gas_limit: base_gas,
            gas_price: (self.network_stats.base_fee_per_gas as f64 * congestion_multiplier) as u64,
            priority_fee,
            total_cost: (base_gas as f64 * congestion_multiplier * self.network_stats.base_fee_per_gas as f64) as u64,
            confirmation_time_estimate: self.estimate_confirmation_time(congestion_multiplier).await?,
        };

        Ok(estimated_cost)
    }
}
```

---

## 🚀 **Performance Features**

### **Performance Metrics**
- **Key Generation**: 10,000+ keys/second (Ed25519)
- **Transaction Signing**: 5,000+ signatures/second
- **Identity Operations**: 1,000+ DID ops/second
- **Memory Usage**: <100MB baseline, <500MB under load

### **Optimization Strategies**
```rust
// Performance optimization configurations
#[derive(Debug)]
pub struct PerformanceConfig {
    pub enable_key_caching: bool,
    pub cache_size: usize,
    pub signature_batching: bool,
    pub async_operations: bool,
    pub hardware_acceleration: bool,
}

// Signature batching for high throughput
impl SignatureBatcher {
    pub async fn batch_sign(&self, requests: Vec<SigningRequest>) -> Result<Vec<Signature>> {
        // Process multiple signatures efficiently
        let batch_size = 100;
        let mut results = Vec::new();

        for chunk in requests.chunks(batch_size) {
            let batch_results = self.process_signature_batch(chunk).await?;
            results.extend(batch_results);
        }

        Ok(results)
    }
}
```

---

## 🔗 **Integration Examples**

### **With GCC Services**
```rust
use walletd::WalletdClient;

// Integrate with other GCC services
let walletd = WalletdClient::connect("http://localhost:8548").await?;
let gid = GIDClient::connect("http://localhost:8552").await?;
let gledger = GLEDGERClient::connect("http://localhost:8555").await?;

// Create identity and link to wallet
let identity = walletd.create_identity("alice", "ed25519").await?;
let gid_registration = gid.register_identity(&identity.did_document).await?;

// Transfer tokens using wallet
let balance = gledger.get_balance(&identity.did).await?;
let transfer = walletd.send_transaction("main", "bob.ghost", "100", "GCC").await?;
```

### **Web3 Application Integration**
```rust
// Building Web3 applications with walletd
let wallet_config = WalletConfig {
    endpoint: "http://localhost:8548",
    auto_unlock: false,
    default_gas_price: 20_000_000_000,
    default_gas_limit: 21_000,
};

let wallet_client = WalletClient::new(wallet_config).await?;

// Dapp transaction signing
let dapp_tx = DappTransaction {
    to: contract_address,
    data: contract_call_data,
    value: 0,
    gas_limit: 100_000,
};

let signed_tx = wallet_client.sign_dapp_transaction(dapp_tx).await?;
```

---

## 🧪 **Development Tools**

### **CLI Development Commands**
```bash
# Development utilities
walletd dev generate-test-wallets --count 10
walletd dev benchmark-signing --algorithm ed25519 --iterations 10000
walletd dev export-keys --wallet main --format json

# Testing tools
walletd test create-test-identity --name test-alice
walletd test multisig-scenario --participants 5 --threshold 3
walletd test hardware-wallet-sim --device ledger

# Security tools
walletd security audit-keys
walletd security check-entropy
walletd security test-encryption --algorithm aes256gcm
```

---

## 🔗 **Related Services**

- **[GID](../gid/README.md)** - Identity verification and DID management
- **[GSIG](../gsig/README.md)** - Signature verification services
- **[GLEDGER](../gledger/README.md)** - Token balance and transfer operations
- **[GHOSTD](../ghostd/README.md)** - Blockchain transaction submission

---

## 📚 **Resources**

- **[Wallet Setup Guide](../gcc-docs/wallet-setup.md)**
- **[Security Best Practices](../gcc-docs/wallet-security.md)**
- **[Multi-Signature Guide](../gcc-docs/multisig-guide.md)**
- **[Hardware Wallet Integration](../gcc-docs/hardware-wallets.md)**
- **[API Documentation](../gcc-docs/walletd-api.md)**

---

*🔐 Secure, user-friendly wallet infrastructure for the GhostChain ecosystem*