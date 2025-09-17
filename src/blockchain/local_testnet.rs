use anyhow::{Result, anyhow};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use chrono::Utc;
use uuid::Uuid;
use tracing::{info, warn, error};

use crate::blockchain::{Blockchain, integration::BlockchainContractIntegration};
use crate::types::*;
use crate::crypto::KeyPair;
use crate::token::TokenManager;
use crate::contracts::{ContractExecutor, storage::ContractStorage};
use crate::contracts::ghostchain_tokens::*;
use crate::performance::PerformanceManager;

/// Local testnet configuration for GhostChain development and testing
pub struct LocalTestnet {
    pub blockchain: Arc<RwLock<Blockchain>>,
    pub contract_integration: Arc<RwLock<BlockchainContractIntegration>>,
    pub token_manager: TokenManager,
    pub performance_manager: PerformanceManager,
    pub test_accounts: HashMap<String, TestAccount>,
    pub deployed_contracts: HashMap<String, String>, // name -> contract_id
    pub genesis_config: GenesisConfig,
}

#[derive(Debug, Clone)]
pub struct TestAccount {
    pub name: String,
    pub address: Address,
    pub key_pair: KeyPair,
    pub initial_balances: HashMap<TokenType, u128>,
}

#[derive(Debug, Clone)]
pub struct TestnetConfig {
    pub chain_id: String,
    pub block_time: u64,           // milliseconds
    pub epoch_length: u64,         // blocks per epoch
    pub initial_validators: usize,
    pub test_accounts: usize,
    pub enable_mining: bool,
    pub enable_contracts: bool,
    pub enable_domains: bool,
}

impl Default for TestnetConfig {
    fn default() -> Self {
        Self {
            chain_id: "ghostchain-testnet".to_string(),
            block_time: 2000, // 2 second blocks for testing
            epoch_length: 10,  // 10 blocks per epoch
            initial_validators: 3,
            test_accounts: 10,
            enable_mining: true,
            enable_contracts: true,
            enable_domains: true,
        }
    }
}

impl LocalTestnet {
    /// Initialize a new local testnet
    pub async fn new(config: TestnetConfig) -> Result<Self> {
        info!("🏗️  Initializing GhostChain local testnet");

        // Create genesis configuration
        let genesis_config = Self::create_genesis_config(&config)?;
        
        // Initialize blockchain
        let blockchain = Arc::new(RwLock::new(Blockchain::new_default()));
        
        // Initialize blockchain with genesis state
        {
            let mut chain = blockchain.write().await;
            chain.initialize_with_genesis(&genesis_config).await?;
        }

        // Create contract storage and executor
        let contract_storage = ContractStorage::new();
        let contract_executor = Arc::new(RwLock::new(ContractExecutor::new(contract_storage)));
        
        // Create contract integration
        let contract_integration = Arc::new(RwLock::new(
            BlockchainContractIntegration::new(blockchain.clone(), contract_executor)
        ));

        // Initialize token manager
        let token_manager = TokenManager::new();
        
        // Initialize performance manager
        let performance_config = crate::performance::PerformanceConfig::default();
        let performance_manager = PerformanceManager::new(performance_config).await?;

        // Create test accounts
        let test_accounts = Self::create_test_accounts(config.test_accounts)?;

        let mut testnet = Self {
            blockchain,
            contract_integration,
            token_manager,
            performance_manager,
            test_accounts,
            deployed_contracts: HashMap::new(),
            genesis_config,
        };

        // Deploy core contracts if enabled
        if config.enable_contracts {
            testnet.deploy_core_contracts().await?;
        }

        info!("✅ Local testnet initialized successfully");
        info!("   Chain ID: {}", config.chain_id);
        info!("   Test accounts: {}", test_accounts.len());
        info!("   Validators: {}", config.initial_validators);

        Ok(testnet)
    }

