use anyhow::Result;
use jsonrpc_core::IoHandler;
use jsonrpc_derive::rpc;
use jsonrpc_http_server::{ServerBuilder, Server};
use jsonrpc_ws_server::{ServerBuilder as WsServerBuilder, Server as WsServer};
use std::sync::Arc;
use tokio::sync::RwLock;
use crate::blockchain::Blockchain;
use crate::blockchain::integration::BlockchainContractIntegration;
use crate::types::*;
use serde::{Serialize, Deserialize};
use std::net::SocketAddr;
use std::collections::HashMap;

pub mod auth;
use auth::{AuthMiddleware, AuthRequest, Permission};

#[rpc]
pub trait GhostChainRpc {
    #[rpc(name = "getBlockHeight", returns = "u64")]
    fn get_block_height(&self) -> jsonrpc_core::Result<u64>;
    
    #[rpc(name = "getBlock", returns = "Option<BlockInfo>")]
    fn get_block(&self, height: u64) -> jsonrpc_core::Result<Option<BlockInfo>>;
    
    #[rpc(name = "getLatestBlock", returns = "Option<BlockInfo>")]
    fn get_latest_block(&self) -> jsonrpc_core::Result<Option<BlockInfo>>;
    
    #[rpc(name = "getBalance", returns = "String")]
    fn get_balance(&self, address: String, token: String) -> jsonrpc_core::Result<String>;
    
    #[rpc(name = "getAccount", returns = "Option<AccountInfo>")]
    fn get_account(&self, address: String) -> jsonrpc_core::Result<Option<AccountInfo>>;
    
    #[rpc(name = "sendTransaction", returns = "String")]
    fn send_transaction(&self, tx: TransactionRequest) -> jsonrpc_core::Result<String>;
    
    #[rpc(name = "getTransaction", returns = "Option<TransactionInfo>")]
    fn get_transaction(&self, _tx_id: String) -> jsonrpc_core::Result<Option<TransactionInfo>>;
    
    #[rpc(name = "getValidators", returns = "Vec<ValidatorInfo>")]
    fn get_validators(&self) -> jsonrpc_core::Result<Vec<ValidatorInfo>>;
    
    #[rpc(name = "getChainInfo", returns = "ChainInfo")]
    fn get_chain_info(&self) -> jsonrpc_core::Result<ChainInfo>;
    
    // Contract-related endpoints
    #[rpc(name = "deployContract", returns = "ContractDeployResult")]
    fn deploy_contract(&self, request: DeployContractRequest) -> jsonrpc_core::Result<ContractDeployResult>;
    
    #[rpc(name = "callContract", returns = "ContractCallResult")]
    fn call_contract(&self, request: CallContractRequest) -> jsonrpc_core::Result<ContractCallResult>;
    
    #[rpc(name = "queryContract", returns = "ContractQueryResult")]
    fn query_contract(&self, request: QueryContractRequest) -> jsonrpc_core::Result<ContractQueryResult>;
    
    #[rpc(name = "getContract", returns = "Option<ContractInfo>")]
    fn get_contract(&self, contract_id: String) -> jsonrpc_core::Result<Option<ContractInfo>>;
    
    // Domain-related endpoints
    #[rpc(name = "registerDomain", returns = "DomainResult")]
    fn register_domain(&self, request: RegisterDomainRequest) -> jsonrpc_core::Result<DomainResult>;
    
    #[rpc(name = "resolveDomain", returns = "Option<DomainData>")]
    fn resolve_domain(&self, domain: String) -> jsonrpc_core::Result<Option<DomainData>>;
    
    #[rpc(name = "transferDomain", returns = "DomainResult")]
    fn transfer_domain(&self, request: TransferDomainRequest) -> jsonrpc_core::Result<DomainResult>;
    
    #[rpc(name = "setDomainRecord", returns = "DomainResult")]
    fn set_domain_record(&self, request: SetRecordRequest) -> jsonrpc_core::Result<DomainResult>;
    
    #[rpc(name = "getDomainsByOwner", returns = "Vec<String>")]
    fn get_domains_by_owner(&self, owner: String) -> jsonrpc_core::Result<Vec<String>>;
    
    // Authentication endpoints
    #[rpc(name = "createApiKey", returns = "String")]
    fn create_api_key(&self, request: CreateApiKeyRequest) -> jsonrpc_core::Result<String>;
    
