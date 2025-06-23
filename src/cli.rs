use clap::{Parser, Subcommand};
use crate::types::*;
use crate::crypto::KeyPair;
use crate::blockchain::Blockchain;
use crate::token::TokenManager;
use crate::network::{NetworkNode, NetworkConfig};
use crate::consensus::{ConsensusModule, ConsensusConfig};
use crate::storage::Storage;
use anyhow::Result;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use std::net::SocketAddr;

#[derive(Parser)]
#[command(name = "ghostchain")]
#[command(about = "GhostChain CLI - A Web5 blockchain with Spirit tokens", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    Account {
        #[command(subcommand)]
        cmd: AccountCommands,
    },
    
    Token {
        #[command(subcommand)]
        cmd: TokenCommands,
    },
    
    Chain {
        #[command(subcommand)]
        cmd: ChainCommands,
    },
    
    Node {
        #[arg(short, long, default_value = "0.0.0.0:7777")]
        bind: String,
        
        #[arg(short, long)]
        validator_key: Option<String>,
        
        #[arg(long)]
        peer: Vec<String>,
        
        #[arg(long, default_value = "ghostchain-devnet")]
        chain_id: String,
        
        #[arg(long)]
        data_dir: Option<String>,
        
        #[arg(long, default_value = "8545")]
        rpc_port: u16,
    },
    
    Rpc {
        #[arg(short, long, default_value = "0.0.0.0:8545")]
        bind: String,
    },
}

#[derive(Subcommand)]
pub enum AccountCommands {
    New,
    
    Balance {
        #[arg(short, long)]
        address: String,
        
        #[arg(short, long, default_value = "spirit")]
        token: String,
    },
    
    Info {
        #[arg(short, long)]
        address: String,
    },
}

#[derive(Subcommand)]
pub enum TokenCommands {
    Transfer {
        #[arg(short, long)]
        from: String,
        
        #[arg(short, long)]
        to: String,
        
        #[arg(short, long)]
        amount: String,
        
        #[arg(long, default_value = "spirit")]
        token: String,
        
        #[arg(long)]
        private_key: String,
    },
    
    Stake {
        #[arg(short, long)]
        staker: String,
        
        #[arg(short, long)]
        amount: String,
        
        #[arg(long)]
        private_key: String,
    },
    
    List,
}

#[derive(Subcommand)]
pub enum ChainCommands {
    Info,
    
    Height,
    
    Block {
        #[arg(short, long)]
        height: u64,
    },
}

pub struct CliHandler {
    pub blockchain: Blockchain,
    pub token_manager: TokenManager,
}

impl CliHandler {
    pub fn new() -> Result<Self> {
        let mut genesis_config = GenesisConfig::default();
        
        let genesis_keypair = KeyPair::generate();
        let genesis_address = genesis_keypair.address();
        
        let mut genesis_balances = HashMap::new();
        genesis_balances.insert(TokenType::Spirit, 500_000_000 * 10u128.pow(18));
        genesis_balances.insert(TokenType::Rlusd, 50_000_000 * 10u128.pow(18));
        
        let genesis_account = Account {
            address: genesis_address.clone(),
            public_key: genesis_keypair.verifying_key.as_bytes().to_vec(),
            balances: genesis_balances,
            nonce: 0,
            soul_id: None,
            staked_amount: 0,
            mana_earned: 0,
        };
        
        genesis_config.genesis_accounts.push((genesis_address, genesis_account));
        
        let blockchain = Blockchain::new(genesis_config)?;
        let token_manager = TokenManager::new();
        
        Ok(Self {
            blockchain,
            token_manager,
        })
    }
    
    pub async fn handle_command(&mut self, cli: Cli) -> Result<()> {
        match cli.command {
            Commands::Account { cmd } => self.handle_account_command(cmd).await,
            Commands::Token { cmd } => self.handle_token_command(cmd).await,
            Commands::Chain { cmd } => self.handle_chain_command(cmd).await,
            Commands::Node { bind, validator_key, peer, chain_id, data_dir, rpc_port } => {
                self.handle_node_command(bind, validator_key, peer, chain_id, data_dir, rpc_port).await
            }
            
            Commands::Rpc { bind } => {
                self.handle_rpc_command(bind).await
            }
        }
    }
    
