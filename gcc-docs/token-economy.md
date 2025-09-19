# 💰 GhostChain 4-Token Economy

> **Comprehensive guide to the GhostChain multi-token economic system**

The GhostChain 4-Token Economy is designed to create a balanced, sustainable ecosystem that incentivizes participation while providing utility across identity, governance, AI operations, and transaction processing.

---

## 🎯 **Economic Philosophy**

### **Core Principles**
- **Utility-Driven Value** - Each token has specific, essential use cases
- **Balanced Incentives** - Rewards for participation and contribution
- **Sustainable Growth** - Long-term viability over short-term speculation
- **User Sovereignty** - Minimal friction for legitimate operations
- **Network Effects** - Value increases with ecosystem adoption

### **Token Interactions**
The four tokens work synergistically to create a robust economic system where each token's value is enhanced by the utility of others.

---

## ⚡ **GCC (Ghost Compute Coin)**

### **Primary Functions**
- **Gas & Transaction Fees** - All network operations consume GCC
- **Computational Resources** - AI operations and smart contract execution
- **Network Security** - Anti-spam and DoS protection
- **Priority Access** - Higher fees for faster processing

### **Economic Model**
```
Supply Model: Deflationary
Initial Supply: 1,000,000,000 GCC
Burn Mechanism: 0.1% of transaction fees burned
Block Rewards: None (pre-mined, distributed over time)
```

### **Usage Examples**
```rust
// Transaction fees
let transfer_fee = calculate_gcc_fee(transaction_size, network_congestion);

// Smart contract deployment
let deployment_cost = base_fee + (contract_size * gcc_per_byte);

// AI operation pricing
let ai_cost = operation_complexity * gcc_rate_per_mflop;
```

### **Fee Structure**
| Operation | Base Cost (GCC) | Dynamic Multiplier |
|-----------|-----------------|-------------------|
| **Identity Creation** | 10 | 1.0x - 5.0x |
| **Domain Resolution** | 1 | 1.0x - 2.0x |
| **Token Transfer** | 5 | 1.0x - 10.0x |
| **Smart Contract Call** | 20 + gas | 1.0x - 20.0x |
| **Signature Verification** | 1 | 1.0x - 2.0x |

---

## 🗳️ **SPIRIT (Governance & Staking Token)**

### **Primary Functions**
- **Governance Participation** - Voting on protocol upgrades and parameters
- **Staking Rewards** - Earn rewards for network validation
- **Proposal Deposits** - Required for submitting governance proposals
- **Validator Bonds** - Collateral for network validators

### **Economic Model**
```
Supply Model: Fixed Supply with Staking Rewards
Total Supply: 100,000,000 SPIRIT
Staking APY: 5-15% (variable based on participation)
Governance Weight: 1 SPIRIT = 1 Vote
Minimum Proposal Deposit: 10,000 SPIRIT
```

### **Governance Framework**
```rust
#[derive(Debug, Serialize, Deserialize)]
pub struct GovernanceProposal {
    pub id: u64,
    pub title: String,
    pub description: String,
    pub proposer: String,           // DID
    pub deposit: TokenAmount,       // SPIRIT deposit
    pub voting_period: Duration,
    pub execution_delay: Duration,
    pub proposal_type: ProposalType,
}

pub enum ProposalType {
    ParameterChange { parameter: String, new_value: String },
    ProtocolUpgrade { code_hash: String },
    TreasurySpend { recipient: String, amount: TokenAmount },
    EmergencyAction { action: EmergencyActionType },
}
```

### **Staking Mechanics**
```rust
// Staking SPIRIT for rewards
let staking_result = gledger.stake_spirit(
    "did:ghost:alice",
    parse_token_amount("1000")?,
    StakingDuration::Days(30)
).await?;

// Governance voting with staked SPIRIT
let vote_result = governance.vote_on_proposal(
    proposal_id,
    VoteChoice::Yes,
    staked_spirit_balance
).await?;
```

### **Reward Distribution**
| Activity | Reward Rate | Requirements |
|----------|-------------|--------------|
| **Staking** | 5-15% APY | Min 100 SPIRIT |
| **Governance Participation** | 2% bonus | Vote on 80%+ proposals |
| **Validator Operation** | 10-20% APY | Min 50,000 SPIRIT bond |
| **Proposal Creation** | Variable | Successful proposals get 50% deposit back |

