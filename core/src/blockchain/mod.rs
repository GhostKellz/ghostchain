pub mod integration;
pub mod local_testnet;

use crate::types::*;
use crate::crypto::hash_data;
use anyhow::{Result, anyhow};
use chrono::Utc;
use std::collections::HashMap;
use serde_json;

#[derive(Debug, Clone)]
pub struct Blockchain {
    pub chain: Vec<Block>,
    pub state: ChainState,
    pub pending_transactions: Vec<Transaction>,
    pub config: GenesisConfig,
}

impl Blockchain {
    pub fn new(config: GenesisConfig) -> Result<Self> {
        let mut state = ChainState {
            accounts: HashMap::new(),
            total_supply: config.initial_supply.clone(),
            validators: HashMap::new(),
            current_epoch: 0,
            contracts: HashMap::new(),
            domains: HashMap::new(),
        };
        
        for (address, account) in &config.genesis_accounts {
            state.accounts.insert(address.clone(), account.clone());
        }
        
        for validator in &config.initial_validators {
            state.validators.insert(validator.address.clone(), validator.clone());
        }
        
        let genesis_block = Block {
            height: 0,
            hash: "0x0".to_string(),
            previous_hash: "0x0".to_string(),
            timestamp: Utc::now(),
            transactions: Vec::new(),
            validator: "ghost_genesis".to_string(),
            state_root: hash_data(b"genesis_state"),
            signature: vec![0; 64],
        };
        
        let genesis_block = Self::compute_block_hash(genesis_block);
        
        Ok(Self {
            chain: vec![genesis_block],
            state,
            pending_transactions: Vec::new(),
            config,
        })
    }
    
    pub fn add_transaction(&mut self, tx: Transaction) -> Result<()> {
        self.validate_transaction(&tx)?;
        self.pending_transactions.push(tx);
        Ok(())
    }
    
    pub fn create_block(&mut self, validator: Address, validator_signature: Vec<u8>) -> Result<Block> {
        let previous_block = self.chain.last().ok_or_else(|| anyhow!("No blocks in chain"))?;
        
        let mut block = Block {
            height: previous_block.height + 1,
            hash: String::new(),
            previous_hash: previous_block.hash.clone(),
            timestamp: Utc::now(),
            transactions: self.pending_transactions.drain(..).collect(),
            validator,
            state_root: self.calculate_state_root(),
            signature: validator_signature,
        };
        
        for tx in &block.transactions {
            self.apply_transaction(tx)?;
        }
        
        block = Self::compute_block_hash(block);
        
        if block.height % self.config.epoch_length == 0 {
            self.state.current_epoch += 1;
        }
        
        Ok(block)
    }
    
    pub fn add_block(&mut self, block: Block) -> Result<()> {
        self.validate_block(&block)?;
        self.chain.push(block);
        Ok(())
    }
    
    fn validate_transaction(&self, tx: &Transaction) -> Result<()> {
        match &tx.tx_type {
            TransactionType::Transfer { from, to: _, token, amount } => {
                let account = self.state.accounts.get(from)
                    .ok_or_else(|| anyhow!("Account not found: {}", from))?;
                
                let balance = account.balances.get(token).unwrap_or(&0);
                if balance < amount {
                    return Err(anyhow!("Insufficient balance"));
                }
                
                if token == &TokenType::Soul {
                    return Err(anyhow!("Soul tokens are non-transferable"));
                }
            }
            TransactionType::Stake { staker, amount } => {
                let account = self.state.accounts.get(staker)
                    .ok_or_else(|| anyhow!("Account not found: {}", staker))?;
                
                let spirit_balance = account.balances.get(&TokenType::Spirit).unwrap_or(&0);
                if spirit_balance < amount {
                    return Err(anyhow!("Insufficient SPIRIT balance for staking"));
                }
            }
            _ => {}
        }
        
        Ok(())
    }
    
