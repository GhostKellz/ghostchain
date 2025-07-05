use crate::types::*;
use crate::contracts::{Contract, ContractABI, MethodSignature, EventSignature, ParameterType, ExecutionContext};
use crate::contracts::gas::{GasMeter, GasOperation};
use crate::contracts::storage::ContractStorage;
use anyhow::{Result, anyhow};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use chrono::Utc;
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainRegistryState {
    pub domains: HashMap<String, DomainData>,
    pub owner_domains: HashMap<Address, Vec<String>>,
    pub tld_configs: HashMap<String, TldConfig>,
    pub registration_fees: HashMap<String, u128>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TldConfig {
    pub tld: String,
    pub enabled: bool,
    pub min_length: u32,
    pub max_length: u32,
    pub registration_fee: u128,
    pub renewal_fee: u128,
    pub admin: Address,
}

pub struct DomainRegistryContract {
    gas_meter: GasMeter,
    storage: ContractStorage,
}

impl DomainRegistryContract {
    pub fn new() -> Self {
        let mut contract = Self {
            gas_meter: GasMeter::new(),
            storage: ContractStorage::new(),
        };
        
        // Initialize default TLD configurations
        contract.init_default_tlds();
        contract
    }
    
    fn init_default_tlds(&mut self) {
        let contract_id = "system.domain_registry";
        
        let tlds = vec![
            TldConfig {
                tld: "ghost".to_string(),
                enabled: true,
                min_length: 3,
                max_length: 63,
                registration_fee: 100 * 10u128.pow(18), // 100 SPIRIT
                renewal_fee: 50 * 10u128.pow(18),       // 50 SPIRIT
                admin: "system".to_string(),
            },
            TldConfig {
                tld: "zkellz".to_string(),
                enabled: true,
                min_length: 2,
                max_length: 63,
                registration_fee: 250 * 10u128.pow(18), // 250 SPIRIT
                renewal_fee: 100 * 10u128.pow(18),      // 100 SPIRIT
                admin: "system".to_string(),
            },
            TldConfig {
                tld: "kz".to_string(),
                enabled: true,
                min_length: 2,
                max_length: 63,
                registration_fee: 500 * 10u128.pow(18), // 500 SPIRIT
                renewal_fee: 200 * 10u128.pow(18),      // 200 SPIRIT
                admin: "system".to_string(),
            },
        ];
        
        for tld in tlds {
            let key = format!("tld_config:{}", tld.tld);
            self.storage.write_json(&contract_id.to_string(), &key, &tld).unwrap();
        }
    }
    
    async fn register_domain(&mut self, ctx: &ExecutionContext, data: &[u8]) -> Result<ContractResult> {
        #[derive(Serialize, Deserialize)]
        struct RegisterDomainArgs {
            domain: String,
            owner: Address,
            records: Vec<DomainRecord>,
        }
        
        let args: RegisterDomainArgs = serde_json::from_slice(data)
            .map_err(|e| anyhow!("Failed to parse register domain args: {}", e))?;
        
        // Validate domain format
        self.validate_domain_name(&args.domain)?;
        
        // Check if domain already exists
        if self.storage.load_domain_data(&ctx.contract_id, &args.domain)?.is_some() {
            return Err(anyhow!("Domain already registered: {}", args.domain));
        }
        
        // Get TLD configuration
        let tld = self.extract_tld(&args.domain)?;
        let tld_config = self.get_tld_config(&ctx.contract_id, &tld)?;
        
        if !tld_config.enabled {
            return Err(anyhow!("TLD not enabled: {}", tld));
        }
        
        // Validate domain length
        let domain_name = args.domain.strip_suffix(&format!(".{}", tld)).unwrap_or(&args.domain);
        if domain_name.len() < tld_config.min_length as usize || domain_name.len() > tld_config.max_length as usize {
            return Err(anyhow!("Invalid domain length for TLD {}: {} (allowed: {}-{})", 
                tld, domain_name.len(), tld_config.min_length, tld_config.max_length));
        }
        
        // Create domain data
        let domain_data = DomainData {
            domain: args.domain.clone(),
            owner: args.owner.clone(),
            records: args.records,
            contract_address: Some(ctx.contract_id.clone()),
            last_updated: Utc::now().timestamp() as u64,
            expiry: None, // TODO: Calculate expiry based on registration period
            signature: vec![], // TODO: Generate signature
        };
        
        // Store domain data
        self.storage.store_domain_data(&ctx.contract_id, &args.domain, &domain_data)?;
        self.storage.store_domain_owner(&ctx.contract_id, &args.domain, &args.owner);
        
        // Update owner's domain list
        let mut owner_domains = self.storage.load_owner_domains(&ctx.contract_id, &args.owner)?
            .unwrap_or_default();
        owner_domains.push(args.domain.clone());
        self.storage.store_owner_domains(&ctx.contract_id, &args.owner, &owner_domains)?;
        
        // Create events
        let events = vec![
            ContractEvent {
                contract_id: ctx.contract_id.clone(),
                event_type: "DomainRegistered".to_string(),
                data: serde_json::to_vec(&domain_data)?,
                topics: vec![args.domain.clone(), args.owner.clone()],
            }
        ];
        
        let gas_used = self.gas_meter.calculate_gas_for_operation(&GasOperation::DomainRegister);
        
        Ok(ContractResult {
            success: true,
            return_data: serde_json::to_vec(&domain_data)?,
            gas_used,
            events,
            error: None,
        })
    }
    
    async fn transfer_domain(&mut self, ctx: &ExecutionContext, data: &[u8]) -> Result<ContractResult> {
        #[derive(Serialize, Deserialize)]
        struct TransferDomainArgs {
            domain: String,
            new_owner: Address,
        }
        
        let args: TransferDomainArgs = serde_json::from_slice(data)?;
        
        // Get domain data
        let mut domain_data = self.storage.load_domain_data(&ctx.contract_id, &args.domain)?
            .ok_or_else(|| anyhow!("Domain not found: {}", args.domain))?;
        
        // Check ownership
        if domain_data.owner != ctx.caller {
            return Err(anyhow!("Only domain owner can transfer domain"));
        }
        
        let old_owner = domain_data.owner.clone();
        
        // Update domain ownership
        domain_data.owner = args.new_owner.clone();
        domain_data.last_updated = Utc::now().timestamp() as u64;
        
        // Store updated domain data
        self.storage.store_domain_data(&ctx.contract_id, &args.domain, &domain_data)?;
        self.storage.store_domain_owner(&ctx.contract_id, &args.domain, &args.new_owner);
        
        // Update old owner's domain list
        if let Some(mut old_domains) = self.storage.load_owner_domains(&ctx.contract_id, &old_owner)? {
            old_domains.retain(|d| d != &args.domain);
            self.storage.store_owner_domains(&ctx.contract_id, &old_owner, &old_domains)?;
        }
        
        // Update new owner's domain list
        let mut new_domains = self.storage.load_owner_domains(&ctx.contract_id, &args.new_owner)?
            .unwrap_or_default();
        new_domains.push(args.domain.clone());
        self.storage.store_owner_domains(&ctx.contract_id, &args.new_owner, &new_domains)?;
        
        let events = vec![
            ContractEvent {
                contract_id: ctx.contract_id.clone(),
                event_type: "DomainTransferred".to_string(),
                data: serde_json::to_vec(&domain_data)?,
                topics: vec![args.domain.clone(), old_owner, args.new_owner.clone()],
            }
        ];
        
        let gas_used = self.gas_meter.calculate_gas_for_operation(&GasOperation::DomainTransfer);
        
        Ok(ContractResult {
            success: true,
            return_data: serde_json::to_vec(&domain_data)?,
            gas_used,
            events,
            error: None,
        })
    }
    
    async fn set_record(&mut self, ctx: &ExecutionContext, data: &[u8]) -> Result<ContractResult> {
        #[derive(Serialize, Deserialize)]
        struct SetRecordArgs {
            domain: String,
            record: DomainRecord,
        }
        
        let args: SetRecordArgs = serde_json::from_slice(data)?;
        
        // Get domain data
        let mut domain_data = self.storage.load_domain_data(&ctx.contract_id, &args.domain)?
            .ok_or_else(|| anyhow!("Domain not found: {}", args.domain))?;
        
        // Check ownership
        if domain_data.owner != ctx.caller {
            return Err(anyhow!("Only domain owner can set records"));
        }
        
        // Update or add record
        if let Some(existing_record) = domain_data.records.iter_mut()
            .find(|r| r.record_type == args.record.record_type && r.name == args.record.name) {
            *existing_record = args.record.clone();
        } else {
            domain_data.records.push(args.record.clone());
        }
        
        domain_data.last_updated = Utc::now().timestamp() as u64;
        
        // Store updated domain data
        self.storage.store_domain_data(&ctx.contract_id, &args.domain, &domain_data)?;
        
        let events = vec![
            ContractEvent {
                contract_id: ctx.contract_id.clone(),
                event_type: "RecordUpdated".to_string(),
                data: serde_json::to_vec(&args.record)?,
                topics: vec![args.domain.clone(), args.record.record_type.clone()],
            }
        ];
        
        let gas_used = self.gas_meter.calculate_gas_for_operation(&GasOperation::DnsRecordUpdate);
        
        Ok(ContractResult {
            success: true,
            return_data: serde_json::to_vec(&domain_data)?,
            gas_used,
            events,
            error: None,
        })
    }
    
    async fn resolve_domain(&self, ctx: &ExecutionContext, data: &[u8]) -> Result<ContractResult> {
        #[derive(Serialize, Deserialize)]
        struct ResolveDomainArgs {
            domain: String,
        }
        
        let args: ResolveDomainArgs = serde_json::from_slice(data)?;
        
        if let Some(domain_data) = self.storage.load_domain_data(&ctx.contract_id, &args.domain)? {
            let gas_used = self.gas_meter.calculate_gas_for_operation(&GasOperation::DomainLookup);
            
            Ok(ContractResult {
                success: true,
                return_data: serde_json::to_vec(&domain_data)?,
                gas_used,
                events: vec![],
                error: None,
            })
        } else {
            Err(anyhow!("Domain not found: {}", args.domain))
        }
    }
    
    fn validate_domain_name(&self, domain: &str) -> Result<()> {
        if domain.is_empty() {
            return Err(anyhow!("Domain name cannot be empty"));
        }
        
        if domain.len() > 253 {
            return Err(anyhow!("Domain name too long"));
        }
        
        // Check for valid characters (basic check)
        if !domain.chars().all(|c| c.is_alphanumeric() || c == '.' || c == '-') {
            return Err(anyhow!("Invalid characters in domain name"));
        }
        
        // Check that it doesn't start or end with hyphen or dot
        if domain.starts_with('-') || domain.ends_with('-') || domain.starts_with('.') || domain.ends_with('.') {
            return Err(anyhow!("Domain name cannot start or end with hyphen or dot"));
        }
        
        Ok(())
    }
    
    fn extract_tld(&self, domain: &str) -> Result<String> {
        domain.split('.').last()
            .ok_or_else(|| anyhow!("Invalid domain format: {}", domain))
            .map(|s| s.to_string())
    }
    
    fn get_tld_config(&self, contract_id: &ContractId, tld: &str) -> Result<TldConfig> {
        let key = format!("tld_config:{}", tld);
        self.storage.read_json(contract_id, &key)?
            .ok_or_else(|| anyhow!("TLD not supported: {}", tld))
    }
}

#[async_trait]
impl Contract for DomainRegistryContract {
    async fn init(&mut self, _ctx: &ExecutionContext, _init_data: &[u8]) -> Result<ContractResult> {
        // Domain registry is already initialized in new()
        Ok(ContractResult {
            success: true,
            return_data: vec![],
            gas_used: 0,
            events: vec![],
            error: None,
        })
    }
    
    async fn call(&mut self, ctx: &ExecutionContext, method: &str, data: &[u8]) -> Result<ContractResult> {
        match method {
            "register_domain" => self.register_domain(ctx, data).await,
            "transfer_domain" => self.transfer_domain(ctx, data).await,
            "set_record" => self.set_record(ctx, data).await,
            "resolve_domain" => self.resolve_domain(ctx, data).await,
            _ => Err(anyhow!("Unknown method: {}", method)),
        }
    }
    
    async fn query(&self, ctx: &ExecutionContext, query: &str, data: &[u8]) -> Result<Vec<u8>> {
        match query {
            "resolve_domain" => {
                let result = self.resolve_domain(ctx, data).await?;
                Ok(result.return_data)
            }
            "get_domain_owner" => {
                #[derive(Serialize, Deserialize)]
                struct GetOwnerArgs {
                    domain: String,
                }
                
                let args: GetOwnerArgs = serde_json::from_slice(data)?;
                if let Some(owner) = self.storage.load_domain_owner(&ctx.contract_id, &args.domain)? {
                    Ok(serde_json::to_vec(&owner)?)
                } else {
                    Err(anyhow!("Domain not found: {}", args.domain))
                }
            }
            "get_owner_domains" => {
                #[derive(Serialize, Deserialize)]
                struct GetOwnerDomainsArgs {
                    owner: Address,
                }
                
                let args: GetOwnerDomainsArgs = serde_json::from_slice(data)?;
                let domains = self.storage.load_owner_domains(&ctx.contract_id, &args.owner)?
                    .unwrap_or_default();
                Ok(serde_json::to_vec(&domains)?)
            }
            _ => Err(anyhow!("Unknown query: {}", query)),
        }
    }
    
    fn get_abi(&self) -> ContractABI {
        ContractABI {
            methods: vec![
                MethodSignature {
                    name: "register_domain".to_string(),
                    inputs: vec![
                        ParameterType::String, // domain
                        ParameterType::Address, // owner
                        ParameterType::Bytes,  // records
                    ],
                    outputs: vec![ParameterType::Bytes], // domain_data
                    gas_cost: 50000,
                },
                MethodSignature {
                    name: "transfer_domain".to_string(),
                    inputs: vec![
                        ParameterType::String, // domain
                        ParameterType::Address, // new_owner
                    ],
                    outputs: vec![ParameterType::Bytes], // domain_data
                    gas_cost: 30000,
                },
                MethodSignature {
                    name: "set_record".to_string(),
                    inputs: vec![
                        ParameterType::String, // domain
                        ParameterType::Bytes,  // record
                    ],
                    outputs: vec![ParameterType::Bytes], // domain_data
                    gas_cost: 10000,
                },
                MethodSignature {
                    name: "resolve_domain".to_string(),
                    inputs: vec![ParameterType::String], // domain
                    outputs: vec![ParameterType::Bytes], // domain_data
                    gas_cost: 100,
                },
            ],
            events: vec![
                EventSignature {
                    name: "DomainRegistered".to_string(),
                    inputs: vec![
                        ParameterType::String, // domain
                        ParameterType::Address, // owner
                    ],
                },
                EventSignature {
                    name: "DomainTransferred".to_string(),
                    inputs: vec![
                        ParameterType::String, // domain
                        ParameterType::Address, // old_owner
                        ParameterType::Address, // new_owner
                    ],
                },
                EventSignature {
                    name: "RecordUpdated".to_string(),
                    inputs: vec![
                        ParameterType::String, // domain
                        ParameterType::String, // record_type
                    ],
                },
            ],
        }
    }
}

// Token Manager Contract for managing SPIRIT, MANA, RLUSD, SOUL tokens
pub struct TokenManagerContract {
    gas_meter: GasMeter,
    storage: ContractStorage,
}

impl TokenManagerContract {
    pub fn new() -> Self {
        Self {
            gas_meter: GasMeter::new(),
            storage: ContractStorage::new(),
        }
    }
}

#[async_trait]
impl Contract for TokenManagerContract {
    async fn init(&mut self, _ctx: &ExecutionContext, _init_data: &[u8]) -> Result<ContractResult> {
        // Initialize token supplies
        let contract_id = &_ctx.contract_id;
        
        // Set initial supplies
        self.storage.store_total_supply(contract_id, &TokenType::Spirit, 1_000_000_000 * 10u128.pow(18));
        self.storage.store_total_supply(contract_id, &TokenType::Rlusd, 100_000_000 * 10u128.pow(18));
        self.storage.store_total_supply(contract_id, &TokenType::Mana, 0); // Mana is earned
        self.storage.store_total_supply(contract_id, &TokenType::Soul, 0);  // Soul is minted
        
        Ok(ContractResult {
            success: true,
            return_data: vec![],
            gas_used: 1000,
            events: vec![],
            error: None,
        })
    }
    
    async fn call(&mut self, ctx: &ExecutionContext, method: &str, data: &[u8]) -> Result<ContractResult> {
        match method {
            "transfer" => self.transfer(ctx, data).await,
            "mint" => self.mint(ctx, data).await,
            "burn" => self.burn(ctx, data).await,
            _ => Err(anyhow!("Unknown method: {}", method)),
        }
    }
    
    async fn query(&self, ctx: &ExecutionContext, query: &str, data: &[u8]) -> Result<Vec<u8>> {
        match query {
            "balance" => {
                #[derive(Serialize, Deserialize)]
                struct BalanceArgs {
                    address: Address,
                    token: TokenType,
                }
                
                let args: BalanceArgs = serde_json::from_slice(data)?;
                let balance = self.storage.load_token_balance(&ctx.contract_id, &args.address, &args.token)?;
                Ok(serde_json::to_vec(&balance)?)
            }
            "total_supply" => {
                #[derive(Serialize, Deserialize)]
                struct TotalSupplyArgs {
                    token: TokenType,
                }
                
                let args: TotalSupplyArgs = serde_json::from_slice(data)?;
                let supply = self.storage.load_total_supply(&ctx.contract_id, &args.token)?;
                Ok(serde_json::to_vec(&supply)?)
            }
            _ => Err(anyhow!("Unknown query: {}", query)),
        }
    }
    
    fn get_abi(&self) -> ContractABI {
        ContractABI {
            methods: vec![
                MethodSignature {
                    name: "transfer".to_string(),
                    inputs: vec![
                        ParameterType::Address, // to
                        ParameterType::U128,    // amount
                        ParameterType::String,  // token (TokenType serialized)
                    ],
                    outputs: vec![ParameterType::Bool],
                    gas_cost: 5000,
                },
                MethodSignature {
                    name: "mint".to_string(),
                    inputs: vec![
                        ParameterType::Address, // to
                        ParameterType::U128,    // amount
                        ParameterType::String,  // token
                    ],
                    outputs: vec![ParameterType::Bool],
                    gas_cost: 10000,
                },
                MethodSignature {
                    name: "burn".to_string(),
                    inputs: vec![
                        ParameterType::Address, // from
                        ParameterType::U128,    // amount
                        ParameterType::String,  // token
                    ],
                    outputs: vec![ParameterType::Bool],
                    gas_cost: 5000,
                },
            ],
            events: vec![
                EventSignature {
                    name: "Transfer".to_string(),
                    inputs: vec![
                        ParameterType::Address, // from
                        ParameterType::Address, // to
                        ParameterType::U128,    // amount
                        ParameterType::String,  // token
                    ],
                },
            ],
        }
    }
}

impl TokenManagerContract {
    async fn transfer(&mut self, ctx: &ExecutionContext, data: &[u8]) -> Result<ContractResult> {
        #[derive(Serialize, Deserialize)]
        struct TransferArgs {
            to: Address,
            amount: u128,
            token: TokenType,
        }
        
        let args: TransferArgs = serde_json::from_slice(data)?;
        
        // Check Soul token transfer restriction
        if args.token == TokenType::Soul {
            return Err(anyhow!("Soul tokens are non-transferable"));
        }
        
        // Get current balances
        let from_balance = self.storage.load_token_balance(&ctx.contract_id, &ctx.caller, &args.token)?;
        
        if from_balance < args.amount {
            return Err(anyhow!("Insufficient balance"));
        }
        
        let to_balance = self.storage.load_token_balance(&ctx.contract_id, &args.to, &args.token)?;
        
        // Update balances
        self.storage.store_token_balance(&ctx.contract_id, &ctx.caller, &args.token, from_balance - args.amount);
        self.storage.store_token_balance(&ctx.contract_id, &args.to, &args.token, to_balance + args.amount);
        
        let events = vec![
            ContractEvent {
                contract_id: ctx.contract_id.clone(),
                event_type: "Transfer".to_string(),
                data: serde_json::to_vec(&args)?,
                topics: vec![ctx.caller.clone(), args.to.clone()],
            }
        ];
        
        let gas_used = self.gas_meter.calculate_gas_for_operation(&GasOperation::TokenTransfer);
        
        Ok(ContractResult {
            success: true,
            return_data: serde_json::to_vec(&true)?,
            gas_used,
            events,
            error: None,
        })
    }
    
    async fn mint(&mut self, ctx: &ExecutionContext, data: &[u8]) -> Result<ContractResult> {
        #[derive(Serialize, Deserialize)]
        struct MintArgs {
            to: Address,
            amount: u128,
            token: TokenType,
        }
        
        let args: MintArgs = serde_json::from_slice(data)?;
        
        // Only system can mint (for now)
        if ctx.caller != "system" {
            return Err(anyhow!("Only system can mint tokens"));
        }
        
        let current_balance = self.storage.load_token_balance(&ctx.contract_id, &args.to, &args.token)?;
        let current_supply = self.storage.load_total_supply(&ctx.contract_id, &args.token)?;
        
        // Update balance and supply
        self.storage.store_token_balance(&ctx.contract_id, &args.to, &args.token, current_balance + args.amount);
        self.storage.store_total_supply(&ctx.contract_id, &args.token, current_supply + args.amount);
        
        let gas_used = self.gas_meter.calculate_gas_for_operation(&GasOperation::TokenMint);
        
        Ok(ContractResult {
            success: true,
            return_data: serde_json::to_vec(&true)?,
            gas_used,
            events: vec![],
            error: None,
        })
    }
    
    async fn burn(&mut self, ctx: &ExecutionContext, data: &[u8]) -> Result<ContractResult> {
        #[derive(Serialize, Deserialize)]
        struct BurnArgs {
            from: Address,
            amount: u128,
            token: TokenType,
        }
        
        let args: BurnArgs = serde_json::from_slice(data)?;
        
        // Check authorization (only system or token owner)
        if ctx.caller != "system" && ctx.caller != args.from {
            return Err(anyhow!("Not authorized to burn tokens"));
        }
        
        let current_balance = self.storage.load_token_balance(&ctx.contract_id, &args.from, &args.token)?;
        
        if current_balance < args.amount {
            return Err(anyhow!("Insufficient balance to burn"));
        }
        
        let current_supply = self.storage.load_total_supply(&ctx.contract_id, &args.token)?;
        
        // Update balance and supply
        self.storage.store_token_balance(&ctx.contract_id, &args.from, &args.token, current_balance - args.amount);
        self.storage.store_total_supply(&ctx.contract_id, &args.token, current_supply - args.amount);
        
        let gas_used = self.gas_meter.calculate_gas_for_operation(&GasOperation::TokenBurn);
        
        Ok(ContractResult {
            success: true,
            return_data: serde_json::to_vec(&true)?,
            gas_used,
            events: vec![],
            error: None,
        })
    }
}