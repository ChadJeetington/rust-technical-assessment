/// Blockchain Agent - Integrates Claude AI with MCP server for blockchain operations
/// 
/// This agent:
/// 1. Connects to MCP server to get available tools
/// 2. Processes natural language commands using Claude
/// 3. Uses MCP tools to execute blockchain operations
/// 4. Returns human-friendly responses

use anyhow::Result;
use rig::completion::{Chat, Message, Prompt};
use rig::providers::anthropic::{self, CLAUDE_3_HAIKU};
use rig::client::CompletionClient;
use rmcp::{
    transport::StreamableHttpClientTransport,
    model::{ClientInfo, ClientCapabilities, Implementation, Tool},
    ServiceExt,
};
use tracing::{debug, error, info};

/// The main blockchain agent that combines Claude AI with MCP tools
pub struct BlockchainAgent {
    /// Claude AI agent configured with MCP tools
    claude_agent: rig::agent::Agent<anthropic::completion::CompletionModel>,
    /// MCP server URL for blockchain operations
    mcp_server_url: String,
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
            mcp_server_url: mcp_server_url.to_string(),
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

    /// Process a command with chat history for conversational interactions
    pub async fn process_chat(&self, user_input: &str, history: Vec<Message>) -> Result<String> {
        debug!("📝 Processing chat command: {}", user_input);
        
        // Use Claude's chat functionality with conversation history
        let response = self.claude_agent
            .chat(user_input, history)
            .await
            .map_err(|e| {
                error!("❌ Claude chat processing failed: {}", e);
                anyhow::anyhow!("Failed to process chat with Claude: {}", e)
            })?;
            
        debug!("🤖 Claude chat response: {}", response);
        
        Ok(response)
    }

    /// Generate the system prompt for Claude
    fn get_system_prompt() -> String {
        r#"
You are an expert Ethereum blockchain assistant with access to powerful blockchain tools via an MCP server.

Your capabilities include:
- Checking ETH and token balances for any address
- Sending ETH transactions between addresses  
- Verifying if smart contracts are deployed at specific addresses
- Interacting with the Ethereum blockchain through Foundry tools

Key addresses you should know:
- Alice: 0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266 (default sender)
- Bob: 0x70997970C51812dc3A010C7d01b50e0d17dc79C8

When users ask about blockchain operations:
1. Use the available MCP tools to perform the actual blockchain operations
2. Provide clear, informative responses about what was done
3. Include relevant transaction details when applicable
4. If an operation fails, explain what went wrong and suggest alternatives

For transfers:
- Default to using Alice as the sender if not specified
- Validate addresses and amounts before executing
- Provide transaction hashes when available

Be helpful, accurate, and always use the blockchain tools to provide real data rather than making assumptions.
"#.trim().to_string()
    }
}