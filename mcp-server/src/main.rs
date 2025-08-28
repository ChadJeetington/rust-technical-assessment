//! MCP Server for Ethereum Blockchain Operations
//! 
//! This server exposes Foundry functionality as MCP tools following the PRD requirements:
//! 1. balance - Get ETH balance of an address
//! 2. transfer - Send ETH between addresses  
//! 3. is_contract_deployed - Check if contract is deployed at address
//!
//! Connects to anvil network at 127.0.0.1:8545 as specified in PRD

use anyhow::Result;
use rmcp::transport::streamable_http_server::{
    StreamableHttpService, session::local::LocalSessionManager,
};
use tracing_subscriber::{self, EnvFilter};

mod blockchain_service;
use blockchain_service::BlockchainService;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging to stderr (stdout is reserved for MCP protocol)
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive(tracing::Level::INFO.into()))
        .with_writer(std::io::stderr)
        .with_ansi(false)
        .init();

    let port = 8080;
    let host = "127.0.0.1";
    
    tracing::info!("🚀 Starting MCP Blockchain Server");
    tracing::info!("🌐 HTTP Server listening on http://{}:{}", host, port);
    tracing::info!("📡 Connecting to anvil network at 127.0.0.1:8545");

    // Create a pre-initialized blockchain service since StreamableHttpService expects sync factory
    let blockchain_service = BlockchainService::new().await
        .map_err(|e| anyhow::anyhow!("Failed to create blockchain service: {}", e))?;
    
    // Create StreamableHttpService with sync constructor
    let service = StreamableHttpService::new(
        move || Ok(blockchain_service.clone()),
        LocalSessionManager::default().into(),
        Default::default(),
    );

    // Create axum router with MCP service
    let router = axum::Router::new().nest_service("/mcp", service);
    let tcp_listener = tokio::net::TcpListener::bind(format!("{}:{}", host, port)).await?;
    
    tracing::info!("✅ MCP Blockchain Server ready on port {} - exposing balance, transfer, and is_contract_deployed tools", port);
    tracing::info!("🔗 RIG clients can connect to: http://{}:{}/mcp", host, port);

    // Start the axum server with graceful shutdown
    axum::serve(tcp_listener, router)
        .with_graceful_shutdown(async {
            tokio::signal::ctrl_c().await.unwrap();
            tracing::info!("🛑 MCP server shutting down...");
        })
        .await?;
    Ok(())
}
