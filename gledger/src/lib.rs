// GLedger - Ghost Ledger Service
//
// Provides transaction processing and state management for GhostChain
// Double-entry accounting with audit trails and identity integration

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap, VecDeque};
use tokio::sync::RwLock;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use ghostchain_shared::types::Address;

/// Account types in the ledger system
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AccountType {
    Asset,      // Regular user accounts
    Liability,  // System liabilities
    Equity,     // Protocol equity
    Revenue,    // Revenue accounts (fees, rewards)
    Expense,    // Expense accounts (gas, operations)
}

/// Token types supported by the ledger
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TokenType {
    GCC,    // Gas & transaction fees
    SPIRIT, // Governance & voting
    MANA,   // Utility & rewards
    GHOST,  // Brand & collectibles
}

impl TokenType {
    pub fn symbol(&self) -> &'static str {
        match self {
            TokenType::GCC => "GCC",
            TokenType::SPIRIT => "SPIRIT",
            TokenType::MANA => "MANA",
            TokenType::GHOST => "GHOST",
        }
    }
}

/// Ledger account
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Account {
    pub address: Address,
    pub account_type: AccountType,
    pub token_type: TokenType,
    pub balance: Decimal,
    pub locked_balance: Decimal,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub metadata: AccountMetadata,
}

/// Account metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountMetadata {
    pub name: Option<String>,
    pub description: Option<String>,
    pub owner_gid: Option<String>, // Ghost Identity
    pub tags: Vec<String>,
}

impl Default for AccountMetadata {
    fn default() -> Self {
        Self {
            name: None,
            description: None,
            owner_gid: None,
            tags: vec![],
        }
    }
}

/// Transaction journal entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JournalEntry {
    pub entry_id: String,
    pub transaction_id: String,
    pub account_address: Address,
    pub token_type: TokenType,
    pub debit: Option<Decimal>,
    pub credit: Option<Decimal>,
    pub description: String,
    pub reference: Option<String>,
    pub timestamp: DateTime<Utc>,
    pub metadata: TransactionMetadata,
}

/// Transaction metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionMetadata {
    pub initiator_gid: Option<String>,
    pub transaction_type: TransactionType,
    pub block_height: Option<u64>,
    pub gas_used: Option<Decimal>,
    pub tags: Vec<String>,
}

/// Transaction types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransactionType {
    Transfer,
    Mint,
    Burn,
    Stake,
    Unstake,
    Reward,
    Fee,
    Penalty,
    Custom(String),
}

/// Complete transaction with journal entries
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    pub transaction_id: String,
    pub entries: Vec<JournalEntry>,
    pub total_debits: Decimal,
    pub total_credits: Decimal,
    pub timestamp: DateTime<Utc>,
    pub status: TransactionStatus,
    pub metadata: TransactionMetadata,
}

/// Transaction status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransactionStatus {
    Pending,
    Confirmed,
    Failed,
    Reverted,
}

/// Audit trail entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    pub audit_id: String,
    pub transaction_id: String,
    pub action: AuditAction,
    pub actor_gid: Option<String>,
    pub timestamp: DateTime<Utc>,
    pub details: serde_json::Value,
    pub signature: Option<String>,
}

/// Audit actions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuditAction {
    TransactionCreated,
    TransactionConfirmed,
    TransactionFailed,
    AccountCreated,
    AccountUpdated,
    BalanceChanged,
    Custom(String),
}

/// Ledger errors
#[derive(Debug, thiserror::Error)]
pub enum LedgerError {
    #[error("Account not found: {address}")]
    AccountNotFound { address: Address },

    #[error("Insufficient balance: {address} has {available}, need {required}")]
    InsufficientBalance { address: Address, available: Decimal, required: Decimal },

    #[error("Transaction not balanced: debits={debits}, credits={credits}")]
    TransactionNotBalanced { debits: Decimal, credits: Decimal },