    #[rpc(name = "revokeApiKey", returns = "bool")]
    fn revoke_api_key(&self, key_id: String) -> jsonrpc_core::Result<bool>;
    
    #[rpc(name = "revokeSession", returns = "bool")]
    fn revoke_session(&self, session_id: String) -> jsonrpc_core::Result<bool>;
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

// Contract-related structures
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeployContractRequest {
    pub deployer: String,
    pub contract_code: String, // hex-encoded bytes
    pub init_data: String,     // hex-encoded bytes
    pub gas_limit: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractDeployResult {
    pub contract_id: String,
    pub transaction_hash: String,
    pub gas_used: String,
    pub success: bool,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallContractRequest {
    pub caller: String,
    pub contract_id: String,
    pub method: String,
    pub data: String,    // hex-encoded bytes
    pub gas_limit: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractCallResult {
    pub transaction_hash: String,
    pub return_data: String, // hex-encoded bytes
    pub gas_used: String,
    pub success: bool,
    pub events: Vec<ContractEventInfo>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryContractRequest {
    pub contract_id: String,
    pub query: String,
    pub data: String, // hex-encoded bytes
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractQueryResult {
    pub return_data: String, // hex-encoded bytes
    pub success: bool,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractEventInfo {
    pub event_type: String,
    pub data: String, // hex-encoded bytes
    pub topics: Vec<String>,
}

// Domain-related structures
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterDomainRequest {
    pub domain: String,
    pub owner: String,
    pub records: Vec<DomainRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferDomainRequest {
    pub domain: String,
    pub new_owner: String,
    pub current_owner: String, // For authorization
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetRecordRequest {
    pub domain: String,
    pub owner: String, // For authorization
    pub record: DomainRecord,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainResult {
    pub transaction_hash: String,
    pub success: bool,
    pub gas_used: String,
    pub error: Option<String>,
}

// Authentication structures
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateApiKeyRequest {
    pub key_id: String,
    pub permissions: Vec<String>, // String representations of Permission enum
    pub expires_in_days: Option<u32>,
    pub rate_limit: Option<u32>,
}

pub struct RpcImpl {
    blockchain: Arc<RwLock<Blockchain>>,
    contract_integration: Arc<RwLock<BlockchainContractIntegration>>,
    auth: Arc<AuthMiddleware>,
}

impl RpcImpl {
    pub fn new(blockchain: Arc<RwLock<Blockchain>>) -> Self {
        let contract_integration = Arc::new(RwLock::new(
            BlockchainContractIntegration::new(blockchain.clone())
        ));
        let auth = Arc::new(AuthMiddleware::new(false)); // Auth disabled by default for development
        
        Self { 
            blockchain,
            contract_integration,
            auth,
        }
    }
    
    pub fn new_with_auth(blockchain: Arc<RwLock<Blockchain>>, require_auth: bool) -> Self {
        let contract_integration = Arc::new(RwLock::new(
            BlockchainContractIntegration::new(blockchain.clone())
        ));
        let auth = Arc::new(AuthMiddleware::new(require_auth));
        
        Self { 
            blockchain,
            contract_integration,
            auth,
        }
    }
    
    fn convert_error(e: anyhow::Error) -> jsonrpc_core::Error {
        jsonrpc_core::Error::internal_error()
    }
    
    async fn check_auth(&self, method: &str, auth_request: Option<AuthRequest>) -> Result<(), jsonrpc_core::Error> {
        let required_permission = Permission::get_required_permission_for_method(method);
        let auth_req = auth_request.unwrap_or(AuthRequest {
            api_key: None,
            session_token: None,
        });
        
        match self.auth.authenticate(&auth_req, &required_permission).await {
            Ok(result) => {
                if result.authenticated {
                    Ok(())
                } else {
                    Err(jsonrpc_core::Error::invalid_request())
                }
            }
            Err(e) => Err(jsonrpc_core::Error::invalid_request())
        }
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
    
    fn get_block(&self, height: u64) -> jsonrpc_core::Result<Option<BlockInfo>> {
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
    
    fn get_latest_block(&self) -> jsonrpc_core::Result<Option<BlockInfo>> {
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
    
    fn get_balance(&self, address: String, token: String) -> jsonrpc_core::Result<String> {
        let blockchain = tokio::task::block_in_place(|| {
            let rt = tokio::runtime::Handle::current();
            rt.block_on(self.blockchain.read())
        });
        
        let token_type = match token.as_str() {
            "SPIRIT" | "SPR" => TokenType::Spirit,
            "MANA" | "MNA" => TokenType::Mana,
            "RLUSD" => TokenType::Rlusd,
            "SOUL" => TokenType::Soul,
            _ => return Err(Self::convert_error(anyhow::anyhow!("Unknown token type"))),
        };
        
        let balance = blockchain.get_balance(&address, &token_type);
        Ok(balance.to_string())
    }
    
    fn get_account(&self, address: String) -> jsonrpc_core::Result<Option<AccountInfo>> {
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
    
    fn send_transaction(&self, tx: TransactionRequest) -> jsonrpc_core::Result<String> {
        let token_type = match tx.token.as_str() {
            "SPIRIT" | "SPR" => TokenType::Spirit,
            "MANA" | "MNA" => TokenType::Mana,
            "RLUSD" => TokenType::Rlusd,
            "SOUL" => TokenType::Soul,
            _ => return Err(Self::convert_error(anyhow::anyhow!("Unknown token type"))),
        };
        
        let amount = tx.amount.parse::<u128>()
            .map_err(|_| Self::convert_error(anyhow::anyhow!("Invalid amount")))?;
        let gas_price = tx.gas_price.parse::<u128>()
            .map_err(|_| Self::convert_error(anyhow::anyhow!("Invalid gas price")))?;
        
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
        }).map_err(Self::convert_error)?;
        
        Ok(tx_id)
    }
    
    fn get_transaction(&self, tx_id: String) -> jsonrpc_core::Result<Option<TransactionInfo>> {
        // In a real implementation, this would look up the transaction
        // For now, return None as we don't have transaction indexing yet
        Ok(None)
    }
    
    fn get_validators(&self) -> jsonrpc_core::Result<Vec<ValidatorInfo>> {
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
    
    fn get_chain_info(&self) -> jsonrpc_core::Result<ChainInfo> {
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
    
    // Contract methods
    fn deploy_contract(&self, request: DeployContractRequest) -> jsonrpc_core::Result<ContractDeployResult> {
        let contract_code = hex::decode(&request.contract_code)
            .map_err(|e| Self::convert_error(anyhow::anyhow!("Invalid hex in contract_code: {}", e)))?;
        let init_data = hex::decode(&request.init_data)
            .map_err(|e| Self::convert_error(anyhow::anyhow!("Invalid hex in init_data: {}", e)))?;
        let gas_limit = request.gas_limit.parse::<u128>()
            .map_err(|e| Self::convert_error(anyhow::anyhow!("Invalid gas_limit: {}", e)))?;
        
        let result = tokio::task::block_in_place(|| {
            let rt = tokio::runtime::Handle::current();
            rt.block_on(async {
                let mut integration = self.contract_integration.write().await;
                integration.deploy_contract(
                    &request.deployer,
                    &contract_code,
                    &init_data,
                    gas_limit,
                ).await
            })
        }).map_err(Self::convert_error)?;
        
        Ok(ContractDeployResult {
            contract_id: "deployed_contract_id".to_string(), // TODO: Get actual contract ID
            transaction_hash: "0xdeploytx".to_string(), // TODO: Generate actual tx hash
            gas_used: result.gas_used.to_string(),
            success: result.success,
            error: result.error,
        })
    }
    
    fn call_contract(&self, request: CallContractRequest) -> jsonrpc_core::Result<ContractCallResult> {
        let data = hex::decode(&request.data)
            .map_err(|e| Self::convert_error(anyhow::anyhow!("Invalid hex in data: {}", e)))?;
        let gas_limit = request.gas_limit.parse::<u128>()
            .map_err(|e| Self::convert_error(anyhow::anyhow!("Invalid gas_limit: {}", e)))?;
        
        let result = tokio::task::block_in_place(|| {
            let rt = tokio::runtime::Handle::current();
            rt.block_on(async {
                let mut integration = self.contract_integration.write().await;
                integration.call_contract(
                    &request.caller,
                    &request.contract_id,
                    &request.method,
                    &data,
                    gas_limit,
                ).await
            })
        }).map_err(Self::convert_error)?;
        
        let events = result.events.iter().map(|e| ContractEventInfo {
            event_type: e.event_type.clone(),
            data: hex::encode(&e.data),
            topics: e.topics.clone(),
        }).collect();
        
        Ok(ContractCallResult {
            transaction_hash: "0xcalltx".to_string(), // TODO: Generate actual tx hash
            return_data: hex::encode(&result.return_data),
            gas_used: result.gas_used.to_string(),
            success: result.success,
            events,
            error: result.error,
        })
    }
    
    fn query_contract(&self, request: QueryContractRequest) -> jsonrpc_core::Result<ContractQueryResult> {
        let data = hex::decode(&request.data)
            .map_err(|e| Self::convert_error(anyhow::anyhow!("Invalid hex in data: {}", e)))?;
        
        let result = tokio::task::block_in_place(|| {
            let rt = tokio::runtime::Handle::current();
            rt.block_on(async {
                let integration = self.contract_integration.read().await;
                integration.query_contract(
                    &request.contract_id,
                    &request.query,
                    &data,
                ).await
            })
        });
        
        match result {
            Ok(return_data) => Ok(ContractQueryResult {
                return_data: hex::encode(&return_data),
                success: true,
                error: None,
            }),
            Err(e) => Ok(ContractQueryResult {
                return_data: String::new(),
                success: false,
                error: Some(e.to_string()),
            }),
        }
    }
    
    fn get_contract(&self, contract_id: String) -> jsonrpc_core::Result<Option<ContractInfo>> {
        let blockchain = tokio::task::block_in_place(|| {
            let rt = tokio::runtime::Handle::current();
            rt.block_on(self.blockchain.read())
        });
        
        Ok(blockchain.state.contracts.get(&contract_id).cloned())
    }
    
    // Domain methods
    fn register_domain(&self, request: RegisterDomainRequest) -> jsonrpc_core::Result<DomainResult> {
        let result = tokio::task::block_in_place(|| {
            let rt = tokio::runtime::Handle::current();
            rt.block_on(async {
                let mut integration = self.contract_integration.write().await;
                integration.register_domain(
                    &request.domain,
                    &request.owner,
                    request.records,
                ).await
            })
        }).map_err(Self::convert_error)?;
        
        Ok(DomainResult {
            transaction_hash: "0xdomaintx".to_string(), // TODO: Generate actual tx hash
            success: result.success,
            gas_used: result.gas_used.to_string(),
            error: result.error,
        })
    }
    
    fn resolve_domain(&self, domain: String) -> jsonrpc_core::Result<Option<DomainData>> {
        let result = tokio::task::block_in_place(|| {
            let rt = tokio::runtime::Handle::current();
            rt.block_on(async {
                let integration = self.contract_integration.read().await;
                integration.resolve_domain(&domain).await
            })
        });
        
        match result {
            Ok(domain_data) => Ok(Some(domain_data)),
            Err(_) => Ok(None),
        }
    }
    
    fn transfer_domain(&self, request: TransferDomainRequest) -> jsonrpc_core::Result<DomainResult> {
        // TODO: Implement domain transfer via contract call
        // For now, return a placeholder result
        Ok(DomainResult {
            transaction_hash: "0xtransfertx".to_string(),
            success: true,
            gas_used: "30000".to_string(),
            error: None,
        })
    }
    
    fn set_domain_record(&self, request: SetRecordRequest) -> jsonrpc_core::Result<DomainResult> {
        // TODO: Implement record setting via contract call
        // For now, return a placeholder result
        Ok(DomainResult {
            transaction_hash: "0xrecordtx".to_string(),
            success: true,
            gas_used: "10000".to_string(),
            error: None,
        })
    }
    
    fn get_domains_by_owner(&self, owner: String) -> jsonrpc_core::Result<Vec<String>> {
        let result: Result<Vec<u8>> = tokio::task::block_in_place(|| {
            let rt = tokio::runtime::Handle::current();
            rt.block_on(async {
                let integration = self.contract_integration.read().await;
                let query_data = serde_json::to_vec(&serde_json::json!({
                    "owner": owner
                })).unwrap();

                integration.query_contract(
                    &"system.domain_registry".to_string(),
                    "get_owner_domains",
                    &query_data,
                ).await
            })
        });

        match result {
            Ok(data) => {
                let domains: Vec<String> = serde_json::from_slice(&data)
                    .unwrap_or_default();
                Ok(domains)
            }
            Err(_) => Ok(vec![]),
        }
    }
    
    // Authentication methods
    fn create_api_key(&self, request: CreateApiKeyRequest) -> jsonrpc_core::Result<String> {
        // This should require admin permissions in practice
        let permissions: Vec<Permission> = request.permissions.iter()
            .filter_map(|p| match p.as_str() {
                "ReadBlockchain" => Some(Permission::ReadBlockchain),
                "ReadAccounts" => Some(Permission::ReadAccounts),
                "ReadContracts" => Some(Permission::ReadContracts),
                "ReadDomains" => Some(Permission::ReadDomains),
                "SendTransactions" => Some(Permission::SendTransactions),
                "DeployContracts" => Some(Permission::DeployContracts),
                "CallContracts" => Some(Permission::CallContracts),
                "RegisterDomains" => Some(Permission::RegisterDomains),
                "ManageValidators" => Some(Permission::ManageValidators),
                "ManageApiKeys" => Some(Permission::ManageApiKeys),
                "ManageSystem" => Some(Permission::ManageSystem),
                "FullAccess" => Some(Permission::FullAccess),
                _ => None,
            })
            .collect();
        
        let raw_key = uuid::Uuid::new_v4().to_string(); // Generate a random API key
        
        let result = tokio::task::block_in_place(|| {
            let rt = tokio::runtime::Handle::current();
            rt.block_on(async {
                self.auth.create_api_key(
                    request.key_id,
                    &raw_key,
                    permissions,
                    request.expires_in_days,
                    request.rate_limit,
                ).await
            })
        }).map_err(Self::convert_error)?;
        
        // In production, you'd want to return the raw key securely
        // For now, we'll return it directly (this is insecure!)
        Ok(format!("{}:{}", result, raw_key))
    }
    
    fn revoke_api_key(&self, key_id: String) -> jsonrpc_core::Result<bool> {
        let result = tokio::task::block_in_place(|| {
            let rt = tokio::runtime::Handle::current();
            rt.block_on(async {
                self.auth.revoke_api_key(&key_id).await
            })
        });
        
        match result {
            Ok(_) => Ok(true),
            Err(e) => Err(Self::convert_error(e)),
        }
    }
    
    fn revoke_session(&self, session_id: String) -> jsonrpc_core::Result<bool> {
        let result = tokio::task::block_in_place(|| {
            let rt = tokio::runtime::Handle::current();
            rt.block_on(async {
                self.auth.revoke_session(&session_id).await
            })
        });
        
        match result {
            Ok(_) => Ok(true),
            Err(e) => Err(Self::convert_error(e)),
        }
    }
}

pub struct RpcServer {
    http_server: Option<Server>,
    ws_server: Option<WsServer>,
}

impl RpcServer {
    pub fn new(blockchain: Arc<RwLock<Blockchain>>, http_addr: SocketAddr) -> Result<Self> {
        let mut io = IoHandler::new();
        let rpc_impl = RpcImpl::new(blockchain);
        io.extend_with(rpc_impl.to_delegate());
        
        let http_server = ServerBuilder::new(io.clone())
            .start_http(&http_addr)?;
        
        Ok(Self {
            http_server: Some(http_server),
            ws_server: None,
        })
    }
    
    pub fn new_with_websocket(
        blockchain: Arc<RwLock<Blockchain>>, 
        http_addr: SocketAddr,
        ws_addr: SocketAddr
    ) -> Result<Self> {
        let mut io = IoHandler::new();
        let rpc_impl = RpcImpl::new(blockchain);
        io.extend_with(rpc_impl.to_delegate());
        
        let http_server = ServerBuilder::new(io.clone())
            .start_http(&http_addr)?;
            
        let ws_server = WsServerBuilder::new(io)
            .start(&ws_addr)?;
        
        Ok(Self {
            http_server: Some(http_server),
            ws_server: Some(ws_server),
        })
    }
    
    pub fn close(mut self) {
        if let Some(server) = self.http_server.take() {
            server.close();
        }
        if let Some(server) = self.ws_server.take() {
            server.close();
        }
    }
}