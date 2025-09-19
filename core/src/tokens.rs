// 4-Token Economy System for GhostChain
//
// GCC - Gas & transaction fees (utility)
// SPIRIT - Governance & voting (governance)
// MANA - Utility & rewards (incentive)
// GHOST - Brand & collectibles (brand/NFT)

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};
use tokio::sync::RwLock;
use std::sync::Arc;
use chrono::{DateTime, Utc};

/// Token types in the GhostChain ecosystem
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TokenType {
    GCC,    // Gas & transaction fees
    SPIRIT, // Governance & voting
    MANA,   // Utility & rewards
    GHOST,  // Brand & collectibles
}

impl TokenType {
    /// Get token decimals
    pub fn decimals(&self) -> u8 {
        match self {
            TokenType::GCC => 18,    // Standard ERC20 decimals
            TokenType::SPIRIT => 18,
            TokenType::MANA => 18,
            TokenType::GHOST => 0,    // NFT-like, no decimals
        }
    }

    /// Get token symbol
    pub fn symbol(&self) -> &str {
        match self {
            TokenType::GCC => "GCC",
            TokenType::SPIRIT => "SPIRIT",
            TokenType::MANA => "MANA",
            TokenType::GHOST => "GHOST",
        }
    }

    /// Get token name
    pub fn name(&self) -> &str {
        match self {
            TokenType::GCC => "GhostChain Coin",
            TokenType::SPIRIT => "Spirit Governance Token",
            TokenType::MANA => "Mana Utility Token",
            TokenType::GHOST => "Ghost Collectible Token",
        }
    }

    /// Check if token is transferable
    pub fn is_transferable(&self) -> bool {
        match self {
            TokenType::GCC => true,
            TokenType::SPIRIT => true,
            TokenType::MANA => true,
            TokenType::GHOST => false, // Soulbound by default
        }
    }
}

/// Token balance for an account
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TokenBalance {
    pub gcc: u128,
    pub spirit: u128,
    pub mana: u128,
    pub ghost: u128,
    pub locked_balances: LockedBalances,
}

impl TokenBalance {
    /// Get balance for specific token type
    pub fn get(&self, token_type: TokenType) -> u128 {
        match token_type {
            TokenType::GCC => self.gcc,
            TokenType::SPIRIT => self.spirit,
            TokenType::MANA => self.mana,
            TokenType::GHOST => self.ghost,
        }
    }

    /// Set balance for specific token type
    pub fn set(&mut self, token_type: TokenType, amount: u128) {
        match token_type {
            TokenType::GCC => self.gcc = amount,
            TokenType::SPIRIT => self.spirit = amount,
            TokenType::MANA => self.mana = amount,
            TokenType::GHOST => self.ghost = amount,
        }
    }

    /// Add to balance
    pub fn add(&mut self, token_type: TokenType, amount: u128) -> Result<()> {
        let current = self.get(token_type);
        let new_balance = current.checked_add(amount)
            .ok_or_else(|| anyhow!("Balance overflow"))?;
        self.set(token_type, new_balance);
        Ok(())
    }

    /// Subtract from balance
    pub fn subtract(&mut self, token_type: TokenType, amount: u128) -> Result<()> {
        let current = self.get(token_type);
        let new_balance = current.checked_sub(amount)
            .ok_or_else(|| anyhow!("Insufficient balance"))?;
        self.set(token_type, new_balance);
        Ok(())
    }

    /// Get available balance (total - locked)
    pub fn available(&self, token_type: TokenType) -> u128 {
        let total = self.get(token_type);
        let locked = self.locked_balances.get(token_type);
        total.saturating_sub(locked)
    }
}

/// Locked token balances (for staking, governance, etc.)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LockedBalances {
    pub gcc: u128,
    pub spirit: u128,
    pub mana: u128,
    pub ghost: u128,
}

impl LockedBalances {
    pub fn get(&self, token_type: TokenType) -> u128 {
        match token_type {
            TokenType::GCC => self.gcc,
            TokenType::SPIRIT => self.spirit,
            TokenType::MANA => self.mana,
            TokenType::GHOST => self.ghost,
        }
    }

    pub fn set(&mut self, token_type: TokenType, amount: u128) {
        match token_type {
            TokenType::GCC => self.gcc = amount,
            TokenType::SPIRIT => self.spirit = amount,
            TokenType::MANA => self.mana = amount,
            TokenType::GHOST => self.ghost = amount,
        }
    }
}

/// Token transfer record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenTransfer {
    pub from: crate::Address,
    pub to: crate::Address,
    pub token_type: TokenType,
    pub amount: u128,
    pub timestamp: DateTime<Utc>,
    pub transaction_hash: String,
    pub memo: Option<String>,
}

