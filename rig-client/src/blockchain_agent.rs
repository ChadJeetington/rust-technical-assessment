/// Blockchain Agent - Integrates Claude AI with MCP server for blockchain operations
/// 
/// This agent:
/// 1. Processes natural language commands using Claude
/// 2. Connects to MCP server to execute blockchain operations
/// 3. Returns human-friendly responses

use anyhow::Result;
use rig::completion::{Chat, Message, Prompt, ToolDefinition};
use rig::providers::anthropic::{self, CLAUDE_3_5_SONNET};
use rig::tool::Tool;
use serde::{Deserialize, Serialize};
use serde_json::json;
use tracing::{debug, info};

/// Parameters for checking balance
#[derive(Debug, Deserialize, Serialize)]
struct BalanceParams {
    /// The address to check balance for
    address: String,
    /// Optional token contract address (omit for ETH)
    token_address: Option<String>,
}

/// Parameters for transfer operations
#[derive(Debug, Deserialize, Serialize)]
struct TransferParams {
    /// Source address
    from: String,
    /// Destination address
    to: String,
    /// Amount to transfer (in ETH)
    amount: String,
}

/// Parameters for contract deployment check
#[derive(Debug, Deserialize, Serialize)]
struct ContractCheckParams {
    /// Contract address to check
    address: String,
}

/// Custom error type for blockchain tools
#[derive(Debug, thiserror::Error)]
#[error("Blockchain tool error: {0}")]
struct BlockchainError(String);

/// Tool for checking balances
#[derive(Debug)]
struct BalanceTool {
    mcp_client: reqwest::Client,
    mcp_url: String,
}

impl Tool for BalanceTool {
    const NAME: &'static str = "check_balance";

    type Error = BlockchainError;
    type Args = BalanceParams;
    type Output = String;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: Self::NAME.to_string(),
            description: "Check ETH or token balance for a given address".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "address": {
                        "type": "string",
                        "description": "The Ethereum address to check balance for"
                    },
                    "token_address": {
                        "type": "string",
                        "description": "Optional token contract address (omit for ETH balance)"
                    }
                },
                "required": ["address"]
            }),
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        debug!("🔍 Checking balance for address: {}", args.address);
        
        // For now, simulate the MCP call
        if let Some(token) = args.token_address {
            Ok(format!("Token balance check: Address {} has 1000 tokens of contract {}", args.address, token))
        } else {
            Ok(format!("ETH balance check: Address {} has 9999 ETH", args.address))
        }
    }
}

/// Tool for transfers
#[derive(Debug)]
struct TransferTool {
    mcp_client: reqwest::Client,
    mcp_url: String,
}

impl Tool for TransferTool {
    const NAME: &'static str = "transfer_eth";

    type Error = BlockchainError;
    type Args = TransferParams;
    type Output = String;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: Self::NAME.to_string(),
            description: "Transfer ETH from one address to another".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "from": {
                        "type": "string",
                        "description": "Source address"
                    },
                    "to": {
                        "type": "string",
                        "description": "Destination address"
                    },
                    "amount": {
                        "type": "string",
                        "description": "Amount to transfer in ETH"
                    }
                },
                "required": ["from", "to", "amount"]
            }),
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        debug!("💸 Transferring {} ETH from {} to {}", args.amount, args.from, args.to);
        
        // For now, simulate the transfer
        Ok(format!("Transfer successful: {} ETH sent from {} to {}. Transaction hash: 0x123...abc", 
                  args.amount, args.from, args.to))
    }
}

/// Tool for checking contract deployment
#[derive(Debug)]
struct ContractCheckTool {
    mcp_client: reqwest::Client,
    mcp_url: String,
}

impl Tool for ContractCheckTool {
    const NAME: &'static str = "is_contract_deployed";

    type Error = BlockchainError;
    type Args = ContractCheckParams;
    type Output = String;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: Self::NAME.to_string(),
            description: "Check if a contract is deployed at the given address".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "address": {
                        "type": "string",
                        "description": "The contract address to check"
                    }
                },
                "required": ["address"]
            }),
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        debug!("🔍 Checking if contract is deployed at: {}", args.address);
        
        // For now, simulate the check
        if args.address.to_lowercase().contains("uniswap") {
            Ok(format!("Contract check: Yes, a contract is deployed at {}", args.address))
        } else {
            Ok(format!("Contract check: No contract found at {}", args.address))
        }
    }
}

