# 🕶️ Guardian Framework

> **Zero-trust privacy framework for decentralized identity and access control**

The Guardian Framework is GhostChain's comprehensive zero-trust privacy system that provides policy-driven access control, ephemeral identities, and anonymous delegation capabilities.

---

## 🎯 **Core Principles**

### **Zero-Trust Architecture**
- **Never Trust, Always Verify** - Every operation requires explicit verification
- **Principle of Least Privilege** - Minimum necessary permissions granted
- **Continuous Verification** - Ongoing policy evaluation during operations
- **Context-Aware Decisions** - Decisions based on real-time context

### **Privacy-First Design**
- **Ephemeral Identities** - Temporary identities for sensitive operations
- **Anonymous Delegation** - Delegate permissions without revealing identity
- **Selective Disclosure** - Share only necessary information
- **Plausible Deniability** - Cryptographic privacy guarantees

---

## 🏗️ **Architecture Components**

```
┌─────────────────────────────────────────────────────────────────┐
│                    Guardian Framework                           │
├─────────────────┬─────────────────┬─────────────────────────────┤
│ Policy Engine   │ Identity Engine │ Cryptographic Engine       │
│                 │                 │                             │
│ • Rule Engine   │ • Ephemeral Mgr │ • Ed25519 Operations        │
│ • Context Eval  │ • Delegation    │ • Anonymous Signatures     │
│ • Approval Flow │ • Identity Mgmt │ • Post-Quantum Ready       │
└─────────────────┴─────────────────┴─────────────────────────────┘
           │                │                        │
           ▼                ▼                        ▼
┌─────────────────┐ ┌─────────────────┐ ┌─────────────────────┐
│ Access Control  │ │ Privacy Layer   │ │ Cryptographic Ops   │
│                 │ │                 │ │                     │
│ • Permissions   │ │ • Ephemeral IDs │ │ • Signature Gen     │
│ • Roles         │ │ • Anonymous Ops │ │ • Verification      │
│ • Time Windows  │ │ • Delegation    │ │ • Hash Operations   │
└─────────────────┘ └─────────────────┘ └─────────────────────┘
```

---

## 🔐 **Permission System**

### **Permission Hierarchy**

Guardian implements a comprehensive permission system with inheritance:

```rust
pub enum Permission {
    // Domain operations
    ResolveDomain,
    RegisterDomain,
    UpdateDomain,
    TransferDomain,

    // Identity operations
    CreateIdentity,
    UpdateIdentity,
    DelegateIdentity,
    RevokeIdentity,

    // Token operations
    TransferTokens,
    MintTokens,
    BurnTokens,
    StakeTokens,

    // Contract operations
    DeployContract,
    ExecuteContract,
    UpgradeContract,

    // Administrative
    AdminAccess,
    PolicyManagement,

    // Custom permission
    Custom(String),
}
```

### **Permission Implications**

Permissions can imply other permissions through a hierarchical system:

```rust
impl Permission {
    pub fn implies(&self, other: &Permission) -> bool {
        match (self, other) {
            // Admin access implies everything
            (Permission::AdminAccess, _) => true,

            // Policy management implies some admin operations
            (Permission::PolicyManagement, Permission::CreateIdentity) => true,
            (Permission::PolicyManagement, Permission::RevokeIdentity) => true,

            // Contract deployment implies execution
            (Permission::DeployContract, Permission::ExecuteContract) => true,

            // Domain registration implies update
            (Permission::RegisterDomain, Permission::UpdateDomain) => true,

            // Exact match
            (a, b) => a == b,
        }
    }
}
```

---

## 🏛️ **Policy Engine**

### **Policy Structure**

