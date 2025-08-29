//! Combined MCP Server Implementation
//! 
//! This module combines blockchain and search functionality into a single MCP server.
//! Provides both blockchain tools and Brave Search API tools.

use anyhow::Result;
use rmcp::{
    ErrorData as McpError, ServerHandler,
    handler::server::{router::tool::ToolRouter, wrapper::Parameters},
    model::{CallToolResult, ServerCapabilities, ServerInfo},
    tool, tool_handler, tool_router,
};
use tracing::info;

use crate::services::blockchain::BlockchainService;
use crate::services::search::SearchService;

/// Combined MCP Service that includes both blockchain and search functionality
#[derive(Clone)]
pub struct CombinedService {
    /// Blockchain service for Ethereum operations
    blockchain: BlockchainService,
    /// Search service for web search operations
    search: SearchService,
    /// Tool router for MCP
    tool_router: ToolRouter<Self>,
}

#[tool_router]
impl CombinedService {
    /// Create a new combined service instance
    pub async fn new() -> Result<Self> {
        info!("🔧 Creating combined MCP service");
        
        // Create blockchain service
        let blockchain = BlockchainService::new().await
            .map_err(|e| anyhow::anyhow!("Failed to create blockchain service: {}", e))?;
        
        // Create search service
        let search = SearchService::new().await
            .map_err(|e| anyhow::anyhow!("Failed to create search service: {}", e))?;
        
        Ok(Self {
            blockchain,
            search,
            tool_router: Self::tool_router(),
        })
    }

    // Blockchain tools - delegate to blockchain service
    #[tool(description = "Get the balance of an account in wei")]
    async fn balance(
        &self,
        Parameters(request): Parameters<crate::services::blockchain::BalanceRequest>,
    ) -> Result<CallToolResult, McpError> {
        self.blockchain.balance(Parameters(request)).await
    }

    #[tool(description = "Send ETH from Alice to a recipient")]
    async fn send_eth(
        &self,
        Parameters(request): Parameters<crate::services::blockchain::TransferRequest>,
    ) -> Result<CallToolResult, McpError> {
        self.blockchain.send_eth(Parameters(request)).await
    }

    #[tool(description = "Check if a contract is deployed at the given address")]
    async fn is_contract_deployed(
        &self,
        Parameters(request): Parameters<crate::services::blockchain::ContractDeploymentRequest>,
    ) -> Result<CallToolResult, McpError> {
        self.blockchain.is_contract_deployed(Parameters(request)).await
    }

    #[tool(description = "Get ERC-20 token balance for an account")]
    async fn token_balance(
        &self,
        Parameters(request): Parameters<crate::services::blockchain::TokenBalanceRequest>,
    ) -> Result<CallToolResult, McpError> {
        self.blockchain.token_balance(Parameters(request)).await
    }

    #[tool(description = "Get list of available test accounts")]
    async fn get_accounts(&self) -> Result<CallToolResult, McpError> {
        self.blockchain.get_accounts().await
    }

    #[tool(description = "Get private keys for test accounts")]
    async fn get_private_keys(&self) -> Result<CallToolResult, McpError> {
        self.blockchain.get_private_keys().await
    }

    #[tool(description = "Get default addresses (Alice and Bob)")]
    async fn get_default_addresses(&self) -> Result<CallToolResult, McpError> {
        self.blockchain.get_default_addresses().await
    }

    #[tool(description = "Swap tokens using Uniswap V2 Router - integrates with search API to find contract addresses")]
    async fn swap_tokens(
        &self,
        Parameters(request): Parameters<crate::services::blockchain::SwapRequest>,
    ) -> Result<CallToolResult, McpError> {
        self.blockchain.swap_tokens(Parameters(request)).await
    }

    // Search tools - delegate to search service
    #[tool(description = "Search the web using Brave Search API")]
    async fn web_search(
        &self,
        Parameters(request): Parameters<crate::services::search::WebSearchRequest>,
    ) -> Result<CallToolResult, McpError> {
        self.search.web_search(Parameters(request)).await
    }

    #[tool(description = "Get current token price information")]
    async fn get_token_price(
        &self,
        Parameters(request): Parameters<crate::services::search::TokenPriceRequest>,
    ) -> Result<CallToolResult, McpError> {
        self.search.get_token_price(Parameters(request)).await
    }

    #[tool(description = "Search for smart contract information")]
    async fn get_contract_info(
        &self,
        Parameters(request): Parameters<crate::services::search::ContractInfoRequest>,
    ) -> Result<CallToolResult, McpError> {
        self.search.get_contract_info(Parameters(request)).await
    }

    #[tool(description = "Handle swap intent by searching for DEX contracts and token prices")]
    async fn handle_swap_intent(
        &self,
        Parameters(request): Parameters<crate::services::search::SwapIntentRequest>,
    ) -> Result<CallToolResult, McpError> {
        self.search.handle_swap_intent(Parameters(request)).await
    }
}

/// Implement the MCP ServerHandler trait
#[tool_handler]
impl ServerHandler for CombinedService {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            instructions: Some(
                "Combined MCP server providing both blockchain operations and web search capabilities. Includes Ethereum blockchain tools and Brave Search API integration.".to_string()
            ),
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            ..Default::default()
        }
    }
}

