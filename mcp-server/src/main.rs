//! MCP Server for Ethereum Blockchain Operations
//! 
//! This server exposes Foundry functionality as MCP tools following the PRD requirements:
//! 1. balance - Get ETH balance of an address
//! 2. transfer - Send ETH between addresses  
//! 3. is_contract_deployed - Check if contract is deployed at address
//!
//! Connects to anvil network at 127.0.0.1:8545 as specified in PRD

use anyhow::Result;
use rmcp::{ServiceExt, transport::stdio};
use tracing_subscriber::{self, EnvFilter};

mod blockchain_server;
use blockchain_server::BlockchainServer;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging to stderr (stdout is reserved for MCP protocol)
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive(tracing::Level::INFO.into()))
        .with_writer(std::io::stderr)
        .with_ansi(false)
        .init();

    tracing::info!("🚀 Starting MCP Blockchain Server");
    tracing::info!("📡 Connecting to anvil network at 127.0.0.1:8545");

    // Create and start the blockchain server following PRD requirements
    let service = BlockchainServer::new()
        .await?
        .serve(stdio())
        .await
        .inspect_err(|e| {
            tracing::error!("❌ MCP server error: {:?}", e);
        })?;

    tracing::info!("✅ MCP Blockchain Server ready - exposing balance, transfer, and is_contract_deployed tools");

    // Wait for the service to complete
    service.waiting().await?;
    Ok(())
}
