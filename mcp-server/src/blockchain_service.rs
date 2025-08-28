//! Blockchain MCP Server Implementation
//! 
//! This module implements the core blockchain functionality as MCP tools.
//! Following the PRD example exactly - using Cast struct directly from foundry.
//! 
//! Tools exposed:
//! - balance: Get ETH balance of an address (exact PRD example implementation)
//! - transfer: Send ETH between addresses using Cast::send
//! - is_contract_deployed: Check if contract code exists using Cast::code

use alloy_ens::NameOrAddress;
use alloy_network::AnyNetwork;
use alloy_primitives::{Address, U256};
use alloy_provider::{Provider, ProviderBuilder, RootProvider};
use alloy_rpc_types::TransactionRequest;
use alloy_serde::WithOtherFields;
use cast::Cast;
use eyre::Result;
use rmcp::{
    ErrorData as McpError, ServerHandler,
    handler::server::{router::tool::ToolRouter, wrapper::Parameters},
    model::{CallToolResult, Content, ServerCapabilities, ServerInfo},
    schemars::JsonSchema,
    tool, tool_handler, tool_router,
};
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use tracing::info;

/// Request structure for balance queries
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct BalanceRequest {
    #[schemars(description = "The address or ENS name to check balance for")]
    pub who: String,
}

/// Request structure for ETH transfers
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct TransferRequest {
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

/// Blockchain MCP Service - Following PRD Example Exactly
/// 
/// This matches the "MyMcp" struct from the PRD example, using Cast directly
#[derive(Clone)]
pub struct BlockchainService {
    /// Provider for blockchain connection (we'll create Cast on-demand)
    provider: RootProvider<AnyNetwork>,
    /// Alice's address (account 0 from PRD)
    alice_address: Address,
    /// Alice's private key for transactions
    alice_private_key: String,
    /// Tool router for MCP
    tool_router: ToolRouter<Self>,
}

#[tool_router]
impl BlockchainService {
    /// Create a new blockchain service instance
    pub async fn new() -> Result<Self> {
        let rpc_url = "http://127.0.0.1:8545";
        
        // Alice's address and private key from PRD (account 0)
        let alice_address = Address::from_str("0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266")?;
        let alice_private_key = "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80".to_string();

        // Create provider connection to anvil
        let provider = ProviderBuilder::<_, _, AnyNetwork>::default()
            .connect(rpc_url)
            .await?;

        info!("🔗 Blockchain service configured for anvil network at {}", rpc_url);
        info!("👤 Alice address: {}", alice_address);

        Ok(Self {
            provider,
            alice_address,
            alice_private_key,
            tool_router: Self::tool_router(),
        })
    }

    /// Get the balance of an account in wei - Following PRD Example Pattern
    #[tool(description = "Get the balance of an account in wei")]
    async fn balance(
        &self,
        Parameters(BalanceRequest { who }): Parameters<BalanceRequest>,
    ) -> Result<CallToolResult, McpError> {
        let address = NameOrAddress::from(who)
            .resolve(&self.provider)
            .await
            .unwrap();
        let balance = self.provider.get_balance(address).await.unwrap();

        Ok(CallToolResult::success(vec![Content::text(
            balance.to_string(),
        )]))
    }

    /// Send ETH from Alice to another address using Cast::send
    #[tool(description = "Send ETH from Alice to another address")]
    async fn send_eth(
        &self,
        Parameters(TransferRequest { to, amount }): Parameters<TransferRequest>,
    ) -> Result<CallToolResult, McpError> {
        let to_address = NameOrAddress::from(to)
            .resolve(&self.provider)
            .await
            .unwrap();
        
        // Parse amount to wei
        let amount_wei = U256::from_str(&format!("{}000000000000000000", amount.replace(".", "")))
            .unwrap();
        
        // Create transaction request
        let tx = TransactionRequest::default()
            .to(to_address)
            .value(amount_wei)
            .from(self.alice_address);
        
        let tx = WithOtherFields::new(tx);
        
        // Create Cast instance and send transaction
        let cast = Cast::new(self.provider.clone());
        let pending_tx = cast.send(tx).await.unwrap();
        let tx_hash = *pending_tx.tx_hash();
        
        Ok(CallToolResult::success(vec![Content::text(
            format!("Transaction sent: {}", tx_hash)
        )]))
    }

    /// Check if a contract is deployed using Cast::code
    #[tool(description = "Check if a contract is deployed at the specified address")]
    async fn is_contract_deployed(
        &self,
        Parameters(ContractDeploymentRequest { address }): Parameters<ContractDeploymentRequest>,
    ) -> Result<CallToolResult, McpError> {
        let addr = Address::from_str(&address).unwrap();
        
        // Create Cast instance and check if there's code at the address
        let cast = Cast::new(self.provider.clone());
        let code = cast.code(addr, None, false).await.unwrap();
        
        // Contract is deployed if code is not "0x" (empty)
        let is_deployed = !code.is_empty() && code != "0x";
        
        Ok(CallToolResult::success(vec![Content::text(format!(
            "Contract at {} is {}",
            address,
            if is_deployed { "DEPLOYED" } else { "NOT DEPLOYED" }
        ))]))
    }
}

/// Implement the MCP ServerHandler trait
#[tool_handler]
impl ServerHandler for BlockchainService {
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
