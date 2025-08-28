#!/bin/bash

# Test script for RIG Client with MCP Server
# This script tests the complete PRD functionality

set -e

echo "🧪 Testing RIG Client with MCP Server"
echo "======================================"

# Check if MCP server is running
echo "📋 Checking if MCP server is running..."
if ! curl -s http://127.0.0.1:8080/mcp > /dev/null 2>&1; then
    echo "❌ MCP server is not running on http://127.0.0.1:8080/mcp"
    echo "💡 Start the MCP server first:"
    echo "   cd mcp-server && cargo run"
    exit 1
fi
echo "✅ MCP server is running"

# Check if .env file exists with required variables
echo "📋 Checking environment variables..."
if [ ! -f .env ]; then
    echo "⚠️  No .env file found. Creating one with required variables..."
    cat > .env << EOF
# Anthropic API Key (required for RIG client)
ANTHROPIC_API_KEY=your_anthropic_api_key_here

# Private key for Alice (required for transactions)
ALICE_PRIVATE_KEY=0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80
EOF
    echo "📝 Created .env file. Please add your ANTHROPIC_API_KEY"
    echo "💡 You can get a free API key from: https://console.anthropic.com/"
    exit 1
fi

# Check if ANTHROPIC_API_KEY is set
if ! grep -q "ANTHROPIC_API_KEY=" .env || grep -q "ANTHROPIC_API_KEY=your_anthropic_api_key_here" .env; then
    echo "❌ ANTHROPIC_API_KEY not set in .env file"
    echo "💡 Please add your Anthropic API key to the .env file"
    exit 1
fi
echo "✅ Environment variables configured"

# Build the RIG client
echo "🔨 Building RIG client..."
cd rig-client
cargo build --release
echo "✅ RIG client built successfully"

# Test the client with a simple command
echo "🧪 Testing RIG client with MCP server..."
echo "📝 Running: cargo run -- --mcp-server http://127.0.0.1:8080/mcp"
echo "💡 In the REPL, try these PRD commands:"
echo "   • test-connection"
echo "   • send 1 ETH from Alice to Bob"
echo "   • How much USDC does Alice have?"
echo "   • Is Uniswap V2 Router (0x7a250d5630B4cF539739dF2C5dAcb4c659F2488D) deployed?"
echo "   • quit"
echo ""
echo "🚀 Starting RIG client REPL..."

# Run the client
cargo run -- --mcp-server http://127.0.0.1:8080/mcp
