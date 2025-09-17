use anyhow::{Result, anyhow};
use sled::{Db, Tree};
use crate::types::*;
use std::path::Path;
use std::collections::HashMap;

pub mod optimized;

pub struct Storage {
    db: Db,
    blocks: Tree,
    accounts: Tree,
    transactions: Tree,
    validators: Tree,
    metadata: Tree,
}

impl Storage {
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
        let db = sled::open(path)?;
        
        Ok(Self {
            blocks: db.open_tree("blocks")?,
            accounts: db.open_tree("accounts")?,
            transactions: db.open_tree("transactions")?,
            validators: db.open_tree("validators")?,
            metadata: db.open_tree("metadata")?,
            db,
        })
    }
    
    pub fn in_memory() -> Result<Self> {
        let db = sled::Config::new().temporary(true).open()?;
        
        Ok(Self {
            blocks: db.open_tree("blocks")?,
            accounts: db.open_tree("accounts")?,
            transactions: db.open_tree("transactions")?,
            validators: db.open_tree("validators")?,
            metadata: db.open_tree("metadata")?,
            db,
        })
    }
    
    // Block operations
    pub fn save_block(&self, block: &Block) -> Result<()> {
        let key = block.height.to_be_bytes();
        let value = bincode::serialize(block)?;
        self.blocks.insert(key, value)?;
        
        // Also save by hash for quick lookup
        let hash_key = format!("hash:{}", block.hash);
        self.blocks.insert(hash_key.as_bytes(), &key)?;
        
        // Update latest block height
        self.metadata.insert("latest_block", &key)?;
        
        Ok(())
    }
    
    pub fn get_block(&self, height: u64) -> Result<Option<Block>> {
        let key = height.to_be_bytes();
        match self.blocks.get(key)? {
            Some(data) => Ok(Some(bincode::deserialize(&data)?)),
            None => Ok(None),
        }
    }
    
    pub fn get_block_by_hash(&self, hash: &str) -> Result<Option<Block>> {
        let hash_key = format!("hash:{}", hash);
        match self.blocks.get(hash_key.as_bytes())? {
            Some(height_bytes) => {
                let height = u64::from_be_bytes(height_bytes.as_ref().try_into()?);
                self.get_block(height)
            }
            None => Ok(None),
        }
    }
    
    pub fn get_latest_block(&self) -> Result<Option<Block>> {
        match self.metadata.get("latest_block")? {
            Some(height_bytes) => {
                let height = u64::from_be_bytes(height_bytes.as_ref().try_into()?);
                self.get_block(height)
            }
            None => Ok(None),
        }
    }
    
    pub fn get_block_range(&self, start: u64, end: u64) -> Result<Vec<Block>> {
        let mut blocks = Vec::new();
        
        for height in start..=end {
            if let Some(block) = self.get_block(height)? {
                blocks.push(block);
            }
        }
        
        Ok(blocks)
    }
    
    // Account operations
    pub fn save_account(&self, account: &Account) -> Result<()> {
        let key = account.address.as_bytes();
        let value = bincode::serialize(account)?;
        self.accounts.insert(key, value)?;
        Ok(())
    }
    
    pub fn get_account(&self, address: &Address) -> Result<Option<Account>> {
        match self.accounts.get(address.as_bytes())? {
            Some(data) => Ok(Some(bincode::deserialize(&data)?)),
            None => Ok(None),
        }
    }
    
    pub fn update_account<F>(&self, address: &Address, update_fn: F) -> Result<()>
    where
        F: FnOnce(&mut Account),
    {
        let mut account = self.get_account(address)?
            .ok_or_else(|| anyhow!("Account not found"))?;
        
        update_fn(&mut account);
        self.save_account(&account)?;
        Ok(())
    }
    
    // Transaction operations
    pub fn save_transaction(&self, tx: &Transaction) -> Result<()> {
        let key = tx.id.as_bytes();
        let value = bincode::serialize(tx)?;
        self.transactions.insert(key, value)?;
        
        // Index by block height if included in a block
        // This would be called when a transaction is included in a block
        
        Ok(())
    }
    
    pub fn get_transaction(&self, tx_id: &uuid::Uuid) -> Result<Option<Transaction>> {
        match self.transactions.get(tx_id.as_bytes())? {
            Some(data) => Ok(Some(bincode::deserialize(&data)?)),
            None => Ok(None),
        }
    }
    
    // Validator operations
    pub fn save_validator(&self, validator: &ValidatorInfo) -> Result<()> {
        let key = validator.address.as_bytes();
        let value = bincode::serialize(validator)?;
        self.validators.insert(key, value)?;
        Ok(())
    }
    
    pub fn get_validator(&self, address: &Address) -> Result<Option<ValidatorInfo>> {
        match self.validators.get(address.as_bytes())? {
            Some(data) => Ok(Some(bincode::deserialize(&data)?)),
            None => Ok(None),
        }
    }
    
    pub fn get_all_validators(&self) -> Result<Vec<ValidatorInfo>> {
        let mut validators = Vec::new();
        
        for item in self.validators.iter() {
            let (_, value) = item?;
            let validator: ValidatorInfo = bincode::deserialize(&value)?;
            validators.push(validator);
        }
        
        Ok(validators)
    }
    
    // State management
    pub fn save_chain_state(&self, state: &ChainState) -> Result<()> {
        // Save all accounts
        for (_, account) in &state.accounts {
            self.save_account(account)?;
        }
        
        // Save all validators
        for (_, validator) in &state.validators {
            self.save_validator(validator)?;
        }
        
        // Save metadata
        self.metadata.insert("current_epoch", &state.current_epoch.to_be_bytes())?;
        
        // Save total supply
        let supply_data = bincode::serialize(&state.total_supply)?;
        self.metadata.insert("total_supply", supply_data)?;
        
        Ok(())
    }
    
    pub fn load_chain_state(&self) -> Result<ChainState> {
        let mut state = ChainState {
            accounts: HashMap::new(),
            total_supply: HashMap::new(),
            validators: HashMap::new(),
            current_epoch: 0,
            contracts: HashMap::new(),
            domains: HashMap::new(),
        };
        
        // Load all accounts
        for item in self.accounts.iter() {
            let (key, value) = item?;
            let address = String::from_utf8(key.to_vec())?;
            let account: Account = bincode::deserialize(&value)?;
            state.accounts.insert(address, account);
        }
        
        // Load all validators
        for item in self.validators.iter() {
            let (key, value) = item?;
            let address = String::from_utf8(key.to_vec())?;
            let validator: ValidatorInfo = bincode::deserialize(&value)?;
            state.validators.insert(address, validator);
        }
        
        // Load metadata
        if let Some(epoch_bytes) = self.metadata.get("current_epoch")? {
            state.current_epoch = u64::from_be_bytes(epoch_bytes.as_ref().try_into()?);
        }
        
        if let Some(supply_data) = self.metadata.get("total_supply")? {
            state.total_supply = bincode::deserialize(&supply_data)?;
        }
        
        Ok(state)
    }
    
    // Utility methods
    pub fn flush(&self) -> Result<()> {
        self.db.flush()?;
        Ok(())
    }
    
    pub fn checkpoint(&self) -> Result<()> {
        // Create a checkpoint of the current state
        // This could be used for snapshots or backups
        self.flush()?;
        Ok(())
    }
    
    pub fn get_db_size(&self) -> Result<u64> {
        Ok(self.db.size_on_disk()?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    
    #[test]
    fn test_block_storage() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let storage = Storage::new(temp_dir.path())?;
        
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
        
        storage.save_block(&block)?;
        
        let loaded = storage.get_block(1)?;
        assert!(loaded.is_some());
        assert_eq!(loaded.unwrap().hash, "test_hash");
        
        let by_hash = storage.get_block_by_hash("test_hash")?;
        assert!(by_hash.is_some());
        
        Ok(())
    }
    
    #[test]
    fn test_account_storage() -> Result<()> {
        let storage = Storage::in_memory()?;
        
        let mut account = Account {
            address: "test_address".to_string(),
            public_key: vec![1, 2, 3],
            balances: HashMap::new(),
            nonce: 0,
            soul_id: None,
            staked_amount: 0,
            mana_earned: 0,
        };
        
        account.balances.insert(TokenType::Spirit, 1000);
        
        storage.save_account(&account)?;
        
        let loaded = storage.get_account(&"test_address".to_string())?;
        assert!(loaded.is_some());
        assert_eq!(loaded.unwrap().balances.get(&TokenType::Spirit), Some(&1000));
        
        Ok(())
    }
}