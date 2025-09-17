use crate::types::*;
use crate::blockchain::Blockchain;
use anyhow::{Result, anyhow};
use std::collections::HashMap;
use rand::{Rng, thread_rng};
use std::sync::Arc;
use tokio::sync::RwLock;
use async_trait::async_trait;

#[derive(Debug, Clone)]
pub struct ConsensusConfig {
    pub min_stake: u128,
    pub block_time_ms: u64,
    pub epoch_length: u64,
    pub max_validators: usize,
    pub slashing_rate: f64,
}

impl Default for ConsensusConfig {
    fn default() -> Self {
        Self {
            min_stake: 100_000 * 10u128.pow(18), // 100k SPIRIT minimum
            block_time_ms: 6000,
            epoch_length: 100,
            max_validators: 100,
            slashing_rate: 0.1,
        }
    }
}

#[async_trait]
pub trait ConsensusEngine: Send + Sync {
    async fn select_validator(&self, validators: &HashMap<Address, ValidatorInfo>) -> Result<Address>;
    async fn validate_block(&self, block: &Block, chain: &Blockchain) -> Result<bool>;
    async fn finalize_block(&self, block: &Block, chain: &mut Blockchain) -> Result<()>;
}

pub struct ProofOfStake {
    config: ConsensusConfig,
    validator_performance: Arc<RwLock<HashMap<Address, ValidatorPerformance>>>,
}

#[derive(Debug, Clone, Default)]
struct ValidatorPerformance {
    blocks_proposed: u64,
    blocks_missed: u64,
    last_block_time: Option<chrono::DateTime<chrono::Utc>>,
    slash_count: u32,
}

impl ProofOfStake {
    pub fn new(config: ConsensusConfig) -> Self {
        Self {
            config,
            validator_performance: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    fn calculate_validator_weight(&self, validator: &ValidatorInfo, performance: &ValidatorPerformance) -> f64 {
        let base_weight = validator.staked_amount as f64;
        
        // Apply performance multiplier
        let total_blocks = performance.blocks_proposed + performance.blocks_missed;
        let success_rate = if total_blocks > 0 {
            performance.blocks_proposed as f64 / total_blocks as f64
        } else {
            1.0
        };
        
        // Apply slash penalty
        let slash_penalty = (1.0 - self.config.slashing_rate).powi(performance.slash_count as i32);
        
        base_weight * success_rate * slash_penalty
    }
}

#[async_trait]
impl ConsensusEngine for ProofOfStake {
    async fn select_validator(&self, validators: &HashMap<Address, ValidatorInfo>) -> Result<Address> {
        let active_validators: Vec<_> = validators
            .iter()
            .filter(|(_, v)| v.is_active && v.staked_amount >= self.config.min_stake)
            .collect();
        
        if active_validators.is_empty() {
            return Err(anyhow!("No active validators available"));
        }
        
        let performance = self.validator_performance.read().await;
        
        // Calculate weighted selection
        let mut weights = Vec::new();
        let mut total_weight = 0.0;
        
        for (address, validator) in &active_validators {
            let perf = performance.get(*address).cloned().unwrap_or_default();
            let weight = self.calculate_validator_weight(validator, &perf);
            weights.push((address.clone(), weight));
            total_weight += weight;
        }
        
        // Random selection based on stake weight  
        let mut rng = thread_rng();
        let selection: f64 = rng.r#gen::<f64>() * total_weight;
        
        let mut cumulative = 0.0;
        for (address, weight) in weights {
            cumulative += weight;
            if cumulative >= selection {
                return Ok(address.clone());
            }
        }
        
        // Fallback to the last validator
        Ok(active_validators.last().unwrap().0.clone())
    }
    
    async fn validate_block(&self, block: &Block, chain: &Blockchain) -> Result<bool> {
        // Verify validator is active
        let validator = chain.state.validators.get(&block.validator)
            .ok_or_else(|| anyhow!("Validator not found"))?;
        
        if !validator.is_active {
            return Ok(false);
        }
        
        if validator.staked_amount < self.config.min_stake {
            return Ok(false);
        }
        
        // Verify block timing
        if let Some(last_block) = chain.chain.last() {
            let time_diff = block.timestamp.signed_duration_since(last_block.timestamp);
            if time_diff.num_milliseconds() < self.config.block_time_ms as i64 {
                return Ok(false);
            }
        }
        
        // TODO: Verify validator signature
        
        Ok(true)
    }
    
    async fn finalize_block(&self, block: &Block, chain: &mut Blockchain) -> Result<()> {
        let mut performance = self.validator_performance.write().await;
        
        let validator_perf = performance.entry(block.validator.clone())
            .or_insert_with(ValidatorPerformance::default);
        
        validator_perf.blocks_proposed += 1;
        validator_perf.last_block_time = Some(block.timestamp);
        
        // Update epoch if needed
        if block.height % self.config.epoch_length == 0 {
            // Recalculate validator set for next epoch
            let mut validator_list: Vec<_> = chain.state.validators.iter()
                .filter(|(_, v)| v.staked_amount >= self.config.min_stake)
                .collect();
            
            validator_list.sort_by(|a, b| b.1.staked_amount.cmp(&a.1.staked_amount));
            
            // Keep only top validators
            validator_list.truncate(self.config.max_validators);
            
            // Collect addresses of active validators
            let active_addrs: std::collections::HashSet<_> = validator_list.iter().map(|(a, _)| (*a).clone()).collect();
            
            // Update active status
            for (addr, validator) in chain.state.validators.iter_mut() {
                validator.is_active = active_addrs.contains(addr);
            }
        }
        
        Ok(())
    }
}

pub struct ConsensusModule {
    engine: Arc<dyn ConsensusEngine>,
    blockchain: Arc<RwLock<Blockchain>>,
}

impl ConsensusModule {
    pub fn new(blockchain: Arc<RwLock<Blockchain>>, config: ConsensusConfig) -> Self {
        Self {
            engine: Arc::new(ProofOfStake::new(config)),
            blockchain,
        }
    }
    
    pub async fn produce_block(&self, validator_address: Address, validator_signature: Vec<u8>) -> Result<Block> {
        let mut chain = self.blockchain.write().await;
        
        // Validate this node can produce blocks
        let validator = chain.state.validators.get(&validator_address)
            .ok_or_else(|| anyhow!("Not a validator"))?;
        
        if !validator.is_active {
            return Err(anyhow!("Validator not active"));
        }
        
        // Create and validate block
        let block = chain.create_block(validator_address, validator_signature)?;
        
        if !self.engine.validate_block(&block, &chain).await? {
            return Err(anyhow!("Block validation failed"));
        }
        
        Ok(block)
    }
    
    pub async fn process_block(&self, block: Block) -> Result<()> {
        let mut chain = self.blockchain.write().await;
        
        if !self.engine.validate_block(&block, &chain).await? {
            return Err(anyhow!("Invalid block"));
        }
        
        chain.add_block(block.clone())?;
        self.engine.finalize_block(&block, &mut chain).await?;
        
        Ok(())
    }
    
    pub async fn get_next_validator(&self) -> Result<Address> {
        let chain = self.blockchain.read().await;
        self.engine.select_validator(&chain.state.validators).await
    }
}