/// Token minting event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenMint {
    pub recipient: crate::Address,
    pub token_type: TokenType,
    pub amount: u128,
    pub reason: MintReason,
    pub timestamp: DateTime<Utc>,
    pub authority: crate::Address,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MintReason {
    Genesis,
    BlockReward,
    StakingReward,
    ManaReward,
    GovernanceReward,
    Custom(String),
}

/// Token burning event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenBurn {
    pub from: crate::Address,
    pub token_type: TokenType,
    pub amount: u128,
    pub reason: BurnReason,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BurnReason {
    GasFee,
    Penalty,
    Voluntary,
    Governance,
    Custom(String),
}

/// Token staking info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StakeInfo {
    pub staker: crate::Address,
    pub token_type: TokenType,
    pub amount: u128,
    pub locked_until: DateTime<Utc>,
    pub reward_rate: f64,
    pub accumulated_rewards: u128,
}

/// Token economics configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenEconomics {
    // Supply caps
    pub gcc_max_supply: Option<u128>,
    pub spirit_max_supply: Option<u128>,
    pub mana_max_supply: Option<u128>,
    pub ghost_max_supply: Option<u128>,

    // Current supplies
    pub gcc_current_supply: u128,
    pub spirit_current_supply: u128,
    pub mana_current_supply: u128,
    pub ghost_current_supply: u128,

    // Economics parameters
    pub gcc_inflation_rate: f64,
    pub spirit_governance_threshold: u128,
    pub mana_reward_multiplier: f64,
    pub ghost_rarity_tiers: Vec<u128>,

    // Gas pricing
    pub base_gas_price_gcc: u128,
    pub spirit_gas_discount: f64,  // Discount for SPIRIT holders
    pub mana_gas_cashback: f64,    // MANA cashback on gas fees
}

impl Default for TokenEconomics {
    fn default() -> Self {
        Self {
            // Initial supply caps (None = unlimited)
            gcc_max_supply: Some(1_000_000_000 * 10u128.pow(18)), // 1 billion GCC
            spirit_max_supply: Some(100_000_000 * 10u128.pow(18)), // 100 million SPIRIT
            mana_max_supply: None, // Unlimited MANA (earned through activity)
            ghost_max_supply: Some(10_000), // 10,000 unique GHOST NFTs

            // Start with zero supply (will be minted at genesis)
            gcc_current_supply: 0,
            spirit_current_supply: 0,
            mana_current_supply: 0,
            ghost_current_supply: 0,

            // Economics parameters
            gcc_inflation_rate: 0.02, // 2% annual inflation
            spirit_governance_threshold: 1000 * 10u128.pow(18), // 1000 SPIRIT to vote
            mana_reward_multiplier: 1.5, // 1.5x MANA rewards for activities
            ghost_rarity_tiers: vec![1, 10, 100, 1000], // Rarity levels

            // Gas pricing
            base_gas_price_gcc: 1_000_000_000, // 1 gwei in GCC
            spirit_gas_discount: 0.1, // 10% discount for SPIRIT holders
            mana_gas_cashback: 0.05, // 5% cashback in MANA
        }
    }
}

/// Main token system managing all 4 tokens
pub struct TokenSystem {
    balances: Arc<RwLock<HashMap<crate::Address, TokenBalance>>>,
    economics: Arc<RwLock<TokenEconomics>>,
    transfer_history: Arc<RwLock<Vec<TokenTransfer>>>,
    stakes: Arc<RwLock<HashMap<crate::Address, Vec<StakeInfo>>>>,
}