    fn apply_transaction(&mut self, tx: &Transaction) -> Result<()> {
        match &tx.tx_type {
            TransactionType::Transfer { from, to, token, amount } => {
                let from_account = self.state.accounts.get_mut(from)
                    .ok_or_else(|| anyhow!("Account not found"))?;
                
                *from_account.balances.get_mut(token).unwrap() -= amount;
                
                let to_account = self.state.accounts.entry(to.clone())
                    .or_insert_with(|| Account {
                        address: to.clone(),
                        public_key: vec![],
                        balances: HashMap::new(),
                        nonce: 0,
                        soul_id: None,
                        staked_amount: 0,
                        mana_earned: 0,
                    });
                
                *to_account.balances.entry(*token).or_insert(0) += amount;
            }
            TransactionType::CreateAccount { address, public_key } => {
                self.state.accounts.insert(address.clone(), Account {
                    address: address.clone(),
                    public_key: public_key.clone(),
                    balances: HashMap::new(),
                    nonce: 0,
                    soul_id: None,
                    staked_amount: 0,
                    mana_earned: 0,
                });
            }
            TransactionType::Stake { staker, amount } => {
                let account = self.state.accounts.get_mut(staker).unwrap();
                *account.balances.get_mut(&TokenType::Spirit).unwrap() -= amount;
                account.staked_amount += amount;
                
                let validator = self.state.validators.entry(staker.clone())
                    .or_insert_with(|| ValidatorInfo {
                        address: staker.clone(),
                        staked_amount: 0,
                        is_active: false,
                        commission_rate: 0.1,
                        delegators: HashMap::new(),
                    });
                
                validator.staked_amount += amount;
                if validator.staked_amount >= 100_000 * 10u128.pow(18) {
                    validator.is_active = true;
                }
            }
            TransactionType::MintSoul { recipient, soul_id, .. } => {
                let account = self.state.accounts.get_mut(recipient).unwrap();
                account.soul_id = Some(*soul_id);
                *account.balances.entry(TokenType::Soul).or_insert(0) += 1;
            }
            TransactionType::ContributeProof { contributor, mana_reward, .. } => {
                let account = self.state.accounts.get_mut(contributor).unwrap();
                account.mana_earned += mana_reward;
                *account.balances.entry(TokenType::Mana).or_insert(0) += mana_reward;
            }
            TransactionType::DeployContract { deployer, contract_code, init_data } => {
                // Contract deployment will be handled by the contract executor
                // For now, just log that a contract was deployed
                println!("Contract deployed by {}: {} bytes", deployer, contract_code.len());
            }
            TransactionType::CallContract { caller, contract_id, method, data } => {
                // Contract calls will be handled by the contract executor
                // For now, just log the contract call
                println!("Contract call from {} to {}: {}", caller, contract_id, method);
            }
            _ => {}
        }
        
        Ok(())
    }
    
    fn validate_block(&self, block: &Block) -> Result<()> {
        let previous = self.chain.last().ok_or_else(|| anyhow!("No previous block"))?;
        
        if block.height != previous.height + 1 {
            return Err(anyhow!("Invalid block height"));
        }
        
        if block.previous_hash != previous.hash {
            return Err(anyhow!("Invalid previous hash"));
        }
        
        Ok(())
    }
    
    fn compute_block_hash(mut block: Block) -> Block {
        let block_data = serde_json::to_vec(&(&block.height, &block.previous_hash, &block.timestamp, &block.transactions)).unwrap();
        block.hash = hash_data(&block_data);
        block
    }
    
    fn calculate_state_root(&self) -> String {
        let state_data = serde_json::to_vec(&self.state).unwrap();
        hash_data(&state_data)
    }
    
    pub fn get_balance(&self, address: &Address, token: &TokenType) -> u128 {
        self.state.accounts
            .get(address)
            .and_then(|acc| acc.balances.get(token))
            .copied()
            .unwrap_or(0)
    }
    
    pub fn get_account(&self, address: &Address) -> Option<&Account> {
        self.state.accounts.get(address)
    }

    /// Initialize blockchain with genesis configuration (needed for local testnet)
    pub async fn initialize_with_genesis(&mut self, genesis_config: &GenesisConfig) -> Result<()> {
        // Update config
        self.config = genesis_config.clone();
        
        // Update state with genesis accounts
        for (address, account) in &genesis_config.genesis_accounts {
            self.state.accounts.insert(address.clone(), account.clone());
        }
        
        // Update total supply
        self.state.total_supply = genesis_config.initial_supply.clone();
        
        // Set up initial validators
        for validator in &genesis_config.initial_validators {
            self.state.validators.insert(validator.address.clone(), validator.clone());
        }
        
        Ok(())
    }

    /// Create a new blockchain with default genesis (for compatibility)
    pub fn new_default() -> Self {
        let default_config = GenesisConfig::default();
        Self::new(default_config).unwrap_or_else(|_| {
            // Fallback if genesis creation fails
            Self {
                chain: Vec::new(),
                state: ChainState {
                    accounts: HashMap::new(),
                    total_supply: HashMap::new(),
                    validators: HashMap::new(),
                    current_epoch: 0,
                    contracts: HashMap::new(),
                    domains: HashMap::new(),
                },
                pending_transactions: Vec::new(),
                config: default_config.clone(),
            }
        })
    }
}