---

## ✨ **MANA (AI & Smart Contract Token)**

### **Primary Functions**
- **AI Operations** - Machine learning inference and training
- **Smart Contract Execution** - Complex computational operations
- **Oracle Services** - Data feed subscriptions and queries
- **Developer Incentives** - Rewards for creating valuable contracts

### **Economic Model**
```
Supply Model: Inflationary (AI-driven demand)
Initial Supply: 500,000,000 MANA
Inflation Rate: 3-7% annually (based on AI usage)
Burn Mechanism: 20% of AI operation fees burned
Oracle Rewards: 1% of supply annually to data providers
```

### **AI Operation Pricing**
```rust
#[derive(Debug)]
pub struct AIOperationCost {
    pub base_cost: TokenAmount,           // Base MANA cost
    pub compute_multiplier: f64,          // Based on operation complexity
    pub data_size_multiplier: f64,        // Based on input/output size
    pub model_complexity_multiplier: f64, // Based on model parameters
}

// Example AI pricing
let inference_cost = calculate_ai_cost(
    AIOperation::ImageClassification,
    model_size,
    input_data_size
);
```

### **Smart Contract Integration**
```rust
// Deploy contract with MANA payment
let contract_deployment = ContractDeployment {
    code: contract_bytecode,
    constructor_args: init_params,
    mana_payment: parse_token_amount("100")?,
    max_gas: 1_000_000,
};

// Execute AI-enabled smart contract
let ai_contract_call = ContractCall {
    contract_address: ai_contract_addr,
    method: "predict_price",
    args: market_data,
    mana_budget: parse_token_amount("50")?,
};
```

### **Developer Incentives**
| Contribution | Reward (MANA) | Frequency |
|--------------|---------------|-----------|
| **Popular Contract** | 1-100 per usage | Per transaction |
| **AI Model Training** | 1000-10000 | Per model |
| **Oracle Data Feed** | 10-100 per query | Per query |
| **Bug Bounty** | 100-10000 | Per valid bug |

---

## 👻 **GHOST (Identity & Domain Token)**

### **Primary Functions**
- **Identity Registration** - Create and maintain DID identities
- **Domain Registration** - Register and renew .ghost domains
- **Premium Features** - Enhanced privacy and priority services
- **Reputation System** - Stake GHOST for identity verification

### **Economic Model**
```
Supply Model: Burn-to-Mint (Deflationary with controlled inflation)
Initial Supply: 10,000,000 GHOST
Burn Rate: 90% of registration fees burned
Mint Mechanism: Burn other tokens to mint GHOST (limited rate)
Domain Renewal: 50 GHOST annually
Identity Premium: 100 GHOST for enhanced features
```

### **Domain Economics**
```rust
#[derive(Debug)]
pub struct DomainPricing {
    pub base_price: TokenAmount,      // 100 GHOST base
    pub length_multiplier: f64,       // Shorter = more expensive
    pub premium_multiplier: f64,      // Premium names cost more
    pub renewal_discount: f64,        // Loyalty discount
}

// Domain pricing examples
let domain_costs = vec![
    ("a.ghost", 10000),              // Single letter: 10,000 GHOST
    ("ai.ghost", 5000),              // Two letter: 5,000 GHOST
    ("alice.ghost", 100),            // Standard: 100 GHOST
    ("mycompany.ghost", 100),        // Standard: 100 GHOST
];
```

### **Identity Premium Features**
```rust
// Premium identity features unlocked with GHOST
let premium_features = PremiumIdentityFeatures {
    enhanced_privacy: true,           // Advanced ephemeral identities
    priority_support: true,           // Faster customer service
    custom_branding: true,            // Personalized identity appearance
    advanced_delegation: true,        // Complex delegation patterns
    multi_sig_support: true,          // Multiple signing keys
    backup_recovery: true,            // Enhanced recovery options
};
```

