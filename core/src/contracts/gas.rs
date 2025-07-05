use anyhow::{Result, anyhow};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct GasMeter {
    gas_schedule: GasSchedule,
}

#[derive(Debug, Clone)]
pub struct GasSchedule {
    // Basic operations
    pub base_cost: u128,
    pub step_cost: u128,
    pub memory_word: u128,
    pub storage_read: u128,
    pub storage_write: u128,
    pub storage_delete: u128,
    
    // Contract operations
    pub contract_call: u128,
    pub contract_create: u128,
    pub event_emit: u128,
    
    // Domain-specific operations
    pub domain_register: u128,
    pub domain_transfer: u128,
    pub dns_record_update: u128,
    pub domain_lookup: u128,
    
    // Token operations
    pub token_transfer: u128,
    pub token_mint: u128,
    pub token_burn: u128,
    
    // Crypto operations
    pub hash_operation: u128,
    pub signature_verify: u128,
    pub signature_recover: u128,
}

impl Default for GasSchedule {
    fn default() -> Self {
        Self {
            // Basic operations (similar to Ethereum gas costs)
            base_cost: 21000,
            step_cost: 1,
            memory_word: 3,
            storage_read: 800,
            storage_write: 20000,
            storage_delete: 5000,
            
            // Contract operations
            contract_call: 700,
            contract_create: 32000,
            event_emit: 375,
            
            // Domain operations (optimized for DNS use case)
            domain_register: 50000,
            domain_transfer: 30000,
            dns_record_update: 10000,
            domain_lookup: 100,
            
            // Token operations
            token_transfer: 5000,
            token_mint: 10000,
            token_burn: 5000,
            
            // Crypto operations
            hash_operation: 60,
            signature_verify: 3000,
            signature_recover: 3000,
        }
    }
}

impl GasMeter {
    pub fn new() -> Self {
        Self {
            gas_schedule: GasSchedule::default(),
        }
    }
    
    pub fn new_with_schedule(gas_schedule: GasSchedule) -> Self {
        Self {
            gas_schedule,
        }
    }
    
    pub fn calculate_gas_for_operation(&self, operation: &GasOperation) -> u128 {
        match operation {
            GasOperation::BaseTransaction => self.gas_schedule.base_cost,
            GasOperation::Step(count) => self.gas_schedule.step_cost * count,
            GasOperation::MemoryAllocate(words) => self.gas_schedule.memory_word * words,
            GasOperation::StorageRead(slots) => self.gas_schedule.storage_read * slots,
            GasOperation::StorageWrite(slots) => self.gas_schedule.storage_write * slots,
            GasOperation::StorageDelete(slots) => self.gas_schedule.storage_delete * slots,
            GasOperation::ContractCall => self.gas_schedule.contract_call,
            GasOperation::ContractCreate => self.gas_schedule.contract_create,
            GasOperation::EventEmit => self.gas_schedule.event_emit,
            GasOperation::DomainRegister => self.gas_schedule.domain_register,
            GasOperation::DomainTransfer => self.gas_schedule.domain_transfer,
            GasOperation::DnsRecordUpdate => self.gas_schedule.dns_record_update,
            GasOperation::DomainLookup => self.gas_schedule.domain_lookup,
            GasOperation::TokenTransfer => self.gas_schedule.token_transfer,
            GasOperation::TokenMint => self.gas_schedule.token_mint,
            GasOperation::TokenBurn => self.gas_schedule.token_burn,
            GasOperation::HashOperation => self.gas_schedule.hash_operation,
            GasOperation::SignatureVerify => self.gas_schedule.signature_verify,
            GasOperation::SignatureRecover => self.gas_schedule.signature_recover,
        }
    }
    
    pub fn calculate_total_gas(&self, operations: &[GasOperation]) -> u128 {
        operations.iter()
            .map(|op| self.calculate_gas_for_operation(op))
            .sum()
    }
    