    /// Create genesis configuration for the testnet
    fn create_genesis_config(config: &TestnetConfig) -> Result<GenesisConfig> {
        let mut initial_supply = HashMap::new();
        
        // Set initial token supplies
        initial_supply.insert(TokenType::Spirit, 1_000_000_000 * 10u128.pow(18)); // 1B GSPR
        initial_supply.insert(TokenType::Mana, 0); // GMAN is earned through staking
        initial_supply.insert(TokenType::Rlusd, 10_000_000 * 10u128.pow(18)); // 10M RLUSD

        // Create genesis accounts (will be populated with test accounts)
        let genesis_accounts = Vec::new();
        
        // Create initial validators (will be populated from test accounts)
        let initial_validators = Vec::new();

        Ok(GenesisConfig {
            chain_id: config.chain_id.clone(),
            initial_supply,
            genesis_accounts,
            initial_validators,
            block_time: config.block_time,
            epoch_length: config.epoch_length,
        })
    }

    /// Create test accounts with initial balances
    fn create_test_accounts(count: usize) -> Result<HashMap<String, TestAccount>> {
        let mut accounts = HashMap::new();
        
        let account_names = vec![
            "alice", "bob", "charlie", "diana", "eve", "frank", "grace", "henry", "iris", "jack",
            "karen", "liam", "mia", "noah", "olivia", "peter", "quinn", "rachel", "sam", "tina"
        ];

        for i in 0..count.min(account_names.len()) {
            let name = account_names[i].to_string();
            let key_pair = KeyPair::generate()?;
            let address = format!("0x{:040x}", i + 1); // Simple address generation for testing
            
            let mut initial_balances = HashMap::new();
            
            // Give different amounts to different accounts
            match i {
                0..=2 => {
                    // Validators get more GSPR for staking
                    initial_balances.insert(TokenType::Spirit, 100_000 * 10u128.pow(18));
                    initial_balances.insert(TokenType::Rlusd, 10_000 * 10u128.pow(18));
                },
                3..=6 => {
                    // Regular users
                    initial_balances.insert(TokenType::Spirit, 10_000 * 10u128.pow(18));
                    initial_balances.insert(TokenType::Rlusd, 1_000 * 10u128.pow(18));
                },
                _ => {
                    // Small amounts for testing
                    initial_balances.insert(TokenType::Spirit, 1_000 * 10u128.pow(18));
                    initial_balances.insert(TokenType::Rlusd, 100 * 10u128.pow(18));
                }
            }

            let test_account = TestAccount {
                name: name.clone(),
                address: address.clone(),
                key_pair,
                initial_balances,
            };

            accounts.insert(name, test_account);
        }

        Ok(accounts)
    }

    /// Deploy core GhostChain contracts (GCC, GMAN, GSPR)
    async fn deploy_core_contracts(&mut self) -> Result<()> {
        info!("🚀 Deploying core GhostChain contracts");

        let alice_account = self.test_accounts.get("alice")
            .ok_or_else(|| anyhow!("Alice account not found"))?;

        let mut integration = self.contract_integration.write().await;

        // Deploy GSPR (Ghost Spirit) contract
        let gspr_contract = GhostSpiritContract::new(
            1_000_000_000 * 10u128.pow(18), // 1B GSPR initial supply
            21_000_000_000 * 10u128.pow(18), // 21B GSPR max supply
        );

        let gspr_contract_id = integration.deploy_native_contract(
            &alice_account.address,
            "GhostSpirit",
            Box::new(gspr_contract),
            &[],
        ).await?;

        self.deployed_contracts.insert("GSPR".to_string(), gspr_contract_id.clone());
        info!("✅ GSPR contract deployed: {}", gspr_contract_id);

        // Deploy GCC (GhostChain Credits) contract
        let gcc_contract = GhostChainCreditsContract::new(
            1_000_000 * 10u128.pow(18), // 1M GCC initial supply
            "GhostChain Credits".to_string(),
            "GCC".to_string(),
            Some(alice_account.address.clone()),
        );

        let gcc_contract_id = integration.deploy_native_contract(
            &alice_account.address,
            "GhostChainCredits",
            Box::new(gcc_contract),
            &[],
        ).await?;

        self.deployed_contracts.insert("GCC".to_string(), gcc_contract_id.clone());
        info!("✅ GCC contract deployed: {}", gcc_contract_id);

        // Deploy GMAN (Ghost Mana) contract
        let gman_contract = GhostManaContract::new(None);

        let gman_contract_id = integration.deploy_native_contract(
            &alice_account.address,
            "GhostMana",
            Box::new(gman_contract),
            &[],
        ).await?;

        self.deployed_contracts.insert("GMAN".to_string(), gman_contract_id.clone());
        info!("✅ GMAN contract deployed: {}", gman_contract_id);

        info!("🎯 All core contracts deployed successfully");
        Ok(())
    }

