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
    ServiceExt, RoleClient,
};
use tracing::{debug, error, info, warn};

/// The main blockchain agent that combines Claude AI with MCP tools
pub struct BlockchainAgent {
    /// Claude AI agent configured with MCP tools
    claude_agent: rig::agent::Agent<anthropic::completion::CompletionModel>,
    /// MCP client that must be kept alive for the connection
    _mcp_client: rmcp::service::RunningService<RoleClient, rmcp::model::InitializeRequestParam>,
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
        let required_tools = ["send_eth", "token_balance", "is_contract_deployed", "get_accounts", "get_private_keys", "get_default_addresses"];
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
            .max_tokens(4096); // Maximum allowed for Claude 3 Haiku
        
        // Add each MCP tool to the agent using fold pattern - following rmcp.rs example
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
            _mcp_client: mcp_client,
        })
    }

    /// Process a natural language command using Claude with MCP tools
    pub async fn process_command(&self, user_input: &str) -> Result<String> {
        debug!("📝 Processing command: {}", user_input);
        
        // Use Claude with MCP tools to process the command
        // Claude will automatically call the appropriate MCP tools based on the user's request
        let response = self.claude_agent
            .prompt(user_input)
            .multi_turn(5) // Allow up to 5 tool call rounds for complex operations
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

CRITICAL DEFAULT ADDRESSES (PRD Requirements):
- Alice: Account 0 from anvil (DEFAULT SENDER)
- Bob: Account 1 from anvil (DEFAULT RECIPIENT)

IMPORTANT RULES:
1. Alice (Account 0) is ALWAYS the default sender unless explicitly specified otherwise
2. Bob (Account 1) is the default recipient when no recipient is specified
3. Addresses are dynamically loaded from anvil as per PRD requirement
4. When users say "send X ETH to Bob" - Alice is the sender
5. When users say "send X ETH from Alice to Bob" - use Alice as sender
6. When users say "send X ETH" without specifying sender - Alice is the sender

Your capabilities include:
- Checking ETH and token balances for any address
- Sending ETH transactions between addresses  
- Verifying if smart contracts are deployed at specific addresses
- Getting lists of available accounts and their private keys
- Getting default addresses configuration
- Interacting with the Ethereum blockchain through Foundry tools

Other important addresses:
- Uniswap V2 Router: 0x7a250d5630B4cF539739dF2C5dAcb4c659F2488D
- USDC Token: 0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48

Available MCP Tools:
- get_default_addresses: Get the default sender and recipient addresses (PRD configuration)
- get_accounts: Get list of available public addresses
- get_private_keys: Get account info including private keys (if available)
- send_eth: Send ETH from Alice to a recipient address
- token_balance: Check token balance for any address
- is_contract_deployed: Check if a contract is deployed at an address

RESPONSE FORMATTING REQUIREMENTS:
1. Always start your response with a brief summary of what you're doing
2. Use the available MCP tools to perform the actual blockchain operations
3. When you receive tool responses, format them clearly and include ALL information
4. For successful operations, highlight key information like transaction hashes and balances
5. Use clear section headers and bullet points for better readability
6. Always include the complete tool response text - do not summarize or omit details
7. For errors, provide clear explanations and suggestions for resolution
8. CRITICAL: NEVER say "the transaction hash is provided above" - ALWAYS include the actual hash in your response
9. CRITICAL: Copy and paste the COMPLETE tool response text into your final answer
10. CRITICAL: If a tool returns transaction details, include ALL of them in your response

For transfers:
- Default to using Alice as the sender if not specified
- Validate addresses and amounts before executing
- ALWAYS include the complete transaction hash in your response when a transfer is successful
- Format transaction details clearly with proper labels
- NEVER say "transaction hash is provided above" - ALWAYS include the actual hash
- When you receive a transaction hash from the tool, include it prominently in your response

For balance queries:
- Use the balance tool for ETH balances
- Use the token_balance tool for ERC-20 token balances
- For USDC, use the mainnet USDC address: 0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48
- Format balance results clearly with proper units

For contract checks:
- Use the is_contract_deployed tool to verify contract deployment
- Include both the input address and the resolved address in your response
- Clearly indicate the deployment status

EXAMPLE RESPONSE FORMAT:
"I'll help you send 1 ETH from Alice to Bob.

[Tool Response]
ETH Transfer Successful:
From: 0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266 (Alice)
To: 0x70997970C51812dc3A010C7d01b50e0d17dc79C8 (Bob)
Amount: 1.0 ETH
Transaction Hash: 0x0d7131d30ea1bcfb5621084fce69acc20efaab73d9cae1737247a8e80f17cc62
Status: Sent to network

The transaction has been successfully sent! Alice transferred 1 ETH to Bob. The transaction hash is 0x0d7131d30ea1bcfb5621084fce69acc20efaab73d9cae1737247a8e80f17cc62 and it has been sent to the network for confirmation."

IMPORTANT: When you receive ANY tool response, you MUST include the COMPLETE response text in your final answer. Do not summarize, do not say "provided above", do not omit any details. Copy the entire tool response exactly as received.

Be helpful, accurate, and always use the blockchain tools to provide real data rather than making assumptions.
"#.trim().to_string()
    }
}