    pub fn validate_gas_limit(&self, operations: &[GasOperation], gas_limit: u128) -> Result<()> {
        let total_gas = self.calculate_total_gas(operations);
        if total_gas > gas_limit {
            return Err(anyhow!("Gas limit exceeded: {} > {}", total_gas, gas_limit));
        }
        Ok(())
    }
    
    pub fn estimate_gas_for_method(&self, method: &str, data_size: usize) -> u128 {
        let base_gas = match method {
            // Domain registry methods
            "register_domain" => self.gas_schedule.domain_register,
            "transfer_domain" => self.gas_schedule.domain_transfer,
            "set_record" | "update_record" => self.gas_schedule.dns_record_update,
            "resolve_domain" | "get_domain_owner" => self.gas_schedule.domain_lookup,
            
            // Token methods
            "transfer" => self.gas_schedule.token_transfer,
            "mint" => self.gas_schedule.token_mint,
            "burn" => self.gas_schedule.token_burn,
            
            // Default for unknown methods
            _ => self.gas_schedule.contract_call,
        };
        
        // Add gas for data processing (roughly 1 gas per 32 bytes)
        let data_gas = (data_size as u128 + 31) / 32;
        
        base_gas + data_gas
    }
}

#[derive(Debug, Clone)]
pub enum GasOperation {
    BaseTransaction,
    Step(u128),
    MemoryAllocate(u128),
    StorageRead(u128),
    StorageWrite(u128),
    StorageDelete(u128),
    ContractCall,
    ContractCreate,
    EventEmit,
    DomainRegister,
    DomainTransfer,
    DnsRecordUpdate,
    DomainLookup,
    TokenTransfer,
    TokenMint,
    TokenBurn,
    HashOperation,
    SignatureVerify,
    SignatureRecover,
}

#[derive(Debug)]
pub struct GasTracker {
    gas_used: u128,
    gas_limit: u128,
    operations: Vec<(GasOperation, u128)>,
}

impl GasTracker {
    pub fn new(gas_limit: u128) -> Self {
        Self {
            gas_used: 0,
            gas_limit,
            operations: Vec::new(),
        }
    }
    
    pub fn consume_gas(&mut self, operation: GasOperation, gas_cost: u128) -> Result<()> {
        if self.gas_used + gas_cost > self.gas_limit {
            return Err(anyhow!("Out of gas: {} + {} > {}", self.gas_used, gas_cost, self.gas_limit));
        }
        
        self.gas_used += gas_cost;
        self.operations.push((operation, gas_cost));
        Ok(())
    }
    
    pub fn gas_used(&self) -> u128 {
        self.gas_used
    }
    
    pub fn gas_remaining(&self) -> u128 {
        self.gas_limit - self.gas_used
    }
    
    pub fn get_operations(&self) -> &[(GasOperation, u128)] {
        &self.operations
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_gas_calculation() {
        let gas_meter = GasMeter::new();
        
        let operations = vec![
            GasOperation::BaseTransaction,
            GasOperation::DomainRegister,
            GasOperation::StorageWrite(1),
        ];
        
        let total_gas = gas_meter.calculate_total_gas(&operations);
        assert_eq!(total_gas, 21000 + 50000 + 20000); // 91,000 gas
    }
    
    #[test]
    fn test_gas_tracker() {
        let mut tracker = GasTracker::new(100000);
        
        assert!(tracker.consume_gas(GasOperation::BaseTransaction, 21000).is_ok());
        assert_eq!(tracker.gas_used(), 21000);
        assert_eq!(tracker.gas_remaining(), 79000);
        
        // Try to exceed gas limit
        assert!(tracker.consume_gas(GasOperation::DomainRegister, 90000).is_err());
    }
    
    #[test]
    fn test_method_gas_estimation() {
        let gas_meter = GasMeter::new();
        
        let domain_gas = gas_meter.estimate_gas_for_method("register_domain", 100);
        assert!(domain_gas > 50000); // Should include base domain register cost + data cost
        
        let transfer_gas = gas_meter.estimate_gas_for_method("transfer", 32);
        assert!(transfer_gas >= 5000); // Should include base transfer cost
    }
}