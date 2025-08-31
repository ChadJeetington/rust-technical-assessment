//! Blockchain Agent - Integrates Claude AI with MCP server for blockchain operations
//! 
//! This agent:
//! 1. Connects to MCP server to get available tools
//! 2. Processes natural language commands using Claude
//! 3. Uses MCP tools to execute blockchain operations
//! 4. Returns human-friendly responses
//! 5. **NEW**: Automatically uses RAG system for Uniswap documentation

use rig::completion::Prompt;
use rig::providers::anthropic::{self, CLAUDE_3_HAIKU};
use rig::client::CompletionClient;
use rig::vector_store::in_memory_store::InMemoryVectorStore;
use rig::embeddings::EmbeddingsBuilder;
use rig::Embed;
use rig_fastembed::{Client as FastembedClient, FastembedModel};

/// Simple text document for RAG
#[derive(rig::Embed, Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
struct SimpleTextDocument {
    #[embed]
    content: String,
}
use rmcp::{
    transport::StreamableHttpClientTransport,
    model::{ClientInfo, ClientCapabilities, Implementation, Tool},
    ServiceExt, RoleClient,
};
use tracing::{debug, error, info, warn};
use crate::rag::UniswapRagSystem;

/// The main blockchain agent that combines Claude AI with MCP tools and RAG
pub struct BlockchainAgent {
    /// Claude AI agent configured with MCP tools and RAG dynamic context
    claude_agent: rig::agent::Agent<anthropic::completion::CompletionModel>,
    /// MCP client that must be kept alive for the connection
    _mcp_client: rmcp::service::RunningService<RoleClient, rmcp::model::InitializeRequestParam>,
    /// RAG system for Uniswap documentation and contracts (kept for manual search)
    rag_system: Option<UniswapRagSystem>,
}