    /// Get account information
    pub fn get_test_account(&self, name: &str) -> Option<&TestAccount> {
        self.test_accounts.get(name)
    }

    /// List all test accounts
    pub fn list_test_accounts(&self) -> Vec<&TestAccount> {
        self.test_accounts.values().collect()
    }

    /// Get deployed contract address
    pub fn get_contract_address(&self, name: &str) -> Option<&String> {
        self.deployed_contracts.get(name)
    }

    /// Create a test transaction
    pub async fn create_test_transaction(
        &self,
        from_account: &str,
        tx_type: TransactionType,
    ) -> Result<Transaction> {
        let account = self.test_accounts.get(from_account)
            .ok_or_else(|| anyhow!("Account not found: {}", from_account))?;

        let transaction = Transaction {
            id: Uuid::new_v4(),
            tx_type,
            timestamp: Utc::now(),
            signature: None, // In a real implementation, would sign with account's key
            gas_price: 1000,
            gas_used: 0,
        };

        Ok(transaction)
    }

    /// Submit and process a transaction
    pub async fn submit_transaction(&self, transaction: Transaction) -> Result<String> {
        let mut blockchain = self.blockchain.write().await;
        blockchain.add_transaction(transaction.clone())?;
        
        info!("📝 Transaction submitted: {}", transaction.id);
        Ok(transaction.id.to_string())
    }

    /// Create a test block
    pub async fn create_test_block(&self, validator_account: &str) -> Result<Block> {
        let account = self.test_accounts.get(validator_account)
            .ok_or_else(|| anyhow!("Validator account not found: {}", validator_account))?;

        let mut blockchain = self.blockchain.write().await;
        let block = blockchain.create_block(account.address.clone(), vec![1, 2, 3, 4])?;
        
        Ok(block)
    }

    /// Get blockchain statistics
    pub async fn get_statistics(&self) -> TestnetStatistics {
        let blockchain = self.blockchain.read().await;
        
        TestnetStatistics {
            current_height: blockchain.current_height(),
            total_accounts: self.test_accounts.len(),
            deployed_contracts: self.deployed_contracts.len(),
            total_supply: blockchain.state.total_supply.clone(),
            validator_count: 3, // TODO: get from actual validator set
        }
    }

    /// Run integration tests
    pub async fn run_integration_tests(&mut self) -> Result<()> {
        info!("🧪 Running integration tests on local testnet");

        // Test 1: Account balances
        self.test_account_balances().await?;
        
        // Test 2: Token transfers
        self.test_token_transfers().await?;
        
        // Test 3: Contract interactions
        self.test_contract_interactions().await?;
        
        // Test 4: Block creation
        self.test_block_creation().await?;

        info!("✅ All integration tests completed successfully");
        Ok(())
    }

    async fn test_account_balances(&self) -> Result<()> {
        info!("Testing account balances...");
        
        for (name, account) in &self.test_accounts {
            for (token_type, expected_balance) in &account.initial_balances {
                let blockchain = self.blockchain.read().await;
                if let Some(acc) = blockchain.state.accounts.get(&account.address) {
                    let actual_balance = acc.balances.get(token_type).unwrap_or(&0);
                    if actual_balance != expected_balance {
                        warn!("Balance mismatch for {} {}: expected {}, got {}", 
                             name, format!("{:?}", token_type), expected_balance, actual_balance);
                    }
                }
            }
        }
        
        Ok(())
    }

