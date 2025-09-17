use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;
use chrono::{DateTime, Utc};

pub type Address = String;
pub type Hash = String;
pub type ContractId = String;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum TokenType {
    Spirit,     
    Mana,       
    Rlusd,      
    Soul,       
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Account {
    pub address: Address,
    pub public_key: Vec<u8>,
    pub balances: HashMap<TokenType, u128>,
    pub nonce: u64,
    pub soul_id: Option<Uuid>,
    pub staked_amount: u128,
    pub mana_earned: u128,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransactionType {
    Transfer {
        from: Address,
        to: Address,
        token: TokenType,
        amount: u128,
    },
    CreateAccount {
        address: Address,
        public_key: Vec<u8>,
    },
    Stake {
        staker: Address,
        amount: u128,
    },
    Unstake {
        staker: Address,
        amount: u128,
    },
    MintSoul {
        recipient: Address,
        soul_id: Uuid,
        metadata: HashMap<String, String>,
    },
    ContributeProof {
        contributor: Address,
        proof_type: String,
        mana_reward: u128,
    },
    DeployContract {
        deployer: Address,
        contract_code: Vec<u8>,
        init_data: Vec<u8>,
    },
    CallContract {
        caller: Address,
        contract_id: ContractId,
        method: String,
        data: Vec<u8>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    pub id: Uuid,
    pub tx_type: TransactionType,
    pub timestamp: DateTime<Utc>,
    pub signature: Option<Vec<u8>>,
    pub gas_price: u128,
    pub gas_used: u128,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Block {
    pub height: u64,
    pub hash: Hash,
    pub previous_hash: Hash,
    pub timestamp: DateTime<Utc>,
    pub transactions: Vec<Transaction>,
    pub validator: Address,
    pub state_root: Hash,
    pub signature: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainState {
    pub accounts: HashMap<Address, Account>,
    pub total_supply: HashMap<TokenType, u128>,
    pub validators: HashMap<Address, ValidatorInfo>,
    pub current_epoch: u64,
    pub contracts: HashMap<ContractId, ContractInfo>,
    pub domains: HashMap<String, DomainData>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidatorInfo {
    pub address: Address,
    pub staked_amount: u128,
    pub is_active: bool,
    pub commission_rate: f64,
    pub delegators: HashMap<Address, u128>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenesisConfig {
    pub chain_id: String,
    pub initial_supply: HashMap<TokenType, u128>,
    pub genesis_accounts: Vec<(Address, Account)>,
    pub initial_validators: Vec<ValidatorInfo>,
    pub block_time: u64,
    pub epoch_length: u64,
}

impl Default for GenesisConfig {
    fn default() -> Self {
        let mut initial_supply = HashMap::new();
        initial_supply.insert(TokenType::Spirit, 1_000_000_000 * 10u128.pow(18));
        initial_supply.insert(TokenType::Rlusd, 100_000_000 * 10u128.pow(18)); 
        
        Self {
            chain_id: "ghostchain-mainnet".to_string(),
            initial_supply,
            genesis_accounts: Vec::new(),
            initial_validators: Vec::new(),
            block_time: 6000,
            epoch_length: 100,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractInfo {
    pub id: ContractId,
    pub deployer: Address,
    pub code: Vec<u8>,
    pub state: HashMap<String, Vec<u8>>,
    pub created_at: DateTime<Utc>,
    pub gas_used: u128,
    pub contract_type: ContractType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ContractType {
    Native,
    Wasm,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractCall {
    pub contract_id: ContractId,
    pub caller: Address,
    pub method: String,
    pub data: Vec<u8>,
    pub gas_limit: u128,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractResult {
    pub success: bool,
    pub return_data: Vec<u8>,
    pub gas_used: u128,
    pub events: Vec<ContractEvent>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractEvent {
    pub contract_id: ContractId,
    pub event_type: String,
    pub data: Vec<u8>,
    pub topics: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainRecord {
    pub record_type: String,
    pub name: String,
    pub value: String,
    pub ttl: u32,
    pub priority: Option<u16>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainData {
    pub domain: String,
    pub owner: Address,
    pub records: Vec<DomainRecord>,
    pub contract_address: Option<ContractId>,
    pub last_updated: u64,
    pub expiry: Option<u64>,
    pub signature: Vec<u8>,
}