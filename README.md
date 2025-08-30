# Rust Technical Assessment - AI Agent System

An AI agent system that can interact with Ethereum blockchain using natural language commands. The system consists of two main components:

## Architecture

```
             ┌─────────────────┐    MCP Protocol    ┌──────────────────┐
             │   RIG Agent     │◄──────────────────►│   MCP Server     │
             │   (Client)      │                    │                  │
             ├─────────────────┤                    ├──────────────────┤
User   ◄───► │ • CLI REPL      │                    │ • Foundry - Cast │
Claude ◄───► │ • LLM API Key   │                    │ • Tx Generation  │
             │ • User Input    │                    │ • State Fork     │
             │ • Response      │                    │ • Anthr MCP SDK  │
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

## Prerequisites

- Rust 1.70+ and Cargo
- Foundry (for blockchain operations)
- Anthropic API key
- Brave Search API key (optional, for search functionality)

## Environment Setup

### 1. Clone and Setup

```bash
git clone <repository-url>
cd rust-technical-assessment
```

### 2. Environment Variables

Create a `.env` file in the root directory:

```bash
# Required: Anthropic API key for RIG client
ANTHROPIC_API_KEY=your_anthropic_api_key_here

# Optional: Brave Search API key for search functionality
BRAVE_SEARCH_API_KEY=your_brave_search_api_key_here

# Optional: Private key for Alice (account 0) for transactions
ALICE_PRIVATE_KEY=0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80

# Optional: Server configuration
MCP_SERVER_HOST=127.0.0.1
MCP_SERVER_PORT=8080
MCP_PATH=/mcp

# Optional: Logging
RUST_LOG=info
```

### 3. Start Foundry Anvil

```bash
# Start anvil with mainnet fork (uses PRD-provided Alchemy key)
anvil --fork-url https://eth-mainnet.g.alchemy.com/v2/4UjEl1ULr2lQYsGR5n7gGKd3pzgAzxKs
```

This will start a local Ethereum node with these test accounts:

```
Available Accounts
==================
(0) 0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266 (10000.000000000000000000 ETH) - Alice
(1) 0x70997970C51812dc3A010C7d01b50e0d17dc79C8 (10000.000000000000000000 ETH) - Bob
(2) 0x3C44CdDdB6a900fa2b585dd299e03d12FA4293BC (10000.000000000000000000 ETH)
...
```

## Building and Running

### 1. Build the Project

```bash
# Build both mcp-server and rig-client
cargo build --release
```

### 2. Start the MCP Server

```bash
# Start the MCP server (blockchain + search tools)
cd mcp-server
cargo run --release
```

The server will start on `http://127.0.0.1:8080/mcp`

### 3. Start the RIG Client

```bash
# In a new terminal, start the RIG client
cd rig-client
cargo run --release
```

## Usage Examples

Once both server and client are running, you can use natural language commands:

```
> send 1 ETH from Alice to Bob
> How much USDC does Alice have?
> Is Uniswap V2 Router (0x7a250d5630B4cF539739dF2C5dAcb4c659F2488D) deployed?
> Use Uniswap V2 Router to swap 10 ETH for USDC on Alice's account
```

## Testing

### Run All Tests

```bash
# Run all tests
cargo test

# Run specific test categories
cargo test --test service_creation_tests
cargo test --test token_balance_tests
cargo test --test swap_tests
```

### Test Structure

- `mcp-server/tests/` - Unit and integration tests for MCP server
- `rig-client/tests/` - Tests for RIG client functionality
- `scripts/tests/` - Shell scripts for end-to-end testing

## Project Structure

```
rust-technical-assessment/
├── Cargo.toml                 # Workspace configuration
├── Cargo.lock                 # Locked dependencies
├── .env.example              # Environment template
├── .gitignore                # Git ignore rules
├── mcp-server/               # MCP Server implementation
│   ├── src/
│   │   ├── services/         # Blockchain and search services
│   │   ├── server.rs         # HTTP server setup
│   │   └── main.rs           # Server entry point
│   └── tests/                # Server tests
├── rig-client/               # RIG Client implementation
│   ├── src/
│   └── tests/                # Client tests
└── scripts/                  # Build and test scripts
```

## Configuration Management

The project uses a centralized configuration system:

- Environment variables for sensitive data (API keys, private keys)
- Default values for non-sensitive configuration
- Validation of required configuration on startup
- Clear error messages for missing configuration

## Error Handling

The project follows Rust best practices for error handling:

- Custom error types using `thiserror`
- Proper error propagation with `?` operator
- No `unwrap()` calls in production code
- Graceful degradation when optional services are unavailable

## Security Considerations

- Private keys are loaded from environment variables only
- No hardcoded API keys in source code
- `.env` files are gitignored
- Clear separation between test and production configuration

## Development

### Adding New Tools

1. Add tool definition in `mcp-server/src/services/`
2. Implement the tool logic
3. Add tests in `mcp-server/tests/`
4. Update documentation

### Code Quality

The project includes:
- Proper error handling patterns
- Comprehensive test coverage
- Clear documentation
- Consistent code formatting
- Dependency version pinning

## Troubleshooting

### Common Issues

1. **Anvil not running**: Make sure anvil is started before running tests
2. **Missing API keys**: Check your `.env` file and environment variables
3. **Port conflicts**: Change `MCP_SERVER_PORT` in your environment
4. **Transaction failures**: Ensure `ALICE_PRIVATE_KEY` is set correctly

### Debug Mode

```bash
# Run with debug logging
RUST_LOG=debug cargo run --bin mcp-server
```

## Contributing

1. Follow Rust coding standards
2. Add tests for new functionality
3. Update documentation
4. Ensure all tests pass before submitting

## License

[Add your license information here]