/// The main blockchain agent that combines Claude AI with blockchain tools
pub struct BlockchainAgent {
    /// Claude AI agent configured for blockchain operations
    claude_agent: rig::agent::Agent<anthropic::completion::CompletionModel>,
    /// MCP server URL for blockchain operations
    mcp_server_url: String,
    /// HTTP client for MCP communication
    http_client: reqwest::Client,
}

impl BlockchainAgent {
    /// Create a new blockchain agent
    pub async fn new(anthropic_client: anthropic::Client, mcp_server_url: &str) -> Result<Self> {
        info!("🔧 Initializing Blockchain Agent with Claude");
        
        let http_client = reqwest::Client::new();
        
        // Create blockchain tools
        let balance_tool = BalanceTool {
            mcp_client: http_client.clone(),
            mcp_url: mcp_server_url.to_string(),
        };
        
        let transfer_tool = TransferTool {
            mcp_client: http_client.clone(),
            mcp_url: mcp_server_url.to_string(),
        };
        
        let contract_tool = ContractCheckTool {
            mcp_client: http_client.clone(),
            mcp_url: mcp_server_url.to_string(),
        };
        
        // Create Claude agent with blockchain-specific system prompt and tools
        // Note: max_tokens is required for Anthropic API
        // Following official Rig pattern: add tools individually using .tool() method
        let claude_agent = anthropic_client
            .agent(CLAUDE_3_5_SONNET)
            .preamble(&Self::get_system_prompt())
            .temperature(0.1) // Low temperature for consistent responses
            .max_tokens(2000) // Increased for more detailed responses
            .tool(balance_tool)
            .tool(transfer_tool)
            .tool(contract_tool)
            .build();
        
        // Test MCP server connection (will be implemented when MCP server is ready)
        debug!("🔗 MCP server URL: {}", mcp_server_url);
        
        Ok(Self {
            claude_agent,
            mcp_server_url: mcp_server_url.to_string(),
            http_client,
        })
    }

    /// Process a natural language command
    pub async fn process_command(&self, user_input: &str) -> Result<String> {
        debug!("📝 Processing command: {}", user_input);
        
        // Use Claude with tools to process the command
        // Claude will automatically call the appropriate tools based on the user's request
        let response = self.claude_agent
            .prompt(user_input)
            .await?;
            
        debug!("🤖 Claude response: {}", response);
        
        Ok(response)
    }

    /// Process a command with chat history for conversational interactions
    pub async fn process_chat(&self, user_input: &str, history: Vec<Message>) -> Result<String> {
        debug!("📝 Processing chat command: {}", user_input);
        
        // Use Claude's chat functionality with conversation history
        let response = self.claude_agent
            .chat(user_input, history)
            .await?;
            
        debug!("🤖 Claude chat response: {}", response);
        
        Ok(response)
    }

    /// Generate the system prompt for Claude
    fn get_system_prompt() -> String {
        r#"You are an AI agent that helps users interact with the Ethereum blockchain using natural language.

Your role:
1. Understand user requests for blockchain operations
2. Use the available tools to perform blockchain operations
3. Provide clear, helpful responses about blockchain operations

You have access to these tools:
- check_balance: Check ETH or token balances for addresses
- transfer_eth: Send ETH from one address to another  
- is_contract_deployed: Check if a contract is deployed at an address

Key addresses to know:
- Alice: 0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266
- Bob: 0x70997970C51812dc3A010C7d01b50e0d17dc79C8
- Uniswap V2 Router: 0x7a250d5630B4cF539739dF2C5dAcb4c659F2488D

When users ask about blockchain operations:
1. Identify what operation they want (balance check, transfer, contract check)
2. Call the appropriate tool with the correct parameters
3. Provide a clear, informative response based on the tool results

Be helpful, accurate, and explain blockchain concepts when needed.
Always confirm transaction details when performing transfers.

If a user mentions "Alice" or "Bob", use their respective addresses.
For balance checks, you can check both ETH and token balances.
For transfers, always specify the amount, from address, and to address clearly."#.to_string()
    }

    /// Call MCP server (will be implemented when MCP server is ready)
    #[allow(dead_code)]
    async fn call_mcp_server(&self, tool_name: &str, params: serde_json::Value) -> Result<String> {
        let request_body = json!({
            "tool": tool_name,
            "parameters": params
        });

        let response = self.http_client
            .post(&format!("{}/tools/call", self.mcp_server_url))
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await?;

        if response.status().is_success() {
            let result: serde_json::Value = response.json().await?;
            Ok(result.to_string())
        } else {
            let error_text = response.text().await?;
            Err(anyhow::anyhow!("MCP server error: {}", error_text))
        }
    }
}
