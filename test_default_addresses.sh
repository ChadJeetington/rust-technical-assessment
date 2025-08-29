#!/bin/bash

# Test script to verify default addresses configuration
echo "🧪 Testing Default Addresses Configuration"
echo "=========================================="

# Check if .env file exists
if [ -f ".env" ]; then
    echo "✅ .env file found"
    echo "📋 Environment variables:"
    grep -E "^(ALICE|BOB)_" .env | while read line; do
        echo "   $line"
    done
else
    echo "❌ .env file not found"
fi

echo ""
echo "🔍 Testing MCP Server Configuration:"
echo "===================================="

# Check if MCP server is running
if curl -s http://127.0.0.1:8080/mcp > /dev/null 2>&1; then
    echo "✅ MCP server is running on port 8080"
else
    echo "❌ MCP server is not running on port 8080"
    echo "   Start it with: cd mcp-server && cargo run"
fi

echo ""
echo "🔍 Testing RIG Client Configuration:"
echo "===================================="

# Check if RIG client can be built
if cd rig-client && cargo check > /dev/null 2>&1; then
    echo "✅ RIG client can be built"
else
    echo "❌ RIG client build failed"
fi

echo ""
echo "📋 Summary of Default Addresses (PRD Requirements):"
echo "=================================================="
echo "👤 Alice (Default Sender): Account 0 from anvil"
echo "👤 Bob (Default Recipient): Account 1 from anvil"
echo ""
echo "💡 Test Commands:"
echo "   • 'send 1 ETH from Alice to Bob'"
echo "   • 'send 0.5 ETH to Bob' (Alice is default sender)"
echo "   • 'get default addresses' (shows configuration)"
echo ""
echo "🔧 To set up private key for transactions:"
echo "   export ALICE_PRIVATE_KEY=0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80"
