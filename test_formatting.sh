#!/bin/bash

echo "🧪 Testing Improved Response Formatting"
echo "========================================"

# Check if MCP server is running
if ! curl -s http://127.0.0.1:8080/mcp > /dev/null 2>&1; then
    echo "❌ MCP server is not running. Please start it first:"
    echo "   cargo run --package mcp-server"
    exit 1
fi

echo "✅ MCP server is running"

# Check environment variables
if [ -z "$ANTHROPIC_API_KEY" ]; then
    echo "❌ ANTHROPIC_API_KEY not set"
    echo "   Please set your Claude API key:"
    echo "   export ANTHROPIC_API_KEY='your-api-key-here'"
    exit 1
fi

echo "✅ Environment variables configured"

echo ""
echo "🚀 Testing improved formatting with a simple balance query..."
echo "   This will show how the new formatting makes responses more readable"
echo ""

# Run a simple test command
echo "How much ETH does Alice have?" | cargo run --package rig-client -- --mcp-server http://127.0.0.1:8080/mcp 2>/dev/null | head -20

echo ""
echo "🎉 Formatting test completed!"
echo "   The response should now be properly formatted with:"
echo "   • Visual separators"
echo "   • Proper indentation"
echo "   • Clear section breaks"
echo "   • Better readability"