Policies are composed of rules that evaluate conditions and return decisions:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Policy {
    pub id: String,
    pub rules: Vec<PolicyRule>,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyRule {
    pub condition: PolicyCondition,
    pub action: PolicyAction,
    pub priority: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PolicyCondition {
    HasRole(String),
    TokenBalance { token_type: TokenType, min_amount: u128 },
    TimeWindow { start: String, end: String },
    DomainOwnership(String),
    TransactionVelocity { max_per_hour: u32 },
    MultiSigRequired { threshold: u32 },
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PolicyAction {
    Allow,
    Deny,
    RequireApproval,
    RequireMultiSig,
    RequireEphemeral,
    Delay { until: chrono::DateTime<chrono::Utc> },
}
```

### **Policy Evaluation**

The policy engine evaluates all applicable policies for each operation:

```rust
pub async fn evaluate_policy(
    &self,
    identity_doc: &IdentityDocument,
    permission: &Permission,
    context: PolicyContext,
) -> Result<PolicyDecision> {
    let applicable_policies = self.get_applicable_policies(permission).await?;

    let mut decisions = Vec::new();

    for policy in applicable_policies {
        for rule in &policy.rules {
            if self.evaluate_condition(&rule.condition, identity_doc, &context).await? {
                decisions.push((rule.action.clone(), rule.priority));
            }
        }
    }

    // Sort by priority and return highest priority decision
    decisions.sort_by(|a, b| b.1.cmp(&a.1));

    match decisions.first() {
        Some((PolicyAction::Allow, _)) => Ok(PolicyDecision::Allow),
        Some((PolicyAction::Deny, _)) => Ok(PolicyDecision::Deny("Policy violation".to_string())),
        Some((PolicyAction::RequireEphemeral, _)) => Ok(PolicyDecision::RequireEphemeral),
        // ... other actions
        None => Ok(PolicyDecision::Deny("No applicable policy".to_string())),
    }
}
```

---

## 👻 **Ephemeral Identities**

### **Ephemeral Identity Creation**

Ephemeral identities are temporary identities for privacy-sensitive operations:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EphemeralIdentity {
    pub identity_id: String,
    pub public_key: Ed25519KeyPair,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub expires_at: chrono::DateTime<chrono::Utc>,
    pub delegation_signature: Option<Vec<u8>>,
}

impl GuardianCrypto {
    pub fn generate_ephemeral_identity(&self) -> Result<EphemeralIdentity> {
        let keypair = self.generate_ed25519_keypair()?;
        let now = chrono::Utc::now();
        let expires_at = now + chrono::Duration::hours(1); // 1 hour ephemeral

        Ok(EphemeralIdentity {
            identity_id: uuid::Uuid::new_v4().to_string(),
            public_key: keypair,
            created_at: now,
            expires_at,
            delegation_signature: None,
        })
    }
}
```

### **Ephemeral Usage Patterns**

Ephemeral identities enable various privacy-preserving patterns:

```rust
// Anonymous transaction
let ephemeral = guardian.create_ephemeral_identity(&parent_gid, Duration::hours(1)).await?;
let anonymous_tx = create_anonymous_transaction(ephemeral.identity_id, tx_data).await?;

// Privacy-preserving voting
let voting_identity = guardian.create_voting_ephemeral(&voter_gid).await?;
let vote = cast_anonymous_vote(voting_identity, proposal_id, vote_choice).await?;

// Temporary service access
let service_identity = guardian.create_service_ephemeral(&user_gid, service_permissions).await?;
let service_result = access_service_anonymously(service_identity, service_request).await?;
```

---

## 🎭 **Anonymous Delegation**

### **Delegation Mechanism**

Anonymous delegation allows permissions to be granted without revealing the delegator:

```rust
pub async fn create_delegation_token(
    &self,
    delegator_gid: &str,
    delegate_gid: &str,
    permissions: Vec<Permission>,
    duration: chrono::Duration,
) -> Result<DelegationToken> {
    let now = chrono::Utc::now();
    let expires_at = now + duration;

    // Create delegation payload
    let delegation_data = DelegationData {
        delegator: delegator_gid.to_string(),
        delegate: delegate_gid.to_string(),
        permissions: permissions.clone(),
        issued_at: now,
        expires_at,
        nonce: uuid::Uuid::new_v4().to_string(),
    };

    // Sign with delegator's key (anonymous signature)
    let signature = self.create_anonymous_signature(&delegation_data)?;

    Ok(DelegationToken {
        delegation_data,
        signature,
        anonymous_proof: self.generate_anonymous_proof(&delegation_data)?,
    })
}
```

### **Zero-Knowledge Delegation Verification**

Delegation can be verified without revealing the delegator's identity:

```rust
pub fn verify_anonymous_delegation(
    &self,
    delegation_token: &DelegationToken,
    required_permission: &Permission,
) -> Result<bool> {
    // Verify the delegation is valid without revealing delegator
    let proof_valid = self.verify_anonymous_proof(&delegation_token.anonymous_proof)?;

    // Check permission is included
    let permission_valid = delegation_token.delegation_data.permissions
        .iter()
        .any(|p| p.implies(required_permission));

    // Check expiration
    let time_valid = chrono::Utc::now() < delegation_token.delegation_data.expires_at;

    Ok(proof_valid && permission_valid && time_valid)
}
```

---

## 🔑 **Access Token System**

### **Guardian Access Tokens**

Guardian access tokens provide secure, time-bound access with cryptographic verification:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessToken {
    pub token_id: String,
    pub identity: String,
    pub permissions: Vec<Permission>,
    pub issued_at: chrono::DateTime<chrono::Utc>,
    pub expires_at: chrono::DateTime<chrono::Utc>,
    pub signature: Vec<u8>,
    pub ephemeral_key: Option<Ed25519KeyPair>,
}

impl AccessToken {
    pub fn is_valid(&self) -> bool {
        chrono::Utc::now() < self.expires_at
    }

    pub fn has_permission(&self, permission: &Permission) -> bool {
        self.permissions.iter().any(|p| p.implies(permission))
    }

    pub fn time_until_expiry(&self) -> chrono::Duration {
        self.expires_at - chrono::Utc::now()
    }
}
```

### **Token-Based Operations**

Operations can be performed using Guardian access tokens:

```rust
// Create token with specific permissions
let token = guardian.create_guardian_token(
    "did:ghost:alice",
    vec![Permission::TransferTokens, Permission::RegisterDomain],
    Some(policy_context)
).await?;

// Use token for authorized operations
let transfer_result = gledger.transfer_with_token(&token, transfer_request).await?;
let domain_result = cns.register_with_token(&token, domain_request).await?;
```

---

## 📋 **Policy Examples**

### **Time-Based Access Policy**

```rust
let business_hours_policy = Policy {
    id: "business_hours".to_string(),
    rules: vec![
        PolicyRule {
            condition: PolicyCondition::TimeWindow {
                start: "09:00".to_string(),
                end: "17:00".to_string(),
            },
            action: PolicyAction::Allow,
            priority: 50,
        },
        PolicyRule {
            condition: PolicyCondition::Custom("after_hours".to_string()),
            action: PolicyAction::RequireApproval,
            priority: 40,
        },
    ],
    enabled: true,
};
```

### **Token Balance Policy**

```rust
let high_value_policy = Policy {
    id: "high_value_operations".to_string(),
    rules: vec![
        PolicyRule {
            condition: PolicyCondition::TokenBalance {
                token_type: TokenType::SPIRIT,
                min_amount: 10000,
            },
            action: PolicyAction::RequireEphemeral,
            priority: 90,
        },
    ],
    enabled: true,
};
```

### **Multi-Signature Policy**

```rust
let multi_sig_policy = Policy {
    id: "large_transfers".to_string(),
    rules: vec![
        PolicyRule {
            condition: PolicyCondition::Custom("transfer_amount_gt_10000".to_string()),
            action: PolicyAction::RequireMultiSig,
            priority: 95,
        },
    ],
    enabled: true,
};
```

---

## 🔒 **Security Guarantees**

### **Cryptographic Properties**

Guardian provides the following cryptographic guarantees:

- **Identity Authenticity** - All identities cryptographically verified
- **Permission Integrity** - Permissions cannot be forged or modified
- **Anonymity** - Ephemeral operations provide plausible deniability
- **Non-Repudiation** - All operations have audit trails
- **Forward Secrecy** - Ephemeral key compromise doesn't affect past operations

### **Privacy Properties**

- **Unlinkability** - Ephemeral operations cannot be linked to parent identity
- **Untraceability** - Anonymous delegations hide delegator identity
- **Selective Disclosure** - Only necessary information revealed
- **Temporal Privacy** - Time-bound access reduces exposure

---

## 🚀 **Performance Considerations**

### **Policy Evaluation Optimization**

```rust
// Policy caching for frequently accessed policies
pub struct PolicyCache {
    cached_decisions: LruCache<String, PolicyDecision>,
    policy_index: HashMap<Permission, Vec<String>>,
}

// Async policy evaluation with caching
pub async fn evaluate_policy_cached(
    &self,
    identity: &str,
    permission: &Permission,
    context: &PolicyContext,
) -> Result<PolicyDecision> {
    let cache_key = format!("{}:{}:{}", identity, permission.as_str(), context.hash());

    if let Some(cached_decision) = self.policy_cache.get(&cache_key) {
        return Ok(cached_decision.clone());
    }

    let decision = self.evaluate_policy_uncached(identity, permission, context).await?;
    self.policy_cache.put(cache_key, decision.clone());

    Ok(decision)
}
```

### **Ephemeral Identity Management**

```rust
// Efficient ephemeral identity cleanup
pub async fn cleanup_expired_ephemeral_identities(&self) -> Result<usize> {
    let now = chrono::Utc::now();
    let mut cleanup_count = 0;

    let mut cache = self.ephemeral_cache.write().await;
    cache.retain(|_, identity| {
        if identity.expires_at < now {
            cleanup_count += 1;
            false
        } else {
            true
        }
    });

    Ok(cleanup_count)
}
```

---

## 📊 **Monitoring & Observability**

### **Guardian Metrics**

```rust
#[derive(Debug, Serialize)]
pub struct GuardianMetrics {
    pub policy_evaluations_per_second: f64,
    pub cache_hit_rate: f64,
    pub ephemeral_identities_active: u64,
    pub delegation_tokens_active: u64,
    pub policy_decisions_by_type: HashMap<String, u64>,
    pub average_evaluation_time_ms: f64,
}
```

### **Audit Logging**

```rust
#[derive(Debug, Serialize)]
pub struct GuardianAuditLog {
    pub event_id: String,
    pub event_type: GuardianEventType,
    pub identity: String,
    pub permission: String,
    pub decision: String,
    pub policy_applied: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub context: serde_json::Value,
}
```

---

*🕶️ Guardian Framework - Zero-trust privacy for the decentralized future*