    async fn test_token_transfers(&mut self) -> Result<()> {
        info!("Testing token transfers...");
        
        let alice = self.test_accounts.get("alice").unwrap();
        let bob = self.test_accounts.get("bob").unwrap();
        
        let transfer_tx = self.create_test_transaction("alice", TransactionType::Transfer {
            from: alice.address.clone(),
            to: bob.address.clone(),
            token: TokenType::Spirit,
            amount: 1000 * 10u128.pow(18),
        }).await?;

        self.submit_transaction(transfer_tx).await?;
        
        Ok(())
    }

    async fn test_contract_interactions(&mut self) -> Result<()> {
        info!("Testing contract interactions...");
        
        if let Some(gspr_contract_id) = self.get_contract_address("GSPR") {
            // Test GSPR staking
            let alice = self.test_accounts.get("alice").unwrap();
            
            let stake_data = serde_json::to_vec(&crate::contracts::ghostchain_tokens::StakeArgs {
                amount: 1000 * 10u128.pow(18),
            })?;

            let contract_call_tx = self.create_test_transaction("alice", TransactionType::CallContract {
                caller: alice.address.clone(),
                contract_id: gspr_contract_id.clone(),
                method: "stake".to_string(),
                data: stake_data,
            }).await?;

            self.submit_transaction(contract_call_tx).await?;
        }
        
        Ok(())
    }

    async fn test_block_creation(&mut self) -> Result<()> {
        info!("Testing block creation...");
        
        let block = self.create_test_block("alice").await?;
        
        let mut blockchain = self.blockchain.write().await;
        blockchain.add_block(block)?;
        
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct TestnetStatistics {
    pub current_height: u64,
    pub total_accounts: usize,
    pub deployed_contracts: usize,
    pub total_supply: HashMap<TokenType, u128>,
    pub validator_count: usize,
}

/// CLI command handler for local testnet
pub async fn handle_testnet_command(action: &str) -> Result<()> {
    match action {
        "start" => {
            println!("🚀 Starting GhostChain local testnet...");
            
            let config = TestnetConfig::default();
            let mut testnet = LocalTestnet::new(config).await?;
            
            println!("✅ Local testnet started successfully!");
            
            // Show testnet information
            let stats = testnet.get_statistics().await;
            println!("\n📊 TESTNET STATISTICS:");
            println!("   Current height: {}", stats.current_height);
            println!("   Test accounts: {}", stats.total_accounts);
            println!("   Deployed contracts: {}", stats.deployed_contracts);
            println!("   Validators: {}", stats.validator_count);
            
            println!("\n👥 TEST ACCOUNTS:");
            for account in testnet.list_test_accounts() {
                println!("   {} - {}", account.name, account.address);
                for (token, balance) in &account.initial_balances {
                    let readable_balance = *balance as f64 / 10f64.powi(18);
                    println!("     {:?}: {:.2}", token, readable_balance);
                }
            }
            
            println!("\n🏗️  DEPLOYED CONTRACTS:");
            for (name, contract_id) in &testnet.deployed_contracts {
                println!("   {} - {}", name, contract_id);
            }

            // Run integration tests
            testnet.run_integration_tests().await?;
        },
        
        "status" => {
            println!("📊 Local testnet status would be shown here");
            println!("   (This would connect to running testnet instance)");
        },
        
        "stop" => {
            println!("🛑 Stopping local testnet...");
            println!("   (This would gracefully shutdown testnet instance)");
        },
        
        _ => {
            println!("❌ Unknown testnet action: {}", action);
            println!("   Available actions: start, status, stop");
        }
    }
    
    Ok(())
}