### **Burn-to-Mint Mechanism**
```rust
// Burn other tokens to mint GHOST (rate-limited)
let ghost_mint_rates = hashmap! {
    TokenType::GCC => 1000,      // 1000 GCC = 1 GHOST
    TokenType::SPIRIT => 100,    // 100 SPIRIT = 1 GHOST
    TokenType::MANA => 500,      // 500 MANA = 1 GHOST
};

// Execute burn-to-mint transaction
let mint_result = gledger.burn_for_ghost_mint(
    "did:ghost:alice",
    TokenType::GCC,
    parse_token_amount("1000")?,
    "Domain registration payment"
).await?;
```

---

## 🔄 **Token Interactions & Utility**

### **Cross-Token Operations**

```rust
// Multi-token operations demonstrate ecosystem synergy
let ecosystem_operation = EcosystemOperation {
    // GCC: Pay for transaction fees
    gcc_fee: parse_token_amount("5")?,

    // SPIRIT: Stake for governance participation
    spirit_stake: parse_token_amount("1000")?,

    // MANA: Execute AI-powered smart contract
    mana_ai_cost: parse_token_amount("50")?,

    // GHOST: Register premium identity
    ghost_identity_cost: parse_token_amount("100")?,
};
```

### **Economic Incentive Alignment**

| User Type | Primary Tokens | Incentive Mechanism |
|-----------|----------------|-------------------|
| **End Users** | GCC, GHOST | Low fees, premium features |
| **Developers** | MANA, SPIRIT | Contract rewards, governance |
| **Validators** | SPIRIT, GCC | Staking rewards, fee sharing |
| **AI Providers** | MANA, GCC | Usage rewards, compute fees |
| **Domain Traders** | GHOST, GCC | Domain appreciation, transfer fees |

---

## 📊 **Economic Analytics**

### **Token Metrics Tracking**
```rust
#[derive(Debug, Serialize)]
pub struct TokenEconomics {
    pub circulating_supply: HashMap<TokenType, TokenAmount>,
    pub total_burned: HashMap<TokenType, TokenAmount>,
    pub staking_ratio: f64,                    // % of SPIRIT staked
    pub transaction_volume: HashMap<TokenType, TokenAmount>,
    pub holder_distribution: Vec<BalanceRange>,
    pub cross_token_flows: Vec<TokenFlow>,
}

// Real-time economic indicators
let economic_health = EconomicHealthIndicators {
    token_velocity: calculate_token_velocity(),
    network_value_to_transactions: calculate_nvt_ratio(),
    staking_participation: calculate_staking_ratio(),
    burn_rate: calculate_burn_rate(),
    mint_rate: calculate_mint_rate(),
};
```

### **Supply Dynamics**

```
┌─────────────────────────────────────────────────────────────────┐
│                    Token Supply Overview                       │
├─────────────────┬─────────────────┬─────────────────────────────┤
│      GCC        │     SPIRIT      │      MANA       │   GHOST   │
│  Deflationary   │  Fixed Supply   │  Inflationary   │ Burn-Mint │
├─────────────────┼─────────────────┼─────────────────┼───────────┤
│ 1B → decreasing │    100M fixed   │ 500M → growing  │ 10M → var │
│ Tx fees burned  │ Staking rewards │ AI demand driven│ Domain reg│
│ 0.1% burn rate  │ 5-15% APY       │ 3-7% inflation  │ 90% burn  │
└─────────────────┴─────────────────┴─────────────────┴───────────┘
```

---

## 🎯 **Economic Governance**

### **Parameter Management**
Key economic parameters are governed by SPIRIT token holders:

```rust
// Governable economic parameters
let economic_parameters = EconomicParameters {
    gcc_base_fee: 1,                    // Base transaction fee
    spirit_staking_apy: 10.0,           // Staking reward rate
    mana_inflation_rate: 5.0,           // Annual inflation %
    ghost_burn_rate: 90.0,              // % of fees burned
    cross_token_exchange_rates: rates,  // Burn-to-mint rates
};

// Governance proposal to adjust parameters
let parameter_proposal = GovernanceProposal {
    proposal_type: ProposalType::ParameterChange {
        parameter: "gcc_base_fee".to_string(),
        new_value: "2".to_string(),        // Increase base fee
    },
    deposit: parse_token_amount("10000")?,  // 10,000 SPIRIT deposit
    // ... other fields
};
```

