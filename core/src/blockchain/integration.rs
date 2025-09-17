use crate::types::*;
use crate::contracts::{ContractExecutor, ExecutionContext};
use crate::contracts::storage::ContractStorage;
use crate::blockchain::Blockchain;
use anyhow::{Result, anyhow};
use chrono::Utc;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug)]
pub struct BlockchainContractIntegration {
    contract_executor: ContractExecutor,
    blockchain: Arc<RwLock<Blockchain>>,
}

impl BlockchainContractIntegration {
    pub fn new(blockchain: Arc<RwLock<Blockchain>>) -> Self {
        let storage = ContractStorage::new();
        let contract_executor = ContractExecutor::new(storage);
        
        Self {
            contract_executor,
            blockchain,
        }
    }
    
    pub async fn deploy_contract(
        &mut self,
        deployer: &Address,
        contract_code: &[u8],
        init_data: &[u8],
        gas_limit: u128,
    ) -> Result<ContractResult> {
        let mut blockchain = self.blockchain.write().await;
        
        // Check deployer account exists and has sufficient balance for gas
        let deployer_account = blockchain.state.accounts.get(deployer)
            .ok_or_else(|| anyhow!("Deployer account not found: {}", deployer))?;
        
        // Estimate gas cost for deployment
        let estimated_gas = 32000 + (contract_code.len() as u128 * 200); // Base cost + code size
        if gas_limit < estimated_gas {
            return Err(anyhow!("Gas limit too low: {} < {}", gas_limit, estimated_gas));
        }
        
        // Deploy contract
        let result = self.contract_executor.deploy_contract(
            deployer,
            contract_code,
            init_data,
            gas_limit,
            &mut blockchain.state,
        ).await?;
        
        // Update deployer's balance to pay for gas (assuming SPIRIT token used for gas)
        if let Some(deployer_account) = blockchain.state.accounts.get_mut(deployer) {
            let gas_cost = result.gas_used * 1000; // 1000 wei per gas unit
            let spirit_balance = deployer_account.balances.get(&TokenType::Spirit).unwrap_or(&0);
            
            if spirit_balance < &gas_cost {
                return Err(anyhow!("Insufficient SPIRIT balance for gas: {} < {}", spirit_balance, gas_cost));
            }
            
            *deployer_account.balances.get_mut(&TokenType::Spirit).unwrap() -= gas_cost;
        }
        
        Ok(result)
    }
    
    pub async fn call_contract(
        &mut self,
        caller: &Address,
        contract_id: &ContractId,
        method: &str,
        data: &[u8],
        gas_limit: u128,
    ) -> Result<ContractResult> {
        let mut blockchain = self.blockchain.write().await;
        
        // Check caller account exists
        let _caller_account = blockchain.state.accounts.get(caller)
            .ok_or_else(|| anyhow!("Caller account not found: {}", caller))?;
        
        // Call contract
        let result = self.contract_executor.call_contract(
            caller,
            contract_id,
            method,
            data,
            gas_limit,
            &mut blockchain.state,
        ).await?;
        
        // Update caller's balance to pay for gas
        if let Some(caller_account) = blockchain.state.accounts.get_mut(caller) {
            let gas_cost = result.gas_used * 1000; // 1000 wei per gas unit
            let spirit_balance = caller_account.balances.get(&TokenType::Spirit).unwrap_or(&0);
            
            if spirit_balance < &gas_cost {
                return Err(anyhow!("Insufficient SPIRIT balance for gas: {} < {}", spirit_balance, gas_cost));
            }
            
            *caller_account.balances.get_mut(&TokenType::Spirit).unwrap() -= gas_cost;
        }
        
        Ok(result)
    }
    
    pub async fn query_contract(
        &self,
        contract_id: &ContractId,
        query: &str,
        data: &[u8],
    ) -> Result<Vec<u8>> {
        let blockchain = self.blockchain.read().await;
        
        self.contract_executor.query_contract(
            contract_id,
            query,
            data,
            &blockchain.state,
        ).await
    }
    
    pub async fn execute_transaction(&mut self, tx: &Transaction) -> Result<ContractResult> {
        match &tx.tx_type {
            TransactionType::DeployContract { deployer, contract_code, init_data } => {
                self.deploy_contract(
                    deployer,
                    contract_code,
                    init_data,
                    tx.gas_used, // Use gas_used as gas_limit for now
                ).await
            }
            TransactionType::CallContract { caller, contract_id, method, data } => {
                self.call_contract(
                    caller,
                    contract_id,
                    method,
                    data,
                    tx.gas_used,
                ).await
            }
            _ => Err(anyhow!("Transaction type not supported for contract execution")),
        }
    }
    
