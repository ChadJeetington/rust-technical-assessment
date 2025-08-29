//! CLI REPL interface for the RIG client

use rustyline::{error::ReadlineError, DefaultEditor};
use tracing::error;

use crate::{BlockchainAgent, Result};

/// CLI REPL interface for interacting with the blockchain agent
pub struct Repl {
    agent: BlockchainAgent,
}

impl Repl {
    /// Create a new REPL instance
    pub fn new(agent: BlockchainAgent) -> Self {
        Self { agent }
    }

    /// Start the interactive REPL
    pub async fn run(&self) -> Result<()> {
        let mut rl = DefaultEditor::new()
            .map_err(|e| crate::ClientError::Cli(format!("Failed to create editor: {}", e)))?;
        
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
                    if let Err(e) = rl.add_history_entry(input) {
                        error!("Failed to add to history: {}", e);
                    }
                    
                    // Handle exit commands
                    if matches!(input.to_lowercase().as_str(), "quit" | "exit" | "q") {
                        println!("👋 Goodbye!");
                        break;
                    }
                    
                    // Handle help
                    if matches!(input.to_lowercase().as_str(), "help" | "h") {
                        Self::print_help();
                        continue;
                    }
                    
                    // Handle test command
                    if matches!(input.to_lowercase().as_str(), "test" | "test-connection") {
                        match self.agent.test_connection().await {
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
                    match self.agent.process_command(input).await {
                        Ok(response) => {
                            // Format the response for better readability
                            let formatted_response = Self::format_response(&response);
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
                    return Err(crate::ClientError::Cli(format!("Readline error: {}", err)));
                }
            }
        }
        
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
                    formatted.push('\n');
                }
            } else if i < lines.len() - 1 {
                // Add spacing between sections but not at the end
                formatted.push('\n');
            }
        }
        
        // Add closing separator
        formatted.push_str("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
        formatted.push('\n');
        
        formatted
    }

    /// Print help information
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
}