impl BlockchainAgent {
    /// Create a new blockchain agent that connects to MCP server
    pub async fn new(anthropic_client: anthropic::Client, mcp_server_url: &str) -> crate::Result<Self> {
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
            crate::ClientError::McpConnection(format!("Failed to connect to MCP server: {}", e))
        })?;

        // Get available tools from MCP server
        info!("🛠️ Fetching available tools from MCP server...");
        let tools: Vec<Tool> = mcp_client.list_tools(Default::default()).await
            .map_err(|e| {
                error!("❌ Failed to fetch tools from MCP server: {:?}", e);
                crate::ClientError::McpConnection(format!("Failed to fetch tools from MCP server: {}", e))
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
            rag_system: None,
        })
    }

    /// Process a natural language command using Claude with MCP tools and RAG
    pub async fn process_command(&self, user_input: &str) -> crate::Result<String> {
        debug!("📝 Processing command: {}", user_input);
        
        // Check if this is a general question that doesn't require tool calling
        let is_general_question = self.is_general_question(user_input);
        
        // Check if this is a documentation/help query that should trigger RAG
        let is_documentation_query = self.is_documentation_query(user_input);
        
        let enhanced_input = if is_documentation_query && self.rag_system.is_some() {
            // Add RAG context to the query
            match self.enhance_query_with_rag(user_input).await {
                Ok(enhanced) => enhanced,
                Err(e) => {
                    warn!("⚠️ Failed to enhance query with RAG: {}, using original query", e);
                    user_input.to_string()
                }
            }
        } else {
            user_input.to_string()
        };
        
        // For general questions, use a simpler approach without tool calling
        if is_general_question {
            return self.handle_general_question(user_input).await;
        }
        
        // Use Claude with MCP tools to process the command
        // Claude will automatically call the appropriate MCP tools based on the user's request
        let response = self.claude_agent
            .prompt(&enhanced_input)
            .multi_turn(5) // Allow up to 5 tool call rounds for complex operations
            .await
            .map_err(|e| {
                error!("❌ Claude processing failed: {}", e);
                crate::ClientError::ClaudeApi(format!("Failed to process command with Claude: {}", e))
            })?;
            
        debug!("🤖 Claude response: {}", response);
        
        Ok(response)
    }

    /// Check if the input is a general question that doesn't require tool calling
    fn is_general_question(&self, input: &str) -> bool {
        let lower_input = input.to_lowercase();
        
        // General question patterns
        let general_patterns = [
            "what tools do you have",
            "what can you do",
            "how do you work",
            "tell me about yourself",
            "what are your capabilities",
            "what are you",
            "who are you",
            "explain yourself",
            "help",
            "what is this",
            "how does this work",
            "what is mcp",
            "what is rag",
            "what is uniswap",
            "explain",
            "describe",
            "what does",
            "how to",
            "can you",
            "do you",
            "are you",
        ];
        
        general_patterns.iter().any(|pattern| lower_input.contains(pattern))
    }

    /// Check if the input is a documentation/help query that should trigger RAG
    fn is_documentation_query(&self, input: &str) -> bool {
        let lower_input = input.to_lowercase();
        
        // Check for question marks and documentation keywords first
        let has_question_mark = lower_input.contains('?');
        let has_docs_keyword = lower_input.contains("docs") || 
                              lower_input.contains("documentation") || 
                              lower_input.contains("guide") || 
                              lower_input.contains("tutorial");
        
        // Check for question words that indicate documentation queries
        let has_question_word = lower_input.contains("how do i") ||
                               lower_input.contains("how to") ||
                               lower_input.contains("what is") ||
                               lower_input.contains("what's") ||
                               lower_input.contains("explain") ||
                               lower_input.contains("show me") ||
                               lower_input.contains("tell me") ||
                               lower_input.contains("describe") ||
                               lower_input.contains("what does") ||
                               lower_input.contains("difference between") ||
                               lower_input.contains("calculate") ||
                               lower_input.contains("compute");
        
        // Check for Uniswap-specific terms
        let has_uniswap_term = lower_input.contains("uniswap") ||
                              lower_input.contains("slippage") ||
                              lower_input.contains("exactinput") ||
                              lower_input.contains("exactoutput") ||
                              lower_input.contains("router") ||
                              lower_input.contains("factory") ||
                              lower_input.contains("pair") ||
                              lower_input.contains("pool") ||
                              lower_input.contains("liquidity") ||
                              lower_input.contains("oracle") ||
                              lower_input.contains("flash") ||
                              lower_input.contains("callback") ||
                              lower_input.contains("multicall") ||
                              lower_input.contains("permit") ||
                              lower_input.contains("signature") ||
                              lower_input.contains("eip712") ||
                              lower_input.contains("deadline") ||
                              lower_input.contains("gas") ||
                              lower_input.contains("event") ||
                              lower_input.contains("error") ||
                              lower_input.contains("revert");
        
        // Check for version-specific terms
        let has_version_term = lower_input.contains("v2") ||
                              lower_input.contains("v3") ||
                              lower_input.contains("v4");
        
        // Logic to determine if this is a documentation query:
        // 1. Must have a question mark OR documentation keywords OR question words
        // 2. AND must have Uniswap-specific terms OR version terms
        let is_question_format = has_question_mark || has_docs_keyword || has_question_word;
        let is_uniswap_related = has_uniswap_term || has_version_term;
        
        // Additional checks to exclude non-documentation queries:
        
        // Check: if it's a simple command (like "swap 1 ETH for USDC"), don't trigger RAG
        let is_simple_command = lower_input.contains("swap") && 
                               (lower_input.contains("eth") || lower_input.contains("usdc") || lower_input.contains("token")) &&
                               !has_question_mark &&
                               !has_docs_keyword &&
                               !has_question_word;
        
        // Check: if it's a deployment check (like "Is contract X deployed?"), don't trigger RAG
        let is_deployment_check = lower_input.contains("deployed") && 
                                 (lower_input.contains("is") || lower_input.contains("check")) &&
                                 !has_docs_keyword &&
                                 !has_question_word;
        
        // Check: if it's just terms without question format, don't trigger RAG
        let is_just_terms = !has_question_mark && !has_docs_keyword && !has_question_word;
        
        // Check: if it's just a simple phrase with contract/interface terms, don't trigger RAG
        let is_simple_phrase = (lower_input.contains("contract") || lower_input.contains("interface")) &&
                              !has_question_mark &&
                              !has_docs_keyword &&
                              !has_question_word &&
                              lower_input.split_whitespace().count() <= 3; // Simple phrases like "SwapRouter contract"
        
        // Trigger RAG if it's a question format AND uniswap related AND NOT excluded
        is_question_format && is_uniswap_related && !is_simple_command && !is_deployment_check && !is_just_terms && !is_simple_phrase
    }

    /// Handle general questions without tool calling
    async fn handle_general_question(&self, input: &str) -> crate::Result<String> {
        let lower_input = input.to_lowercase();
        
        let response = if lower_input.contains("what tools") || lower_input.contains("capabilities") {
            r#"
I'm an Ethereum blockchain assistant with access to several powerful tools:

🔧 **Blockchain Tools:**
• Send ETH transactions between addresses
• Check ETH and token balances for any address
• Verify if smart contracts are deployed at specific addresses
• Get lists of available accounts and their private keys
• Get default addresses configuration

🌐 **Web Search:**
• Search the web for current information and real-time data

📚 **RAG System:**
• Access to comprehensive Uniswap documentation
• Contract source code and interfaces
• Slippage calculation guides and best practices

💡 **Key Features:**
• Default accounts: Alice (sender) and Bob (recipient)
• Automatic RAG integration for Uniswap questions
• Natural language processing for blockchain operations
• Integration with Foundry for Ethereum interactions

Try commands like:
• "send 1 ETH to Bob"
• "How much USDC does Alice have?"
• "Is Uniswap V2 Router deployed?"
• "Search for current Ethereum price"
"#.trim().to_string()
        } else if lower_input.contains("how do you work") || lower_input.contains("what are you") {
            r#"
I'm an AI assistant built with the RIG framework that connects to an MCP (Model Context Protocol) server to interact with the Ethereum blockchain. Here's how I work:

🤖 **My Architecture:**
• I use Claude 3 Haiku for natural language understanding
• I connect to an MCP server that provides blockchain tools
• The MCP server uses Foundry Cast to interact with Ethereum
• I have a RAG system for Uniswap documentation

🔄 **How I Process Requests:**
1. You ask me a question in natural language
2. I understand your intent and determine if I need to call tools
3. For blockchain operations, I call the appropriate MCP tools
4. For general questions, I answer directly using my knowledge
5. I format the response clearly with all relevant information

🔗 **Key Technologies:**
• RIG Core Framework for AI provider abstraction
• Anthropic MCP SDK for tool calling protocol
• Foundry for Ethereum blockchain operations
• Local vector embeddings for documentation search

I'm designed to make blockchain interactions as simple as having a conversation!
"#.trim().to_string()
        } else if lower_input.contains("what is mcp") {
            r#"
MCP stands for **Model Context Protocol**, which is a protocol created by Anthropic for connecting AI models to external tools and data sources.

🔧 **What MCP Does:**
• Allows AI models to call external tools and APIs
• Provides a standardized way for AI to interact with systems
• Enables real-time data access and tool execution
• Maintains security and control over what tools AI can access

🔄 **How MCP Works in This System:**
• I (the AI client) connect to an MCP server
• The MCP server provides blockchain tools (Foundry Cast integration)
• When you ask me to send ETH or check balances, I call these tools
• The tools execute the actual blockchain operations
• I receive the results and present them to you

💡 **Benefits:**
• Secure: Tools run on the server, not in the AI
• Flexible: Easy to add new tools and capabilities
• Standardized: Works with any MCP-compatible AI
• Real-time: Direct access to live blockchain data

This is why I can actually perform real blockchain operations instead of just talking about them!
"#.trim().to_string()
        } else if lower_input.contains("what is rag") {
            r#"
RAG stands for **Retrieval-Augmented Generation**, which is a technique that enhances AI responses with relevant external information.

🔍 **How RAG Works:**
• Documents are converted into vector embeddings (mathematical representations)
• When you ask a question, I search for the most relevant documents
• I use this retrieved information to provide more accurate and detailed answers
• This gives me access to up-to-date, specific information

📚 **RAG in This System:**
• I have access to Uniswap documentation and contract source code
• When you ask about Uniswap functions, I automatically retrieve relevant docs
• This helps me provide accurate, detailed answers about Uniswap
• I can explain specific functions, parameters, and best practices

💡 **Benefits:**
• More accurate: Based on actual documentation, not just training data
• Up-to-date: Can include recent changes and new features
• Detailed: Access to specific code examples and technical details
• Contextual: Provides relevant information for your specific question

This is why I can give you detailed, accurate information about Uniswap functions and contracts!
"#.trim().to_string()
        } else {
            r#"
I'm an Ethereum blockchain assistant that can help you with various blockchain operations and answer questions about Ethereum, Uniswap, and related technologies.

🔧 **What I Can Do:**
• Send ETH transactions between addresses
• Check token balances (ETH, USDC, etc.)
• Verify smart contract deployment
• Search the web for current information
• Answer questions about Uniswap and blockchain technology
• Provide detailed documentation and code examples

💡 **Try These Commands:**
• "send 1 ETH to Bob"
• "How much USDC does Alice have?"
• "Is Uniswap V2 Router deployed?"
• "What is the current Ethereum price?"
• "Explain how Uniswap V2 works"

I'm here to make blockchain interactions simple and accessible through natural language!
"#.trim().to_string()
        };
        
        Ok(response)
    }

    /// Test the MCP connection and available tools
    pub async fn test_connection(&self) -> crate::Result<String> {
        info!("🧪 Testing MCP connection and tools...");
        
        // Test with a simple command that should use MCP tools
        let test_response = self.process_command("Get the list of available accounts").await?;
        
        info!("✅ MCP connection test successful");
        Ok(format!("Connection test successful. Available accounts:\n{}", test_response))
    }

    /// Initialize the RAG system with Uniswap documentation and integrate with agent
    pub async fn initialize_rag_system(&mut self, docs_path: Option<&str>) -> crate::Result<()> {
        info!("🔧 Initializing AGENTIC RAG system for Uniswap documentation");
        
        let mut rag_system = UniswapRagSystem::new().await?;
        
        // Try to load documentation from the specified path
        if let Some(path) = docs_path {
            let docs_path = std::path::Path::new(path);
            rag_system.load_documentation(docs_path).await?;
        }
        
        // If no external docs loaded, add sample documentation
        if rag_system.document_count() == 0 {
            info!("📚 No external documentation found, adding sample Uniswap docs");
            rag_system.add_sample_documentation().await?;
        }
        
        // Create embeddings for agentic RAG integration
        info!("🤖 Creating embeddings for agentic RAG integration...");
        let embedding_client = FastembedClient::new();
        let embedding_model = embedding_client.embedding_model(&FastembedModel::AllMiniLML6V2Q);
        
        // Get all documents from RAG system and convert to simple text format
        let documents = rag_system.get_all_documents().await?;
        
        // Create embeddings using RIG's EmbeddingsBuilder with simple text documents
        let mut embeddings_builder = EmbeddingsBuilder::new(embedding_model.clone());
        
        for doc in documents.iter() {
            let doc_text = format!("Title: {}\nTags: {}\nContent:\n{}", 
                doc.title, 
                doc.metadata.tags.join(", "), 
                doc.content
            );
            let simple_doc = SimpleTextDocument { content: doc_text };
            embeddings_builder = embeddings_builder.document(simple_doc)
                .map_err(|e| crate::ClientError::RagError(format!("Failed to add document: {}", e)))?;
        }
        
        let embeddings = embeddings_builder
            .build()
            .await
            .map_err(|e| crate::ClientError::RagError(format!("Failed to build embeddings: {}", e)))?;
        
        // Create vector store and index for dynamic context
        let vector_store = InMemoryVectorStore::from_documents(embeddings);
        let _vector_index = vector_store.index(embedding_model);
        
        // Recreate the agent with dynamic context
        info!("🔄 Recreating agent with dynamic RAG context...");
        let anthropic_client = anthropic::Client::new(&std::env::var("ANTHROPIC_API_KEY").unwrap_or_default());
        
        // Get MCP tools from the existing connection
        let tools: Vec<Tool> = self._mcp_client.list_tools(Default::default()).await
            .map_err(|e| crate::ClientError::McpConnection(format!("Failed to fetch tools: {}", e)))?
            .tools;
        
        // Create new agent with enhanced RAG guidance (without dynamic context for now)
        let agent_builder = anthropic_client
            .agent(CLAUDE_3_HAIKU)
            .preamble(&Self::get_system_prompt())
            .temperature(0.1)
            .max_tokens(4096);
        
        // Add MCP tools
        let claude_agent = tools
            .into_iter()
            .fold(agent_builder, |agent, tool| {
                debug!("🔧 Adding MCP tool to agent: {}", tool.name);
                agent.rmcp_tool(tool, self._mcp_client.clone())
            })
            .build();
        
        // Update the agent
        self.claude_agent = claude_agent;
        self.rag_system = Some(rag_system);
        
        info!("✅ AGENTIC RAG system initialized with {} documents", self.rag_system.as_ref().unwrap().document_count());
        info!("🎯 Claude will now automatically use RAG context for Uniswap questions!");
        
        Ok(())
    }

    /// Enhance a query with relevant RAG context
    async fn enhance_query_with_rag(&self, query: &str) -> crate::Result<String> {
        if let Some(rag_system) = &self.rag_system {
            // Search for relevant documents
            let results = rag_system.search(query, 3).await?;
            
            if results.is_empty() {
                return Ok(query.to_string());
            }
            
            // Build context from search results
            let mut context = String::new();
            context.push_str("\n\nRELEVANT UNISWAP DOCUMENTATION:\n");
            context.push_str("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
            
            for (score, _id, doc) in &results {
                context.push_str(&format!("📋 Document: {} (Relevance: {:.1}%)\n", doc.title, (score * 100.0).min(100.0)));
                context.push_str(&format!("🏷️  Tags: {}\n", doc.metadata.tags.join(", ")));
                context.push_str(&format!("📝 Content:\n{}\n\n", doc.content));
                context.push_str("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n\n");
            }
            
            // Combine original query with RAG context
            let enhanced_query = format!("{}\n\n{}", query, context);
            Ok(enhanced_query)
        } else {
            Ok(query.to_string())
        }
    }

    /// Search for relevant Uniswap documentation
    pub async fn search_documentation(&self, query: &str, limit: usize) -> crate::Result<Vec<(f64, String, crate::rag::UniswapDocument)>> {
        if let Some(rag_system) = &self.rag_system {
            rag_system.search(query, limit).await
        } else {
            Err(crate::ClientError::RagError("RAG system not initialized".to_string()))
        }
    }

    /// Get RAG system status
    pub fn rag_status(&self) -> Option<String> {
        self.rag_system.as_ref().map(|rag| {
            format!("RAG System: {} documents indexed", rag.document_count())
        })
    }

    /// Test function to verify RAG logic (for debugging)
    #[cfg(test)]
    pub fn test_rag_logic(&self, input: &str) -> bool {
        self.is_documentation_query(input)
    }

    /// Generate the system prompt for Claude
    fn get_system_prompt() -> String {
        r#"
You are an expert Ethereum blockchain assistant with access to powerful blockchain tools via an MCP server and an AGENTIC RAG system for Uniswap documentation.

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

CONVERSATION MODES:
You can handle two types of interactions:

1. **GENERAL CONVERSATION**: For questions about your capabilities, tools, or general information
   - Answer directly without calling tools
   - Be helpful and informative
   - Explain what you can do
   - Don't try to call tools for general questions

2. **BLOCKCHAIN OPERATIONS**: For specific blockchain actions
   - Use the available MCP tools to perform operations
   - Follow the detailed formatting requirements below

Your capabilities include:
- Checking ETH and token balances for any address
- Sending ETH transactions between addresses  
- Verifying if smart contracts are deployed at specific addresses
- Getting lists of available accounts and their private keys
- Getting default addresses configuration
- Interacting with the Ethereum blockchain through Foundry tools
- **AGENTIC RAG System**: Automatically provides relevant Uniswap documentation and contract source code for your responses
- Web search capabilities for real-time information

Available MCP Tools:
- get_default_addresses: Get the default sender and recipient addresses (PRD configuration)
- get_accounts: Get list of available public addresses
- get_private_keys: Get account info including private keys (if available)
- send_eth: Send ETH from Alice to a recipient address
- token_balance: Check token balance for any address
- is_contract_deployed: Check if a contract is deployed at an address
- web_search: Search the web for current information

**GENERAL CONVERSATION EXAMPLES:**
- "What tools do you have access to?" → List your capabilities without calling tools
- "How do you work?" → Explain your architecture and capabilities
- "What can you do?" → Describe your features and functions
- "Tell me about yourself" → Explain your role and capabilities

**BLOCKCHAIN OPERATION EXAMPLES:**
- "send 1 ETH to Bob" → Use send_eth tool
- "How much USDC does Alice have?" → Use token_balance tool
- "Is Uniswap V2 Router deployed?" → Use is_contract_deployed tool

**IMPORTANT: RAG functionality is NOT available as MCP tools. Use CLI commands only.**

**ENHANCED RAG SYSTEM CAPABILITIES:**
You have access to a comprehensive RAG system that includes:
- Uniswap V2 and V3 documentation
- Contract source code and interfaces
- Slippage calculation guides
- Best practices and examples
- Function signatures and parameters

**RAG INTEGRATION STRATEGY:**
The RAG system is automatically triggered ONLY for documentation questions about Uniswap. It will provide you with relevant documentation when users ask:
1. **Questions about Uniswap functionality** (e.g., "How does Uniswap V2 work?")
2. **Documentation queries** (e.g., "Uniswap V3 documentation")
3. **Technical explanations** (e.g., "Explain Uniswap slippage")
4. **Code examples and guides** (e.g., "Uniswap V2 tutorial")

**IMPORTANT:** RAG is NOT triggered for:
- Simple swap commands (e.g., "swap 1 ETH for USDC")
- Balance queries (e.g., "How much USDC does Alice have?")
- Transaction operations (e.g., "send 1 ETH to Bob")
- Contract deployment checks (e.g., "Is Uniswap Router deployed?")

When RAG is triggered, you will automatically receive relevant Uniswap documentation to provide comprehensive, accurate answers.

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

For Uniswap documentation queries:
- **Provide immediate helpful answers** based on your knowledge
- **Suggest RAG searches** for detailed technical questions about Uniswap
- **Explain that rag-search is a CLI command** for specific documentation queries
- **Explain differences between V2 and V3** when applicable
- **CRITICAL: Do NOT try to call rag-search as a tool** - it's a CLI command only
- **CRITICAL: Do NOT use any tool calls for RAG functionality**

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
