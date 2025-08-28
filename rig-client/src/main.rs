/// RIG AI Agent Client for Ethereum Blockchain Interaction
/// 
/// This client provides a CLI REPL interface that uses Claude API for natural language
/// processing and connects to an MCP server for blockchain operations.

use anyhow::Result;
use clap::Parser;
use dotenv::dotenv;
use rig::providers::anthropic;
use rustyline::{error::ReadlineError, DefaultEditor};
use std::env;
use tracing::{info, error};

mod blockchain_agent;
use blockchain_agent::BlockchainAgent;

#[derive(Parser, Debug)]
#[command(name = "rig-client")]
#[command(about = "AI Agent for Ethereum blockchain interaction via natural language")]
#[command(version)]
struct Args {
    /// Enable verbose logging
    #[arg(short, long)]
    verbose: bool,
    
    /// MCP server URL (default: local)
    #[arg(long, default_value = "http://127.0.0.1:3000")]
    mcp_server: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Load environment variables from .env file
    dotenv().ok();
    
    let args = Args::parse();
    
    // Initialize logging
    let log_level = if args.verbose {
        tracing::Level::DEBUG
    } else {
        tracing::Level::INFO
    };
    
    tracing_subscriber::fmt()
        .with_max_level(log_level)
        .with_target(false)
        .init();

    info!("🚀 Starting RIG AI Agent Client");
    
    // Initialize Claude client
    let anthropic_client = match env::var("ANTHROPIC_API_KEY") {
        Ok(_) => anthropic::Client::from_env(),
        Err(_) => {
            error!("❌ ANTHROPIC_API_KEY environment variable not set");
            error!("Please set your Claude API key in the .env file");
            return Ok(());
        }
    };

    // Create blockchain agent with Claude
    let agent = BlockchainAgent::new(anthropic_client, &args.mcp_server).await?;
    
    info!("🤖 Claude AI Agent initialized");
    info!("🔗 Connected to MCP server at: {}", args.mcp_server);
    
    // Start CLI REPL
    start_repl(agent).await?;
    
    Ok(())
}

async fn start_repl(agent: BlockchainAgent) -> Result<()> {
    let mut rl = DefaultEditor::new()?;
    
    println!("\n🔥 Ethereum AI Agent Ready!");
    println!("💡 Try these commands:");
    println!("   • send 1 ETH from Alice to Bob");
    println!("   • How much ETH does Alice have?");
    println!("   • Is Uniswap V2 Router deployed?");
    println!("   • Type 'quit' or 'exit' to stop\n");

    loop {
        match rl.readline("🤖 > ") {
            Ok(line) => {
                let input = line.trim();
                
                if input.is_empty() {
                    continue;
                }
                
                // Add to history
                rl.add_history_entry(input)?;
                
                // Handle exit commands
                if matches!(input.to_lowercase().as_str(), "quit" | "exit" | "q") {
                    println!("👋 Goodbye!");
                    break;
                }
                
                // Handle help
                if matches!(input.to_lowercase().as_str(), "help" | "h") {
                    print_help();
                    continue;
                }
                
                // Process user input with Claude
                match agent.process_command(input).await {
                    Ok(response) => {
                        println!("🤖 {}\n", response);
                    }
                    Err(e) => {
                        error!("❌ Error processing command: {}", e);
                        println!("❌ Sorry, I encountered an error: {}\n", e);
                    }
                }
            }
            Err(ReadlineError::Interrupted) => {
                println!("👋 Goodbye!");
                break;
            }
            Err(ReadlineError::Eof) => {
                println!("👋 Goodbye!");
                break;
            }
            Err(err) => {
                error!("Error reading input: {}", err);
                break;
            }
        }
    }
    
    Ok(())
}

fn print_help() {
    println!("\n📚 Available Commands:");
    println!("  Blockchain Operations:");
    println!("    • send [amount] ETH from [sender] to [recipient]");
    println!("    • How much [token] does [address] have?");
    println!("    • Is [contract name] deployed?");
    println!("  \n  General:");
    println!("    • help, h - Show this help");
    println!("    • quit, exit, q - Exit the program");
    println!("  \n  Examples:");
    println!("    • send 1 ETH from Alice to Bob");
    println!("    • How much USDC does Alice have?");
    println!("    • Is Uniswap V2 Router deployed?");
    println!();
}
