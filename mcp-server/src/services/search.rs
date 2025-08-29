//! Brave Search API MCP Server Implementation
//! 
//! This module implements Brave Search API functionality as MCP tools.
//! Provides web search capabilities for blockchain-related queries.
//! 
//! Tools exposed:
//! - web_search: Search the web using Brave Search API
//! - get_token_price: Get token prices from search results
//! - get_contract_info: Search for contract information

use anyhow::Result;
use reqwest::Client;
use rmcp::{
    ErrorData as McpError,
    handler::server::{router::tool::ToolRouter, wrapper::Parameters},
    model::{CallToolResult, Content},
    tool, tool_router,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::env;
use tracing::{info, error};

/// Request structure for web searches
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct WebSearchRequest {
    #[schemars(description = "Search query")]
    pub query: String,
    #[schemars(description = "Number of results to return (default: 10)")]
    pub count: Option<u32>,
    #[schemars(description = "Country code (default: 'us')")]
    pub country: Option<String>,
    #[schemars(description = "Search language (default: 'en')")]
    pub search_lang: Option<String>,
}

/// Request structure for token price searches
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct TokenPriceRequest {
    #[schemars(description = "Token symbol (e.g., 'USDC', 'ETH')")]
    pub token: String,
    #[schemars(description = "Base currency (default: 'USD')")]
    pub base_currency: Option<String>,
}

/// Request structure for contract information searches
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct ContractInfoRequest {
    #[schemars(description = "Contract name or address (e.g., 'Uniswap V2 Router')")]
    pub contract: String,
    #[schemars(description = "Network (e.g., 'ethereum', 'polygon')")]
    pub network: Option<String>,
}

/// Brave Search API response structure
#[derive(Debug, Serialize, Deserialize)]
struct BraveSearchResponse {
    query: QueryInfo,
    web: Option<WebResults>,
}

#[derive(Debug, Serialize, Deserialize)]
struct QueryInfo {
    original: String,
    show_strict_warning: bool,
    is_navigational: bool,
    spellcheck_off: bool,
    locale: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct WebResults {
    results: Vec<WebResult>,
}

#[derive(Debug, Serialize, Deserialize)]
struct WebResult {
    title: String,
    url: String,
    description: String,
}

/// Search result structure for MCP responses
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SearchResult {
    #[schemars(description = "Result title")]
    pub title: String,
    #[schemars(description = "Result URL")]
    pub url: String,
    #[schemars(description = "Result description")]
    pub description: String,
}

/// Search response structure
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct SearchResponse {
    #[schemars(description = "Search query")]
    pub query: String,
    #[schemars(description = "Search results")]
    pub results: Vec<SearchResult>,
    #[schemars(description = "Total number of results")]
    pub total_results: usize,
}

/// Brave Search MCP Service
#[derive(Clone)]
pub struct SearchService {
    /// HTTP client for API requests
    client: Client,
    /// Brave Search API key
    api_key: String,
    /// Base URL for Brave Search API
    base_url: String,
    /// Tool router for MCP
    tool_router: ToolRouter<Self>,
}

#[tool_router]
impl SearchService {
    /// Create a new Search service instance
    pub async fn new() -> Result<Self> {
        info!("🔍 Creating Brave Search service");
        
        // Get API key from environment
        let api_key = env::var("BRAVE_SEARCH_API_KEY")
            .map_err(|_| anyhow::anyhow!("BRAVE_SEARCH_API_KEY environment variable not set"))?;
        
        // Create HTTP client
        let client = Client::new();
        
        Ok(Self {
            client,
            api_key,
            base_url: "https://api.search.brave.com/res/v1/web/search".to_string(),
            tool_router: Self::tool_router(),
        })
    }

    /// Perform a web search using Brave Search API
    #[tool(description = "Search the web using Brave Search API")]
    pub async fn web_search(
        &self,
        Parameters(WebSearchRequest { query, count, country, search_lang }): Parameters<WebSearchRequest>,
    ) -> Result<CallToolResult, McpError> {
        info!("🔍 Performing web search: {}", query);
        
        // Build request parameters
        let params = vec![
            ("q", query.clone()),
            ("count", count.unwrap_or(10).to_string()),
            ("country", country.unwrap_or_else(|| "us".to_string())),
            ("search_lang", search_lang.unwrap_or_else(|| "en".to_string())),
        ];
        
        // Make API request
        let response = self.client
            .get(&self.base_url)
            .header("X-Subscription-Token", &self.api_key)
            .query(&params)
            .send()
            .await
            .map_err(|e| {
                error!("Failed to make Brave Search API request: {}", e);
                McpError::internal_error(format!("API request failed: {}", e), None)
            })?;
        
        // Parse response
        let search_response: BraveSearchResponse = response.json().await
            .map_err(|e| {
                error!("Failed to parse Brave Search API response: {}", e);
                McpError::internal_error(format!("Failed to parse response: {}", e), None)
            })?;
        
        // Convert to our response format
        let results: Vec<SearchResult> = search_response.web
            .map(|web| web.results.into_iter().map(|r| SearchResult {
                title: r.title,
                url: r.url,
                description: r.description,
            }).collect())
            .unwrap_or_default();
        
        let search_response = SearchResponse {
            query,
            results: results.clone(),
            total_results: results.len(),
        };
        
        info!("✅ Web search completed with {} results", search_response.total_results);
        
        Ok(CallToolResult::success(vec![Content::text(
            serde_json::to_string_pretty(&search_response)
                .map_err(|e| McpError::internal_error(format!("Failed to serialize response: {}", e), None))?
        )]))
    }

    /// Get token price information
    #[tool(description = "Get current token price information")]
    pub async fn get_token_price(
        &self,
        Parameters(TokenPriceRequest { token, base_currency }): Parameters<TokenPriceRequest>,
    ) -> Result<CallToolResult, McpError> {
        info!("💰 Getting price for token: {}", token);
        
        // Create search query for token price
        let query = format!("{} {} price", 
            token, 
            base_currency.unwrap_or_else(|| "USD".to_string())
        );
        
        // Use web search to find price information
        let search_request = WebSearchRequest {
            query,
            count: Some(5),
            country: Some("us".to_string()),
            search_lang: Some("en".to_string()),
        };
        
        // Call web search internally
        let search_result = self.web_search(Parameters(search_request)).await?;
        
        info!("✅ Token price search completed");
        
        Ok(search_result)
    }

    /// Get contract information
    #[tool(description = "Search for smart contract information")]
    pub async fn get_contract_info(
        &self,
        Parameters(ContractInfoRequest { contract, network }): Parameters<ContractInfoRequest>,
    ) -> Result<CallToolResult, McpError> {
        info!("📋 Getting contract info for: {}", contract);
        
        // Create search query for contract information
        let network = network.unwrap_or_else(|| "ethereum".to_string());
        let query = format!("{} {} contract address", contract, network);
        
        // Use web search to find contract information
        let search_request = WebSearchRequest {
            query,
            count: Some(5),
            country: Some("us".to_string()),
            search_lang: Some("en".to_string()),
        };
        
        // Call web search internally
        let search_result = self.web_search(Parameters(search_request)).await?;
        
        info!("✅ Contract info search completed");
        
        Ok(search_result)
    }
}


