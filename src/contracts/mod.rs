use crate::types::*;
use anyhow::{Result, anyhow};
use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use async_trait::async_trait;
use chrono::Utc;

pub mod gas;
pub mod native;
pub mod storage;
pub mod deployment;
pub mod ghostchain_tokens;

use gas::GasMeter;
use storage::ContractStorage;

#[derive(Debug, Clone)]
pub struct ExecutionContext {
    pub caller: Address,
    pub contract_id: ContractId,
    pub block_height: u64,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub gas_limit: u128,
    pub chain_state: ChainState,
}

#[async_trait]
pub trait Contract: Send + Sync {
    async fn init(&mut self, ctx: &ExecutionContext, init_data: &[u8]) -> Result<ContractResult>;
    async fn call(&mut self, ctx: &ExecutionContext, method: &str, data: &[u8]) -> Result<ContractResult>;
    async fn query(&self, ctx: &ExecutionContext, query: &str, data: &[u8]) -> Result<Vec<u8>>;
    fn get_abi(&self) -> ContractABI;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractABI {
    pub methods: Vec<MethodSignature>,
    pub events: Vec<EventSignature>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MethodSignature {
    pub name: String,
    pub inputs: Vec<ParameterType>,
    pub outputs: Vec<ParameterType>,
    pub gas_cost: u128,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventSignature {
    pub name: String,
    pub inputs: Vec<ParameterType>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ParameterType {
    String,
    Address,
    U128,
    U64,
    Bytes,
    Bool,
}

pub struct ContractExecutor {
    storage: ContractStorage,
    gas_meter: GasMeter,
    native_contracts: HashMap<String, Box<dyn Contract>>,
}

impl std::fmt::Debug for ContractExecutor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ContractExecutor")
            .field("storage", &self.storage)
            .field("gas_meter", &self.gas_meter)
            .field("native_contracts", &format_args!("{} contracts", self.native_contracts.len()))
            .finish()
    }
}

impl ContractExecutor {
    pub fn new(storage: ContractStorage) -> Self {
        let mut executor = Self {
            storage,
            gas_meter: GasMeter::new(),
            native_contracts: HashMap::new(),
        };
        
        // Register system contracts
        executor.register_system_contracts();
        executor
    }
    
    fn register_system_contracts(&mut self) {
        // Register the domain registry contract
        self.native_contracts.insert(
            "system.domain_registry".to_string(),
            Box::new(native::DomainRegistryContract::new())
        );
        
        // Register token contract
        self.native_contracts.insert(
            "system.token_manager".to_string(),
            Box::new(native::TokenManagerContract::new())
        );
    }
    
    pub async fn deploy_contract(
        &mut self,
        deployer: &Address,
        contract_code: &[u8],
        init_data: &[u8],
        gas_limit: u128,
        chain_state: &mut ChainState,
    ) -> Result<ContractResult> {
        let contract_id = self.generate_contract_id(deployer, contract_code);
        
        // Check if contract already exists
        if chain_state.contracts.contains_key(&contract_id) {
            return Err(anyhow!("Contract already exists"));
        }
        
        let ctx = ExecutionContext {
            caller: deployer.clone(),
            contract_id: contract_id.clone(),
            block_height: 0, // TODO: get from current block
            timestamp: Utc::now(),
            gas_limit,
            chain_state: chain_state.clone(),
        };
        
        // Determine contract type
        let contract_type = if self.is_native_contract(&contract_code) {
            ContractType::Native
        } else {
            ContractType::Wasm
        };
        
        // Create contract info
        let contract_info = ContractInfo {
            id: contract_id.clone(),
            deployer: deployer.clone(),
            code: contract_code.to_vec(),
            state: HashMap::new(),
            created_at: Utc::now(),
            gas_used: 0,
            contract_type: contract_type.clone(),
        };
        
        // Add to chain state
        chain_state.contracts.insert(contract_id.clone(), contract_info);
        
        // Initialize contract
        match contract_type {
            ContractType::Native => {
                if let Some(contract) = self.native_contracts.get_mut(&contract_id) {
                    contract.init(&ctx, init_data).await
                } else {
                    Err(anyhow!("Native contract not found: {}", contract_id))
                }
            }
            ContractType::Wasm => {
                // TODO: Implement WASM contract initialization
                Err(anyhow!("WASM contracts not yet implemented"))
            }
        }
    }
    
    pub async fn call_contract(
        &mut self,
        caller: &Address,
        contract_id: &ContractId,
        method: &str,
        data: &[u8],
        gas_limit: u128,
        chain_state: &mut ChainState,
    ) -> Result<ContractResult> {
        // Get contract info
        let contract_info = chain_state.contracts.get(contract_id)
            .ok_or_else(|| anyhow!("Contract not found: {}", contract_id))?
            .clone();
        
        let ctx = ExecutionContext {
            caller: caller.clone(),
            contract_id: contract_id.clone(),
            block_height: 0, // TODO: get from current block
            timestamp: Utc::now(),
            gas_limit,
            chain_state: chain_state.clone(),
        };
        
        match contract_info.contract_type {
            ContractType::Native => {
                if let Some(contract) = self.native_contracts.get_mut(contract_id) {
                    let result = contract.call(&ctx, method, data).await?;
                    
                    // Update contract state in chain state
                    if let Some(contract_info) = chain_state.contracts.get_mut(contract_id) {
                        contract_info.gas_used += result.gas_used;
                    }
                    
                    Ok(result)
                } else {
                    Err(anyhow!("Native contract not found: {}", contract_id))
                }
            }
            ContractType::Wasm => {
                // TODO: Implement WASM contract execution
                Err(anyhow!("WASM contracts not yet implemented"))
            }
        }
    }
    
    pub async fn query_contract(
        &self,
        contract_id: &ContractId,
        query: &str,
        data: &[u8],
        chain_state: &ChainState,
    ) -> Result<Vec<u8>> {
        let contract_info = chain_state.contracts.get(contract_id)
            .ok_or_else(|| anyhow!("Contract not found: {}", contract_id))?;
        
        let ctx = ExecutionContext {
            caller: "system".to_string(),
            contract_id: contract_id.clone(),
            block_height: 0,
            timestamp: Utc::now(),
            gas_limit: 0,
            chain_state: chain_state.clone(),
        };
        
        match contract_info.contract_type {
            ContractType::Native => {
                if let Some(contract) = self.native_contracts.get(contract_id) {
                    contract.query(&ctx, query, data).await
                } else {
                    Err(anyhow!("Native contract not found: {}", contract_id))
                }
            }
            ContractType::Wasm => {
                Err(anyhow!("WASM contracts not yet implemented"))
            }
        }
    }
    
    fn generate_contract_id(&self, deployer: &Address, code: &[u8]) -> ContractId {
        use sha2::{Sha256, Digest};
        let mut hasher = Sha256::new();
        hasher.update(deployer.as_bytes());
        hasher.update(code);
        format!("0x{:x}", hasher.finalize())
    }
    
    fn is_native_contract(&self, code: &[u8]) -> bool {
        // Check if this is a known native contract
        // For now, we'll check against a magic prefix
        code.starts_with(b"GHOST_NATIVE_CONTRACT")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_contract_executor_creation() {
        let storage = ContractStorage::new();
        let executor = ContractExecutor::new(storage);
        
        assert_eq!(executor.native_contracts.len(), 2); // domain_registry + token_manager
    }
}