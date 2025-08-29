//! MCP Server for Ethereum Blockchain Operations
//! 
//! This server exposes Foundry functionality as MCP tools following the PRD requirements:
//! 1. balance - Get ETH balance of an address
//! 2. transfer - Send ETH between addresses  
//! 3. is_contract_deployed - Check if contract is deployed at address
//!
//! Connects to anvil network at 127.0.0.1:8545 as specified in PRD

use anyhow::Result;

use mcp_server::server::{McpServer, ServerConfig, init_logging};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    init_logging();

    // Create server configuration
    let config = ServerConfig::default();
    
    // Create and start the MCP server
    let server = McpServer::new(config).await?;
    server.start().await?;
    
    Ok(())
}
