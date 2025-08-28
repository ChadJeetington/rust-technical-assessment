/// Blockchain Agent - Integrates Claude AI with MCP server for blockchain operations
/// 
/// This agent:
/// 1. Processes natural language commands using Claude
/// 2. Connects to MCP server to execute blockchain operations
/// 3. Returns human-friendly responses

use anyhow::Result;
use rig::completion::Prompt;
use rig::providers::anthropic::{self, CLAUDE_3_5_SONNET};
use serde_json::json;
use tracing::{debug, info};

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
        
        // Create Claude agent with blockchain-specific system prompt
        let claude_agent = anthropic_client
            .agent(CLAUDE_3_5_SONNET)
            .preamble(&Self::get_system_prompt())
            .temperature(0.1) // Low temperature for consistent responses
            .max_tokens(1000)
            .build();

        let http_client = reqwest::Client::new();
        
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
        
        // Step 1: Use Claude to understand the intent and generate MCP calls
        let claude_response = self.claude_agent
            .prompt(format!("User command: {}", user_input))
            .await?;
            
        debug!("🤖 Claude response: {}", claude_response);
        
        // Step 2: Parse Claude's response to identify blockchain operations needed
        // For now, we'll simulate MCP calls since the MCP server isn't implemented yet
        let blockchain_result = self.simulate_blockchain_operation(user_input).await?;
        
        // Step 3: Have Claude format the final response for the user
        let final_response = self.claude_agent
            .prompt(format!(
                "The user asked: '{}'\n\
                 The blockchain operation result was: {}\n\
                 Please provide a friendly, informative response to the user about what happened.",
                user_input, blockchain_result
            ))
            .await?;
            
        Ok(final_response)
    }

    /// Generate the system prompt for Claude
    fn get_system_prompt() -> String {
        r#"You are an AI agent that helps users interact with the Ethereum blockchain using natural language.

Your role:
1. Understand user requests for blockchain operations
2. Work with an MCP server that provides blockchain tools
3. Provide clear, helpful responses about blockchain operations

Available blockchain operations (via MCP server):
- balance: Check ETH or token balances for addresses
- transfer: Send ETH from one address to another  
- is_contract_deployed: Check if a contract is deployed at an address

Key addresses to know:
- Alice: 0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266
- Bob: 0x70997970C51812dc3A010C7d01b50e0d17dc79C8
- Uniswap V2 Router: 0x7a250d5630B4cF539739dF2C5dAcb4c659F2488D

When users ask about blockchain operations:
1. Identify what operation they want (balance check, transfer, contract check)
2. Extract the relevant parameters (addresses, amounts, etc.)
3. Provide clear, informative responses

Be helpful, accurate, and explain blockchain concepts when needed.
Always confirm transaction details before executing operations."#.to_string()
    }

    /// Simulate blockchain operations (placeholder until MCP server is ready)
    async fn simulate_blockchain_operation(&self, user_input: &str) -> Result<String> {
        let input_lower = user_input.to_lowercase();
        
        if input_lower.contains("send") && input_lower.contains("eth") {
            Ok("Transaction simulated: 1 ETH sent from Alice to Bob. Transaction hash: 0x123...abc".to_string())
        } else if input_lower.contains("balance") || input_lower.contains("how much") {
            Ok("Balance query simulated: Alice has 9999 ETH".to_string())
        } else if input_lower.contains("deployed") || input_lower.contains("uniswap") {
            Ok("Contract check simulated: Uniswap V2 Router is deployed at the specified address".to_string())
        } else {
            Ok("Blockchain operation simulated successfully".to_string())
        }
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