    async fn handle_account_command(&self, cmd: AccountCommands) -> Result<()> {
        match cmd {
            AccountCommands::New => {
                let keypair = KeyPair::generate();
                let address = keypair.address();
                
                println!("New GhostChain account created!");
                println!("Address: {}", address);
                println!("Private Key: {}", hex::encode(keypair.signing_key.as_bytes()));
                println!("Public Key: {}", hex::encode(keypair.verifying_key.as_bytes()));
                println!("\n⚠️  Save your private key securely! It cannot be recovered.");
            }
            
            AccountCommands::Balance { address, token } => {
                let token_type = self.parse_token_type(&token)?;
                let balance = self.blockchain.get_balance(&address, &token_type);
                let formatted = self.token_manager.format_amount(&token_type, balance);
                
                println!("Balance for {}: {}", address, formatted);
            }
            
            AccountCommands::Info { address } => {
                if let Some(account) = self.blockchain.get_account(&address) {
                    println!("Account Information:");
                    println!("Address: {}", account.address);
                    println!("Nonce: {}", account.nonce);
                    println!("Staked: {} SPR", account.staked_amount / 10u128.pow(18));
                    println!("MANA Earned: {} MNA", account.mana_earned / 10u128.pow(18));
                    
                    if let Some(soul_id) = &account.soul_id {
                        println!("Soul ID: {}", soul_id);
                    }
                    
                    println!("\nBalances:");
                    for (token, balance) in &account.balances {
                        let formatted = self.token_manager.format_amount(token, *balance);
                        println!("  {}", formatted);
                    }
                } else {
                    println!("Account not found: {}", address);
                }
            }
        }
        
        Ok(())
    }
    
    async fn handle_token_command(&mut self, cmd: TokenCommands) -> Result<()> {
        match cmd {
            TokenCommands::Transfer { from, to, amount, token, private_key: _ } => {
                let token_type = self.parse_token_type(&token)?;
                let amount_wei = self.token_manager.parse_amount(&token_type, &amount)?;
                
                let tx = self.token_manager.create_transfer_tx(
                    from.clone(),
                    to.clone(),
                    token_type,
                    amount_wei,
                    1_000_000_000,
                )?;
                
                self.blockchain.add_transaction(tx)?;
                
                println!("Transfer queued successfully!");
                println!("From: {} To: {}", from, to);
                println!("Amount: {}", self.token_manager.format_amount(&token_type, amount_wei));
            }
            
            TokenCommands::Stake { staker, amount, private_key: _ } => {
                let amount_wei = self.token_manager.parse_amount(&TokenType::Spirit, &amount)?;
                
                let tx = self.token_manager.create_stake_tx(
                    staker.clone(),
                    amount_wei,
                    1_000_000_000,
                )?;
                
                self.blockchain.add_transaction(tx)?;
                
                println!("Stake transaction queued!");
                println!("Staker: {}", staker);
                println!("Amount: {} SPR", amount);
            }
            
            TokenCommands::List => {
                println!("GhostChain Tokens:");
                println!("==================");
                
                for (_, config) in &self.token_manager.token_configs {
                    println!("\n{} ({})", config.name, config.symbol);
                    println!("  Type: {:?}", config.token_type);
                    println!("  Decimals: {}", config.decimals);
                    println!("  Transferable: {}", config.transferable);
                    
                    if config.total_supply > 0 {
                        println!("  Total Supply: {}", 
                            self.token_manager.format_amount(&config.token_type, config.total_supply));
                    }
                }
            }
        }
        
        Ok(())
    }
    
