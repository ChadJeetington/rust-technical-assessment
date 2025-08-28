# MCP Server Testing Guide

This guide shows you how to test all your Foundry tool commands to verify they're working correctly with passed parameters and see the returned results.

## Prerequisites

1. **Start Anvil** (in a separate terminal):
   ```bash
   anvil --fork-url https://eth-mainnet.g.alchemy.com/v2/4UjEl1ULr2lQYsGR5n7gGKd3pzgAzxKs
   ```

2. **Start the MCP Server** (in another terminal):
   ```bash
   cd rust-technical-assessment/mcp-server
   cargo run
   ```
   
   The server should start on `http://127.0.0.1:8080`

## Testing Options

### Option 1: Quick Bash Script Testing (Recommended)

Run the comprehensive bash script that tests all tools:

```bash
../test_tools.sh
```

This script will:
- Check server connectivity
- Test all three tools (balance, send_eth, is_contract_deployed) 
- Run the exact PRD examples
- Show JSON responses for each call

### Option 2: Basic Rust Tests

Run the basic compilation and structure tests:

```bash
cargo test --test server_tests -- --nocapture
```

This validates:
- BlockchainService can be created
- Request structures serialize correctly  
- PRD addresses are valid

## Test Cases Covered

### 1. Balance Tool Tests
- ✅ Alice's balance (10000 ETH from anvil)
- ✅ Bob's balance 
- ✅ Other test accounts
- ✅ Zero address
- ✅ Invalid address formats
- ✅ ENS names (may fail on local anvil)

### 2. Send ETH Tool Tests
- ✅ Send 1 ETH from Alice to Bob
- ✅ Send small amounts (0.1 ETH)
- ✅ Send to multiple recipients
- ✅ Verify balance changes after transfers
- ✅ Invalid recipient addresses
- ✅ Custom amounts

### 3. Contract Deployment Tool Tests
- ✅ Check EOA addresses (should be NOT DEPLOYED)
- ✅ Check known contract addresses (Uniswap V2 Router)
- ✅ Check zero address
- ✅ Check random addresses
- ✅ Invalid address formats

### 4. PRD Requirements Tests
Tests the exact examples from the PRD:
- ✅ "send 1 ETH from Alice to Bob"
- ✅ "How much USDC does Alice have?" (using ETH)
- ✅ "Is Uniswap V2 Router (0x7a250d5630B4cF539739dF2C5dAcb4c659F2488D) deployed?"

## Expected Results

### Alice's Initial Balance
- Should show ~10000 ETH (10000000000000000000000 wei)

### Bob's Initial Balance  
- Should show ~10000 ETH (10000000000000000000000 wei)

### Transfer Results
- Should return transaction hash
- Balances should update after transfers
- Alice's balance decreases, recipient's balance increases

### Contract Deployment
- EOA addresses: "NOT DEPLOYED"
- Contract addresses: "DEPLOYED" or "NOT DEPLOYED" depending on fork state
- Uniswap V2 Router: May show "NOT DEPLOYED" on fresh anvil fork

## Troubleshooting

### Server Connection Issues
```
❌ Server connection failed
💡 Make sure to start the server first: cargo run
```
**Solution**: Ensure the MCP server is running on port 8080

### Anvil Connection Issues
```
❌ Error: failed to get balance
```
**Solution**: Ensure anvil is running on port 8545 with the fork URL

### Transaction Failures
```
❌ Transfer error: transaction failed
```
**Solution**: Check that Alice has sufficient balance and anvil is mining blocks

## Sample Test Output

```
🧪 Testing balance tool...
📋 Test 1: Alice's balance
✅ Alice's balance: CallToolResult { 
    content: [Content { text: "10000000000000000000000" }], 
    is_error: Some(false) 
}

📋 PRD Test 1: Send 1 ETH from Alice to Bob
✅ PRD Transfer result: CallToolResult { 
    content: [Content { text: "Transaction sent: 0x1234..." }], 
    is_error: Some(false) 
}

📋 PRD Test 3: Is Uniswap V2 Router deployed?
✅ PRD Router deployment: CallToolResult { 
    content: [Content { text: "Contract at 0x7a250d5630B4cF539739dF2C5dAcb4c659F2488D is NOT DEPLOYED" }], 
    is_error: Some(false) 
}
```

## Advanced Testing

### Custom Parameters
The interactive test allows you to input custom:
- Addresses for balance checks
- Recipients and amounts for transfers  
- Contract addresses for deployment checks

### Error Testing
Both test suites include error condition testing:
- Invalid address formats
- Missing parameters
- Non-existent tools
- Network connectivity issues

This comprehensive testing ensures all your Foundry tool commands work correctly with the MCP framework!
