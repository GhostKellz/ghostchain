use anyhow::{Result, anyhow};
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::RwLock;
use crate::storage::Storage;
use crate::performance::{PerformanceManager, MultiLevelCache, BatchOperation};
use crate::types::*;
use tracing::{debug, info, warn};

/// High-performance storage wrapper with caching and batch operations
pub struct OptimizedStorage {
    inner: Arc<Storage>,
    cache: Arc<RwLock<MultiLevelCache>>,
    performance_manager: Arc<PerformanceManager>,
    write_buffer: Arc<RwLock<Vec<BatchOperation>>>,
}

impl OptimizedStorage {
    pub fn new(storage: Storage, performance_manager: Arc<PerformanceManager>) -> Self {
        Self {
            inner: Arc::new(storage),
            cache: Arc::new(RwLock::new(MultiLevelCache::new())),
            performance_manager,
            write_buffer: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Get block with caching
    pub async fn get_block(&self, height: u64) -> Result<Option<Block>> {
        let start_time = Instant::now();
        
        // Check cache first
        let cache_key = format!("block:{}", height);
        if let Some(cached_block) = self.performance_manager.get_cached::<String, Block>(cache_key.clone()).await {
            self.performance_manager.record_operation("get_block_cached", start_time.elapsed()).await;
            return Ok(Some(cached_block));
        }

        // Fetch from storage
        let block = self.inner.get_block(height)?;
        
        // Cache the result if found
        if let Some(ref block) = block {
            let _ = self.performance_manager.set_cached(cache_key, block.clone()).await;
        }

        self.performance_manager.record_operation("get_block_storage", start_time.elapsed()).await;
        Ok(block)
    }

    /// Get block by hash with caching
    pub async fn get_block_by_hash(&self, hash: &str) -> Result<Option<Block>> {
        let start_time = Instant::now();
        
        // Check cache first
        let cache_key = format!("block_hash:{}", hash);
        if let Some(cached_block) = self.performance_manager.get_cached::<String, Block>(cache_key.clone()).await {
            self.performance_manager.record_operation("get_block_by_hash_cached", start_time.elapsed()).await;
            return Ok(Some(cached_block));
        }

        // Fetch from storage
        let block = self.inner.get_block_by_hash(hash)?;
        
        // Cache the result if found
        if let Some(ref block) = block {
            let _ = self.performance_manager.set_cached(cache_key, block.clone()).await;
            // Also cache by height
            let height_key = format!("block:{}", block.height);
            let _ = self.performance_manager.set_cached(height_key, block.clone()).await;
        }

        self.performance_manager.record_operation("get_block_by_hash_storage", start_time.elapsed()).await;
        Ok(block)
    }

    /// Get block range with optimized fetching
    pub async fn get_block_range(&self, start: u64, end: u64) -> Result<Vec<Block>> {
        let start_time = Instant::now();
        let mut blocks = Vec::new();
        let mut missing_heights = Vec::new();

        // Check cache for each block
        for height in start..=end {
            let cache_key = format!("block:{}", height);
            if let Some(cached_block) = self.performance_manager.get_cached::<String, Block>(cache_key).await {
                blocks.push((height, cached_block));
            } else {
                missing_heights.push(height);
            }
        }

        // Fetch missing blocks from storage
        if !missing_heights.is_empty() {
            let storage_blocks = self.inner.get_block_range(
                *missing_heights.first().unwrap(),
                *missing_heights.last().unwrap(),
            )?;

            for block in storage_blocks {
                // Cache the block
                let cache_key = format!("block:{}", block.height);
                let _ = self.performance_manager.set_cached(cache_key, block.clone()).await;
                blocks.push((block.height, block));
            }
        }

        // Sort blocks by height
        blocks.sort_by_key(|(height, _)| *height);
        let result = blocks.into_iter().map(|(_, block)| block).collect();

        self.performance_manager.record_operation("get_block_range", start_time.elapsed()).await;
        Ok(result)
    }

    /// Save block with batching
    pub async fn save_block(&self, block: &Block) -> Result<()> {
        let start_time = Instant::now();
        
        // Add to write buffer for batch processing
        let data = bincode::serialize(block)?;
        let operation = BatchOperation::BlockInsert {
            height: block.height,
            data,
        };
        
        self.performance_manager.batch_process(vec![operation]).await?;
        
        // Also save immediately for critical operations
        self.inner.save_block(block)?;
        
        // Cache the block
        let cache_key = format!("block:{}", block.height);
        let _ = self.performance_manager.set_cached(cache_key, block.clone()).await;
        
        let hash_key = format!("block_hash:{}", block.hash);
        let _ = self.performance_manager.set_cached(hash_key, block.clone()).await;

        self.performance_manager.record_operation("save_block", start_time.elapsed()).await;
        Ok(())
    }

    /// Get account with caching
    pub async fn get_account(&self, address: &Address) -> Result<Option<Account>> {
        let start_time = Instant::now();
        
        // Check cache first
        let cache_key = format!("account:{}", address);
        if let Some(cached_account) = self.performance_manager.get_cached::<String, Account>(cache_key.clone()).await {
            self.performance_manager.record_operation("get_account_cached", start_time.elapsed()).await;
            return Ok(Some(cached_account));
        }

        // Fetch from storage
        let account = self.inner.get_account(address)?;
        
        // Cache the result if found
        if let Some(ref account) = account {
            let _ = self.performance_manager.set_cached(cache_key, account.clone()).await;
        }

        self.performance_manager.record_operation("get_account_storage", start_time.elapsed()).await;
        Ok(account)
    }

    /// Save account with batching
    pub async fn save_account(&self, account: &Account) -> Result<()> {
        let start_time = Instant::now();
        
        // Add to write buffer for batch processing
        let data = bincode::serialize(account)?;
        let operation = BatchOperation::AccountUpdate {
            address: account.address.clone(),
            data,
        };
        
        self.performance_manager.batch_process(vec![operation]).await?;
        
        // Also save immediately for critical operations
        self.inner.save_account(account)?;
        
        // Cache the account
        let cache_key = format!("account:{}", account.address);
        let _ = self.performance_manager.set_cached(cache_key, account.clone()).await;

        self.performance_manager.record_operation("save_account", start_time.elapsed()).await;
        Ok(())
    }

    /// Update account with optimized caching
    pub async fn update_account<F>(&self, address: &Address, update_fn: F) -> Result<()>
    where
        F: FnOnce(&mut Account) + Send + 'static,
    {
        let start_time = Instant::now();
        
        // Get account (with caching)
        let mut account = self.get_account(address).await?
            .ok_or_else(|| anyhow!("Account not found"))?;
        
        // Apply update
        update_fn(&mut account);
        
        // Save updated account
        self.save_account(&account).await?;

        self.performance_manager.record_operation("update_account", start_time.elapsed()).await;
        Ok(())
    }

    /// Get transaction with caching
    pub async fn get_transaction(&self, tx_id: &uuid::Uuid) -> Result<Option<Transaction>> {
        let start_time = Instant::now();
        
        // Check cache first
        let cache_key = format!("tx:{}", tx_id);
        if let Some(cached_tx) = self.performance_manager.get_cached::<String, Transaction>(cache_key.clone()).await {
            self.performance_manager.record_operation("get_transaction_cached", start_time.elapsed()).await;
            return Ok(Some(cached_tx));
        }

        // Fetch from storage
        let transaction = self.inner.get_transaction(tx_id)?;
        
        // Cache the result if found
        if let Some(ref tx) = transaction {
            let _ = self.performance_manager.set_cached(cache_key, tx.clone()).await;
        }

        self.performance_manager.record_operation("get_transaction_storage", start_time.elapsed()).await;
        Ok(transaction)
    }

    /// Save transaction with batching
    pub async fn save_transaction(&self, tx: &Transaction) -> Result<()> {
        let start_time = Instant::now();
        
        // Add to write buffer for batch processing
        let data = bincode::serialize(tx)?;
        let operation = BatchOperation::TransactionStore {
            tx_id: tx.id.to_string(),
            data,
        };
        
        self.performance_manager.batch_process(vec![operation]).await?;
        
        // Also save immediately for critical operations
        self.inner.save_transaction(tx)?;
        
        // Cache the transaction
        let cache_key = format!("tx:{}", tx.id);
        let _ = self.performance_manager.set_cached(cache_key, tx.clone()).await;

        self.performance_manager.record_operation("save_transaction", start_time.elapsed()).await;
        Ok(())
    }

    /// Batch save multiple transactions
    pub async fn save_transactions(&self, transactions: &[Transaction]) -> Result<()> {
        let start_time = Instant::now();
        
        // Create batch operations
        let mut batch_ops = Vec::new();
        for tx in transactions {
            let data = bincode::serialize(tx)?;
            batch_ops.push(BatchOperation::TransactionStore {
                tx_id: tx.id.to_string(),
                data,
            });
        }
        
        // Process batch
        self.performance_manager.batch_process(batch_ops).await?;
        
        // Save to storage and cache
        for tx in transactions {
            self.inner.save_transaction(tx)?;
            let cache_key = format!("tx:{}", tx.id);
            let _ = self.performance_manager.set_cached(cache_key, tx.clone()).await;
        }

        self.performance_manager.record_operation("save_transactions_batch", start_time.elapsed()).await;
        Ok(())
    }

    /// Get validator with caching
    pub async fn get_validator(&self, address: &Address) -> Result<Option<ValidatorInfo>> {
        let start_time = Instant::now();
        
        // Check cache first
        let cache_key = format!("validator:{}", address);
        if let Some(cached_validator) = self.performance_manager.get_cached::<String, ValidatorInfo>(cache_key.clone()).await {
            self.performance_manager.record_operation("get_validator_cached", start_time.elapsed()).await;
            return Ok(Some(cached_validator));
        }

        // Fetch from storage
        let validator = self.inner.get_validator(address)?;
        
        // Cache the result if found
        if let Some(ref validator) = validator {
            let _ = self.performance_manager.set_cached(cache_key, validator.clone()).await;
        }

        self.performance_manager.record_operation("get_validator_storage", start_time.elapsed()).await;
        Ok(validator)
    }

    /// Save validator with caching
    pub async fn save_validator(&self, validator: &ValidatorInfo) -> Result<()> {
        let start_time = Instant::now();
        
        // Save to storage
        self.inner.save_validator(validator)?;
        
        // Cache the validator
        let cache_key = format!("validator:{}", validator.address);
        let _ = self.performance_manager.set_cached(cache_key, validator.clone()).await;

        self.performance_manager.record_operation("save_validator", start_time.elapsed()).await;
        Ok(())
    }

    /// Get all validators with caching
    pub async fn get_all_validators(&self) -> Result<Vec<ValidatorInfo>> {
        let start_time = Instant::now();
        
        // Check cache first
        let cache_key = "all_validators".to_string();
        if let Some(cached_validators) = self.performance_manager.get_cached::<String, Vec<ValidatorInfo>>(cache_key.clone()).await {
            self.performance_manager.record_operation("get_all_validators_cached", start_time.elapsed()).await;
            return Ok(cached_validators);
        }

        // Fetch from storage
        let validators = self.inner.get_all_validators()?;
        
        // Cache the result
        let _ = self.performance_manager.set_cached(cache_key, validators.clone()).await;

        self.performance_manager.record_operation("get_all_validators_storage", start_time.elapsed()).await;
        Ok(validators)
    }

    /// Save chain state with optimized operations
    pub async fn save_chain_state(&self, state: &ChainState) -> Result<()> {
        let start_time = Instant::now();
        
        // Save to storage
        self.inner.save_chain_state(state)?;
        
        // Cache accounts
        for (address, account) in &state.accounts {
            let cache_key = format!("account:{}", address);
            let _ = self.performance_manager.set_cached(cache_key, account.clone()).await;
        }
        
        // Cache validators
        for (address, validator) in &state.validators {
            let cache_key = format!("validator:{}", address);
            let _ = self.performance_manager.set_cached(cache_key, validator.clone()).await;
        }
        
        // Cache validator list
        let validators: Vec<ValidatorInfo> = state.validators.values().cloned().collect();
        let _ = self.performance_manager.set_cached("all_validators".to_string(), validators).await;

        self.performance_manager.record_operation("save_chain_state", start_time.elapsed()).await;
        Ok(())
    }

    /// Load chain state with caching
    pub async fn load_chain_state(&self) -> Result<ChainState> {
        let start_time = Instant::now();
        
        // Load from storage
        let state = self.inner.load_chain_state()?;
        
        // Pre-populate cache
        for (address, account) in &state.accounts {
            let cache_key = format!("account:{}", address);
            let _ = self.performance_manager.set_cached(cache_key, account.clone()).await;
        }
        
        for (address, validator) in &state.validators {
            let cache_key = format!("validator:{}", address);
            let _ = self.performance_manager.set_cached(cache_key, validator.clone()).await;
        }

        self.performance_manager.record_operation("load_chain_state", start_time.elapsed()).await;
        Ok(state)
    }

    /// Flush all pending operations
    pub async fn flush(&self) -> Result<()> {
        let start_time = Instant::now();
        
        // Flush storage
        self.inner.flush()?;
        
        // Optimize performance manager
        self.performance_manager.optimize_storage().await?;

        self.performance_manager.record_operation("flush", start_time.elapsed()).await;
        Ok(())
    }

    /// Create checkpoint with optimization
    pub async fn checkpoint(&self) -> Result<()> {
        let start_time = Instant::now();
        
        // Create storage checkpoint
        self.inner.checkpoint()?;
        
        // Optimize caches
        self.performance_manager.optimize_storage().await?;

        self.performance_manager.record_operation("checkpoint", start_time.elapsed()).await;
        Ok(())
    }

    /// Get storage statistics
    pub async fn get_stats(&self) -> Result<StorageStats> {
        let db_size = self.inner.get_db_size()?;
        let health = self.performance_manager.health_check().await;
        
        Ok(StorageStats {
            db_size_bytes: db_size,
            cache_hit_rate: health.cache_hit_rate,
            active_connections: health.active_connections,
            pending_operations: health.pending_batches,
            estimated_cache_size: health.memory_usage,
        })
    }

    /// Optimize storage performance
    pub async fn optimize(&self) -> Result<()> {
        let start_time = Instant::now();
        
        info!("Starting storage optimization...");
        
        // Flush pending operations
        self.flush().await?;
        
        // Optimize performance manager
        self.performance_manager.optimize_storage().await?;
        
        // Clear old cache entries
        let mut cache = self.cache.write().await;
        cache.cleanup_all();
        
        info!("Storage optimization completed in {:?}", start_time.elapsed());
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct StorageStats {
    pub db_size_bytes: u64,
    pub cache_hit_rate: f64,
    pub active_connections: usize,
    pub pending_operations: usize,
    pub estimated_cache_size: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::performance::PerformanceConfig;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_optimized_storage_block_operations() {
        let temp_dir = TempDir::new().unwrap();
        let storage = Storage::new(temp_dir.path()).unwrap();
        let performance_config = PerformanceConfig::default();
        let performance_manager = Arc::new(PerformanceManager::new(performance_config).unwrap());
        
        let optimized = OptimizedStorage::new(storage, performance_manager);
        
        let block = Block {
            height: 1,
            hash: "test_hash".to_string(),
            previous_hash: "prev_hash".to_string(),
            timestamp: chrono::Utc::now(),
            transactions: vec![],
            validator: "validator1".to_string(),
            state_root: "state_root".to_string(),
            signature: vec![0; 64],
        };
        
        // Save block
        optimized.save_block(&block).await.unwrap();
        
        // Get block (should be cached)
        let retrieved = optimized.get_block(1).await.unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().hash, "test_hash");
        
        // Get by hash (should be cached)
        let by_hash = optimized.get_block_by_hash("test_hash").await.unwrap();
        assert!(by_hash.is_some());
    }

    #[tokio::test]
    async fn test_optimized_storage_account_operations() {
        let temp_dir = TempDir::new().unwrap();
        let storage = Storage::new(temp_dir.path()).unwrap();
        let performance_config = PerformanceConfig::default();
        let performance_manager = Arc::new(PerformanceManager::new(performance_config).unwrap());
        
        let optimized = OptimizedStorage::new(storage, performance_manager);
        
        let account = Account {
            address: "test_address".to_string(),
            public_key: vec![1, 2, 3],
            balances: std::collections::HashMap::new(),
            nonce: 0,
            soul_id: None,
            staked_amount: 0,
            mana_earned: 0,
        };
        
        // Save account
        optimized.save_account(&account).await.unwrap();
        
        // Get account (should be cached)
        let retrieved = optimized.get_account(&"test_address".to_string()).await.unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().address, "test_address");
        
        // Update account
        optimized.update_account(&"test_address".to_string(), |acc| {
            acc.nonce = 1;
        }).await.unwrap();
        
        // Verify update
        let updated = optimized.get_account(&"test_address".to_string()).await.unwrap();
        assert_eq!(updated.unwrap().nonce, 1);
    }

    #[tokio::test]
    async fn test_optimized_storage_stats() {
        let temp_dir = TempDir::new().unwrap();
        let storage = Storage::new(temp_dir.path()).unwrap();
        let performance_config = PerformanceConfig::default();
        let performance_manager = Arc::new(PerformanceManager::new(performance_config).unwrap());
        
        let optimized = OptimizedStorage::new(storage, performance_manager);
        
        let stats = optimized.get_stats().await.unwrap();
        assert!(stats.db_size_bytes >= 0);
        assert!(stats.cache_hit_rate >= 0.0);
    }
}