    #[error("Invalid transaction: {reason}")]
    InvalidTransaction { reason: String },

    #[error("Account already exists: {address}")]
    AccountAlreadyExists { address: Address },
}

/// Account manager
pub struct AccountManager {
    accounts: RwLock<HashMap<(Address, TokenType), Account>>,
}

impl AccountManager {
    pub fn new() -> Self {
        Self {
            accounts: RwLock::new(HashMap::new()),
        }
    }

    /// Create a new account
    pub async fn create_account(
        &self,
        address: Address,
        account_type: AccountType,
        token_type: TokenType,
        metadata: AccountMetadata,
    ) -> Result<()> {
        let mut accounts = self.accounts.write().await;

        let key = (address, token_type);
        if accounts.contains_key(&key) {
            return Err(LedgerError::AccountAlreadyExists { address }.into());
        }

        let account = Account {
            address,
            account_type,
            token_type,
            balance: Decimal::ZERO,
            locked_balance: Decimal::ZERO,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            metadata,
        };

        accounts.insert(key, account);
        Ok(())
    }

    /// Get account
    pub async fn get_account(&self, address: &Address, token_type: TokenType) -> Result<Account> {
        let accounts = self.accounts.read().await;
        let key = (*address, token_type);
        accounts.get(&key)
            .cloned()
            .ok_or_else(|| LedgerError::AccountNotFound { address: *address }.into())
    }

    /// Get or create account
    pub async fn get_or_create_account(
        &self,
        address: Address,
        token_type: TokenType,
        account_type: AccountType,
    ) -> Result<Account> {
        // Try to get existing account
        if let Ok(account) = self.get_account(&address, token_type).await {
            return Ok(account);
        }

        // Create new account
        self.create_account(address, account_type, token_type, AccountMetadata::default()).await?;
        self.get_account(&address, token_type).await
    }

    /// Update account balance
    pub async fn update_balance(
        &self,
        address: &Address,
        token_type: TokenType,
        new_balance: Decimal,
    ) -> Result<()> {
        let mut accounts = self.accounts.write().await;
        let key = (*address, token_type);

        let account = accounts.get_mut(&key)
            .ok_or_else(|| LedgerError::AccountNotFound { address: *address })?;

        account.balance = new_balance;
        account.updated_at = Utc::now();
        Ok(())
    }

    /// Lock tokens
    pub async fn lock_tokens(
        &self,
        address: &Address,
        token_type: TokenType,
        amount: Decimal,
    ) -> Result<()> {
        let mut accounts = self.accounts.write().await;
        let key = (*address, token_type);

        let account = accounts.get_mut(&key)
            .ok_or_else(|| LedgerError::AccountNotFound { address: *address })?;

        let available = account.balance - account.locked_balance;
        if available < amount {
            return Err(LedgerError::InsufficientBalance {
                address: *address,
                available,
                required: amount,
            }.into());
        }

        account.locked_balance += amount;
        account.updated_at = Utc::now();
        Ok(())
    }

    /// Get all accounts for an address
    pub async fn get_accounts_for_address(&self, address: &Address) -> Vec<Account> {
        let accounts = self.accounts.read().await;
        accounts.values()
            .filter(|account| account.address == *address)
            .cloned()
            .collect()
    }
}

/// Transaction processor
pub struct TransactionProcessor {
    journal: RwLock<VecDeque<JournalEntry>>,
    transactions: RwLock<HashMap<String, Transaction>>,
    audit_trail: RwLock<Vec<AuditEntry>>,
}

impl TransactionProcessor {
    pub fn new() -> Self {
        Self {
            journal: RwLock::new(VecDeque::new()),
            transactions: RwLock::new(HashMap::new()),
            audit_trail: RwLock::new(Vec::new()),
        }
    }

