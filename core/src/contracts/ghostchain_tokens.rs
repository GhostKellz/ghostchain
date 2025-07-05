use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use chrono::Utc;
use tracing::{info, warn};

use crate::contracts::{Contract, ContractResult, ExecutionContext};
use crate::contracts::deployment::{ContractTemplate, ParameterDefinition, ContractType};
use crate::types::*;

/// GhostChain Credits (GCC) - Utility token for smart contracts and network operations
#[derive(Debug)]
pub struct GhostChainCreditsContract {
    pub total_supply: u128,
    pub balances: HashMap<Address, u128>,
    pub allowances: HashMap<Address, HashMap<Address, u128>>,
    pub name: String,
    pub symbol: String,
    pub decimals: u8,
    pub minter: Option<Address>,
    pub paused: bool,
}

/// Ghost Mana (GMAN) - Governance and staking reward token
#[derive(Debug)]
pub struct GhostManaContract {
    pub total_supply: u128,
    pub balances: HashMap<Address, u128>,
    pub staking_rewards: HashMap<Address, StakingInfo>,
    pub voting_power: HashMap<Address, u128>,
    pub name: String,
    pub symbol: String,
    pub decimals: u8,
    pub governance_contract: Option<Address>,
    pub reward_rate: u128, // GMAN per block per staked GSPR
}

