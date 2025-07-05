use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::process::Command;
use std::path::PathBuf;
use crate::types::*;

/// ZVM (Zig Virtual Machine) Integration for GhostChain
/// Enables smart contract execution via ZVM
#[derive(Debug, Clone)]
pub struct ZvmExecutor {
    zvm_binary_path: PathBuf,
    contract_storage: HashMap<String, DeployedContract>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeployedContract {
    pub address: String,
    pub bytecode: Vec<u8>,
    pub abi: Option<String>,
    pub owner: String,
    pub deployed_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractExecutionResult {
    pub success: bool,
    pub return_data: Vec<u8>,
    pub gas_used: u64,
    pub logs: Vec<ContractLog>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractLog {
    pub address: String,
    pub topics: Vec<String>,
    pub data: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractCall {
    pub contract_address: String,
    pub function_selector: String,
    pub input_data: Vec<u8>,
    pub caller: String,
    pub value: u128,
    pub gas_limit: u64,
}

impl ZvmExecutor {
    pub fn new(zvm_binary_path: PathBuf) -> Self {
        Self {
            zvm_binary_path,
            contract_storage: HashMap::new(),
        }
    }

    /// Deploy a new smart contract
    pub async fn deploy_contract(
        &mut self,
        bytecode: &[u8],
        constructor_args: &[u8],
        deployer: &str,
        gas_limit: u64,
    ) -> Result<String> {
        let bytecode_hex = hex::encode(bytecode);
        let args_hex = hex::encode(constructor_args);

        let output = Command::new(&self.zvm_binary_path)
            .arg("deploy")
            .arg("--bytecode")
            .arg(&bytecode_hex)
            .arg("--constructor-args")
            .arg(&args_hex)
            .arg("--deployer")
            .arg(deployer)
            .arg("--gas-limit")
            .arg(&gas_limit.to_string())
            .arg("--json")
            .output()
            .map_err(|e| anyhow!("Failed to execute ZVM binary: {}", e))?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow!("Contract deployment failed: {}", error));
        }

        let json_output = String::from_utf8_lossy(&output.stdout);
        let result: serde_json::Value = serde_json::from_str(&json_output)
            .map_err(|e| anyhow!("Failed to parse ZVM response: {}", e))?;

        let contract_address = result["contract_address"]
            .as_str()
            .ok_or_else(|| anyhow!("No contract address in response"))?
            .to_string();

        // Store contract metadata
        let deployed_contract = DeployedContract {
            address: contract_address.clone(),
            bytecode: bytecode.to_vec(),
            abi: None, // Could be parsed from metadata
            owner: deployer.to_string(),
            deployed_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        };

        self.contract_storage.insert(contract_address.clone(), deployed_contract);

        Ok(contract_address)
    }

    /// Call a smart contract function
    pub async fn call_contract(&self, call: &ContractCall) -> Result<ContractExecutionResult> {
        let input_hex = hex::encode(&call.input_data);

        let output = Command::new(&self.zvm_binary_path)
            .arg("call")
            .arg("--contract")
            .arg(&call.contract_address)
            .arg("--function")
            .arg(&call.function_selector)
            .arg("--input")
            .arg(&input_hex)
            .arg("--caller")
            .arg(&call.caller)
            .arg("--value")
            .arg(&call.value.to_string())
            .arg("--gas-limit")
            .arg(&call.gas_limit.to_string())
            .arg("--json")
            .output()
            .map_err(|e| anyhow!("Failed to execute ZVM binary: {}", e))?;

        let json_output = String::from_utf8_lossy(&output.stdout);
        let result: ContractExecutionResult = serde_json::from_str(&json_output)
            .map_err(|e| anyhow!("Failed to parse ZVM response: {}", e))?;

        Ok(result)
    }

    /// Execute a read-only contract call (view function)
    pub async fn view_contract(
        &self,
        contract_address: &str,
        function_selector: &str,
        input_data: &[u8],
    ) -> Result<Vec<u8>> {
        let input_hex = hex::encode(input_data);

        let output = Command::new(&self.zvm_binary_path)
            .arg("view")
            .arg("--contract")
            .arg(contract_address)
            .arg("--function")
            .arg(function_selector)
            .arg("--input")
            .arg(&input_hex)
            .arg("--json")
            .output()
            .map_err(|e| anyhow!("Failed to execute ZVM binary: {}", e))?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow!("View call failed: {}", error));
        }

        let json_output = String::from_utf8_lossy(&output.stdout);
        let result: serde_json::Value = serde_json::from_str(&json_output)
            .map_err(|e| anyhow!("Failed to parse ZVM response: {}", e))?;

        let return_data_hex = result["return_data"]
            .as_str()
            .ok_or_else(|| anyhow!("No return data in response"))?;

        let return_data = hex::decode(return_data_hex)
            .map_err(|e| anyhow!("Failed to decode return data: {}", e))?;

        Ok(return_data)
    }