    /// Create a new transaction
    pub async fn create_transaction(
        &self,
        entries: Vec<JournalEntry>,
        metadata: TransactionMetadata,
    ) -> Result<String> {
        // Validate transaction is balanced
        let total_debits: Decimal = entries.iter()
            .filter_map(|e| e.debit)
            .sum();
        let total_credits: Decimal = entries.iter()
            .filter_map(|e| e.credit)
            .sum();

        if total_debits != total_credits {
            return Err(LedgerError::TransactionNotBalanced {
                debits: total_debits,
                credits: total_credits,
            }.into());
        }

        let transaction_id = uuid::Uuid::new_v4().to_string();
        let now = Utc::now();

        // Create transaction
        let transaction = Transaction {
            transaction_id: transaction_id.clone(),
            entries: entries.clone(),
            total_debits,
            total_credits,
            timestamp: now,
            status: TransactionStatus::Pending,
            metadata: metadata.clone(),
        };

        // Store transaction
        let mut transactions = self.transactions.write().await;
        transactions.insert(transaction_id.clone(), transaction);

        // Add entries to journal
        let mut journal = self.journal.write().await;
        for entry in entries {
            journal.push_back(entry);
        }

        // Create audit entry
        self.add_audit_entry(AuditEntry {
            audit_id: uuid::Uuid::new_v4().to_string(),
            transaction_id: transaction_id.clone(),
            action: AuditAction::TransactionCreated,
            actor_gid: metadata.initiator_gid,
            timestamp: now,
            details: serde_json::to_value(&metadata)?,
            signature: None,
        }).await;

        Ok(transaction_id)
    }

    /// Confirm a transaction
    pub async fn confirm_transaction(&self, transaction_id: &str) -> Result<()> {
        let mut transactions = self.transactions.write().await;
        let transaction = transactions.get_mut(transaction_id)
            .ok_or_else(|| anyhow!("Transaction not found: {}", transaction_id))?;

        transaction.status = TransactionStatus::Confirmed;

        // Create audit entry
        self.add_audit_entry(AuditEntry {
            audit_id: uuid::Uuid::new_v4().to_string(),
            transaction_id: transaction_id.to_string(),
            action: AuditAction::TransactionConfirmed,
            actor_gid: None,
            timestamp: Utc::now(),
            details: serde_json::Value::Null,
            signature: None,
        }).await;

        Ok(())
    }

    /// Get transaction
    pub async fn get_transaction(&self, transaction_id: &str) -> Option<Transaction> {
        let transactions = self.transactions.read().await;
        transactions.get(transaction_id).cloned()
    }

    /// Get transactions for account
    pub async fn get_transactions_for_account(&self, address: &Address) -> Vec<Transaction> {
        let transactions = self.transactions.read().await;
        transactions.values()
            .filter(|tx| tx.entries.iter().any(|entry| entry.account_address == *address))
            .cloned()
            .collect()
    }

    /// Add audit entry
    async fn add_audit_entry(&self, entry: AuditEntry) {
        let mut audit_trail = self.audit_trail.write().await;
        audit_trail.push(entry);
    }

    /// Get audit trail
    pub async fn get_audit_trail(&self) -> Vec<AuditEntry> {
        let audit_trail = self.audit_trail.read().await;
        audit_trail.clone()
    }
}

/// Main ledger service
pub struct GLedgerService {
    account_manager: AccountManager,
    transaction_processor: TransactionProcessor,
}

impl GLedgerService {
    pub fn new() -> Self {
        Self {
            account_manager: AccountManager::new(),
            transaction_processor: TransactionProcessor::new(),
        }
    }

