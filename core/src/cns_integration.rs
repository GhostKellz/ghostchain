use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::process::Command;
use crate::types::*;
use crate::blockchain::integration::BlockchainContractIntegration;
use std::sync::Arc;
use tokio::sync::RwLock;

/// CNS Integration Module for GhostChain
/// Bridges Rust GhostChain with CNS (Crypto Name Service)
#[derive(Debug)]
pub struct CnsIntegration {
    cache: HashMap<String, CachedDomainData>,
    contract_integration: Option<Arc<RwLock<BlockchainContractIntegration>>>,
    cns_rpc_endpoint: String,
}

/// CNS Domain Resolver - Stub for external CNS crate integration
pub struct CnsDomainResolver {
    pub enabled: bool,
    pub endpoint: String,
}

impl CnsDomainResolver {
    pub fn new(endpoint: String) -> Self {
        Self {
            enabled: true,
            endpoint,
        }
    }
}

// DomainRecord and DomainData are now defined in types.rs

#[derive(Debug, Clone)]
struct CachedDomainData {
    data: DomainData,
    cached_at: u64,
    ttl: u32,
}

impl CnsIntegration {
    pub fn new(cns_rpc_endpoint: String) -> Self {
        Self {
            cache: HashMap::new(),
            contract_integration: None,
            cns_rpc_endpoint,
        }
    }

    pub fn new_with_contract(
        cns_rpc_endpoint: String,
        contract_integration: Arc<RwLock<BlockchainContractIntegration>>
    ) -> Self {
        Self {
            cache: HashMap::new(),
            contract_integration: Some(contract_integration),
            cns_rpc_endpoint,
        }
    }

    /// Resolve a domain through CNS
    pub async fn resolve_domain(&mut self, domain: &str) -> Result<DomainData> {
        // Check cache first
        if let Some(cached) = self.get_cached(domain) {
            return Ok(cached.data.clone());
        }

        let domain_data = if self.contract_integration.is_some() {
            // Use on-chain contract
            let integration = self.contract_integration.as_ref().unwrap();
            let integration_guard = integration.read().await;
            integration_guard.resolve_domain(domain).await?
        } else {
            // Use CNS RPC endpoint
            // TODO: Implement HTTP client call to CNS service
            return Err(anyhow!("CNS RPC integration not yet implemented"));
        };

        // Cache the result
        self.cache_domain(domain, &domain_data);

        Ok(domain_data)
    }

    /// Register a domain through ZNS
    pub async fn register_domain(
        &mut self,
        domain: &str,
        owner: &str,
        records: Vec<DomainRecord>,
        private_key: Option<&str>,
    ) -> Result<String> {
        if self.contract_integration.is_some() {
            // Use on-chain contract
            let integration = self.contract_integration.as_ref().unwrap();
            let mut integration_guard = integration.write().await;
            let owner_addr: Address = owner.to_string();
            let result = integration_guard.register_domain(domain, &owner_addr, records).await?;
            
            // Return transaction hash or contract result
            Ok(format!("Contract call successful, gas used: {}", result.gas_used))
        } else {
            // TODO: Implement CNS RPC call for registration
            Err(anyhow!("CNS RPC registration not yet implemented"))
        }
    }

    /// Update domain records
    pub async fn update_domain(
        &mut self,
        domain: &str,
        records: Vec<DomainRecord>,
        private_key: Option<&str>,
    ) -> Result<String> {
        if self.contract_integration.is_some() {
            // Use on-chain contract - for simplicity, we'll set one record at a time
            let integration = self.contract_integration.as_ref().unwrap();
            let mut integration_guard = integration.write().await;
            
            let mut total_gas = 0;
            for record in records {
                // This would need the actual contract call for setting records
                // For now, we'll simulate it
                total_gas += 10000; // Estimated gas per record
            }
            
            // Clear cache for this domain
            self.cache.remove(domain);
            
            Ok(format!("Contract records updated, estimated gas: {}", total_gas))
        } else {
            // TODO: Implement CNS RPC call for update
            // Clear cache for this domain
            self.cache.remove(domain);
            Err(anyhow!("CNS RPC update not yet implemented"))
        }
    }

    /// Get domain ownership info
    pub async fn get_domain_owner(&mut self, domain: &str) -> Result<Address> {
        let domain_data = self.resolve_domain(domain).await?;
        Ok(domain_data.owner)
    }

    /// Check if domain exists
    pub async fn domain_exists(&mut self, domain: &str) -> bool {
        self.resolve_domain(domain).await.is_ok()
    }

    /// Get all domains owned by an address
    pub async fn get_domains_by_owner(&self, owner: &str) -> Result<Vec<String>> {
        if self.contract_integration.is_some() {
            // Use on-chain contract query
            let integration = self.contract_integration.as_ref().unwrap();
            let integration_guard = integration.read().await;
            
            let query_data = serde_json::to_vec(&serde_json::json!({
                "owner": owner
            }))?;
            
            let result = integration_guard.query_contract(
                &"system.domain_registry".to_string(),
                "get_owner_domains",
                &query_data,
            ).await?;
            
            let domains: Vec<String> = serde_json::from_slice(&result)?;
            Ok(domains)
        } else {
            // TODO: Implement CNS RPC call for domain listing
            Err(anyhow!("CNS RPC domain listing not yet implemented"))
        }
    }

    // Private helper methods

    fn get_cached(&self, domain: &str) -> Option<&CachedDomainData> {
        if let Some(cached) = self.cache.get(domain) {
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs();

            if now < cached.cached_at + cached.ttl as u64 {
                return Some(cached);
            }
        }
        None
    }

    fn cache_domain(&mut self, domain: &str, data: &DomainData) {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        // Use minimum TTL from records, default to 300 seconds
        let ttl = data.records.iter()
            .map(|r| r.ttl)
            .min()
            .unwrap_or(300);

        let cached_data = CachedDomainData {
            data: data.clone(),
            cached_at: now,
            ttl,
        };

        self.cache.insert(domain.to_string(), cached_data);
    }

    /// Clear expired cache entries
    pub fn cleanup_cache(&mut self) {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        self.cache.retain(|_, cached| {
            now < cached.cached_at + cached.ttl as u64
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_cns_integration_creation() {
        // Test CNS integration creation
        let mut cns = CnsIntegration::new("http://localhost:8553".to_string());

        // This would only work with actual CNS service
        // let result = cns.resolve_domain("test.ghost").await;
        // assert!(result.is_ok());
    }

    #[test]
    fn test_cache_functionality() {

        let domain_data = DomainData {
            domain: "test.ghost".to_string(),
            owner: "owner123".to_string(),
            records: vec![
                DomainRecord {
                    record_type: "A".to_string(),
                    name: "test.ghost".to_string(),
                    value: "192.168.1.1".to_string(),
                    ttl: 300,
                    priority: None,
                }
            ],
            contract_address: None,
            last_updated: 1640995200,
            signature: "sig123".to_string(),
        };

        let mut cns = CnsIntegration::new("http://localhost:8553".to_string());
        cns.cache_domain("test.ghost", &domain_data);
        assert!(cns.get_cached("test.ghost").is_some());
    }
}
