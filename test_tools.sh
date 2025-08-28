#!/bin/bash

# Quick test script for MCP server tools
# Usage: ./test_tools.sh

echo "🔧 MCP Server Tool Testing Script"
echo "=================================="

SERVER_URL="http://127.0.0.1:8080"

# Check if server is running
echo "🔗 Checking server connectivity..."
if ! curl -s "$SERVER_URL/mcp" > /dev/null; then
    echo "❌ Server not accessible at $SERVER_URL"
    echo "💡 Make sure to start the server: cd mcp-server && cargo run"
    exit 1
fi
echo "✅ Server is running"

echo ""
echo "🧪 Testing Balance Tool..."
echo "========================="

# Test Alice's balance
echo "📋 Alice's balance:"
curl -s -X POST "$SERVER_URL/mcp/call_tool" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "balance",
    "arguments": {
      "who": "0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266"
    }
  }' | jq '.'

echo ""
echo "📋 Bob's balance:"
curl -s -X POST "$SERVER_URL/mcp/call_tool" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "balance",
    "arguments": {
      "who": "0x70997970C51812dc3A010C7d01b50e0d17dc79C8"
    }
  }' | jq '.'

echo ""
echo "🧪 Testing Send ETH Tool..."
echo "=========================="

# Send 0.1 ETH from Alice to Bob
echo "📋 Sending 0.1 ETH from Alice to Bob:"
curl -s -X POST "$SERVER_URL/mcp/call_tool" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "send_eth",
    "arguments": {
      "to": "0x70997970C51812dc3A010C7d01b50e0d17dc79C8",
      "amount": "0.1"
    }
  }' | jq '.'

echo ""
echo "⏳ Waiting for transaction to be mined..."
sleep 3

echo ""
echo "📋 Bob's balance after transfer:"
curl -s -X POST "$SERVER_URL/mcp/call_tool" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "balance",
    "arguments": {
      "who": "0x70997970C51812dc3A010C7d01b50e0d17dc79C8"
    }
  }' | jq '.'

echo ""
echo "🧪 Testing Contract Deployment Tool..."
echo "====================================="

# Check if Uniswap V2 Router is deployed
echo "📋 Checking Uniswap V2 Router deployment:"
curl -s -X POST "$SERVER_URL/mcp/call_tool" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "is_contract_deployed",
    "arguments": {
      "address": "0x7a250d5630B4cF539739dF2C5dAcb4c659F2488D"
    }
  }' | jq '.'

echo ""
echo "📋 Checking Alice (EOA) deployment status:"
curl -s -X POST "$SERVER_URL/mcp/call_tool" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "is_contract_deployed",
    "arguments": {
      "address": "0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266"
    }
  }' | jq '.'

echo ""
echo "🎯 PRD Requirements Test"
echo "======================="

echo "📋 PRD Test 1: Send 1 ETH from Alice to Bob"
curl -s -X POST "$SERVER_URL/mcp/call_tool" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "send_eth",
    "arguments": {
      "to": "0x70997970C51812dc3A010C7d01b50e0d17dc79C8",
      "amount": "1.0"
    }
  }' | jq '.'

sleep 2

echo ""
echo "📋 PRD Test 2: How much ETH does Alice have?"
curl -s -X POST "$SERVER_URL/mcp/call_tool" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "balance",
    "arguments": {
      "who": "0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266"
    }
  }' | jq '.'

echo ""
echo "📋 PRD Test 3: Is Uniswap V2 Router deployed?"
curl -s -X POST "$SERVER_URL/mcp/call_tool" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "is_contract_deployed",
    "arguments": {
      "address": "0x7a250d5630B4cF539739dF2C5dAcb4c659F2488D"
    }
  }' | jq '.'

echo ""
echo "🎉 All tests completed!"
echo ""
echo "📚 For more comprehensive testing, run:"
echo "   cargo test --test manual_test -- --nocapture"
echo "   cargo test --test integration_tests -- --nocapture"
