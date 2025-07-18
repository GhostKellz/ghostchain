# 🪙 GhostChain Token System Design

## Multi-Token Ecosystem

GhostChain features a **four-token system** for comprehensive utility:

### 1. **GCC (GhostChain Coin)** - Native Currency 💰
- **Symbol**: GCC 
- **Decimals**: 18
- **Purpose**: Native gas token & transaction fees
- **Supply**: Dynamic (minted via block rewards)
- **Use Cases**: Network fees, validator staking

### 2. **SPIRIT** - Governance Token 👻  
- **Symbol**: SPIRIT
- **Decimals**: 18
- **Purpose**: Governance & voting rights
- **Supply**: Fixed supply with time-locked release
- **Use Cases**: DAO governance, protocol upgrades, treasury decisions

### 3. **MANA** - Utility Token ⚡
- **Symbol**: MANA
- **Decimals**: 18  
- **Purpose**: DApp interactions & premium features
- **Supply**: Dynamic (minted for ecosystem rewards)
- **Use Cases**: Smart contract execution, premium services, rewards

### 4. **GHOST** - Brand Token 👻
- **Symbol**: GHOST (👻)
- **Decimals**: 18
- **Purpose**: Brand identity & special features
- **Supply**: Limited edition minting
- **Use Cases**: Collectibles, special domains (.ghost), exclusive features

### Core Functions

## Core Token Functions

#### 1. **GCC Operations (Gas & Fees)**
```zig
pub fn payGasFee(from: Address, gcc_amount: u64) !void
pub fn getGCCBalance(address: Address) u64
pub fn transferGCC(from: Address, to: Address, amount: u64) !void
```

#### 2. **SPIRIT Operations (Governance)**
```zig
pub fn getSPIRITBalance(address: Address) u64
pub fn delegateVotingPower(from: Address, to: Address, amount: u64) !void
pub fn createProposal(proposer: Address, spirit_required: u64) !ProposalId
```

#### 3. **MANA Operations (Utility)**
```zig
pub fn getMANABalance(address: Address) u64
pub fn spendMANA(user: Address, service: ServiceType, amount: u64) !void
pub fn earnMANA(user: Address, action: ActionType, amount: u64) !void
```

#### 4. **GHOST Operations (Brand & Collectibles)**
```zig
pub fn getGHOSTBalance(address: Address) u64
pub fn mintGHOST(to: Address, amount: u64, metadata: []const u8) !void
pub fn registerGhostDomain(owner: Address, domain: []const u8, ghost_cost: u64) !void
```

#### 5. **Multi-Token Management**
```zig
pub const TokenType = enum { GCC, SPIRIT, MANA, GHOST };
pub fn getBalance(address: Address, token: TokenType) u64
pub fn transfer(from: Address, to: Address, token: TokenType, amount: u64) !void
```

## Implementation Strategy

### **Native Integration**
- All three tokens built into blockchain core
- **GCC**: Automatic gas deduction from transactions  
- **SPIRIT**: Governance voting weight calculation
- **MANA**: DApp interaction rewards & spending

### **Account State Structure**
```zig
pub const Account = struct {
    gcc_balance: u64,      // Gas & transaction fees
    spirit_balance: u64,   // Governance voting power
    mana_balance: u64,     // Utility & rewards
    ghost_balance: u64,    // Brand & collectibles 👻
    nonce: u64,
    // ... other fields
};
```

## Economic Model

### **GCC (Native Gas Token)**
- **Block Rewards**: 1 GCC per block (12 second blocks)
- **Transaction Fees**: Variable gas pricing
- **Staking Yield**: 5-8% APY for validators
- **Minimum Stake**: 1000 GCC

### **SPIRIT (Governance)**  
- **Total Supply**: 100,000,000 SPIRIT (fixed)
- **Distribution**: Time-locked release over 4 years
- **Voting Power**: 1 SPIRIT = 1 vote
- **Proposal Threshold**: 10,000 SPIRIT to create proposals

### **MANA (Utility)**
- **Minting**: Earned through network participation
- **Burning**: Spent on premium features
- **Rewards**: Smart contract interactions, validator delegations
- **Services**: Premium ZNS domains, contract storage, etc.

### **GHOST (Brand & Collectibles)** 👻
- **Total Supply**: 1,000,000 GHOST (limited edition)
- **Distribution**: Special events, achievements, community rewards
- **Use Cases**: .ghost domains, exclusive features, collectible value
- **Burn Rate**: Deflationary through premium domain registrations

This creates a **balanced four-token economy** with clear utility separation!