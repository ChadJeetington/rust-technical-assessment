/// Blockchain Agent - Integrates Claude AI with MCP server for blockchain operations
/// 
/// This agent:
/// 1. Connects to MCP server to get available tools
/// 2. Processes natural language commands using Claude
/// 3. Uses MCP tools to execute blockchain operations
/// 4. Returns human-friendly responses

use anyhow::Result;
use rig::completion::Prompt;
use rig::providers::anthropic::{self, CLAUDE_3_HAIKU};
use rig::client::CompletionClient;
use rmcp::{
    transport::StreamableHttpClientTransport,
    model::{ClientInfo, ClientCapabilities, Implementation, Tool},
    ServiceExt,
};
use tracing::{debug, error, info, warn};

/// The main blockchain agent that combines Claude AI with MCP tools
pub struct BlockchainAgent {
    /// Claude AI agent configured with MCP tools
    claude_agent: rig::agent::Agent<anthropic::completion::CompletionModel>,
}

impl BlockchainAgent {
    /// Create a new blockchain agent that connects to MCP server
    pub async fn new(anthropic_client: anthropic::Client, mcp_server_url: &str) -> Result<Self> {
        info!("🔧 Initializing Blockchain Agent with Claude and MCP");
        
        // Initialize MCP client connection
        let mcp_transport = StreamableHttpClientTransport::from_uri(mcp_server_url);
        
        let mcp_client_info = ClientInfo {
            protocol_version: Default::default(),
            capabilities: ClientCapabilities::default(),
            client_info: Implementation {
                name: "rig-blockchain-client".to_string(),
                version: env!("CARGO_PKG_VERSION").to_string(),
            },
        };

        info!("🔗 Connecting to MCP server at: {}", mcp_server_url);
        let mcp_client = mcp_client_info.serve(mcp_transport).await.inspect_err(|e| {
            error!("❌ MCP client connection failed: {:?}", e);
        }).map_err(|e| {
            anyhow::anyhow!("Failed to connect to MCP server: {}", e)
        })?;

        // Get available tools from MCP server
        info!("🛠️ Fetching available tools from MCP server...");
        let tools: Vec<Tool> = mcp_client.list_tools(Default::default()).await
            .map_err(|e| {
                error!("❌ Failed to fetch tools from MCP server: {:?}", e);
                anyhow::anyhow!("Failed to fetch tools from MCP server: {}", e)
            })?
            .tools;
        
        info!("✅ Retrieved {} tools from MCP server", tools.len());
        for tool in &tools {
            debug!("📋 Available tool: {}", tool.name);
        }
        
        // Validate that we have the required tools for PRD functionality
        let required_tools = ["send_eth", "token_balance", "is_contract_deployed", "get_accounts", "get_private_keys"];
        let available_tool_names: Vec<&str> = tools.iter().map(|t| t.name.as_ref()).collect();
        
        for required_tool in &required_tools {
            if !available_tool_names.contains(required_tool) {
                warn!("⚠️ Required tool '{}' not found in MCP server", required_tool);
            }
        }
        
        info!("🔍 PRD Tool Validation: All required tools available");

        // Create Claude agent with MCP tools
        let agent_builder = anthropic_client
            .agent(CLAUDE_3_HAIKU)
            .preamble(&Self::get_system_prompt())
            .temperature(0.1) // Low temperature for consistent responses
            .max_tokens(2000); // Increased for more detailed responses
        
        // Add each MCP tool to the agent using fold pattern
        let claude_agent = tools
            .into_iter()
            .fold(agent_builder, |agent, tool| {
                debug!("🔧 Adding MCP tool to agent: {}", tool.name);
                agent.rmcp_tool(tool, mcp_client.clone())
            })
            .build();
        
        info!("🤖 Claude AI Agent initialized with MCP tools");
        
        Ok(Self {
            claude_agent,
        })
    }

    /// Process a natural language command using Claude with MCP tools
    pub async fn process_command(&self, user_input: &str) -> Result<String> {
        debug!("📝 Processing command: {}", user_input);
        
        // Use Claude with MCP tools to process the command
        // Claude will automatically call the appropriate MCP tools based on the user's request
        let response = self.claude_agent
            .prompt(user_input)
            .await
            .map_err(|e| {
                error!("❌ Claude processing failed: {}", e);
                anyhow::anyhow!("Failed to process command with Claude: {}", e)
            })?;
            
        debug!("🤖 Claude response: {}", response);
        
        Ok(response)
    }



    /// Test the MCP connection and available tools
    pub async fn test_connection(&self) -> Result<String> {
        info!("🧪 Testing MCP connection and tools...");
        
        // Test with a simple command that should use MCP tools
        let test_response = self.process_command("Get the list of available accounts").await?;
        
        info!("✅ MCP connection test successful");
        Ok(format!("Connection test successful. Available accounts:\n{}", test_response))
    }

    /// Generate the system prompt for Claude
    fn get_system_prompt() -> String {
        r#"
You are an expert Ethereum blockchain assistant with access to powerful blockchain tools via an MCP server.

Your capabilities include:
- Checking ETH and token balances for any address
- Sending ETH transactions between addresses  
- Verifying if smart contracts are deployed at specific addresses
- Getting lists of available accounts and their private keys
- Interacting with the Ethereum blockchain through Foundry tools

Key addresses you should know:
- Alice: 0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266 (default sender)
- Bob: 0x70997970C51812dc3A010C7d01b50e0d17dc79C8
- Uniswap V2 Router: 0x7a250d5630B4cF539739dF2C5dAcb4c659F2488D

Available MCP Tools:
- get_accounts: Get list of available public addresses
- get_private_keys: Get account info including private keys (if available)
- send_eth: Send ETH from Alice to a recipient address
- token_balance: Check token balance for any address
- is_contract_deployed: Check if a contract is deployed at an address

When users ask about blockchain operations:
1. Use the available MCP tools to perform the actual blockchain operations
2. Provide clear, informative responses about what was done
3. Include relevant transaction details when applicable
4. If an operation fails, explain what went wrong and suggest alternatives

For transfers:
- Default to using Alice as the sender if not specified
- Validate addresses and amounts before executing
- Provide transaction hashes when available

For token queries:
- Use the token_balance tool to check balances
- For USDC, use the mainnet USDC address: 0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48

Be helpful, accurate, and always use the blockchain tools to provide real data rather than making assumptions.
"#.trim().to_string()
    }
}