impl TokenSystem {
    pub fn new() -> Self {
        Self {
            balances: Arc::new(RwLock::new(HashMap::new())),
            economics: Arc::new(RwLock::new(TokenEconomics::default())),
            transfer_history: Arc::new(RwLock::new(Vec::new())),
            stakes: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Get balance for an address
    pub async fn get_balance(&self, address: &crate::Address) -> TokenBalance {
        let balances = self.balances.read().await;
        balances.get(address).cloned().unwrap_or_default()
    }

    /// Transfer tokens between addresses
    pub async fn transfer(
        &self,
        from: &crate::Address,
        to: &crate::Address,
        token_type: TokenType,
        amount: u128,
    ) -> Result<String> {
        // Check if token is transferable
        if !token_type.is_transferable() {
            return Err(anyhow!("{} tokens are not transferable", token_type.symbol()));
        }

        let mut balances = self.balances.write().await;

        // Get or create sender balance
        let sender_balance = balances.entry(from.clone()).or_default();

        // Check available balance
        if sender_balance.available(token_type) < amount {
            return Err(anyhow!("Insufficient {} balance", token_type.symbol()));
        }

        // Subtract from sender
        sender_balance.subtract(token_type, amount)?;

        // Add to recipient
        let recipient_balance = balances.entry(to.clone()).or_default();
        recipient_balance.add(token_type, amount)?;

        // Record transfer
        let transfer = TokenTransfer {
            from: from.clone(),
            to: to.clone(),
            token_type,
            amount,
            timestamp: Utc::now(),
            transaction_hash: format!("0x{}", hex::encode(rand::random::<[u8; 32]>())),
            memo: None,
        };

        let tx_hash = transfer.transaction_hash.clone();
        drop(balances);

        let mut history = self.transfer_history.write().await;
        history.push(transfer);

        Ok(tx_hash)
    }

    /// Mint new tokens
    pub async fn mint(
        &self,
        recipient: &crate::Address,
        token_type: TokenType,
        amount: u128,
        reason: MintReason,
        authority: &crate::Address,
    ) -> Result<()> {
        let mut economics = self.economics.write().await;

        // Check max supply
        let (max_supply, current_supply) = match token_type {
            TokenType::GCC => (economics.gcc_max_supply, &mut economics.gcc_current_supply),
            TokenType::SPIRIT => (economics.spirit_max_supply, &mut economics.spirit_current_supply),
            TokenType::MANA => (economics.mana_max_supply, &mut economics.mana_current_supply),
            TokenType::GHOST => (economics.ghost_max_supply, &mut economics.ghost_current_supply),
        };

        if let Some(max) = max_supply {
            if *current_supply + amount > max {
                return Err(anyhow!("Minting would exceed max supply for {}", token_type.symbol()));
            }
        }

        // Update current supply
        *current_supply += amount;
        drop(economics);

        // Add to recipient balance
        let mut balances = self.balances.write().await;
        let balance = balances.entry(recipient.clone()).or_default();
        balance.add(token_type, amount)?;

        Ok(())
    }

    /// Burn tokens
    pub async fn burn(
        &self,
        from: &crate::Address,
        token_type: TokenType,
        amount: u128,
        reason: BurnReason,
    ) -> Result<()> {
        let mut balances = self.balances.write().await;
        let balance = balances.entry(from.clone()).or_default();

        // Check balance
        if balance.available(token_type) < amount {
            return Err(anyhow!("Insufficient {} balance to burn", token_type.symbol()));
        }

        // Burn tokens
        balance.subtract(token_type, amount)?;
        drop(balances);

        // Update current supply
        let mut economics = self.economics.write().await;
        match token_type {
            TokenType::GCC => economics.gcc_current_supply = economics.gcc_current_supply.saturating_sub(amount),
            TokenType::SPIRIT => economics.spirit_current_supply = economics.spirit_current_supply.saturating_sub(amount),
            TokenType::MANA => economics.mana_current_supply = economics.mana_current_supply.saturating_sub(amount),
            TokenType::GHOST => economics.ghost_current_supply = economics.ghost_current_supply.saturating_sub(amount),
        }

        Ok(())
    }

    /// Stake tokens for rewards
    pub async fn stake(
        &self,
        staker: &crate::Address,
        token_type: TokenType,
        amount: u128,
        lock_duration_days: u64,
    ) -> Result<()> {
        // Only SPIRIT and GCC can be staked
        if token_type != TokenType::SPIRIT && token_type != TokenType::GCC {
            return Err(anyhow!("Only SPIRIT and GCC tokens can be staked"));
        }

        let mut balances = self.balances.write().await;
        let balance = balances.entry(staker.clone()).or_default();

        // Check available balance
        if balance.available(token_type) < amount {
            return Err(anyhow!("Insufficient {} balance to stake", token_type.symbol()));
        }

        // Lock the tokens
        balance.locked_balances.set(
            token_type,
            balance.locked_balances.get(token_type) + amount
        );
        drop(balances);

        // Create stake info
        let stake = StakeInfo {
            staker: staker.clone(),
            token_type,
            amount,
            locked_until: Utc::now() + chrono::Duration::days(lock_duration_days as i64),
            reward_rate: if token_type == TokenType::SPIRIT { 0.10 } else { 0.05 }, // 10% for SPIRIT, 5% for GCC
            accumulated_rewards: 0,
        };

        let mut stakes = self.stakes.write().await;
        stakes.entry(staker.clone()).or_default().push(stake);

        Ok(())
    }

    /// Calculate gas cost with token-based discounts
    pub async fn calculate_gas_cost(
        &self,
        sender: &crate::Address,
        base_gas: u128,
    ) -> Result<(u128, TokenType, u128)> {
        let economics = self.economics.read().await;
        let balance = self.get_balance(sender).await;

        let mut gas_cost = base_gas * economics.base_gas_price_gcc;
        let mut mana_cashback = 0u128;

        // Apply SPIRIT holder discount
        if balance.spirit > economics.spirit_governance_threshold {
            let discount = (gas_cost as f64 * economics.spirit_gas_discount) as u128;
            gas_cost = gas_cost.saturating_sub(discount);
        }

        // Calculate MANA cashback
        if balance.mana > 0 {
            mana_cashback = (gas_cost as f64 * economics.mana_gas_cashback) as u128;
        }

        Ok((gas_cost, TokenType::GCC, mana_cashback))
    }

    /// Distribute block rewards
    pub async fn distribute_block_rewards(
        &self,
        validator: &crate::Address,
        block_number: u64,
    ) -> Result<()> {
        // Base rewards
        let gcc_reward = 100 * 10u128.pow(18); // 100 GCC per block
        let mana_reward = 10 * 10u128.pow(18);  // 10 MANA per block

        // Mint rewards
        self.mint(validator, TokenType::GCC, gcc_reward, MintReason::BlockReward, validator).await?;
        self.mint(validator, TokenType::MANA, mana_reward, MintReason::BlockReward, validator).await?;

        // Every 1000 blocks, distribute SPIRIT to validators
        if block_number % 1000 == 0 {
            let spirit_reward = 1 * 10u128.pow(18); // 1 SPIRIT every 1000 blocks
            self.mint(validator, TokenType::SPIRIT, spirit_reward, MintReason::GovernanceReward, validator).await?;
        }

        Ok(())
    }

    /// Get economics configuration
    pub async fn get_economics(&self) -> TokenEconomics {
        self.economics.read().await.clone()
    }
}

impl Default for TokenSystem {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_token_transfer() {
        let system = TokenSystem::new();

        let alice = crate::Address::default();
        let bob = crate::Address::from([1u8; 32]);
        let authority = crate::Address::from([2u8; 32]);

        // Mint tokens to Alice
        system.mint(&alice, TokenType::GCC, 1000 * 10u128.pow(18), MintReason::Genesis, &authority).await.unwrap();

        // Transfer to Bob
        let tx_hash = system.transfer(&alice, &bob, TokenType::GCC, 100 * 10u128.pow(18)).await.unwrap();
        assert!(!tx_hash.is_empty());

        // Check balances
        let alice_balance = system.get_balance(&alice).await;
        let bob_balance = system.get_balance(&bob).await;

        assert_eq!(alice_balance.gcc, 900 * 10u128.pow(18));
        assert_eq!(bob_balance.gcc, 100 * 10u128.pow(18));
    }

    #[tokio::test]
    async fn test_token_staking() {
        let system = TokenSystem::new();

        let staker = crate::Address::default();
        let authority = crate::Address::from([1u8; 32]);

        // Mint SPIRIT tokens
        system.mint(&staker, TokenType::SPIRIT, 1000 * 10u128.pow(18), MintReason::Genesis, &authority).await.unwrap();

        // Stake tokens
        system.stake(&staker, TokenType::SPIRIT, 500 * 10u128.pow(18), 30).await.unwrap();

        // Check balance
        let balance = system.get_balance(&staker).await;
        assert_eq!(balance.spirit, 1000 * 10u128.pow(18));
        assert_eq!(balance.locked_balances.spirit, 500 * 10u128.pow(18));
        assert_eq!(balance.available(TokenType::SPIRIT), 500 * 10u128.pow(18));
    }

    #[tokio::test]
    async fn test_gas_calculation() {
        let system = TokenSystem::new();

        let user = crate::Address::default();
        let authority = crate::Address::from([1u8; 32]);

        // User with no tokens
        let (gas_cost, token, cashback) = system.calculate_gas_cost(&user, 21000).await.unwrap();
        assert_eq!(token, TokenType::GCC);
        assert_eq!(cashback, 0);

        // Give user SPIRIT tokens (above governance threshold)
        system.mint(&user, TokenType::SPIRIT, 2000 * 10u128.pow(18), MintReason::Genesis, &authority).await.unwrap();

        // Should get discount
        let (discounted_gas, _, _) = system.calculate_gas_cost(&user, 21000).await.unwrap();
        assert!(discounted_gas < gas_cost);
    }

    #[test]
    fn test_token_properties() {
        assert_eq!(TokenType::GCC.symbol(), "GCC");
        assert_eq!(TokenType::SPIRIT.decimals(), 18);
        assert!(TokenType::GCC.is_transferable());
        assert!(!TokenType::GHOST.is_transferable()); // Soulbound
    }
}