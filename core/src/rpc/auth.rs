use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use chrono::{DateTime, Utc, Duration};
use sha2::{Sha256, Digest};

#[derive(Debug, Clone)]
pub struct AuthMiddleware {
    api_keys: Arc<RwLock<HashMap<String, ApiKey>>>,
    sessions: Arc<RwLock<HashMap<String, Session>>>,
    require_auth: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKey {
    pub key_id: String,
    pub key_hash: String,
    pub permissions: Vec<Permission>,
    pub created_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    pub rate_limit: Option<u32>, // requests per minute
    pub last_used: Option<DateTime<Utc>>,
    pub usage_count: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub session_id: String,
    pub api_key_id: String,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub last_activity: DateTime<Utc>,
    pub permissions: Vec<Permission>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Permission {
    // Read permissions
    ReadBlockchain,
    ReadAccounts,
    ReadContracts,
    ReadDomains,
    
    // Write permissions
    SendTransactions,
    DeployContracts,
    CallContracts,
    RegisterDomains,
    
    // Admin permissions
    ManageValidators,
    ManageApiKeys,
    ManageSystem,
    
    // Special permissions
    FullAccess,
}

impl Permission {
    pub fn get_required_permission_for_method(method: &str) -> Permission {
        match method {
            "getBlockHeight" | "getBlock" | "getLatestBlock" | "getChainInfo" => Permission::ReadBlockchain,
            "getBalance" | "getAccount" => Permission::ReadAccounts,
            "sendTransaction" => Permission::SendTransactions,
            "getValidators" => Permission::ReadBlockchain,
            "getTransaction" => Permission::ReadBlockchain,
            "deployContract" => Permission::DeployContracts,
            "callContract" => Permission::CallContracts,
            "queryContract" | "getContract" => Permission::ReadContracts,
            "registerDomain" | "transferDomain" | "setDomainRecord" => Permission::RegisterDomains,
            "resolveDomain" | "getDomainsByOwner" => Permission::ReadDomains,
            "createApiKey" | "revokeApiKey" | "revokeSession" => Permission::ManageApiKeys,
            _ => Permission::FullAccess,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthRequest {
    pub api_key: Option<String>,
    pub session_token: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthResult {
    pub authenticated: bool,
    pub permissions: Vec<Permission>,
    pub session_id: Option<String>,
    pub rate_limit_remaining: Option<u32>,
}

impl AuthMiddleware {
    pub fn new(require_auth: bool) -> Self {
        Self {
            api_keys: Arc::new(RwLock::new(HashMap::new())),
            sessions: Arc::new(RwLock::new(HashMap::new())),
            require_auth,
        }
    }
    
    pub async fn create_api_key(
        &self,
        key_id: String,
        raw_key: &str,
        permissions: Vec<Permission>,
        expires_in_days: Option<u32>,
        rate_limit: Option<u32>,
    ) -> Result<String> {
        let key_hash = self.hash_key(raw_key);
        let expires_at = expires_in_days.map(|days| Utc::now() + Duration::days(days as i64));
        
        let api_key = ApiKey {
            key_id: key_id.clone(),
            key_hash,
            permissions,
            created_at: Utc::now(),
            expires_at,
            rate_limit,
            last_used: None,
            usage_count: 0,
        };
        
        let mut keys = self.api_keys.write().await;
        keys.insert(key_id.clone(), api_key);
        
        Ok(key_id)
    }
    
    pub async fn authenticate(&self, auth_request: &AuthRequest, required_permission: &Permission) -> Result<AuthResult> {
        if !self.require_auth {
            return Ok(AuthResult {
                authenticated: true,
                permissions: vec![Permission::FullAccess],
                session_id: None,
                rate_limit_remaining: None,
            });
        }
        
        // Try session authentication first
        if let Some(session_token) = &auth_request.session_token {
            if let Some(auth_result) = self.authenticate_session(session_token, required_permission).await? {
                return Ok(auth_result);
            }
        }
        
        // Try API key authentication
        if let Some(api_key) = &auth_request.api_key {
            return self.authenticate_api_key(api_key, required_permission).await;
        }
        
        Err(anyhow!("No authentication credentials provided"))
    }
    
    async fn authenticate_session(&self, session_token: &str, required_permission: &Permission) -> Result<Option<AuthResult>> {
        let mut sessions = self.sessions.write().await;
        
        if let Some(session) = sessions.get_mut(session_token) {
            // Check if session is expired
            if Utc::now() > session.expires_at {
                sessions.remove(session_token);
                return Ok(None);
            }
            
            // Check permissions
            if !self.has_permission(&session.permissions, required_permission) {
                return Err(anyhow!("Insufficient permissions"));
            }
            
            // Update last activity
            session.last_activity = Utc::now();
            
            Ok(Some(AuthResult {
                authenticated: true,
                permissions: session.permissions.clone(),
                session_id: Some(session_token.to_string()),
                rate_limit_remaining: None, // Sessions don't have rate limits
            }))
        } else {
            Ok(None)
        }
    }
    
    async fn authenticate_api_key(&self, raw_key: &str, required_permission: &Permission) -> Result<AuthResult> {
        let key_hash = self.hash_key(raw_key);
        let mut keys = self.api_keys.write().await;
        
        // Find the API key by hash
        let mut found_key = None;
        for (key_id, api_key) in keys.iter_mut() {
            if api_key.key_hash == key_hash {
                // Check if key is expired
                if let Some(expires_at) = api_key.expires_at {
                    if Utc::now() > expires_at {
                        return Err(anyhow!("API key expired"));
                    }
                }
                
                // Check permissions
                if !self.has_permission(&api_key.permissions, required_permission) {
                    return Err(anyhow!("Insufficient permissions"));
                }
                
                // Check rate limit
                if let Some(rate_limit) = api_key.rate_limit {
                    if let Some(last_used) = api_key.last_used {
                        let time_since_last_use = Utc::now().signed_duration_since(last_used);
                        if time_since_last_use < Duration::minutes(1) && api_key.usage_count >= rate_limit as u64 {
                            return Err(anyhow!("Rate limit exceeded"));
                        }
                        
                        // Reset usage count if more than a minute has passed
                        if time_since_last_use >= Duration::minutes(1) {
                            api_key.usage_count = 0;
                        }
                    }
                }
                
                // Update usage
                api_key.last_used = Some(Utc::now());
                api_key.usage_count += 1;
                
                found_key = Some((key_id.clone(), api_key.clone()));
                break;
            }
        }
        
        if let Some((key_id, api_key)) = found_key {
            // Create a session for this API key
            let session_id = self.create_session(&key_id, &api_key.permissions).await?;
            
            let rate_limit_remaining = api_key.rate_limit.map(|limit| {
                if api_key.usage_count >= limit as u64 {
                    0
                } else {
                    limit - api_key.usage_count as u32
                }
            });
            
            Ok(AuthResult {
                authenticated: true,
                permissions: api_key.permissions,
                session_id: Some(session_id),
                rate_limit_remaining,
            })
        } else {
            Err(anyhow!("Invalid API key"))
        }
    }
    
    async fn create_session(&self, api_key_id: &str, permissions: &[Permission]) -> Result<String> {
        let session_id = uuid::Uuid::new_v4().to_string();
        let session = Session {
            session_id: session_id.clone(),
            api_key_id: api_key_id.to_string(),
            created_at: Utc::now(),
            expires_at: Utc::now() + Duration::hours(24), // Sessions last 24 hours
            last_activity: Utc::now(),
            permissions: permissions.to_vec(),
        };
        
        let mut sessions = self.sessions.write().await;
        sessions.insert(session_id.clone(), session);
        
        Ok(session_id)
    }
    
    pub async fn revoke_session(&self, session_id: &str) -> Result<()> {
        let mut sessions = self.sessions.write().await;
        sessions.remove(session_id);
        Ok(())
    }
    
    pub async fn revoke_api_key(&self, key_id: &str) -> Result<()> {
        let mut keys = self.api_keys.write().await;
        keys.remove(key_id);
        
        // Also remove all sessions for this API key
        let mut sessions = self.sessions.write().await;
        sessions.retain(|_, session| session.api_key_id != key_id);
        
        Ok(())
    }
    
    pub async fn cleanup_expired_sessions(&self) {
        let mut sessions = self.sessions.write().await;
        let now = Utc::now();
        sessions.retain(|_, session| now <= session.expires_at);
    }
    
    fn hash_key(&self, raw_key: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(raw_key.as_bytes());
        format!("{:x}", hasher.finalize())
    }
    
    fn has_permission(&self, user_permissions: &[Permission], required: &Permission) -> bool {
        // FullAccess grants all permissions
        if user_permissions.contains(&Permission::FullAccess) {
            return true;
        }
        
        user_permissions.contains(required)
    }
    
    pub fn get_required_permission_for_method(method: &str) -> Permission {
        match method {
            // Read operations
            "getBlockHeight" | "getBlock" | "getLatestBlock" | "getChainInfo" => Permission::ReadBlockchain,
            "getBalance" | "getAccount" => Permission::ReadAccounts,
            "getContract" | "queryContract" => Permission::ReadContracts,
            "resolveDomain" | "getDomainsByOwner" => Permission::ReadDomains,
            "getValidators" => Permission::ReadBlockchain,
            
            // Write operations
            "sendTransaction" => Permission::SendTransactions,
            "deployContract" => Permission::DeployContracts,
            "callContract" => Permission::CallContracts,
            "registerDomain" | "transferDomain" | "setDomainRecord" => Permission::RegisterDomains,
            
            // Admin operations
            _ => Permission::FullAccess, // Unknown methods require full access
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_api_key_creation_and_auth() {
        let auth = AuthMiddleware::new(true);
        
        // Create an API key
        let key_id = auth.create_api_key(
            "test_key".to_string(),
            "secret123",
            vec![Permission::ReadBlockchain],
            Some(30), // 30 days
            Some(100), // 100 requests per minute
        ).await.unwrap();
        
        assert_eq!(key_id, "test_key");
        
        // Test authentication
        let auth_request = AuthRequest {
            api_key: Some("secret123".to_string()),
            session_token: None,
        };
        
        let result = auth.authenticate(&auth_request, &Permission::ReadBlockchain).await.unwrap();
        assert!(result.authenticated);
        assert!(result.permissions.contains(&Permission::ReadBlockchain));
        assert!(result.session_id.is_some());
    }
    
    #[tokio::test]
    async fn test_insufficient_permissions() {
        let auth = AuthMiddleware::new(true);
        
        // Create an API key with limited permissions
        auth.create_api_key(
            "limited_key".to_string(),
            "secret456",
            vec![Permission::ReadBlockchain],
            Some(30),
            None,
        ).await.unwrap();
        
        let auth_request = AuthRequest {
            api_key: Some("secret456".to_string()),
            session_token: None,
        };
        
        // Should fail when requesting higher permissions
        let result = auth.authenticate(&auth_request, &Permission::DeployContracts).await;
        assert!(result.is_err());
    }
    
    #[tokio::test]
    async fn test_no_auth_required() {
        let auth = AuthMiddleware::new(false); // Auth not required
        
        let auth_request = AuthRequest {
            api_key: None,
            session_token: None,
        };
        
        let result = auth.authenticate(&auth_request, &Permission::DeployContracts).await.unwrap();
        assert!(result.authenticated);
        assert!(result.permissions.contains(&Permission::FullAccess));
    }
}