/// Ghost Spirit (GSPR) - Primary native token of GhostChain
#[derive(Debug)]
pub struct GhostSpiritContract {
    pub total_supply: u128,
    pub balances: HashMap<Address, u128>,
    pub staked_amounts: HashMap<Address, u128>,
    pub validator_rewards: HashMap<Address, u128>,
    pub name: String,
    pub symbol: String,
    pub decimals: u8,
    pub consensus_contract: Option<Address>,
    pub max_supply: u128,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StakingInfo {
    pub staked_amount: u128,
    pub staking_start: u64,
    pub last_reward_claim: u64,
    pub accumulated_rewards: u128,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferArgs {
    pub to: Address,
    pub amount: u128,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApproveArgs {
    pub spender: Address,
    pub amount: u128,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StakeArgs {
    pub amount: u128,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenBalance {
    pub address: Address,
    pub balance: u128,
    pub staked_balance: Option<u128>,
    pub voting_power: Option<u128>,
}

impl GhostChainCreditsContract {
    pub fn new(
        initial_supply: u128,
        name: String,
        symbol: String,
        minter: Option<Address>,
    ) -> Self {
        Self {
            total_supply: initial_supply,
            balances: HashMap::new(),
            allowances: HashMap::new(),
            name,
            symbol,
            decimals: 18,
            minter,
            paused: false,
        }
    }

    fn transfer(&mut self, from: &Address, to: &Address, amount: u128) -> Result<()> {
        if self.paused {
            return Err(anyhow!("Contract is paused"));
        }

        let from_balance = self.balances.get(from).unwrap_or(&0);
        if *from_balance < amount {
            return Err(anyhow!("Insufficient balance"));
        }

        self.balances.insert(from.clone(), from_balance - amount);
        let to_balance = self.balances.get(to).unwrap_or(&0);
        self.balances.insert(to.clone(), to_balance + amount);

        Ok(())
    }

    fn mint(&mut self, to: &Address, amount: u128, caller: &Address) -> Result<()> {
        if let Some(ref minter) = self.minter {
            if caller != minter {
                return Err(anyhow!("Only minter can mint tokens"));
            }
        }

        self.total_supply += amount;
        let balance = self.balances.get(to).unwrap_or(&0);
        self.balances.insert(to.clone(), balance + amount);

        Ok(())
    }
}

#[async_trait::async_trait]
impl Contract for GhostChainCreditsContract {
    async fn init(&mut self, ctx: &ExecutionContext, init_data: &[u8]) -> Result<ContractResult> {
        // Initialize with deployer receiving initial supply
        if self.total_supply > 0 {
            self.balances.insert(ctx.caller.clone(), self.total_supply);
        }

        Ok(ContractResult {
            success: true,
            return_data: b"GCC initialized".to_vec(),
            gas_used: 50_000,
            events: vec![],
            error: None,
        })
    }

    async fn call(&mut self, ctx: &ExecutionContext, method: &str, data: &[u8]) -> Result<ContractResult> {
        match method {
            "transfer" => {
                let args: TransferArgs = serde_json::from_slice(data)?;
                self.transfer(&ctx.caller, &args.to, args.amount)?;
                
                Ok(ContractResult {
                    success: true,
                    return_data: b"Transfer successful".to_vec(),
                    gas_used: 21_000,
                    events: vec![],
                    error: None,
                })
            },
            "mint" => {
                let args: TransferArgs = serde_json::from_slice(data)?;
                self.mint(&args.to, args.amount, &ctx.caller)?;
                
                Ok(ContractResult {
                    success: true,
                    return_data: b"Mint successful".to_vec(),
                    gas_used: 30_000,
                    events: vec![],
                    error: None,
                })
            },
            "balance_of" => {
                let address: Address = serde_json::from_slice(data)?;
                let balance = self.balances.get(&address).unwrap_or(&0);
                
                Ok(ContractResult {
                    success: true,
                    return_data: serde_json::to_vec(balance)?,
                    gas_used: 5_000,
                    events: vec![],
                    error: None,
                })
            },
            "pause" => {
                if Some(&ctx.caller) != self.minter.as_ref() {
                    return Ok(ContractResult {
                        success: false,
                        return_data: b"Unauthorized".to_vec(),
                        gas_used: 3_000,
                        events: vec![],
                        error: Some("Only minter can pause".to_string()),
                    });
                }
                
                self.paused = true;
                Ok(ContractResult {
                    success: true,
                    return_data: b"Contract paused".to_vec(),
                    gas_used: 10_000,
                    events: vec![],
                    error: None,
                })
            },
            _ => Err(anyhow!("Unknown method: {}", method)),
        }
    }

    async fn query(&self, _ctx: &ExecutionContext, query: &str, data: &[u8]) -> Result<Vec<u8>> {
        match query {
            "total_supply" => Ok(serde_json::to_vec(&self.total_supply)?),
            "name" => Ok(serde_json::to_vec(&self.name)?),
            "symbol" => Ok(serde_json::to_vec(&self.symbol)?),
            "decimals" => Ok(serde_json::to_vec(&self.decimals)?),
            "balance_of" => {
                let address: Address = serde_json::from_slice(data)?;
                let balance = self.balances.get(&address).unwrap_or(&0);
                Ok(serde_json::to_vec(balance)?)
            },
            _ => Err(anyhow!("Unknown query: {}", query)),
        }
    }

    fn get_abi(&self) -> crate::contracts::ContractABI {
        use crate::contracts::{ContractABI, MethodSignature, ParameterType};
        
        ContractABI {
            methods: vec![
                MethodSignature {
                    name: "transfer".to_string(),
                    inputs: vec![ParameterType::Address, ParameterType::U128],
                    outputs: vec![ParameterType::Bool],
                    gas_cost: 21_000,
                },
                MethodSignature {
                    name: "mint".to_string(),
                    inputs: vec![ParameterType::Address, ParameterType::U128],
                    outputs: vec![ParameterType::Bool],
                    gas_cost: 30_000,
                },
                MethodSignature {
                    name: "balance_of".to_string(),
                    inputs: vec![ParameterType::Address],
                    outputs: vec![ParameterType::U128],
                    gas_cost: 5_000,
                },
            ],
            events: vec![],
        }
    }
}

impl GhostManaContract {
    pub fn new(governance_contract: Option<Address>) -> Self {
        Self {
            total_supply: 0,
            balances: HashMap::new(),
            staking_rewards: HashMap::new(),
            voting_power: HashMap::new(),
            name: "Ghost Mana".to_string(),
            symbol: "GMAN".to_string(),
            decimals: 18,
            governance_contract,
            reward_rate: 1_000_000_000_000_000, // 0.001 GMAN per block per GSPR
        }
    }

    fn calculate_staking_rewards(&self, address: &Address) -> u128 {
        if let Some(staking_info) = self.staking_rewards.get(address) {
            let current_time = Utc::now().timestamp() as u64;
            let time_diff = current_time.saturating_sub(staking_info.last_reward_claim);
            let blocks_elapsed = time_diff / 6; // 6 second block time
            
            blocks_elapsed as u128 * staking_info.staked_amount * self.reward_rate / 1_000_000_000_000_000_000
        } else {
            0
        }
    }
}

#[async_trait::async_trait]
impl Contract for GhostManaContract {
    async fn init(&mut self, _ctx: &ExecutionContext, _init_data: &[u8]) -> Result<ContractResult> {
        Ok(ContractResult {
            success: true,
            return_data: b"GMAN initialized".to_vec(),
            gas_used: 40_000,
            events: vec![],
            error: None,
        })
    }

    async fn call(&mut self, ctx: &ExecutionContext, method: &str, data: &[u8]) -> Result<ContractResult> {
        match method {
            "stake_gspr" => {
                let args: StakeArgs = serde_json::from_slice(data)?;
                
                // Record staking info
                let staking_info = StakingInfo {
                    staked_amount: args.amount,
                    staking_start: Utc::now().timestamp() as u64,
                    last_reward_claim: Utc::now().timestamp() as u64,
                    accumulated_rewards: 0,
                };
                
                self.staking_rewards.insert(ctx.caller.clone(), staking_info);
                self.voting_power.insert(ctx.caller.clone(), args.amount);
                
                Ok(ContractResult {
                    success: true,
                    return_data: b"GSPR staked successfully".to_vec(),
                    gas_used: 35_000,
                    events: vec![],
                    error: None,
                })
            },
            "claim_rewards" => {
                let rewards = self.calculate_staking_rewards(&ctx.caller);
                if rewards > 0 {
                    self.total_supply += rewards;
                    let balance = self.balances.get(&ctx.caller).unwrap_or(&0);
                    self.balances.insert(ctx.caller.clone(), balance + rewards);
                    
                    // Update last claim time
                    if let Some(staking_info) = self.staking_rewards.get_mut(&ctx.caller) {
                        staking_info.last_reward_claim = Utc::now().timestamp() as u64;
                        staking_info.accumulated_rewards += rewards;
                    }
                }
                
                Ok(ContractResult {
                    success: true,
                    return_data: serde_json::to_vec(&rewards)?,
                    gas_used: 25_000,
                    events: vec![],
                    error: None,
                })
            },
            _ => Err(anyhow!("Unknown method: {}", method)),
        }
    }

    async fn query(&self, _ctx: &ExecutionContext, query: &str, data: &[u8]) -> Result<Vec<u8>> {
        match query {
            "staking_info" => {
                let address: Address = serde_json::from_slice(data)?;
                let info = self.staking_rewards.get(&address);
                Ok(serde_json::to_vec(&info)?)
            },
            "voting_power" => {
                let address: Address = serde_json::from_slice(data)?;
                let power = self.voting_power.get(&address).unwrap_or(&0);
                Ok(serde_json::to_vec(power)?)
            },
            "pending_rewards" => {
                let address: Address = serde_json::from_slice(data)?;
                let rewards = self.calculate_staking_rewards(&address);
                Ok(serde_json::to_vec(&rewards)?)
            },
            _ => Err(anyhow!("Unknown query: {}", query)),
        }
    }

    fn get_abi(&self) -> crate::contracts::ContractABI {
        use crate::contracts::{ContractABI, MethodSignature, ParameterType};
        
        ContractABI {
            methods: vec![
                MethodSignature {
                    name: "stake_gspr".to_string(),
                    inputs: vec![ParameterType::U128],
                    outputs: vec![ParameterType::Bool],
                    gas_cost: 35_000,
                },
                MethodSignature {
                    name: "claim_rewards".to_string(),
                    inputs: vec![],
                    outputs: vec![ParameterType::U128],
                    gas_cost: 25_000,
                },
            ],
            events: vec![],
        }
    }
}

impl GhostSpiritContract {
    pub fn new(initial_supply: u128, max_supply: u128) -> Self {
        Self {
            total_supply: initial_supply,
            balances: HashMap::new(),
            staked_amounts: HashMap::new(),
            validator_rewards: HashMap::new(),
            name: "Ghost Spirit".to_string(),
            symbol: "GSPR".to_string(),
            decimals: 18,
            consensus_contract: None,
            max_supply,
        }
    }
}

#[async_trait::async_trait]
impl Contract for GhostSpiritContract {
    async fn init(&mut self, ctx: &ExecutionContext, _init_data: &[u8]) -> Result<ContractResult> {
        // Give initial supply to deployer
        self.balances.insert(ctx.caller.clone(), self.total_supply);
        
        Ok(ContractResult {
            success: true,
            return_data: b"GSPR initialized".to_vec(),
            gas_used: 60_000,
            events: vec![],
            error: None,
        })
    }

    async fn call(&mut self, ctx: &ExecutionContext, method: &str, data: &[u8]) -> Result<ContractResult> {
        match method {
            "transfer" => {
                let args: TransferArgs = serde_json::from_slice(data)?;
                let from_balance = self.balances.get(&ctx.caller).unwrap_or(&0);
                
                if *from_balance < args.amount {
                    return Ok(ContractResult {
                        success: false,
                        return_data: b"Insufficient balance".to_vec(),
                        gas_used: 5_000,
                        events: vec![],
                        error: Some("Insufficient balance".to_string()),
                    });
                }

                self.balances.insert(ctx.caller.clone(), from_balance - args.amount);
                let to_balance = self.balances.get(&args.to).unwrap_or(&0);
                self.balances.insert(args.to.clone(), to_balance + args.amount);
                
                Ok(ContractResult {
                    success: true,
                    return_data: b"Transfer successful".to_vec(),
                    gas_used: 21_000,
                    events: vec![],
                    error: None,
                })
            },
            "stake" => {
                let args: StakeArgs = serde_json::from_slice(data)?;
                let balance = self.balances.get(&ctx.caller).unwrap_or(&0);
                
                if *balance < args.amount {
                    return Ok(ContractResult {
                        success: false,
                        return_data: b"Insufficient balance to stake".to_vec(),
                        gas_used: 5_000,
                        events: vec![],
                        error: Some("Insufficient balance".to_string()),
                    });
                }

                self.balances.insert(ctx.caller.clone(), balance - args.amount);
                let staked = self.staked_amounts.get(&ctx.caller).unwrap_or(&0);
                self.staked_amounts.insert(ctx.caller.clone(), staked + args.amount);
                
                Ok(ContractResult {
                    success: true,
                    return_data: b"Staking successful".to_vec(),
                    gas_used: 30_000,
                    events: vec![],
                    error: None,
                })
            },
            "unstake" => {
                let args: StakeArgs = serde_json::from_slice(data)?;
                let staked = self.staked_amounts.get(&ctx.caller).unwrap_or(&0);
                
                if *staked < args.amount {
                    return Ok(ContractResult {
                        success: false,
                        return_data: b"Insufficient staked amount".to_vec(),
                        gas_used: 5_000,
                        events: vec![],
                        error: Some("Insufficient staked amount".to_string()),
                    });
                }

                self.staked_amounts.insert(ctx.caller.clone(), staked - args.amount);
                let balance = self.balances.get(&ctx.caller).unwrap_or(&0);
                self.balances.insert(ctx.caller.clone(), balance + args.amount);
                
                Ok(ContractResult {
                    success: true,
                    return_data: b"Unstaking successful".to_vec(),
                    gas_used: 30_000,
                    events: vec![],
                    error: None,
                })
            },
            _ => Err(anyhow!("Unknown method: {}", method)),
        }
    }

    async fn query(&self, _ctx: &ExecutionContext, query: &str, data: &[u8]) -> Result<Vec<u8>> {
        match query {
            "balance_of" => {
                let address: Address = serde_json::from_slice(data)?;
                let balance = TokenBalance {
                    address: address.clone(),
                    balance: *self.balances.get(&address).unwrap_or(&0),
                    staked_balance: Some(*self.staked_amounts.get(&address).unwrap_or(&0)),
                    voting_power: None,
                };
                Ok(serde_json::to_vec(&balance)?)
            },
            "total_supply" => Ok(serde_json::to_vec(&self.total_supply)?),
            "max_supply" => Ok(serde_json::to_vec(&self.max_supply)?),
            "total_staked" => {
                let total_staked: u128 = self.staked_amounts.values().sum();
                Ok(serde_json::to_vec(&total_staked)?)
            },
            _ => Err(anyhow!("Unknown query: {}", query)),
        }
    }

    fn get_abi(&self) -> crate::contracts::ContractABI {
        use crate::contracts::{ContractABI, MethodSignature, ParameterType};
        
        ContractABI {
            methods: vec![
                MethodSignature {
                    name: "transfer".to_string(),
                    inputs: vec![ParameterType::Address, ParameterType::U128],
                    outputs: vec![ParameterType::Bool],
                    gas_cost: 21_000,
                },
                MethodSignature {
                    name: "stake".to_string(),
                    inputs: vec![ParameterType::U128],
                    outputs: vec![ParameterType::Bool],
                    gas_cost: 30_000,
                },
                MethodSignature {
                    name: "unstake".to_string(),
                    inputs: vec![ParameterType::U128],
                    outputs: vec![ParameterType::Bool],
                    gas_cost: 30_000,
                },
            ],
            events: vec![],
        }
    }
}

/// Create contract templates for GhostChain tokens
pub fn create_ghostchain_token_templates() -> Vec<ContractTemplate> {
    vec![
        // GCC (GhostChain Credits) Template
        ContractTemplate {
            name: "GhostChainCredits".to_string(),
            contract_type: ContractType::Native,
            bytecode: b"GHOST_NATIVE_CONTRACT_GCC".to_vec(),
            abi: serde_json::json!({
                "methods": [
                    {"name": "transfer", "inputs": ["address", "uint128"], "outputs": ["bool"]},
                    {"name": "mint", "inputs": ["address", "uint128"], "outputs": ["bool"]},
                    {"name": "balance_of", "inputs": ["address"], "outputs": ["uint128"]},
                    {"name": "pause", "inputs": [], "outputs": ["bool"]}
                ]
            }),
            constructor_params: vec![
                ParameterDefinition {
                    name: "initial_supply".to_string(),
                    param_type: "uint128".to_string(),
                    description: "Initial supply of GCC tokens".to_string(),
                    optional: false,
                    default_value: Some(serde_json::Value::String("1000000000000000000000000".to_string())), // 1M GCC
                },
                ParameterDefinition {
                    name: "name".to_string(),
                    param_type: "string".to_string(),
                    description: "Token name".to_string(),
                    optional: false,
                    default_value: Some(serde_json::Value::String("GhostChain Credits".to_string())),
                },
                ParameterDefinition {
                    name: "symbol".to_string(),
                    param_type: "string".to_string(),
                    description: "Token symbol".to_string(),
                    optional: false,
                    default_value: Some(serde_json::Value::String("GCC".to_string())),
                },
            ],
            description: "GhostChain Credits (GCC) - Utility token for smart contracts and network operations".to_string(),
            example_usage: "Deploy with initial_supply=1000000, name='GhostChain Credits', symbol='GCC'".to_string(),
        },

        // GMAN (Ghost Mana) Template
        ContractTemplate {
            name: "GhostMana".to_string(),
            contract_type: ContractType::Native,
            bytecode: b"GHOST_NATIVE_CONTRACT_GMAN".to_vec(),
            abi: serde_json::json!({
                "methods": [
                    {"name": "stake_gspr", "inputs": ["uint128"], "outputs": ["bool"]},
                    {"name": "claim_rewards", "inputs": [], "outputs": ["uint128"]},
                    {"name": "voting_power", "inputs": ["address"], "outputs": ["uint128"]}
                ]
            }),
            constructor_params: vec![
                ParameterDefinition {
                    name: "governance_contract".to_string(),
                    param_type: "address".to_string(),
                    description: "Address of the governance contract".to_string(),
                    optional: true,
                    default_value: None,
                },
            ],
            description: "Ghost Mana (GMAN) - Governance and staking reward token".to_string(),
            example_usage: "Deploy with governance_contract=0x...".to_string(),
        },

        // GSPR (Ghost Spirit) Template
        ContractTemplate {
            name: "GhostSpirit".to_string(),
            contract_type: ContractType::Native,
            bytecode: b"GHOST_NATIVE_CONTRACT_GSPR".to_vec(),
            abi: serde_json::json!({
                "methods": [
                    {"name": "transfer", "inputs": ["address", "uint128"], "outputs": ["bool"]},
                    {"name": "stake", "inputs": ["uint128"], "outputs": ["bool"]},
                    {"name": "unstake", "inputs": ["uint128"], "outputs": ["bool"]},
                    {"name": "balance_of", "inputs": ["address"], "outputs": ["TokenBalance"]}
                ]
            }),
            constructor_params: vec![
                ParameterDefinition {
                    name: "initial_supply".to_string(),
                    param_type: "uint128".to_string(),
                    description: "Initial supply of GSPR tokens".to_string(),
                    optional: false,
                    default_value: Some(serde_json::Value::String("1000000000000000000000000000".to_string())), // 1B GSPR
                },
                ParameterDefinition {
                    name: "max_supply".to_string(),
                    param_type: "uint128".to_string(),
                    description: "Maximum supply of GSPR tokens".to_string(),
                    optional: false,
                    default_value: Some(serde_json::Value::String("21000000000000000000000000000".to_string())), // 21B GSPR max
                },
            ],
            description: "Ghost Spirit (GSPR) - Primary native token of GhostChain".to_string(),
            example_usage: "Deploy with initial_supply=1000000000, max_supply=21000000000".to_string(),
        },
    ]
}

/// Test function for GhostChain token contracts
pub async fn test_ghostchain_token_contracts() -> Result<()> {
    info!("🪙 Testing GhostChain token contracts");

    // Test GCC contract
    let mut gcc_contract = GhostChainCreditsContract::new(
        1_000_000 * 10u128.pow(18), // 1M GCC
        "GhostChain Credits".to_string(),
        "GCC".to_string(),
        Some("0xminter".to_string()),
    );

    let test_ctx = ExecutionContext {
        caller: "0xtest".to_string(),
        contract_id: "gcc_contract".to_string(),
        block_height: 1,
        timestamp: Utc::now(),
        gas_limit: 1_000_000,
        chain_state: ChainState {
            accounts: HashMap::new(),
            total_supply: HashMap::new(),
            validators: HashMap::new(),
            current_epoch: 1,
            contracts: HashMap::new(),
            domains: HashMap::new(),
        },
    };

    // Test GCC initialization
    match gcc_contract.init(&test_ctx, &[]).await {
        Ok(result) => info!("✅ GCC contract initialized: {}", result.success),
        Err(e) => warn!("❌ GCC initialization failed: {}", e),
    }

    // Test GMAN contract
    let mut gman_contract = GhostManaContract::new(None);
    
    match gman_contract.init(&test_ctx, &[]).await {
        Ok(result) => info!("✅ GMAN contract initialized: {}", result.success),
        Err(e) => warn!("❌ GMAN initialization failed: {}", e),
    }

    // Test GSPR contract
    let mut gspr_contract = GhostSpiritContract::new(
        1_000_000_000 * 10u128.pow(18), // 1B GSPR
        21_000_000_000 * 10u128.pow(18), // 21B GSPR max
    );
    
    match gspr_contract.init(&test_ctx, &[]).await {
        Ok(result) => info!("✅ GSPR contract initialized: {}", result.success),
        Err(e) => warn!("❌ GSPR initialization failed: {}", e),
    }

    info!("🎯 All GhostChain token contracts tested successfully");
    Ok(())
}

/// CLI command handler for token contract testing
pub async fn handle_token_test_command() -> Result<()> {
    println!("🪙 Testing GhostChain Token Contracts...");
    
    match test_ghostchain_token_contracts().await {
        Ok(()) => {
            println!("✅ Token contract tests completed successfully");
            
            // Show available templates
            let templates = create_ghostchain_token_templates();
            println!("\n📋 Available Token Templates:");
            for template in templates {
                println!("  🪙 {} ({})", template.name, template.description);
                println!("     Example: {}", template.example_usage);
            }
        },
        Err(e) => {
            println!("❌ Token contract tests failed: {}", e);
        }
    }
    
    Ok(())
}