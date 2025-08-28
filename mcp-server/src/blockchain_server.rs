//! Blockchain MCP Server Implementation
//! 
//! This module implements the core blockchain functionality as MCP tools.
//! Following the PRD example and using Foundry's cast functionality.
//! 
//! Tools exposed:
//! - balance: Get ETH balance of an address (following PRD example on line 34-50)
//! - transfer: Send ETH between addresses 
//! - is_contract_deployed: Check if contract code exists at address

use std::process::Command;

use anyhow::Result;
use rmcp::{
    ErrorData as McpError, ServerHandler,
    handler::server::{router::tool::ToolRouter, wrapper::Parameters},
    model::{CallToolResult, Content, ServerCapabilities, ServerInfo},
    schemars::JsonSchema,
    tool, tool_handler, tool_router,
};
use serde::{Deserialize, Serialize};
use tracing::{debug, error, info};

/// Request structure for balance queries
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct BalanceRequest {
    #[schemars(description = "The address or ENS name to check balance for")]
    pub address: String,
}

/// Request structure for ETH transfers
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct TransferRequest {
    #[schemars(description = "Sender address (defaults to Alice if not provided)")]
    pub from: Option<String>,
    #[schemars(description = "Recipient address")]
    pub to: String,
    #[schemars(description = "Amount in ETH (e.g., '1.0')")]
    pub amount: String,
}

/// Request structure for contract deployment checks
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct ContractDeploymentRequest {
    #[schemars(description = "Contract address to check")]
    pub address: String,
}

/// Blockchain MCP Server
/// 
/// Exposes Foundry functionality as MCP tools, connecting to anvil at 127.0.0.1:8545
#[derive(Clone)]
pub struct BlockchainServer {
    /// RPC URL for the anvil network
    rpc_url: String,
    /// Alice's address (account 0 from PRD)
    alice_address: String,
    /// Alice's private key for transactions
    alice_private_key: String,
    /// Tool router for MCP
    tool_router: ToolRouter<Self>,
}

#[tool_router]
impl BlockchainServer {
    /// Create a new blockchain server instance
    pub async fn new() -> Result<Self> {
        let rpc_url = "http://127.0.0.1:8545".to_string();
        
        // Alice's address and private key from PRD (account 0)
        let alice_address = "0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266".to_string();
        let alice_private_key = "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80".to_string();

        info!("🔗 Blockchain server configured for anvil network");
        info!("👤 Alice address: {}", alice_address);

        Ok(Self {
            rpc_url,
            alice_address,
            alice_private_key,
            tool_router: Self::tool_router(),
        })
    }

    /// Get the balance of an account using cast balance
    /// Following PRD example structure (lines 34-50)
    #[tool(description = "Get the ETH balance of an address in ether")]
    async fn balance(
        &self,
        Parameters(BalanceRequest { address }): Parameters<BalanceRequest>,
    ) -> Result<CallToolResult, McpError> {
        debug!("💰 Querying balance for address: {}", address);

        // Use cast balance command following Foundry patterns
        let output = Command::new("cast")
            .args([
                "balance",
                &address,
                "--ether", // Return in ETH units
                "--rpc-url",
                &self.rpc_url,
            ])
            .output()
            .map_err(|e| {
                error!("Failed to execute cast balance: {}", e);
                McpError::internal_error(format!("Failed to execute cast balance: {}", e), None)
            })?;

        if !output.status.success() {
            let error_msg = String::from_utf8_lossy(&output.stderr);
            error!("Cast balance failed: {}", error_msg);
            return Err(McpError::internal_error(format!(
                "Cast balance failed: {}",
                error_msg
            ), None));
        }

        let balance = String::from_utf8_lossy(&output.stdout).trim().to_string();
        info!("✅ Balance query successful: {} ETH", balance);

        Ok(CallToolResult::success(vec![Content::text(format!(
            "Balance: {} ETH",
            balance
        ))]))
    }

    /// Transfer ETH between addresses using cast send
    #[tool(description = "Send ETH from one address to another")]
    async fn transfer(
        &self,
        Parameters(TransferRequest { from, to, amount }): Parameters<TransferRequest>,
    ) -> Result<CallToolResult, McpError> {
        // Use Alice as default sender if not specified (PRD requirement)
        let from_address = from.unwrap_or_else(|| self.alice_address.clone());
        
        info!("💸 Transferring {} ETH from {} to {}", amount, from_address, to);

        // Use cast send to perform the transfer
        let output = Command::new("cast")
            .args([
                "send",
                &to,
                "--value",
                &format!("{}ether", amount), // Convert to wei
                "--private-key",
                &self.alice_private_key,
                "--rpc-url",
                &self.rpc_url,
            ])
            .output()
            .map_err(|e| {
                error!("Failed to execute cast send: {}", e);
                McpError::internal_error(format!("Failed to execute cast send: {}", e), None)
            })?;

        if !output.status.success() {
            let error_msg = String::from_utf8_lossy(&output.stderr);
            error!("Cast send failed: {}", error_msg);
            return Err(McpError::internal_error(format!(
                "Cast send failed: {}",
                error_msg
            ), None));
        }

        let tx_hash = String::from_utf8_lossy(&output.stdout).trim().to_string();
        info!("✅ Transfer successful: {}", tx_hash);

        Ok(CallToolResult::success(vec![Content::text(format!(
            "Transfer successful!\nTransaction hash: {}\nSent {} ETH from {} to {}",
            tx_hash, amount, from_address, to
        ))]))
    }

    /// Check if a contract is deployed at the given address
    #[tool(description = "Check if a contract is deployed at the specified address")]
    async fn is_contract_deployed(
        &self,
        Parameters(ContractDeploymentRequest { address }): Parameters<ContractDeploymentRequest>,
    ) -> Result<CallToolResult, McpError> {
        debug!("🔍 Checking contract deployment at address: {}", address);

        // Use cast code to check if there's code at the address
        let output = Command::new("cast")
            .args([
                "code",
                &address,
                "--rpc-url",
                &self.rpc_url,
            ])
            .output()
            .map_err(|e| {
                error!("Failed to execute cast code: {}", e);
                McpError::internal_error(format!("Failed to execute cast code: {}", e), None)
            })?;

        if !output.status.success() {
            let error_msg = String::from_utf8_lossy(&output.stderr);
            error!("Cast code failed: {}", error_msg);
            return Err(McpError::internal_error(format!(
                "Cast code failed: {}",
                error_msg
            ), None));
        }

        let code = String::from_utf8_lossy(&output.stdout).trim().to_string();
        
        // Contract is deployed if code is not "0x" (empty)
        let is_deployed = !code.is_empty() && code != "0x";
        
        info!("✅ Contract deployment check: {} - {}", address, if is_deployed { "DEPLOYED" } else { "NOT DEPLOYED" });

        Ok(CallToolResult::success(vec![Content::text(format!(
            "Contract at {} is {}\nBytecode: {}",
            address,
            if is_deployed { "DEPLOYED" } else { "NOT DEPLOYED" },
            if code.len() > 100 { format!("{}... ({} bytes)", &code[..100], code.len()) } else { code }
        ))]))
    }
}

/// Implement the MCP ServerHandler trait
#[tool_handler]
impl ServerHandler for BlockchainServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            instructions: Some(
                "Ethereum blockchain operations server. Provides tools for balance queries, ETH transfers, and contract deployment checks using Foundry cast commands.".to_string()
            ),
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            ..Default::default()
        }
    }
}
