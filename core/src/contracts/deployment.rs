use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use crate::types::*;
use crate::contracts::{ContractExecutor, ContractResult};
use crate::contracts::gas::{GasMeter, GasOperation};

/// Smart contract deployment and management system
pub struct ContractDeploymentSystem {
    pub executor: Arc<RwLock<ContractExecutor>>,
    pub gas_meter: GasMeter,
    pub deployed_contracts: HashMap<ContractId, DeployedContract>,
    pub contract_templates: HashMap<String, ContractTemplate>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeployedContract {
    pub contract_id: ContractId,
    pub deployer: Address,
    pub contract_type: ContractType,
    pub code_hash: String,
    pub creation_block: u64,
    pub creation_timestamp: u64,
    pub init_data: Vec<u8>,
    pub gas_used: u128,
    pub status: ContractStatus,
    pub metadata: ContractMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ContractType {
    Native,           // Built-in GhostChain contracts
    WASM,            // WebAssembly contracts
    EVM,             // Ethereum Virtual Machine contracts
    ZVM,             // Zig Virtual Machine contracts
    Custom(String),   // Custom contract types
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ContractStatus {
    Active,
    Paused,
    Upgraded,
    Destroyed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractMetadata {
    pub name: String,
    pub description: String,
    pub version: String,
    pub author: String,
    pub license: String,
    pub source_url: Option<String>,
    pub abi: Option<serde_json::Value>,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractTemplate {
    pub name: String,
    pub contract_type: ContractType,
    pub bytecode: Vec<u8>,
    pub abi: serde_json::Value,
    pub constructor_params: Vec<ParameterDefinition>,
    pub description: String,
    pub example_usage: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParameterDefinition {
    pub name: String,
    pub param_type: String,
    pub description: String,
    pub optional: bool,
    pub default_value: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentRequest {
    pub deployer: Address,
    pub contract_type: ContractType,
    pub bytecode: Vec<u8>,
    pub init_data: Vec<u8>,
    pub gas_limit: u128,
    pub metadata: ContractMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentResult {
    pub contract_id: ContractId,
    pub transaction_hash: String,
    pub gas_used: u128,
    pub creation_block: u64,
    pub contract_address: Address,
}

impl ContractDeploymentSystem {
    pub fn new(executor: Arc<RwLock<ContractExecutor>>) -> Self {
        let mut system = Self {
            executor,
            gas_meter: GasMeter::new(),
            deployed_contracts: HashMap::new(),
            contract_templates: HashMap::new(),
        };
        
        // Initialize standard contract templates
        system.init_standard_templates();
        system
    }

    /// Deploy a new smart contract
    pub async fn deploy_contract(&mut self, request: DeploymentRequest) -> Result<DeploymentResult> {
        // Validate deployment request
        self.validate_deployment_request(&request)?;

        // Calculate deployment gas cost
        let deployment_gas = self.calculate_deployment_gas(&request)?;
        if deployment_gas > request.gas_limit {
            return Err(anyhow!("Insufficient gas limit. Required: {}, Provided: {}", deployment_gas, request.gas_limit));
        }

        // Generate contract ID
        let contract_id = self.generate_contract_id(&request);
        
        // Deploy through executor
        let executor = self.executor.write().await;
        let deployment_result = self.execute_deployment(&executor, &request, &contract_id).await?;
        drop(executor);

        // Create deployed contract record
        let deployed_contract = DeployedContract {
            contract_id: contract_id.clone(),
            deployer: request.deployer.clone(),
            contract_type: request.contract_type,
            code_hash: self.calculate_code_hash(&request.bytecode),
            creation_block: self.get_current_block_height(),
            creation_timestamp: chrono::Utc::now().timestamp() as u64,
            init_data: request.init_data,
            gas_used: deployment_result.gas_used,
            status: ContractStatus::Active,
            metadata: request.metadata,
        };

        // Store deployed contract
        self.deployed_contracts.insert(contract_id.clone(), deployed_contract);

        Ok(DeploymentResult {
            contract_id: contract_id.clone(),
            transaction_hash: format!("0x{:064x}", self.hash_string(&format!("deploy_{}", contract_id))),
            gas_used: deployment_result.gas_used,
            creation_block: self.get_current_block_height(),
            contract_address: contract_id,
        })
    }

    /// Deploy contract from template
    pub async fn deploy_from_template(
        &mut self,
        template_name: &str,
        deployer: &Address,
        constructor_args: Vec<serde_json::Value>,
        gas_limit: u128,
    ) -> Result<DeploymentResult> {
        let template = self.contract_templates.get(template_name)
            .ok_or_else(|| anyhow!("Contract template not found: {}", template_name))?
            .clone();

        // Encode constructor arguments
        let init_data = self.encode_constructor_args(&template, constructor_args)?;

        let request = DeploymentRequest {
            deployer: deployer.clone(),
            contract_type: template.contract_type,
            bytecode: template.bytecode,
            init_data,
            gas_limit,
            metadata: ContractMetadata {
                name: template.name.clone(),
                description: template.description.clone(),
                version: "1.0.0".to_string(),
                author: deployer.clone(),
                license: "MIT".to_string(),
                source_url: None,
                abi: Some(template.abi),
                tags: vec!["template".to_string()],
            },
        };

        self.deploy_contract(request).await
    }

    /// Upgrade a contract (if supported)
    pub async fn upgrade_contract(
        &mut self,
        contract_id: &ContractId,
        new_bytecode: Vec<u8>,
        upgrader: &Address,
        gas_limit: u128,
    ) -> Result<String> {
        let contract = self.deployed_contracts.get_mut(contract_id)
            .ok_or_else(|| anyhow!("Contract not found: {}", contract_id))?;

        // Verify upgrader permissions (in real implementation, check contract permissions)
        if contract.deployer != *upgrader {
            return Err(anyhow!("Only contract deployer can upgrade"));
        }

        // Mark old contract as upgraded
        contract.status = ContractStatus::Upgraded;

        // Deploy new version (simplified - real implementation would be more complex)
        let new_code_hash = self.calculate_code_hash(&new_bytecode);
        
        Ok(format!("Contract {} upgraded. New code hash: {}", contract_id, new_code_hash))
    }

    /// Pause/unpause a contract
    pub async fn set_contract_status(
        &mut self,
        contract_id: &ContractId,
        new_status: ContractStatus,
        caller: &Address,
    ) -> Result<()> {
        let contract = self.deployed_contracts.get_mut(contract_id)
            .ok_or_else(|| anyhow!("Contract not found: {}", contract_id))?;

        // Verify permissions
        if contract.deployer != *caller {
            return Err(anyhow!("Only contract deployer can change status"));
        }

        contract.status = new_status;
        Ok(())
    }

    /// Get contract information
    pub fn get_contract_info(&self, contract_id: &ContractId) -> Option<&DeployedContract> {
        self.deployed_contracts.get(contract_id)
    }

    /// Get contracts deployed by an address
    pub fn get_contracts_by_deployer(&self, deployer: &Address) -> Vec<&DeployedContract> {
        self.deployed_contracts
            .values()
            .filter(|contract| &contract.deployer == deployer)
            .collect()
    }

    /// Get contracts by type
    pub fn get_contracts_by_type(&self, contract_type: &ContractType) -> Vec<&DeployedContract> {
        self.deployed_contracts
            .values()
            .filter(|contract| std::mem::discriminant(&contract.contract_type) == std::mem::discriminant(contract_type))
            .collect()
    }

    /// Add a new contract template
    pub fn add_template(&mut self, template: ContractTemplate) {
        self.contract_templates.insert(template.name.clone(), template);
    }

    /// Get available templates
    pub fn get_templates(&self) -> Vec<&ContractTemplate> {
        self.contract_templates.values().collect()
    }

    /// Calculate deployment gas cost
    fn calculate_deployment_gas(&self, request: &DeploymentRequest) -> Result<u128> {
        let base_gas = self.gas_meter.calculate_gas_for_operation(&GasOperation::ContractCreate);
        let code_gas = request.bytecode.len() as u128 * 200; // 200 gas per byte
        let init_gas = request.init_data.len() as u128 * 68; // 68 gas per byte
        
        Ok(base_gas + code_gas + init_gas)
    }

    /// Validate deployment request
    fn validate_deployment_request(&self, request: &DeploymentRequest) -> Result<()> {
        if request.bytecode.is_empty() {
            return Err(anyhow!("Contract bytecode cannot be empty"));
        }

        if request.bytecode.len() > 24576 {
            return Err(anyhow!("Contract bytecode exceeds maximum size (24576 bytes)"));
        }

        if request.gas_limit == 0 {
            return Err(anyhow!("Gas limit must be greater than 0"));
        }

        if request.metadata.name.is_empty() {
            return Err(anyhow!("Contract name cannot be empty"));
        }

        Ok(())
    }

    /// Execute the actual deployment
    async fn execute_deployment(
        &self,
        executor: &ContractExecutor,
        request: &DeploymentRequest,
        contract_id: &ContractId,
    ) -> Result<ContractResult> {
        // In real implementation, this would:
        // 1. Create execution context
        // 2. Initialize contract storage
        // 3. Execute constructor
        // 4. Return deployment result

        // For now, simulate successful deployment
        Ok(ContractResult {
            success: true,
            return_data: vec![],
            gas_used: self.calculate_deployment_gas(request)?,
            events: vec![],
            error: None,
        })
    }

    /// Generate deterministic contract ID
    fn generate_contract_id(&self, request: &DeploymentRequest) -> ContractId {
        format!("0x{:040x}", self.hash_string(&format!(
            "{}{}{}",
            request.deployer,
            self.calculate_code_hash(&request.bytecode),
            chrono::Utc::now().timestamp()
        )))
    }

    /// Calculate code hash
    fn calculate_code_hash(&self, bytecode: &[u8]) -> String {
        use sha2::{Sha256, Digest};
        let mut hasher = Sha256::new();
        hasher.update(bytecode);
        format!("{:x}", hasher.finalize())
    }

    /// Encode constructor arguments
    fn encode_constructor_args(
        &self,
        template: &ContractTemplate,
        args: Vec<serde_json::Value>,
    ) -> Result<Vec<u8>> {
        if args.len() != template.constructor_params.len() {
            return Err(anyhow!(
                "Constructor parameter count mismatch. Expected: {}, Got: {}",
                template.constructor_params.len(),
                args.len()
            ));
        }

        // Simple encoding - in real implementation would use proper ABI encoding
        Ok(serde_json::to_vec(&args)?)
    }

    /// Initialize standard contract templates
    fn init_standard_templates(&mut self) {
        // ERC-20 Token Template
        let erc20_template = ContractTemplate {
            name: "ERC20Token".to_string(),
            contract_type: ContractType::EVM,
            bytecode: self.get_erc20_bytecode(),
            abi: self.get_erc20_abi(),
            constructor_params: vec![
                ParameterDefinition {
                    name: "name".to_string(),
                    param_type: "string".to_string(),
                    description: "Token name".to_string(),
                    optional: false,
                    default_value: None,
                },
                ParameterDefinition {
                    name: "symbol".to_string(),
                    param_type: "string".to_string(),
                    description: "Token symbol".to_string(),
                    optional: false,
                    default_value: None,
                },
                ParameterDefinition {
                    name: "totalSupply".to_string(),
                    param_type: "uint256".to_string(),
                    description: "Total token supply".to_string(),
                    optional: false,
                    default_value: None,
                },
            ],
            description: "Standard ERC-20 token contract".to_string(),
            example_usage: "Deploy with name='MyToken', symbol='MTK', totalSupply=1000000".to_string(),
        };
        self.add_template(erc20_template);

        // NFT Template
        let nft_template = ContractTemplate {
            name: "ERC721NFT".to_string(),
            contract_type: ContractType::EVM,
            bytecode: self.get_erc721_bytecode(),
            abi: self.get_erc721_abi(),
            constructor_params: vec![
                ParameterDefinition {
                    name: "name".to_string(),
                    param_type: "string".to_string(),
                    description: "NFT collection name".to_string(),
                    optional: false,
                    default_value: None,
                },
                ParameterDefinition {
                    name: "symbol".to_string(),
                    param_type: "string".to_string(),
                    description: "NFT collection symbol".to_string(),
                    optional: false,
                    default_value: None,
                },
            ],
            description: "Standard ERC-721 NFT contract".to_string(),
            example_usage: "Deploy with name='MyNFTs', symbol='MNFT'".to_string(),
        };
        self.add_template(nft_template);

        // DAO Template
        let dao_template = ContractTemplate {
            name: "SimpleDAO".to_string(),
            contract_type: ContractType::Custom("DAO".to_string()),
            bytecode: self.get_dao_bytecode(),
            abi: self.get_dao_abi(),
            constructor_params: vec![
                ParameterDefinition {
                    name: "votingToken".to_string(),
                    param_type: "address".to_string(),
                    description: "Token used for voting".to_string(),
                    optional: false,
                    default_value: None,
                },
                ParameterDefinition {
                    name: "proposalThreshold".to_string(),
                    param_type: "uint256".to_string(),
                    description: "Minimum tokens required to create proposal".to_string(),
                    optional: false,
                    default_value: Some(serde_json::Value::Number(serde_json::Number::from(1000))),
                },
            ],
            description: "Simple DAO contract with proposal and voting".to_string(),
            example_usage: "Deploy with votingToken=0x..., proposalThreshold=1000".to_string(),
        };
        self.add_template(dao_template);
    }

    // Placeholder bytecode and ABI methods (in real implementation, these would be actual contract code)
    fn get_erc20_bytecode(&self) -> Vec<u8> {
        b"ERC20_BYTECODE".to_vec()
    }

    fn get_erc20_abi(&self) -> serde_json::Value {
        serde_json::json!([
            {
                "inputs": [
                    {"name": "name", "type": "string"},
                    {"name": "symbol", "type": "string"},
                    {"name": "totalSupply", "type": "uint256"}
                ],
                "name": "constructor",
                "type": "constructor"
            },
            {
                "inputs": [
                    {"name": "to", "type": "address"},
                    {"name": "amount", "type": "uint256"}
                ],
                "name": "transfer",
                "outputs": [{"name": "", "type": "bool"}],
                "type": "function"
            }
        ])
    }

    fn get_erc721_bytecode(&self) -> Vec<u8> {
        b"ERC721_BYTECODE".to_vec()
    }

    fn get_erc721_abi(&self) -> serde_json::Value {
        serde_json::json!([
            {
                "inputs": [
                    {"name": "name", "type": "string"},
                    {"name": "symbol", "type": "string"}
                ],
                "name": "constructor",
                "type": "constructor"
            },
            {
                "inputs": [
                    {"name": "to", "type": "address"},
                    {"name": "tokenId", "type": "uint256"}
                ],
                "name": "mint",
                "outputs": [],
                "type": "function"
            }
        ])
    }

    fn get_dao_bytecode(&self) -> Vec<u8> {
        b"DAO_BYTECODE".to_vec()
    }

    fn get_dao_abi(&self) -> serde_json::Value {
        serde_json::json!([
            {
                "inputs": [
                    {"name": "votingToken", "type": "address"},
                    {"name": "proposalThreshold", "type": "uint256"}
                ],
                "name": "constructor",
                "type": "constructor"
            },
            {
                "inputs": [
                    {"name": "description", "type": "string"},
                    {"name": "target", "type": "address"},
                    {"name": "data", "type": "bytes"}
                ],
                "name": "createProposal",
                "outputs": [{"name": "proposalId", "type": "uint256"}],
                "type": "function"
            }
        ])
    }

    fn hash_string(&self, input: &str) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        input.hash(&mut hasher);
        hasher.finish()
    }

    fn get_current_block_height(&self) -> u64 {
        // In real implementation, would get from blockchain
        chrono::Utc::now().timestamp() as u64
    }
}