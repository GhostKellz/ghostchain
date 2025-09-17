use anyhow::{Result, anyhow};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn, error, debug};
use serde_json;

use crate::domains::{
    MultiDomainResolver,
    ens::ENSResolver,
    unstoppable::UnstoppableResolver,
    web5::Web5Resolver,
    ghost::GhostDomainResolver,
};
use crate::types::*;

/// Comprehensive domain resolution testing framework
pub struct DomainTestingSuite {
    pub multi_resolver: MultiDomainResolver,
    pub test_results: Arc<RwLock<HashMap<String, DomainTestResult>>>,
}

#[derive(Debug, Clone)]
pub struct DomainTestResult {
    pub domain: String,
    pub domain_type: DomainType,
    pub resolution_success: bool,
    pub resolution_time_ms: u64,
    pub resolved_records: Vec<DomainRecord>,
    pub error_message: Option<String>,
    pub test_timestamp: u64,
}

#[derive(Debug, Clone)]
pub enum DomainType {
    ENS,
    UnstoppableDomains,
    Web5DID,
    GhostNative,
}

impl DomainTestingSuite {
    pub fn new() -> Self {
        let multi_resolver = MultiDomainResolver::new();
        
        Self {
            multi_resolver,
            test_results: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Run comprehensive domain resolution tests
    pub async fn run_all_tests(&mut self) -> Result<()> {
        info!("🧪 Starting comprehensive domain resolution tests");

        // Test ENS domains
        self.test_ens_domains().await?;
        
        // Test Unstoppable Domains
        self.test_unstoppable_domains().await?;
        
        // Test Web5 DIDs
        self.test_web5_dids().await?;
        
        // Test Ghost native domains
        self.test_ghost_domains().await?;

        // Generate test report
        self.generate_test_report().await?;

        info!("✅ All domain resolution tests completed");
        Ok(())
    }

    /// Test ENS (.eth) domain resolution
    async fn test_ens_domains(&mut self) -> Result<()> {
        info!("🔍 Testing ENS domain resolution");

        let test_domains = vec![
            "vitalik.eth",
            "ens.eth",
            "ethereum.eth",
            "test.eth",
            "ghostchain.eth", // Test domain we might register
        ];

        for domain in test_domains {
            self.test_single_domain(domain, DomainType::ENS).await;
        }

        Ok(())
    }

    /// Test Unstoppable Domains resolution
    async fn test_unstoppable_domains(&mut self) -> Result<()> {
        info!("🔍 Testing Unstoppable Domains resolution");

        let test_domains = vec![
            "brad.crypto",
            "ethereum.crypto",
            "ghostchain.crypto",
            "test.nft",
            "example.blockchain",
            "sample.x",
            "demo.888",
        ];

        for domain in test_domains {
            self.test_single_domain(domain, DomainType::UnstoppableDomains).await;
        }

        Ok(())
    }

    /// Test Web5 DID resolution
    async fn test_web5_dids(&mut self) -> Result<()> {
        info!("🔍 Testing Web5 DID resolution");

        let test_dids = vec![
            "did:web:example.com",
            "did:key:z6MkiTBz1ymuepAQ4HEHYSF1H8quG5GLVVQR3djdX3mDooWp",
            "did:ethr:0x1234567890123456789012345678901234567890",
            "did:ion:EiClkZMDxPKqC9c-umQfTkR8vvZ9JPhl_xLDI9Nfk38w5w",
            "did:ghost:example", // Ghost-native DID
        ];

        for did in test_dids {
            self.test_single_domain(did, DomainType::Web5DID).await;
        }

        Ok(())
    }

    /// Test Ghost native domain resolution
    async fn test_ghost_domains(&mut self) -> Result<()> {
        info!("🔍 Testing Ghost native domain resolution");

        let test_domains = vec![
            // Core Identity Domains
            "alice.ghost",
            "bob.gcc",
            "validator.sig",
            "0x1234567890abcdef1234567890abcdef12345678.gpk",
            "master.key",
            "node1.pin",
            
            // Decentralized & Blockchain Infrastructure
            "main.bc",
            "registry.zns",
            "node.ops",
            
            // Reserved for Future/Extension Use
            "secure.sid",
            "contract.dvm",
            "temp.tmp",
            "debug.dbg",
            "lib.lib",
            "output.txo",
        ];

        for domain in test_domains {
            self.test_single_domain(domain, DomainType::GhostNative).await;
        }

        Ok(())
    }

    /// Test resolution of a single domain
    async fn test_single_domain(&mut self, domain: &str, domain_type: DomainType) {
        let start_time = std::time::Instant::now();
        
        debug!("Testing domain: {} ({:?})", domain, domain_type);

        let result = match self.multi_resolver.resolve(domain).await {
            Ok(resolution_result) => {
                let elapsed = start_time.elapsed().as_millis() as u64;
                
                DomainTestResult {
                    domain: domain.to_string(),
                    domain_type,
                    resolution_success: true,
                    resolution_time_ms: elapsed,
                    resolved_records: resolution_result.records,
                    error_message: None,
                    test_timestamp: chrono::Utc::now().timestamp() as u64,
                }
            },
            Err(e) => {
                let elapsed = start_time.elapsed().as_millis() as u64;
                
                DomainTestResult {
                    domain: domain.to_string(),
                    domain_type,
                    resolution_success: false,
                    resolution_time_ms: elapsed,
                    resolved_records: vec![],
                    error_message: Some(e.to_string()),
                    test_timestamp: chrono::Utc::now().timestamp() as u64,
                }
            }
        };

        // Store result
        {
            let mut results = self.test_results.write().await;
            results.insert(domain.to_string(), result);
        }
    }

    /// Generate comprehensive test report
    async fn generate_test_report(&self) -> Result<()> {
        let results = self.test_results.read().await;
        
        println!("\n📊 DOMAIN RESOLUTION TEST REPORT");
        println!("════════════════════════════════════════════════");

        let mut stats = HashMap::new();
        let mut total_tests = 0;
        let mut successful_tests = 0;
        let mut total_resolution_time = 0u64;

        for result in results.values() {
            total_tests += 1;
            if result.resolution_success {
                successful_tests += 1;
            }
            total_resolution_time += result.resolution_time_ms;

            let domain_type_key = format!("{:?}", result.domain_type);
            let type_stats = stats.entry(domain_type_key).or_insert((0, 0, 0u64));
            type_stats.0 += 1; // total
            if result.resolution_success {
                type_stats.1 += 1; // successful
            }
            type_stats.2 += result.resolution_time_ms; // total time
        }

        println!("📈 OVERALL STATISTICS:");
        println!("  Total tests: {}", total_tests);
        println!("  Successful: {} ({:.1}%)", successful_tests, 
                (successful_tests as f64 / total_tests as f64) * 100.0);
        println!("  Failed: {}", total_tests - successful_tests);
        println!("  Average resolution time: {:.1}ms", 
                total_resolution_time as f64 / total_tests as f64);

        println!("\n🏷️  BY DOMAIN TYPE:");
        for (domain_type, (total, successful, total_time)) in stats {
            let success_rate = if total > 0 { 
                (successful as f64 / total as f64) * 100.0 
            } else { 
                0.0 
            };
            let avg_time = if total > 0 { 
                total_time as f64 / total as f64 
            } else { 
                0.0 
            };
            
            println!("  {}: {}/{} ({:.1}%) - Avg: {:.1}ms", 
                    domain_type, successful, total, success_rate, avg_time);
        }

        // Show detailed results
        println!("\n📋 DETAILED RESULTS:");
        for result in results.values() {
            let status = if result.resolution_success { "✅" } else { "❌" };
            println!("  {} {} ({:?}) - {}ms", 
                    status, result.domain, result.domain_type, result.resolution_time_ms);
            
            if result.resolution_success {
                println!("     Records: {}", result.resolved_records.len());
                for record in &result.resolved_records {
                    println!("       {} -> {}", record.record_type, record.value);
                }
            } else if let Some(ref error) = result.error_message {
                println!("     Error: {}", error);
            }
        }

        // Show recommendations
        println!("\n💡 RECOMMENDATIONS:");
        let total_success_rate = (successful_tests as f64 / total_tests as f64) * 100.0;
        
        if total_success_rate >= 80.0 {
            println!("  🎉 Excellent! Domain resolution is working well across all types.");
        } else if total_success_rate >= 60.0 {
            println!("  👍 Good performance, but some domain types may need optimization.");
        } else {
            println!("  ⚠️  Several domain types are failing. Check network connectivity and service status.");
        }

        println!("  📝 Note: Many failures are expected in testing without live services.");
        println!("  🔗 For production, ensure ENS, UD, and Web5 services are accessible.");
        println!("  🏠 Ghost domains should work locally with the integrated resolver.");

        Ok(())
    }

    /// Test domain registration (for Ghost domains)
    pub async fn test_ghost_domain_registration(&mut self) -> Result<()> {
        info!("🏗️  Testing Ghost domain registration");

        let test_registrations = vec![
            ("testuser.ghost", "Test user identity"),
            ("contract123.gcc", "Test smart contract"),
            ("validator001.ops", "Test validator node"),
            ("library.lib", "Test contract library"),
        ];

        for (domain, description) in test_registrations {
            info!("Attempting to register: {} ({})", domain, description);

            // Create test records
            let records = vec![
                DomainRecord {
                    record_type: "A".to_string(),
                    name: domain.to_string(),
                    value: "192.168.1.100".to_string(),
                    ttl: 3600,
                    priority: None,
                },
                DomainRecord {
                    record_type: "TXT".to_string(),
                    name: domain.to_string(),
                    value: description.to_string(),
                    ttl: 3600,
                    priority: None,
                },
            ];

            // Try to register via the Ghost domain resolver
            if let Some(ghost_resolver) = self.multi_resolver.get_ghost_resolver() {
                match ghost_resolver.write().await.register_domain(
                    domain,
                    &"0x1234567890123456789012345678901234567890".to_string(),
                    records,
                    None,
                ).await {
                    Ok(result) => {
                        info!("✅ Successfully registered {}: {}", domain, result);
                    },
                    Err(e) => {
                        warn!("❌ Failed to register {}: {}", domain, e);
                    }
                }
            }
        }

        Ok(())
    }

    /// Get test statistics
    pub async fn get_test_statistics(&self) -> HashMap<String, serde_json::Value> {
        let results = self.test_results.read().await;
        
        let mut stats = HashMap::new();
        stats.insert("total_tests".to_string(), serde_json::Value::from(results.len()));
        
        let successful = results.values().filter(|r| r.resolution_success).count();
        stats.insert("successful_tests".to_string(), serde_json::Value::from(successful));
        
        let failed = results.len() - successful;
        stats.insert("failed_tests".to_string(), serde_json::Value::from(failed));
        
        if !results.is_empty() {
            let avg_time: f64 = results.values()
                .map(|r| r.resolution_time_ms as f64)
                .sum::<f64>() / results.len() as f64;
            stats.insert("average_resolution_time_ms".to_string(), serde_json::Value::from(avg_time));
        }

        stats
    }
}

/// CLI command handler for domain testing
pub async fn handle_domain_test_command() -> Result<()> {
    println!("🌐 Testing Multi-Domain Resolution...");
    
    let mut test_suite = DomainTestingSuite::new();
    
    match test_suite.run_all_tests().await {
        Ok(()) => {
            println!("✅ Domain resolution tests completed successfully");
            
            // Also test Ghost domain registration
            if let Err(e) = test_suite.test_ghost_domain_registration().await {
                warn!("Ghost domain registration test failed: {}", e);
            }
        },
        Err(e) => {
            println!("❌ Domain resolution tests failed: {}", e);
        }
    }
    
    Ok(())
}

/// Quick domain resolution test for CLI
pub async fn quick_domain_test(domain: &str) -> Result<()> {
    println!("🔍 Testing domain: {}", domain);
    
    let mut resolver = MultiDomainResolver::new();
    
    match resolver.resolve(domain).await {
        Ok(result) => {
            println!("✅ Domain resolved successfully!");
            println!("   Domain: {}", result.domain);
            println!("   Records: {}", result.records.len());
            for record in result.records {
                println!("     {} -> {}", record.record_type, record.value);
            }
            if let Some(ref metadata) = result.metadata {
                println!("   Metadata: {} entries", metadata.len());
            }
        },
        Err(e) => {
            println!("❌ Domain resolution failed: {}", e);
        }
    }
    
    Ok(())
}