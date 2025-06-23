use anyhow::Result;
use jsonrpc_core::IoHandler;
use jsonrpc_derive::rpc;
use jsonrpc_http_server::{ServerBuilder, Server};
use std::sync::Arc;
use tokio::sync::RwLock;
use crate::blockchain::Blockchain;
use crate::types::*;
use serde::{Serialize, Deserialize};
use std::net::SocketAddr;
use std::collections::HashMap;

#[rpc]
pub trait GhostChainRpc {
    #[rpc(name = "getBlockHeight")]
    fn get_block_height(&self) -> jsonrpc_core::Result<u64>;
    
    #[rpc(name = "getBlock")]
    fn get_block(&self, height: u64) -> jsonrpc_core::Result<Option<BlockInfo>>;
    
    #[rpc(name = "getLatestBlock")]
    fn get_latest_block(&self) -> jsonrpc_core::Result<Option<BlockInfo>>;
    
    #[rpc(name = "getBalance")]
    fn get_balance(&self, address: String, token: String) -> jsonrpc_core::Result<String>;
    
    #[rpc(name = "getAccount")]
    fn get_account(&self, address: String) -> jsonrpc_core::Result<Option<AccountInfo>>;
    
    #[rpc(name = "sendTransaction")]
    fn send_transaction(&self, tx: TransactionRequest) -> jsonrpc_core::Result<String>;
    
    #[rpc(name = "getTransaction")]
    fn get_transaction(&self, _tx_id: String) -> jsonrpc_core::Result<Option<TransactionInfo>>;
    
    #[rpc(name = "getValidators")]
    fn get_validators(&self) -> jsonrpc_core::Result<Vec<ValidatorInfo>>;
    
