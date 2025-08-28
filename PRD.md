# RustTechnicalAssessment
Technical Assessment - Rust Engineer

The use of AI is strongly encouraged! You should leverage AI to accelerate your learning curve. Put the requirements in context to generate as much as code as possible, as long as it make sense. Think of Anthropic using Claude Code to develop Claude Code. We're looking for 10x AI user for fast iterations as an AI company.

If I'm not seeing trace of vibe coding in your submission, that's and red flag 🚩

## Overview
Build an AI agent system that can interact with Ethereum blockchain using natural language commands. The system consists of two main components:

### AI Agent Client
- Built with RIG framework, provides a CLI REPL for user, and incorporate a client with Claude API key that process user's natural language.
- [0xPlaygrounds](https://github.com/0xPlaygrounds)
- [Anthropic Overview](https://www.anthropic.com/)

### MCP Server
- Uses Anthropic Rust SDK to expose Foundry functionality as tools.
- [Model Context Protocol Specification](https://github.com/modelcontextprotocol/spec)
- [GitHub rust-sdk/examples/servers](https://github.com/modelcontextprotocol/rust-sdk/tree/main/examples/servers)

### Transaction Generation
Foundry is a Smart Contract dev tool that also serves as a light client to most EVM chains. You can send RPC call to blockchain endpoint through cast, which is a component of Foundry.

```bash
$ cast balance vitalik.eth --ether --rpc-url https://reth-ethereum.ithaca.xyz/rpc
```

For this project, you can expose cast method directly as MCP tool call like this:
https://github.com/foundry-rs/foundry/blob/c78faa217c4ee7a60894c4f740f5c5a967ffb97b/crates/cast/src/lib.rs#L267

```rust
#[tool(tool_box)]
impl MyMcp {
    #[tool(description = "Get the balance of an account in wei")]
    async fn balance (
        &self,
        #[tool(param)]
        #[schemars(description = "The address or ENS name to check balance for")]
        who: String,
    ) -> Result<CallToolResult, McpError> {
        let address = NameOrAddress::from(who)
            .resolve(&self.provider)
            .await
            .unwrap();
        let balance = self.provider.get_balance(address).await.unwrap();

        Ok(CallToolResult::success(vec![Content::text(
            balance.to_string(),
        )]))
    }
}
```

## Technical Requirements

### Stack Requirements
- Client Framework: RIG (Rust AI agent framework) 
- Server Framework: Anthropic Rust SDK for MCP
- Blockchain Tool: Foundry 
- Language: Rust
- Interface: CLI REPL

### System Architecture
```
             ┌─────────────────┐    MCP Protocol    ┌──────────────────┐
             │   RIG Agent     │◄──────────────────►│   MCP Server     │
             │   (Client)      │                    │                  │
             ├─────────────────┤                    ├──────────────────┤
User   ◄───► │ • CLI REPL      │                    │ • Foundry - Cast │
Claude ◄───► │ • LLM API Key   │                    │ • Tx Generation  │
             │ • User Input    │                    │ • State Fork     │
             │ • Response      │                    │ • Anthropic SDK  │
             └─────────────────┘                    └──────────────────┘
                      │                                       │
                      │                                       │
                      └───────────────┐           ┌───────────┘
                                      │           │
                                 ┌────▼───────────▼──────┐
                                 │   Forked Ethereum     │
                                 │     Test Network      │
                                 │   (via Foundry)       │
                                 └───────────────────────┘
```

## Part 1: Environment Setup

### Prerequisites
- Install Foundry
- Set up Anthropic API key
- Install RIG framework dependencies

### Foundry Test Network
```bash
$ anvil --fork-url  https://eth-mainnet.g.alchemy.com/v2/4UjEl1ULr2lQYsGR5n7gGKd3pzgAzxKs
```

Your MCP server should access 127.0.0.1:8545 with these test accounts:

Available Accounts
==================
```
(0) 0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266 (10000.000000000000000000 ETH)
(1) 0x70997970C51812dc3A010C7d01b50e0d17dc79C8 (10000.000000000000000000 ETH)
(2) 0x3C44CdDdB6a900fa2b585dd299e03d12FA4293BC (10000.000000000000000000 ETH)
(3) 0x90F79bf6EB2c4f870365E785982E1f101E93b906 (10000.000000000000000000 ETH)
(4) 0x15d34AAf54267DB7D7c367839AAf71A00a2C6A65 (10000.000000000000000000 ETH)
(5) 0x9965507D1a55bcC2695C58ba16FB37d819B0A4dc (10000.000000000000000000 ETH)
(6) 0x976EA74026E726554dB657fA54763abd0C3a0aa9 (10000.000000000000000000 ETH)
(7) 0x14dC79964da2C08b23698B3D3cc7Ca32193d9955 (10000.000000000000000000 ETH)
(8) 0x23618e81E3f5cdF7f54C3d65f7FBc0aBf5B21E8f (10000.000000000000000000 ETH)
(9) 0xa0Ee7A142d267C1f36714E4a8F75612F20a79720 (10000.000000000000000000 ETH)
```

Private Keys
==================
```
(0) 0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80
(1) 0x59c6995e998f97a5a0044966f0945389dc9e86dae88c7a8412f4603b6b78690d
...
```

## Part 2: Core Requirements

### 2.1 Basic Functionality (Required)
Your system must handle these basic commands or answer these questions:
```
# Alice is 0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266
# Bob is 0x70997970C51812dc3A010C7d01b50e0d17dc79C8

> send 1 ETH from Alice to Bob
> How much USDC does Alice have?
> Is Uniswap V2 Router (0x7a250d5630B4cF539739dF2C5dAcb4c659F2488D) deployed?
```

Expected Behavior:
- Parse natural language input
- Identify sender (default to account 0)
- Validate recipient address
- Generate transaction using Foundry
- Execute transaction on forked network
- Return transaction hash and confirmation

## Part 3: Bonus

### 3.1 Server-side External API integration 
Integrate external API into your MCP server, for example:
- [Brave Search API](https://brave.com/search/api/)
- [DefiLlama API](https://defillama.com/docs/api)
- [0x API](https://0x.org/docs/0x-swap-api/introduction)

Example command:
```
> Use Uniswap V2 Router to swap 10 ETH for USDC on Alice's account. 
```

Expected Behavior if you integrate standard web search:
- Parse swap intent
- Integrate with a search API to find current Uniswap contract addresses
- Figure out the correct function to use. Hint: you can prompt the AI to call swapExactETHForTokens(uint256,address[],address,uint256)
- The LLM can either search on the internet, 0x, or DefiLlama to figure out the price, or you can prompt it to use parameters you come up with.
- Formulate the call data and execute through the MCP server
- Return swap details and transaction hash

Example: [Warp - The Agentic Development Environment with Claude 4 🤔](https://warp.work)

### 3.2 Client-side RAG for Docs and Source Code
Implement a Retrieval-Augmented Generation (RAG) system on the client side that stores and queries Uniswap documentation and contract source code.

Requirements:
- Document Storage: Ingest and store Uniswap V2/V3 documentation, guides, and contract source code
- Vector Embeddings: Use a local embedding model 
- Context Integration: Provide relevant docs to the LLM for better responses

Example queries:
```
> How do I calculate slippage for Uniswap V3?
# Agent searches RAG system, finds relevant docs, provides detailed answer

> What's the difference between exactInput and exactOutput?
# Agent retrieves function documentation and explains differences

> Show me the SwapRouter contract interface
# Agent finds and displays relevant contract code sections
```