    /// Transfer tokens between accounts
    pub async fn transfer(
        &self,
        from: Address,
        to: Address,
        token_type: TokenType,
        amount: Decimal,
        description: String,
        initiator_gid: Option<String>,
    ) -> Result<String> {
        // Ensure accounts exist
        self.account_manager.get_or_create_account(from, token_type, AccountType::Asset).await?;
        self.account_manager.get_or_create_account(to, token_type, AccountType::Asset).await?;

        // Check balance
        let from_account = self.account_manager.get_account(&from, token_type).await?;
        let available = from_account.balance - from_account.locked_balance;
        if available < amount {
            return Err(LedgerError::InsufficientBalance {
                address: from,
                available,
                required: amount,
            }.into());
        }

        // Create journal entries
        let transaction_id = uuid::Uuid::new_v4().to_string();
        let now = Utc::now();

        let entries = vec![
            JournalEntry {
                entry_id: uuid::Uuid::new_v4().to_string(),
                transaction_id: transaction_id.clone(),
                account_address: from,
                token_type,
                debit: Some(amount),
                credit: None,
                description: format!("Transfer to {}: {}", to, description),
                reference: None,
                timestamp: now,
                metadata: TransactionMetadata {
                    initiator_gid: initiator_gid.clone(),
                    transaction_type: TransactionType::Transfer,
                    block_height: None,
                    gas_used: None,
                    tags: vec![],
                },
            },
            JournalEntry {
                entry_id: uuid::Uuid::new_v4().to_string(),
                transaction_id: transaction_id.clone(),
                account_address: to,
                token_type,
                debit: None,
                credit: Some(amount),
                description: format!("Transfer from {}: {}", from, description),
                reference: None,
                timestamp: now,
                metadata: TransactionMetadata {
                    initiator_gid: initiator_gid.clone(),
                    transaction_type: TransactionType::Transfer,
                    block_height: None,
                    gas_used: None,
                    tags: vec![],
                },
            },
        ];

        // Process transaction
        let tx_id = self.transaction_processor.create_transaction(
            entries,
            TransactionMetadata {
                initiator_gid,
                transaction_type: TransactionType::Transfer,
                block_height: None,
                gas_used: None,
                tags: vec![],
            },
        ).await?;

        // Update balances
        self.account_manager.update_balance(&from, token_type, from_account.balance - amount).await?;
        let to_account = self.account_manager.get_account(&to, token_type).await?;
        self.account_manager.update_balance(&to, token_type, to_account.balance + amount).await?;

        // Confirm transaction
        self.transaction_processor.confirm_transaction(&tx_id).await?;

        Ok(tx_id)
    }

    /// Mint new tokens
    pub async fn mint(
        &self,
        to: Address,
        token_type: TokenType,
        amount: Decimal,
        reason: String,
        authority_gid: Option<String>,
    ) -> Result<String> {
        // Ensure account exists
        self.account_manager.get_or_create_account(to, token_type, AccountType::Asset).await?;

        // Create journal entries (credit to user, debit to system supply account)
        let transaction_id = uuid::Uuid::new_v4().to_string();
        let now = Utc::now();
        let system_address = Address::default(); // System mint address

        let entries = vec![
            JournalEntry {
                entry_id: uuid::Uuid::new_v4().to_string(),
                transaction_id: transaction_id.clone(),
                account_address: system_address,
                token_type,
                debit: Some(amount),
                credit: None,
                description: format!("Mint {} to {}: {}", token_type.symbol(), to, reason),
                reference: None,
                timestamp: now,
                metadata: TransactionMetadata {
                    initiator_gid: authority_gid.clone(),
                    transaction_type: TransactionType::Mint,
                    block_height: None,
                    gas_used: None,
                    tags: vec![],
                },
            },
            JournalEntry {
                entry_id: uuid::Uuid::new_v4().to_string(),
                transaction_id: transaction_id.clone(),
                account_address: to,
                token_type,
                debit: None,
                credit: Some(amount),
                description: format!("Minted {} {}: {}", amount, token_type.symbol(), reason),
                reference: None,
                timestamp: now,
                metadata: TransactionMetadata {
                    initiator_gid: authority_gid.clone(),
                    transaction_type: TransactionType::Mint,
                    block_height: None,
                    gas_used: None,
                    tags: vec![],
                },
            },
        ];

        // Process transaction
        let tx_id = self.transaction_processor.create_transaction(
            entries,
            TransactionMetadata {
                initiator_gid: authority_gid,
                transaction_type: TransactionType::Mint,
                block_height: None,
                gas_used: None,
                tags: vec![],
            },
        ).await?;

        // Update balance
        let account = self.account_manager.get_account(&to, token_type).await?;
        self.account_manager.update_balance(&to, token_type, account.balance + amount).await?;

        // Confirm transaction
        self.transaction_processor.confirm_transaction(&tx_id).await?;

        Ok(tx_id)
    }

