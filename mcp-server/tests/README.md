# MCP Blockchain Server Tests

This directory contains modular tests for the MCP Blockchain Server, organized by functionality for better maintainability and targeted testing.

## Test Structure

The tests have been split into the following modular files:

### Core Test Files

- **`service_creation_tests.rs`** - Tests for BlockchainService instantiation and connection
- **`request_structure_tests.rs`** - Tests for request/response serialization and deserialization
- **`address_validation_tests.rs`** - Tests for PRD addresses and anvil account data validation
- **`account_loading_tests.rs`** - Tests for dynamic account loading functionality
- **`token_balance_tests.rs`** - Tests for USDC and other ERC-20 token balance queries
- **`brave_api_tests.rs`** - Tests for Brave Search API integration and swap intent functionality

### Integration and Legacy Files

- **`integration_tests.rs`** - Integration tests that run all modules together
- **`server_tests.rs`** - Legacy test file (simplified, points to modular tests)
- **`test_usdc_direct.rs`** - Direct USDC testing (existing file)

## Running Tests

### Run All Tests
```bash
cargo test
```

### Run Specific Test Categories

```bash
# Service creation and connection tests
cargo test --test service_creation_tests

# Request/response structure tests
cargo test --test request_structure_tests

# Address validation tests
cargo test --test address_validation_tests

# Account loading tests
cargo test --test account_loading_tests

# Token balance tests
cargo test --test token_balance_tests

# Brave API tests
cargo test --test brave_api_tests

# Integration tests
cargo test --test integration_tests
```

### Run Legacy Tests
```bash
cargo test --test server_tests
```

### Run with Output
```bash
# Run with detailed output
cargo test -- --nocapture

# Run specific test with output
cargo test --test service_creation_tests -- --nocapture
```

## Test Categories Explained

### Service Creation Tests
- Tests BlockchainService instantiation
- Verifies provider connection handling
- Tests graceful error handling when anvil is not running

### Request Structure Tests
- Tests serialization/deserialization of all request/response types
- Validates JSON structure and field presence
- Tests optional fields (like private keys)

### Address Validation Tests
- Validates PRD addresses (Alice, Bob, Uniswap Router)
- Tests anvil default account data format
- Validates address and private key formats

### Account Loading Tests
- Tests dynamic account loading logic
- Validates matching of known addresses with private keys
- Tests handling of unknown addresses

### Token Balance Tests
- Tests USDC token balance queries on forked mainnet
- Tests multiple account balance queries
- Validates token contract interaction

### Brave API Tests
- Tests Brave Search API service creation and connection
- Tests web search request/response serialization
- Tests swap intent request/response serialization
- Tests actual web search functionality
- Tests swap intent functionality with search integration

## Prerequisites

Some tests require anvil to be running with a forked mainnet:

```bash
# Start anvil with mainnet fork
anvil --fork-url https://eth-mainnet.g.alchemy.com/v2/YOUR_API_KEY
```

Some tests require a Brave Search API key:

```bash
# Set Brave Search API key
export BRAVE_SEARCH_API_KEY="BSA-xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx"
```

## Test Output

Tests provide detailed output with:
- 🧪 Test category identification
- 📝 Input and expected output descriptions
- ✅ Success indicators
- ⚠️ Warning messages for expected failures
- 📊 Validation details
- 🔚 Test completion markers

## Adding New Tests

When adding new functionality:

1. **Identify the appropriate test category** or create a new one
2. **Add tests to the relevant modular file**
3. **Update this README** if adding new test categories
4. **Consider adding integration tests** for cross-module functionality

## Test Dependencies

Tests use the following dependencies:
- `tokio` for async testing
- `serde_json` for JSON serialization testing
- `alloy_primitives` for address validation
- `mcp_server::services::blockchain` for service types
