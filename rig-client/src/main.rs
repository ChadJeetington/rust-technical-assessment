//! RIG AI Agent Client for Ethereum Blockchain Interaction
//! 
//! This client provides a CLI REPL interface that uses Claude API for natural language
//! processing and connects to an MCP server for blockchain operations.

use anyhow::Result;
use clap::Parser;
use dotenv::dotenv;
use rig::providers::anthropic::Client;
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
    #[arg(long, default_value = "http://127.0.0.1:8080/mcp")]
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
    
    // Initialize Claude client using the working pattern
    let api_key = match env::var("ANTHROPIC_API_KEY") {
        Ok(key) => key,
        Err(_) => {
            error!("❌ ANTHROPIC_API_KEY environment variable not set");
            error!("Please set your Claude API key in the .env file");
            return Ok(());
        }
    };

    let anthropic_client = Client::new(&api_key);

    // Create blockchain agent with Claude
    let agent = BlockchainAgent::new(anthropic_client, &args.mcp_server).await?;
    
    info!("🤖 Claude AI Agent initialized");
    info!("🔗 Connected to MCP server at: {}", args.mcp_server);
    
    // Start CLI REPL
    start_repl(agent).await?;
    
    Ok(())
}

/// Format MCP tool responses for better readability
fn format_response(response: &str) -> String {
    let mut formatted = String::new();
    
    // Add a visual separator
    formatted.push_str("🤖 Response:\n");
    formatted.push_str("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    
    // Split response into lines and format each line
    let lines: Vec<&str> = response.lines().collect();
    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        if !trimmed.is_empty() {
            // Add indentation for better readability
            formatted.push_str("  ");
            formatted.push_str(trimmed);
            formatted.push('\n');
            
            // Add extra spacing after key sections
            if trimmed.contains("Transaction Hash:") || 
               trimmed.contains("Status:") ||
               trimmed.contains("Balance:") ||
               trimmed.contains("Contract Deployment Check:") ||
               trimmed.contains("Token Balance:") {
                formatted.push_str("\n");
            }
        } else if i < lines.len() - 1 {
            // Add spacing between sections but not at the end
            formatted.push_str("\n");
        }
    }
    
    // Add closing separator
    formatted.push_str("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    formatted.push_str("\n");
    
    formatted
}

async fn start_repl(agent: BlockchainAgent) -> Result<()> {
    let mut rl = DefaultEditor::new()?;
    
    println!("\n🔥 Ethereum AI Agent Ready!");
    println!("💡 Try these PRD commands:");
    println!("   • Identify sender and recipient");
    println!("   • Validate recipient address");
    println!("   • send 1 ETH from Alice to Bob");
    println!("   • send 0.5 ETH to Bob");
    println!("   • How much USDC does Alice have?");
    println!("   • Is Uniswap V2 Router (0x7a250d5630B4cF539739dF2C5dAcb4c659F2488D) deployed?");
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
                
                // Handle test command
                if matches!(input.to_lowercase().as_str(), "test" | "test-connection") {
                    match agent.test_connection().await {
                        Ok(result) => {
                            println!("🧪 {}\n", result);
                        }
                        Err(e) => {
                            error!("❌ Connection test failed: {}", e);
                            println!("❌ Connection test failed: {}\n", e);
                        }
                    }
                    continue;
                }
                
                // Process user input with Claude
                match agent.process_command(input).await {
                    Ok(response) => {
                        // Format the response for better readability
                        let formatted_response = format_response(&response);
                        println!("{}", formatted_response);
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
    println!("  PRD Required Operations:");
    println!("    • send [amount] ETH from [sender] to [recipient]");
    println!("    • send [amount] ETH to [recipient] (Alice is default sender)");
    println!("    • How much [token] does [address] have?");
    println!("    • Is [contract name] deployed?");
    println!("  \n  Additional Operations:");
    println!("    • Get default addresses (Alice/Bob configuration)");
    println!("    • Get list of available accounts");
    println!("    • Check account private keys");
    println!("  \n  General:");
    println!("    • help, h - Show this help");
    println!("    • test, test-connection - Test MCP connection");
    println!("    • quit, exit, q - Exit the program");
    println!("  \n  PRD Examples:");
    println!("    • send 1 ETH from Alice to Bob");
    println!("    • send 0.5 ETH to Bob (Alice is default sender)");
    println!("    • How much USDC does Alice have?");
    println!("    • Is Uniswap V2 Router (0x7a250d5630B4cF539739dF2C5dAcb4c659F2488D) deployed?");
    println!("  \n  Default Addresses (PRD):");
    println!("    • Alice: Account 0 from anvil (Default Sender)");
    println!("    • Bob: Account 1 from anvil (Default Recipient)");
    println!();
}
