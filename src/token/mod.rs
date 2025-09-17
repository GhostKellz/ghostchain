use crate::types::*;
use anyhow::{Result, anyhow};
use std::collections::HashMap;
use uuid::Uuid;
use chrono::Utc;

pub const SPIRIT_DECIMALS: u8 = 18;
pub const MANA_DECIMALS: u8 = 18;
pub const RLUSD_DECIMALS: u8 = 18;

pub struct TokenManager {
    pub token_configs: HashMap<TokenType, TokenConfig>,
}

#[derive(Debug, Clone)]
pub struct TokenConfig {
    pub token_type: TokenType,
    pub name: String,
    pub symbol: String,
    pub decimals: u8,
    pub total_supply: u128,
    pub max_supply: Option<u128>,
    pub transferable: bool,
}

impl TokenManager {
    pub fn new() -> Self {
        let mut token_configs = HashMap::new();
        
        token_configs.insert(TokenType::Spirit, TokenConfig {
            token_type: TokenType::Spirit,
            name: "Spirit Token".to_string(),
            symbol: "SPR".to_string(),
            decimals: SPIRIT_DECIMALS,
            total_supply: 1_000_000_000 * 10u128.pow(SPIRIT_DECIMALS as u32),
            max_supply: Some(1_000_000_000 * 10u128.pow(SPIRIT_DECIMALS as u32)),
            transferable: true,
        });
        
        token_configs.insert(TokenType::Mana, TokenConfig {
            token_type: TokenType::Mana,
            name: "Mana Token".to_string(),
            symbol: "MNA".to_string(),
            decimals: MANA_DECIMALS,
            total_supply: 0,
            max_supply: None,
            transferable: true,
        });
        
        token_configs.insert(TokenType::Rlusd, TokenConfig {
            token_type: TokenType::Rlusd,
            name: "Ripple USD".to_string(),
            symbol: "RLUSD".to_string(),
            decimals: RLUSD_DECIMALS,
            total_supply: 100_000_000 * 10u128.pow(RLUSD_DECIMALS as u32),
            max_supply: None,
            transferable: true,
        });
        
        token_configs.insert(TokenType::Soul, TokenConfig {
            token_type: TokenType::Soul,
            name: "Soul Bound Token".to_string(),
            symbol: "SOUL".to_string(),
            decimals: 0,
            total_supply: 0,
            max_supply: None,
            transferable: false,
        });
        
        Self { token_configs }
    }
    
    pub fn create_transfer_tx(
        &self,
        from: Address,
        to: Address,
        token: TokenType,
        amount: u128,
        gas_price: u128,
    ) -> Result<Transaction> {
        let config = self.token_configs.get(&token)
            .ok_or_else(|| anyhow!("Token type not found"))?;
        
        if !config.transferable {
            return Err(anyhow!("Token {} is non-transferable", config.symbol));
        }
        
        Ok(Transaction {
            id: Uuid::new_v4(),
            tx_type: TransactionType::Transfer {
                from,
                to,
                token,
                amount,
            },
            timestamp: Utc::now(),
            signature: None,
            gas_price,
            gas_used: 21000,
        })
    }
    
    pub fn create_stake_tx(
        &self,
        staker: Address,
        amount: u128,
        gas_price: u128,
    ) -> Result<Transaction> {
        Ok(Transaction {
            id: Uuid::new_v4(),
            tx_type: TransactionType::Stake {
                staker,
                amount,
            },
            timestamp: Utc::now(),
            signature: None,
            gas_price,
            gas_used: 25000,
        })
    }
    
    pub fn create_soul_mint_tx(
        &self,
        recipient: Address,
        metadata: HashMap<String, String>,
        gas_price: u128,
    ) -> Result<Transaction> {
        Ok(Transaction {
            id: Uuid::new_v4(),
            tx_type: TransactionType::MintSoul {
                recipient,
                soul_id: Uuid::new_v4(),
                metadata,
            },
            timestamp: Utc::now(),
            signature: None,
            gas_price,
            gas_used: 50000,
        })
    }
    
    pub fn create_contribution_proof_tx(
        &self,
        contributor: Address,
        proof_type: String,
        mana_reward: u128,
        gas_price: u128,
    ) -> Result<Transaction> {
        Ok(Transaction {
            id: Uuid::new_v4(),
            tx_type: TransactionType::ContributeProof {
                contributor,
                proof_type,
                mana_reward,
            },
            timestamp: Utc::now(),
            signature: None,
            gas_price,
            gas_used: 30000,
        })
    }
    
    pub fn format_amount(&self, token: &TokenType, amount: u128) -> String {
        let config = &self.token_configs[token];
        let divisor = 10u128.pow(config.decimals as u32);
        let whole = amount / divisor;
        let fraction = amount % divisor;
        
        if fraction == 0 {
            format!("{} {}", whole, config.symbol)
        } else {
            let fraction_str = format!("{:0width$}", fraction, width = config.decimals as usize);
            let trimmed = fraction_str.trim_end_matches('0');
            format!("{}.{} {}", whole, trimmed, config.symbol)
        }
    }
    
    pub fn parse_amount(&self, token: &TokenType, amount_str: &str) -> Result<u128> {
        let config = &self.token_configs[token];
        let parts: Vec<&str> = amount_str.split('.').collect();
        
        if parts.len() > 2 {
            return Err(anyhow!("Invalid amount format"));
        }
        
        let whole: u128 = parts[0].parse()?;
        let mut result = whole * 10u128.pow(config.decimals as u32);
        
        if parts.len() == 2 {
            let fraction_str = parts[1];
            if fraction_str.len() > config.decimals as usize {
                return Err(anyhow!("Too many decimal places"));
            }
            
            let padded = format!("{:0<width$}", fraction_str, width = config.decimals as usize);
            let fraction: u128 = padded[..config.decimals as usize].parse()?;
            result += fraction;
        }
        
        Ok(result)
    }
}