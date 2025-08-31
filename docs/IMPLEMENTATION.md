# Implementation Guide - Rust AI Agent System

This guide breaks down the PRD into actionable, testable steps following the exact order from PRD.md to prevent overwhelming code generation and ensure each component works before moving to the next.

## Part 1: Environment Setup (Following PRD Section)

### Step 1.1: Install Foundry
**Goal**: Get Foundry tools installed and working
```bash
# Install Foundry
curl -L https://foundry.paradigm.xyz | bash
foundryup

# Verify installation
forge --version
cast --version
anvil --version
```

**Test Criteria**: All commands return version numbers

### Step 1.2: Set up Anthropic API Key
**Goal**: Ensure Claude API access
- Set `ANTHROPIC_API_KEY` environment variable
- Test with simple API call (optional: use curl to verify)

**Test Criteria**: API key is set and accessible

### Step 1.3: Install RIG Framework Dependencies
**Goal**: Prepare for RIG client development
- Research RIG framework documentation
- Identify required Rust dependencies

**Test Criteria**: Clear understanding of RIG setup requirements

### Step 1.4: Start Foundry Test Network
**Goal**: Get forked Ethereum network running exactly as specified in PRD
```bash
anvil --fork-url https://eth-mainnet.g.alchemy.com/v2/4UjEl1ULr2lQYsGR5n7gGKd3pzgAzxKs
```

**Test Criteria**: 
- Network starts on 127.0.0.1:8545
- Shows exactly 10 test accounts with 10000 ETH each
- Alice (0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266) has correct balance
- Bob (0x70997970C51812dc3A010C7d01b50e0d17dc79C8) has correct balance
- Can query balance: `cast balance 0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266 --rpc-url http://127.0.0.1:8545`

---

## Part 2: Core Requirements - Basic Functionality (Following PRD Section 2.1)

The PRD focuses on implementing the system that can handle these three basic commands:
```
> send 1 ETH from Alice to Bob
> How much USDC does Alice have?
> Is Uniswap V2 Router (0x7a250d5630B4cF539739dF2C5dAcb4c659F2488D) deployed?
```

### Step 2.1: Create Both Project Structures
**Goal**: Set up both components as the PRD describes a two-component system

**MCP Server Structure**:
```
mcp-server/
├── Cargo.toml
├── src/
│   ├── main.rs
│   └── lib.rs
```

**RIG Client Structure**:
```
rig-client/
├── Cargo.toml
├── src/
│   ├── main.rs
│   └── lib.rs
```

**Test Criteria**: Both projects created, `cargo check` passes for both

### Step 2.2: Implement MCP Server with Required Tools
**Goal**: Create MCP server with all three required functionalities

**Implementation Focus** (following PRD architecture):
- Implement `balance` tool using `#[tool]` macro (PRD example)
- Implement `transfer` tool for ETH transfers
- Implement `is_contract_deployed` tool for contract checks
- Connect to anvil network at 127.0.0.1:8545
- Use Anthropic Rust SDK for MCP

**Dependencies** (based on PRD examples):
- Anthropic Rust SDK for MCP
- `tokio` for async
- `serde` and `serde_json`
- Ethereum provider libraries

**Test Criteria**: 
- MCP server starts and exposes all three tools
- Can call each tool independently
- Returns proper `CallToolResult::success()` responses

### Step 2.3: Implement RIG Client with Claude Integration
**Goal**: Create AI Agent Client as specified in PRD

**Implementation Focus** (following PRD requirements):
- CLI REPL interface
- Claude API integration for natural language processing
- MCP client to connect to server
- Handle the three required command types

**Dependencies**:
- RIG framework core
- Claude API client
- MCP client libraries
- `tokio` for async
- `clap` for CLI

**Test Criteria**:
- CLI REPL accepts user input
- Claude API integration works
- Can discover and call MCP server tools

### Step 2.4: Test Required Command #1 - ETH Transfer
**Goal**: Handle "send 1 ETH from Alice to Bob"