    /// Get contract bytecode
    pub async fn get_contract_bytecode(&self, contract_address: &str) -> Result<Vec<u8>> {
        let output = Command::new(&self.zvm_binary_path)
            .arg("get-code")
            .arg("--contract")
            .arg(contract_address)
            .output()
            .map_err(|e| anyhow!("Failed to execute ZVM binary: {}", e))?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow!("Get code failed: {}", error));
        }

        let bytecode_hex = String::from_utf8_lossy(&output.stdout).trim().to_string();
        let bytecode = hex::decode(&bytecode_hex)
            .map_err(|e| anyhow!("Failed to decode bytecode: {}", e))?;

        Ok(bytecode)
    }

    /// Estimate gas for a contract call
    pub async fn estimate_gas(&self, call: &ContractCall) -> Result<u64> {
        let input_hex = hex::encode(&call.input_data);

        let output = Command::new(&self.zvm_binary_path)
            .arg("estimate-gas")
            .arg("--contract")
            .arg(&call.contract_address)
            .arg("--function")
            .arg(&call.function_selector)
            .arg("--input")
            .arg(&input_hex)
            .arg("--caller")
            .arg(&call.caller)
            .arg("--value")
            .arg(&call.value.to_string())
            .output()
            .map_err(|e| anyhow!("Failed to execute ZVM binary: {}", e))?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow!("Gas estimation failed: {}", error));
        }

        let gas_estimate = String::from_utf8_lossy(&output.stdout)
            .trim()
            .parse::<u64>()
            .map_err(|e| anyhow!("Failed to parse gas estimate: {}", e))?;

        Ok(gas_estimate)
    }

    /// Get contract info from local storage
    pub fn get_contract_info(&self, contract_address: &str) -> Option<&DeployedContract> {
        self.contract_storage.get(contract_address)
    }

    /// List all deployed contracts
    pub fn list_contracts(&self) -> Vec<&DeployedContract> {
        self.contract_storage.values().collect()
    }

    /// Check if ZVM binary is available
    pub fn is_available(&self) -> bool {
        Command::new(&self.zvm_binary_path)
            .arg("--version")
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
    }

    /// Get ZVM version
    pub fn get_version(&self) -> Result<String> {
        let output = Command::new(&self.zvm_binary_path)
            .arg("--version")
            .output()
            .map_err(|e| anyhow!("Failed to execute ZVM binary: {}", e))?;

        if !output.status.success() {
            return Err(anyhow!("Failed to get ZVM version"));
        }

        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    }
}

/// Helper functions for contract interaction

/// Create a simple contract call for token transfers
pub fn create_transfer_call(
    contract_address: &str,
    to: &str,
    amount: u128,
    caller: &str,
) -> ContractCall {
    // ERC20 transfer function selector: transfer(address,uint256)
    let function_selector = "a9059cbb".to_string();
    
    // Encode parameters: to address (32 bytes) + amount (32 bytes)
    let mut input_data = Vec::new();
    
    // Pad address to 32 bytes
    let to_bytes = hex::decode(to.trim_start_matches("0x")).unwrap_or_default();
    let mut to_padded = vec![0u8; 32];
    to_padded[32 - to_bytes.len()..].copy_from_slice(&to_bytes);
    input_data.extend_from_slice(&to_padded);
    
    // Encode amount as 32-byte big-endian
    let amount_bytes = amount.to_be_bytes();
    let mut amount_padded = vec![0u8; 32];
    amount_padded[32 - amount_bytes.len()..].copy_from_slice(&amount_bytes);
    input_data.extend_from_slice(&amount_padded);

    ContractCall {
        contract_address: contract_address.to_string(),
        function_selector,
        input_data,
        caller: caller.to_string(),
        value: 0,
        gas_limit: 100_000,
    }
}

/// Create a balance query call
pub fn create_balance_call(contract_address: &str, account: &str) -> ContractCall {
    // ERC20 balanceOf function selector: balanceOf(address)
    let function_selector = "70a08231".to_string();
    
    // Encode parameter: account address (32 bytes)
    let account_bytes = hex::decode(account.trim_start_matches("0x")).unwrap_or_default();
    let mut account_padded = vec![0u8; 32];
    account_padded[32 - account_bytes.len()..].copy_from_slice(&account_bytes);

    ContractCall {
        contract_address: contract_address.to_string(),
        function_selector,
        input_data: account_padded,
        caller: account.to_string(),
        value: 0,
        gas_limit: 50_000,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_transfer_call() {
        let call = create_transfer_call(
            "0x1234567890abcdef",
            "0xabcdef1234567890",
            1000,
            "0xsender123"
        );
        
        assert_eq!(call.function_selector, "a9059cbb");
        assert_eq!(call.contract_address, "0x1234567890abcdef");
        assert_eq!(call.value, 0);
        assert_eq!(call.gas_limit, 100_000);
    }

    #[test]
    fn test_zvm_executor_creation() {
        let zvm_path = PathBuf::from("./zvm");
        let executor = ZvmExecutor::new(zvm_path);
        assert!(executor.contract_storage.is_empty());
    }
}