    // System contract initialization
    pub async fn init_system_contracts(&mut self) -> Result<()> {
        let system_address = "system".to_string();
        
        // Deploy domain registry contract
        let domain_registry_code = b"GHOST_NATIVE_CONTRACT_DOMAIN_REGISTRY".to_vec();
        let domain_registry_init = serde_json::to_vec(&serde_json::json!({
            "name": "Ghost Domain Registry",
            "version": "1.0.0"
        }))?;
        
        let domain_result = self.deploy_contract(
            &system_address,
            &domain_registry_code,
            &domain_registry_init,
            100000,
        ).await?;
        
        println!("Domain registry deployed: gas used = {}", domain_result.gas_used);
        
        // Deploy token manager contract
        let token_manager_code = b"GHOST_NATIVE_CONTRACT_TOKEN_MANAGER".to_vec();
        let token_manager_init = serde_json::to_vec(&serde_json::json!({
            "name": "Ghost Token Manager",
            "version": "1.0.0"
        }))?;
        
        let token_result = self.deploy_contract(
            &system_address,
            &token_manager_code,
            &token_manager_init,
            100000,
        ).await?;
        
        println!("Token manager deployed: gas used = {}", token_result.gas_used);
        
        Ok(())
    }
    
    // Register a domain using the system domain registry
    pub async fn register_domain(
        &mut self,
        domain: &str,
        owner: &Address,
        records: Vec<DomainRecord>,
    ) -> Result<ContractResult> {
        let register_data = serde_json::to_vec(&serde_json::json!({
            "domain": domain,
            "owner": owner,
            "records": records
        }))?;
        
        self.call_contract(
            owner,
            &"system.domain_registry".to_string(),
            "register_domain",
            &register_data,
            60000, // Gas limit for domain registration
        ).await
    }
    
    // Resolve a domain using the system domain registry
    pub async fn resolve_domain(&self, domain: &str) -> Result<DomainData> {
        let query_data = serde_json::to_vec(&serde_json::json!({
            "domain": domain
        }))?;
        
        let result = self.query_contract(
            &"system.domain_registry".to_string(),
            "resolve_domain",
            &query_data,
        ).await?;
        
        let domain_data: DomainData = serde_json::from_slice(&result)?;
        Ok(domain_data)
    }
    
    // Transfer tokens using the token manager
    pub async fn transfer_tokens(
        &mut self,
        from: &Address,
        to: &Address,
        amount: u128,
        token: TokenType,
    ) -> Result<ContractResult> {
        let transfer_data = serde_json::to_vec(&serde_json::json!({
            "to": to,
            "amount": amount,
            "token": token
        }))?;
        
        self.call_contract(
            from,
            &"system.token_manager".to_string(),
            "transfer",
            &transfer_data,
            10000, // Gas limit for token transfer
        ).await
    }
    
    // Get token balance
    pub async fn get_token_balance(&self, address: &Address, token: TokenType) -> Result<u128> {
        let query_data = serde_json::to_vec(&serde_json::json!({
            "address": address,
            "token": token
        }))?;
        
        let result = self.query_contract(
            &"system.token_manager".to_string(),
            "balance",
            &query_data,
        ).await?;
        
        let balance: u128 = serde_json::from_slice(&result)?;
        Ok(balance)
    }

    /// Deploy a native contract (for token contracts and system contracts)
    pub async fn deploy_native_contract(
        &mut self,
        deployer: &Address,
        contract_name: &str,
        contract: Box<dyn crate::contracts::Contract>,
        init_data: &[u8],
    ) -> Result<String> {
        use crate::contracts::storage::ContractStorage;
        use crate::contracts::ContractExecutor;
        
        // Generate contract ID
        let contract_id = format!("native_{}_{}", contract_name, 
            chrono::Utc::now().timestamp());
        
        // Create execution context
        let mut blockchain = self.blockchain.write().await;
        let ctx = crate::contracts::ExecutionContext {
            caller: deployer.clone(),
            contract_id: contract_id.clone(),
            block_height: blockchain.current_height(),
            timestamp: chrono::Utc::now(),
            gas_limit: 1_000_000,
            chain_state: blockchain.state.clone(),
        };

        // Create contract info and add to blockchain state
        let contract_info = crate::types::ContractInfo {
            id: contract_id.clone(),
            deployer: deployer.clone(),
            code: format!("GHOST_NATIVE_CONTRACT_{}", contract_name.to_uppercase()).into_bytes(),
            state: std::collections::HashMap::new(),
            created_at: chrono::Utc::now(),
            gas_used: 0,
            contract_type: crate::types::ContractType::Native,
        };

        blockchain.state.contracts.insert(contract_id.clone(), contract_info);
        
        // Note: In a full implementation, we would store the contract instance
        // and initialize it properly. For now, we just create the record.
        
        Ok(contract_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::blockchain::Blockchain;
    use crate::types::GenesisConfig;
    
    #[tokio::test]
    async fn test_system_contract_deployment() {
        let config = GenesisConfig::default();
        let blockchain = Arc::new(RwLock::new(Blockchain::new(config).unwrap()));
        let mut integration = BlockchainContractIntegration::new(blockchain.clone());
        
        // This would fail in real scenario because system account doesn't exist
        // but demonstrates the integration structure
        assert!(integration.init_system_contracts().await.is_err());
    }
    
    #[tokio::test]
    async fn test_domain_resolution() {
        let config = GenesisConfig::default();
        let blockchain = Arc::new(RwLock::new(Blockchain::new(config).unwrap()));
        let integration = BlockchainContractIntegration::new(blockchain.clone());
        
        // This would fail because domain doesn't exist
        let result = integration.resolve_domain("test.ghost").await;
        assert!(result.is_err());
    }
}