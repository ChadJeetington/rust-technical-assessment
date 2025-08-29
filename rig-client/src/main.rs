//! RIG AI Agent Client for Ethereum Blockchain Interaction
//! 
//! This client provides a CLI REPL interface that uses Claude API for natural language
//! processing and connects to an MCP server for blockchain operations.

use dotenv::dotenv;
use rig::providers::anthropic::Client;
use tracing::info;

use rig_client::{BlockchainAgent, Config, Repl, Result};

#[tokio::main]
async fn main() -> Result<()> {
    // Load environment variables from .env file
    dotenv().ok();
    
    let config = Config::new();
    
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(config.log_level())
        .with_target(false)
        .init();

    info!("🚀 Starting RIG AI Agent Client");
    
    // Initialize Claude client
    let api_key = config.anthropic_api_key()?;
    let anthropic_client = Client::new(&api_key);

    // Create blockchain agent with Claude
    let mut agent = BlockchainAgent::new(anthropic_client, &config.mcp_server).await?;
    
    info!("🤖 Claude AI Agent initialized");
    info!("🔗 Connected to MCP server at: {}", config.mcp_server);
    
    // Initialize RAG system with sample Uniswap documentation
    info!("📚 Initializing RAG system with sample Uniswap documentation");
    agent.initialize_rag_system(None).await?;
    info!("✅ RAG system initialized successfully");
    
    // Start CLI REPL
    let mut repl = Repl::new(agent);
    repl.run().await?;
    
    Ok(())
}
