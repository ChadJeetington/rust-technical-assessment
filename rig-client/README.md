# RIG Client - AI Agent for Ethereum Blockchain

This is the RIG (Rust AI Agent Framework) client that provides a CLI REPL interface for interacting with the Ethereum blockchain using natural language commands. It connects to an MCP (Model Context Protocol) server to execute blockchain operations.

## Features

### PRD Required Functionality ✅
- **ETH Transfers**: Send ETH from Alice to any recipient
- **Token Balance Queries**: Check USDC and other token balances
- **Contract Deployment Verification**: Check if contracts are deployed
- **Natural Language Processing**: Use Claude AI to understand user intent
- **Address Validation**: Automatic validation of recipient addresses

### Additional Features
- **Account Management**: List available accounts and their private keys
- **Interactive REPL**: Command-line interface with history and auto-completion
- **Error Handling**: Comprehensive error messages and suggestions
- **Connection Testing**: Built-in tools to test MCP server connectivity

## Prerequisites

1. **Anthropic API Key**: Get a free API key from [Anthropic Console](https://console.anthropic.com/)
2. **MCP Server**: The blockchain MCP server must be running
3. **Foundry/Anvil**: For blockchain operations (handled by MCP server)

## Setup

### 1. Environment Configuration

Create a `.env` file in the project root:

```bash
# Required: Anthropic API Key for Claude AI
ANTHROPIC_API_KEY=your_anthropic_api_key_here

# Required: Private key for Alice (for transactions)
ALICE_PRIVATE_KEY=0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80
```

### 2. Build and Run

```bash
# Build the client
cargo build --release

# Run the client
cargo run -- --mcp-server http://127.0.0.1:8080/mcp
```

### 3. Quick Test

Use the provided test script:

```bash
./test_rig_client.sh
```

## Usage

### Starting the REPL

```bash
cargo run -- --mcp-server http://127.0.0.1:8080/mcp
```

### Available Commands

#### PRD Required Operations
- `send 1 ETH from Alice to Bob` - Transfer ETH between accounts
- `How much USDC does Alice have?` - Check token balances
- `Is Uniswap V2 Router deployed?` - Verify contract deployment

#### Additional Operations
- `Get list of available accounts` - List all available addresses
- `Check account private keys` - Get account information

#### General Commands
- `help` or `h` - Show help information
- `test` or `test-connection` - Test MCP server connection
- `quit`, `exit`, or `q` - Exit the program

### Example Session

```
🔥 Ethereum AI Agent Ready!
💡 Try these PRD commands:
   • send 1 ETH from Alice to Bob
   • How much USDC does Alice have?
   • Is Uniswap V2 Router (0x7a250d5630B4cF539739dF2C5dAcb4c659F2488D) deployed?
   • Type 'quit' or 'exit' to stop

🤖 > test-connection
🧪 Connection test successful. Available accounts:
[Account list from MCP server]

🤖 > send 1 ETH from Alice to Bob
🤖 [Claude AI response with transaction details]

🤖 > How much USDC does Alice have?
🤖 [Claude AI response with USDC balance]

🤖 > quit
👋 Goodbye!
```

## Architecture

```
┌─────────────────┐    MCP Protocol    ┌──────────────────┐
│   RIG Client    │◄──────────────────►│   MCP Server     │
│   (Claude AI)   │                    │                  │
├─────────────────┤                    ├──────────────────┤
│ • CLI REPL      │                    │ • Blockchain     │
│ • Natural Lang  │                    │ • Foundry/Cast   │
│ • User Input    │                    │ • Transaction    │
│ • Response      │                    │ • State Fork     │
└─────────────────┘                    └──────────────────┘
```

## MCP Tools Integration

The client automatically discovers and uses these MCP tools:

- `get_accounts` - Get list of available public addresses
- `get_private_keys` - Get account info including private keys
- `send_eth` - Send ETH from Alice to a recipient
- `token_balance` - Check token balance for any address
- `is_contract_deployed` - Check if a contract is deployed

## Error Handling

The client provides comprehensive error handling:

- **Connection Errors**: Clear messages when MCP server is unavailable
- **API Errors**: Helpful messages for missing API keys
- **Blockchain Errors**: Detailed error messages from MCP server
- **Validation Errors**: Address and parameter validation feedback

## Development

### Building

```bash
# Debug build
cargo build

# Release build
cargo build --release
```

### Testing

```bash
# Run unit tests
cargo test

# Run with verbose output
cargo test -- --nocapture
```

### Logging

Enable verbose logging:

```bash
cargo run -- --verbose --mcp-server http://127.0.0.1:8080/mcp
```

## Troubleshooting

### Common Issues

1. **"ANTHROPIC_API_KEY not set"**
   - Add your API key to the `.env` file
   - Get a free key from [Anthropic Console](https://console.anthropic.com/)

2. **"MCP server connection failed"**
   - Ensure the MCP server is running: `cd mcp-server && cargo run`
   - Check the server URL in the command line arguments

3. **"Failed to fetch tools from MCP server"**
   - Verify the MCP server is healthy
   - Check network connectivity to the server

4. **"Private key not found"**
   - Ensure `ALICE_PRIVATE_KEY` is set in `.env`
   - Use the default Anvil private key if testing locally

### Debug Mode

Run with debug logging to see detailed information:

```bash
RUST_LOG=debug cargo run -- --verbose --mcp-server http://127.0.0.1:8080/mcp
```

## Contributing

1. Follow Rust coding standards
2. Add tests for new functionality
3. Update documentation for API changes
4. Ensure all PRD requirements are met

## License

This project is part of the Rust Technical Assessment.