    #[rpc(name = "getChainInfo")]
    fn get_chain_info(&self) -> jsonrpc_core::Result<ChainInfo>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockInfo {
    pub height: u64,
    pub hash: String,
    pub previous_hash: String,
    pub timestamp: String,
    pub validator: String,
    pub transaction_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountInfo {
    pub address: String,
    pub balances: HashMap<String, String>,
    pub nonce: u64,
    pub soul_id: Option<String>,
    pub staked_amount: String,
    pub mana_earned: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionRequest {
    pub from: String,
    pub to: String,
    pub token: String,
    pub amount: String,
    pub gas_price: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionInfo {
    pub id: String,
    pub tx_type: String,
    pub timestamp: String,
    pub gas_price: String,
    pub gas_used: String,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainInfo {
    pub chain_id: String,
    pub height: u64,
    pub total_supply: HashMap<String, String>,
    pub current_epoch: u64,
    pub validator_count: usize,
}

pub struct RpcImpl {
    blockchain: Arc<RwLock<Blockchain>>,
}

impl RpcImpl {
    pub fn new(blockchain: Arc<RwLock<Blockchain>>) -> Self {
        Self { blockchain }
    }
    
    fn convert_error(e: anyhow::Error) -> jsonrpc_core::Error {
        jsonrpc_core::Error::internal_error_with_data(e.to_string(), None)
    }
}

impl GhostChainRpc for RpcImpl {
    fn get_block_height(&self) -> jsonrpc_core::Result<u64> {
        let blockchain = tokio::task::block_in_place(|| {
            let rt = tokio::runtime::Handle::current();
            rt.block_on(self.blockchain.read())
        });
        Ok(blockchain.chain.len() as u64 - 1)
    }
    
    fn get_block(&self, height: u64) -> Result<Option<BlockInfo>> {
        let blockchain = tokio::task::block_in_place(|| {
            let rt = tokio::runtime::Handle::current();
            rt.block_on(self.blockchain.read())
        });
        
        let block = blockchain.chain.get(height as usize).map(|b| BlockInfo {
            height: b.height,
            hash: b.hash.clone(),
            previous_hash: b.previous_hash.clone(),
            timestamp: b.timestamp.to_rfc3339(),
            validator: b.validator.clone(),
            transaction_count: b.transactions.len(),
        });
        
        Ok(block)
    }
    
    fn get_latest_block(&self) -> Result<Option<BlockInfo>> {
        let blockchain = tokio::task::block_in_place(|| {
            let rt = tokio::runtime::Handle::current();
            rt.block_on(self.blockchain.read())
        });
        
        let block = blockchain.chain.last().map(|b| BlockInfo {
            height: b.height,
            hash: b.hash.clone(),
            previous_hash: b.previous_hash.clone(),
            timestamp: b.timestamp.to_rfc3339(),
            validator: b.validator.clone(),
            transaction_count: b.transactions.len(),
        });
        
        Ok(block)
    }
    
    fn get_balance(&self, address: String, token: String) -> Result<String> {
        let blockchain = tokio::task::block_in_place(|| {
            let rt = tokio::runtime::Handle::current();
            rt.block_on(self.blockchain.read())
        });
        
        let token_type = match token.as_str() {
            "SPIRIT" | "SPR" => TokenType::Spirit,
            "MANA" | "MNA" => TokenType::Mana,
            "RLUSD" => TokenType::Rlusd,
            "SOUL" => TokenType::Soul,
            _ => return Err(anyhow::anyhow!("Unknown token type")),
        };
        
        let balance = blockchain.get_balance(&address, &token_type);
        Ok(balance.to_string())
    }
    
    fn get_account(&self, address: String) -> Result<Option<AccountInfo>> {
        let blockchain = tokio::task::block_in_place(|| {
            let rt = tokio::runtime::Handle::current();
            rt.block_on(self.blockchain.read())
        });
        
        let account = blockchain.get_account(&address).map(|acc| {
            let mut balances = HashMap::new();
            for (token, amount) in &acc.balances {
                let token_name = match token {
                    TokenType::Spirit => "SPIRIT",
                    TokenType::Mana => "MANA",
                    TokenType::Rlusd => "RLUSD",
                    TokenType::Soul => "SOUL",
                };
                balances.insert(token_name.to_string(), amount.to_string());
            }
            
            AccountInfo {
                address: acc.address.clone(),
                balances,
                nonce: acc.nonce,
                soul_id: acc.soul_id.map(|id| id.to_string()),
                staked_amount: acc.staked_amount.to_string(),
                mana_earned: acc.mana_earned.to_string(),
            }
        });
        
        Ok(account)
    }
    
    fn send_transaction(&self, tx: TransactionRequest) -> Result<String> {
        let token_type = match tx.token.as_str() {
            "SPIRIT" | "SPR" => TokenType::Spirit,
            "MANA" | "MNA" => TokenType::Mana,
            "RLUSD" => TokenType::Rlusd,
            "SOUL" => TokenType::Soul,
            _ => return Err(anyhow::anyhow!("Unknown token type")),
        };
        
        let amount = tx.amount.parse::<u128>()
            .map_err(|_| anyhow::anyhow!("Invalid amount"))?;
        let gas_price = tx.gas_price.parse::<u128>()
            .map_err(|_| anyhow::anyhow!("Invalid gas price"))?;
        
        let transaction = Transaction {
            id: uuid::Uuid::new_v4(),
            tx_type: TransactionType::Transfer {
                from: tx.from,
                to: tx.to,
                token: token_type,
                amount,
            },
            timestamp: chrono::Utc::now(),
            signature: None,
            gas_price,
            gas_used: 21000, // Basic transfer gas
        };
        
        let tx_id = transaction.id.to_string();
        
        tokio::task::block_in_place(|| {
            let rt = tokio::runtime::Handle::current();
            rt.block_on(async {
                let mut blockchain = self.blockchain.write().await;
                blockchain.add_transaction(transaction)
            })
        })?;
        
        Ok(tx_id)
    }
    
    fn get_transaction(&self, tx_id: String) -> Result<Option<TransactionInfo>> {
        // In a real implementation, this would look up the transaction
        // For now, return None as we don't have transaction indexing yet
        Ok(None)
    }
    
    fn get_validators(&self) -> Result<Vec<ValidatorInfo>> {
        let blockchain = tokio::task::block_in_place(|| {
            let rt = tokio::runtime::Handle::current();
            rt.block_on(self.blockchain.read())
        });
        
        let validators: Vec<ValidatorInfo> = blockchain.state.validators
            .values()
            .cloned()
            .collect();
        
        Ok(validators)
    }
    
    fn get_chain_info(&self) -> Result<ChainInfo> {
        let blockchain = tokio::task::block_in_place(|| {
            let rt = tokio::runtime::Handle::current();
            rt.block_on(self.blockchain.read())
        });
        
        let mut total_supply = HashMap::new();
        for (token, amount) in &blockchain.state.total_supply {
            let token_name = match token {
                TokenType::Spirit => "SPIRIT",
                TokenType::Mana => "MANA",
                TokenType::Rlusd => "RLUSD",
                TokenType::Soul => "SOUL",
            };
            total_supply.insert(token_name.to_string(), amount.to_string());
        }
        
        Ok(ChainInfo {
            chain_id: blockchain.config.chain_id.clone(),
            height: blockchain.chain.len() as u64 - 1,
            total_supply,
            current_epoch: blockchain.state.current_epoch,
            validator_count: blockchain.state.validators.len(),
        })
    }
}

pub struct RpcServer {
    server: Option<Server>,
}

impl RpcServer {
    pub fn new(blockchain: Arc<RwLock<Blockchain>>, addr: SocketAddr) -> Result<Self> {
        let mut io = IoHandler::new();
        let rpc_impl = RpcImpl::new(blockchain);
        io.extend_with(rpc_impl.to_delegate());
        
        let server = ServerBuilder::new(io)
            .start_http(&addr)?;
        
        Ok(Self {
            server: Some(server),
        })
    }
    
    pub fn close(mut self) {
        if let Some(server) = self.server.take() {
            server.close();
        }
    }
}