**Expected Behavior** (from PRD):
- Parse natural language input
- Identify sender (default to account 0/Alice)
- Validate recipient address (Bob)
- Generate transaction using Foundry
- Execute transaction on forked network
- Return transaction hash and confirmation

**Test Flow**:
1. User types: "send 1 ETH from Alice to Bob"
2. Claude processes natural language
3. Claude calls transfer tool via MCP
4. MCP server executes transaction
5. Returns transaction hash

**Test Criteria**: Transfer completes successfully, balances updated

### Step 2.5: Test Required Command #2 - Balance Query
**Goal**: Handle "How much USDC does Alice have?" (start with ETH)

**Test Flow**:
1. User types: "How much ETH does Alice have?"
2. Claude processes natural language
3. Claude calls balance tool via MCP with Alice's address
4. MCP server queries blockchain at 127.0.0.1:8545
5. Returns balance result

**Test Criteria**: Balance query returns correct amount

### Step 2.6: Test Required Command #3 - Contract Deployment Check
**Goal**: Handle "Is Uniswap V2 Router deployed?"

**Implementation Focus**:
- Check contract at address 0x7a250d5630B4cF539739dF2C5dAcb4c659F2488D
- Return boolean result about deployment status

**Test Flow**:
1. User types: "Is Uniswap V2 Router deployed?"
2. Claude processes natural language
3. Claude calls is_contract_deployed tool via MCP
4. MCP server checks if code exists at address
5. Returns deployment status

**Test Criteria**: Can check contract deployment status

### Step 2.7: Verify All Basic Functionality Requirements
**Goal**: Test all three PRD Section 2.1 requirements together

**Test Cases** (exact PRD examples):
```
> send 1 ETH from Alice to Bob
> How much USDC does Alice have?
> Is Uniswap V2 Router (0x7a250d5630B4cF539739dF2C5dAcb4c659F2488D) deployed?
```

**Test Criteria**: All three commands work end-to-end with expected PRD behavior

---

## Part 3: Bonus Features (Following PRD Section 3)

### Step 3.1: Server-side External API Integration (PRD Section 3.1)
**Goal**: Integrate one external API as specified in PRD

**Options** (from PRD):
- Brave Search API
- DefiLlama API  
- 0x API

**Implementation Focus**:
- Add external API client to MCP server
- Create tools that use external data
- Handle API rate limits and errors

**Test Criteria**: Can successfully call external API and return data

### Step 3.2: Implement Uniswap Swap Example (PRD Section 3.1)
**Goal**: Handle "Use Uniswap V2 Router to swap 10 ETH for USDC on Alice's account"

**Implementation Focus** (following PRD expected behavior):
- Parse swap intent from natural language
- Use external API to find Uniswap contract addresses
- Identify swapExactETHForTokens function
- Get price data from API or calculate parameters
- Formulate call data and execute via MCP server
- Return swap details and transaction hash

**Test Criteria**: Can execute Uniswap swap with natural language command

### Step 3.3: Client-side RAG for Docs (PRD Section 3.2)
**Goal**: Implement RAG system for Uniswap documentation

**Implementation Focus**:
- Document storage for Uniswap V2/V3 docs and contracts
- Local embedding model integration
- Vector search and retrieval
- Context integration with LLM

**Test Cases** (from PRD):
```
> How do I calculate slippage for Uniswap V3?
> What's the difference between exactInput and exactOutput?
> Show me the SwapRouter contract interface
```

**Test Criteria**: RAG system provides relevant documentation context

---

## Development Guidelines

### For Each Step:
1. **Implement minimal functionality first**
2. **Test thoroughly before moving on**
3. **Use AI to generate code, but verify it works**
4. **Keep components simple and focused**
5. **Add error handling incrementally**

### Testing Strategy:
- Test each component in isolation first
- Integration testing only after components work alone
- Use anvil network for all blockchain testing
- Keep test cases simple and reproducible

### AI Usage:
- Generate code for one step at a time
- Ask for specific implementations, not entire systems
- Verify generated code compiles and runs
- Iterate on small pieces rather than large refactors

This approach ensures you have working software at each step and can identify issues early rather than debugging a complex system all at once.