    /// Get account manager
    pub fn account_manager(&self) -> &AccountManager {
        &self.account_manager
    }

    /// Get transaction processor
    pub fn transaction_processor(&self) -> &TransactionProcessor {
        &self.transaction_processor
    }
}

/// GLedger daemon configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GLedgerConfig {
    pub bind_address: String,
    pub rpc_port: u16,
    pub grpc_port: u16,
    pub enable_audit: bool,
    pub max_journal_entries: usize,
}

impl Default for GLedgerConfig {
    fn default() -> Self {
        Self {
            bind_address: "127.0.0.1".to_string(),
            rpc_port: 8555,
            grpc_port: 9555,
            enable_audit: true,
            max_journal_entries: 1_000_000,
        }
    }
}

/// GLedger daemon service
pub struct GLedgerDaemon {
    service: GLedgerService,
    config: GLedgerConfig,
}

impl GLedgerDaemon {
    pub fn new(config: GLedgerConfig) -> Self {
        Self {
            service: GLedgerService::new(),
            config,
        }
    }

    pub async fn start(&self) -> Result<()> {
        tracing::info!("Starting GLedger daemon on {}:{}", self.config.bind_address, self.config.rpc_port);

        // TODO: Start RPC server
        // TODO: Start gRPC server

        Ok(())
    }

    pub fn service(&self) -> &GLedgerService {
        &self.service
    }
}

impl Default for AccountManager {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for TransactionProcessor {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for GLedgerService {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_account_creation() {
        let manager = AccountManager::new();
        let address = Address::default();

        manager.create_account(
            address,
            AccountType::Asset,
            TokenType::GCC,
            AccountMetadata::default(),
        ).await.unwrap();

        let account = manager.get_account(&address, TokenType::GCC).await.unwrap();
        assert_eq!(account.balance, Decimal::ZERO);
        assert_eq!(account.token_type, TokenType::GCC);
    }

    #[tokio::test]
    async fn test_transfer() {
        let service = GLedgerService::new();
        let alice = Address::from([1u8; 32]);
        let bob = Address::from([2u8; 32]);

        // Mint tokens to Alice
        service.mint(
            alice,
            TokenType::GCC,
            Decimal::from(1000),
            "Initial mint".to_string(),
            None,
        ).await.unwrap();

        // Transfer from Alice to Bob
        let tx_id = service.transfer(
            alice,
            bob,
            TokenType::GCC,
            Decimal::from(100),
            "Test transfer".to_string(),
            None,
        ).await.unwrap();

        assert!(!tx_id.is_empty());

        // Check balances
        let alice_account = service.account_manager.get_account(&alice, TokenType::GCC).await.unwrap();
        let bob_account = service.account_manager.get_account(&bob, TokenType::GCC).await.unwrap();

        assert_eq!(alice_account.balance, Decimal::from(900));
        assert_eq!(bob_account.balance, Decimal::from(100));
    }

    #[tokio::test]
    async fn test_insufficient_balance() {
        let service = GLedgerService::new();
        let alice = Address::from([1u8; 32]);
        let bob = Address::from([2u8; 32]);

        // Try to transfer without balance
        let result = service.transfer(
            alice,
            bob,
            TokenType::GCC,
            Decimal::from(100),
            "Should fail".to_string(),
            None,
        ).await;

        assert!(result.is_err());
    }
}