### **Emergency Economic Controls**
```rust
// Emergency controls for economic stability
let emergency_controls = EmergencyEconomicControls {
    fee_scaling: true,                  // Dynamic fee adjustment
    burn_rate_adjustment: true,         // Modify burn rates
    mint_rate_limiting: true,           // Control GHOST minting
    staking_reward_adjustment: true,    // Modify SPIRIT rewards
    transaction_rate_limiting: true,    // Network congestion control
};
```

---

## 💎 **Token Distribution & Allocation**

### **Initial Distribution**

| Token | Total Supply | Community | Team | Treasury | Ecosystem |
|-------|-------------|-----------|------|----------|-----------|
| **GCC** | 1,000,000,000 | 60% | 15% | 15% | 10% |
| **SPIRIT** | 100,000,000 | 50% | 20% | 20% | 10% |
| **MANA** | 500,000,000 | 55% | 15% | 20% | 10% |
| **GHOST** | 10,000,000 | 70% | 10% | 10% | 10% |

### **Vesting Schedules**
```rust
// Token vesting for different allocations
let vesting_schedules = VestingSchedules {
    team_tokens: VestingSchedule {
        cliff_duration: Duration::days(365),      // 1 year cliff
        vesting_duration: Duration::days(1460),   // 4 year total
        release_frequency: Duration::days(30),    // Monthly releases
    },
    treasury_tokens: VestingSchedule {
        cliff_duration: Duration::days(180),      // 6 month cliff
        vesting_duration: Duration::days(1095),   // 3 year total
        release_frequency: Duration::days(90),    // Quarterly releases
    },
    // Community tokens: No vesting (immediate utility)
    // Ecosystem tokens: Project-based vesting
};
```

---

## 🔮 **Economic Projections**

### **5-Year Economic Model**

```rust
// Projected token economics over 5 years
let economic_projections = EconomicProjections {
    year_1: TokenProjection {
        gcc_supply: 950_000_000,        // 5% burned
        spirit_staked: 70_000_000,      // 70% staking rate
        mana_supply: 515_000_000,       // 3% inflation
        ghost_supply: 8_000_000,        // Domain adoption
    },
    year_5: TokenProjection {
        gcc_supply: 750_000_000,        // 25% total burned
        spirit_staked: 85_000_000,      // 85% staking rate
        mana_supply: 580_000_000,       // AI growth driven
        ghost_supply: 15_000_000,       // Domain ecosystem maturity
    },
};
```

### **Network Value Drivers**
1. **Transaction Volume** - GCC utility drives base value
2. **Governance Participation** - SPIRIT voting weight attracts holders
3. **AI Innovation** - MANA enables cutting-edge applications
4. **Identity Adoption** - GHOST scarcity increases with user growth
5. **Cross-Token Synergies** - Ecosystem effects multiply individual values

---

## 🛡️ **Economic Security**

### **Anti-Manipulation Measures**
```rust
// Economic attack prevention
let security_measures = EconomicSecurityMeasures {
    whale_protection: WhaleProtection {
        max_single_vote_weight: 5.0,        // Max 5% of total votes
        voting_delay: Duration::hours(24),   // Delay for large stakes
        gradual_unstaking: true,             // Prevent dump attacks
    },
    market_manipulation: MarketProtection {
        max_burn_per_block: 1000,           // Limit burn rate manipulation
        mint_rate_limiting: true,            // Prevent mint flooding
        fee_spike_protection: true,          // Limit fee manipulation
    },
    flash_loan_protection: FlashLoanProtection {
        governance_vote_delay: Duration::days(7),  // Prevent flash governance
        staking_minimum_period: Duration::hours(24), // Min staking time
    },
};
```

### **Economic Circuit Breakers**
```rust
// Automatic protections during market stress
let circuit_breakers = CircuitBreakers {
    high_volatility_pause: true,           // Pause during extreme moves
    burn_rate_limits: true,                // Prevent over-burning
    mint_rate_limits: true,                // Prevent over-minting
    fee_escalation_caps: true,             // Cap maximum fees
    emergency_governance: true,            // Fast emergency actions
};
```

---

*💰 Building a sustainable, utility-driven token economy for the decentralized future*