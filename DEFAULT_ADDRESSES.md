# Default Addresses Configuration

This document explains how the default sender and recipient addresses are configured in the Ethereum AI Agent system, as required by the PRD.

## PRD Requirements

According to the PRD, the system must handle these basic commands:
```
# Alice is 0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266
# Bob is 0x70997970C51812dc3A010C7d01b50e0d17dc79C8

> send 1 ETH from Alice to Bob
> How much USDC does Alice have?
> Is Uniswap V2 Router (0x7a250d5630B4cF539739dF2C5dAcb4c659F2488D) deployed?
```

## Default Addresses

### Alice (Default Sender)
- **Address**: Account 0 from anvil (dynamically loaded)
- **Role**: Default sender for all transactions
- **Private Key**: Must be set in environment for transactions
- **Environment Variable**: `ALICE_PRIVATE_KEY`

### Bob (Default Recipient)
- **Address**: Account 1 from anvil (dynamically loaded)
- **Role**: Default recipient when no recipient is specified
- **Private Key**: Not available (for security)

## Configuration

### Environment Variables (.env file)

The system uses a `.env` file primarily for the private key (required for transactions):

```bash
# Alice's private key for transaction signing (Account 0 from anvil)
ALICE_PRIVATE_KEY=0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80

# Optional: Other configuration
ANVIL_RPC_URL=http://127.0.0.1:8545
ANTHROPIC_API_KEY=your_anthropic_api_key_here
```

### MCP Server Configuration

The MCP server dynamically loads addresses from anvil and makes them available to the client:

1. **Alice's address** is Account 0 from anvil (default sender)
2. **Bob's address** is Account 1 from anvil (default recipient)
3. **Alice's private key** is loaded from environment for transaction signing
4. **Address validation** properly resolves "alice" and "bob" to their respective addresses

### Client Configuration

The RIG client has a clear system prompt that explains the default addresses:

```
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
```

## Usage Examples

### Valid Commands

```bash
# Explicit sender and recipient
> send 1 ETH from Alice to Bob

# Default sender (Alice) with explicit recipient
> send 0.5 ETH to Bob

# Default sender (Alice) with address recipient
> send 1 ETH to 0x70997970C51812dc3A010C7d01b50e0d17dc79C8

# Balance queries
> How much ETH does Alice have?
> How much USDC does Bob have?
```

### Address Resolution

The system can resolve addresses in multiple ways:

1. **Names**: "alice" → Alice's address, "bob" → Bob's address
2. **ENS**: "vitalik.eth" → resolved address
3. **Direct**: "0x742d35Cc6634C0532925a3b8D8C9C0C4e8C6C85b"
4. **Account numbers**: "account0" → first anvil account

## Testing

Use the test script to verify configuration:

```bash
./test_default_addresses.sh
```

Or use the MCP tool to get current configuration:

```bash
> get default addresses
```

## Troubleshooting

### Common Issues

1. **"Private key not available"**: Set `ALICE_PRIVATE_KEY` in your `.env` file
2. **"Invalid recipient address"**: Use valid Ethereum addresses or known names (alice, bob)
3. **"MCP server not running"**: Start the MCP server with `cd mcp-server && cargo run`

### Verification Commands

```bash
# Check if addresses are properly configured
> get default addresses

# Test address resolution
> send 0.001 ETH to Bob

# Check account balances
> How much ETH does Alice have?
> How much ETH does Bob have?
```

## Implementation Details

### MCP Server Changes

1. **Dynamic Loading**: Loads Account 0 and Account 1 from anvil as Alice and Bob
2. **Address Validation**: Properly resolves "alice" and "bob" to their addresses
3. **New Tool**: `get_default_addresses` tool shows current configuration
4. **Clear Logging**: Shows default addresses on startup

### Client Changes

1. **Enhanced System Prompt**: Clear rules about default addresses
2. **Better Help Text**: Shows default addresses in help
3. **Tool Validation**: Includes new `get_default_addresses` tool
4. **Example Commands**: Shows proper usage examples

This configuration ensures that the AI agent always knows who the default sender and recipient are, eliminating confusion and following the PRD requirements exactly.