    async fn handle_chain_command(&self, cmd: ChainCommands) -> Result<()> {
        match cmd {
            ChainCommands::Info => {
                println!("GhostChain Information:");
                println!("======================");
                println!("Chain ID: {}", self.blockchain.config.chain_id);
                println!("Current Height: {}", self.blockchain.chain.len() - 1);
                println!("Current Epoch: {}", self.blockchain.state.current_epoch);
                println!("Block Time: {} ms", self.blockchain.config.block_time);
                println!("Epoch Length: {} blocks", self.blockchain.config.epoch_length);
                println!("Active Validators: {}", 
                    self.blockchain.state.validators.values()
                        .filter(|v| v.is_active)
                        .count());
                println!("Pending Transactions: {}", self.blockchain.pending_transactions.len());
            }
            
            ChainCommands::Height => {
                println!("{}", self.blockchain.chain.len() - 1);
            }
            
            ChainCommands::Block { height } => {
                if let Some(block) = self.blockchain.chain.get(height as usize) {
                    println!("Block #{}", block.height);
                    println!("Hash: {}", block.hash);
                    println!("Previous: {}", block.previous_hash);
                    println!("Timestamp: {}", block.timestamp);
                    println!("Validator: {}", block.validator);
                    println!("Transactions: {}", block.transactions.len());
                    println!("State Root: {}", block.state_root);
                } else {
                    println!("Block not found at height {}", height);
                }
            }
        }
        
        Ok(())
    }
    
    async fn handle_node_command(
        &self,
        bind: String,
        validator_key: Option<String>,
        peers: Vec<String>,
        chain_id: String,
        data_dir: Option<String>,
        rpc_port: u16,
    ) -> Result<()> {
        println!("🚀 Starting GhostChain node...");
        println!("Network: {}", bind);
        println!("Chain ID: {}", chain_id);
        
        // Setup storage
        let storage = if let Some(dir) = data_dir {
            println!("📂 Data directory: {}", dir);
            Storage::new(dir)?
        } else {
            println!("💾 Using in-memory storage");
            Storage::in_memory()?
        };
        
        // Initialize blockchain with storage
        let blockchain = Arc::new(RwLock::new(self.blockchain.clone()));
        
        // Setup consensus
        let consensus_config = ConsensusConfig::default();
        let consensus = ConsensusModule::new(blockchain.clone(), consensus_config);
        
        // Setup networking
        let bind_addr: SocketAddr = bind.parse()?;
        let peer_id = format!("ghost-{}", uuid::Uuid::new_v4());
        
        let network_config = NetworkConfig {
            listen_addr: bind_addr,
            peer_id: peer_id.clone(),
            max_peers: 50,
            chain_id: chain_id.clone(),
        };
        
        let network = NetworkNode::new(network_config, blockchain.clone()).await?;
        
        // Connect to initial peers
        for peer_addr in peers {
            let addr: SocketAddr = peer_addr.parse()?;
            println!("🔗 Connecting to peer: {}", addr);
            if let Err(e) = network.connect_to_peer(addr).await {
                println!("⚠️  Failed to connect to {}: {}", addr, e);
            }
        }
        
        // Start RPC server if needed
        println!("🌐 RPC server would start on port {}", rpc_port);
        
        if validator_key.is_some() {
            println!("⚡ Validator mode enabled");
        }
        
        println!("🎯 Node ID: {}", peer_id);
        println!("✅ GhostChain node is running!");
        
        // Start the network node (this will block)
        network.start().await?;
        
        Ok(())
    }
    
    async fn handle_rpc_command(&self, bind: String) -> Result<()> {
        println!("🌐 Starting GhostChain RPC server...");
        println!("✅ RPC server would run on {}", bind);
        println!("📖 RPC functionality is planned for future implementation");
        
        // Keep the server running placeholder
        loop {
            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        }
    }
    
    fn parse_token_type(&self, token: &str) -> Result<TokenType> {
        match token.to_lowercase().as_str() {
            "spirit" | "spr" => Ok(TokenType::Spirit),
            "mana" | "mna" => Ok(TokenType::Mana),
            "rlusd" => Ok(TokenType::Rlusd),
            "soul" => Ok(TokenType::Soul),
            _ => Err(anyhow::anyhow!("Unknown token type: {}", token)),
        }
    }
}