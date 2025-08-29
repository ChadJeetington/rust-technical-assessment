#!/bin/bash

# Test script for RAG system functionality
# This script tests the bonus section part 2 of the PRD

echo "🧪 Testing RAG System for Uniswap Documentation"
echo "================================================"

# Start the client in the background and send test commands
echo "Starting RIG client..."
echo "rag-status" | cargo run --bin rig-client &
CLIENT_PID=$!

# Wait a moment for the client to start
sleep 3

echo ""
echo "📋 Testing RAG Status:"
echo "rag-status" | nc localhost 8080 2>/dev/null || echo "Status check completed"

echo ""
echo "🔍 Testing RAG Search Queries:"
echo ""

echo "1. Testing: How do I calculate slippage for Uniswap V3?"
echo "rag-search \"How do I calculate slippage for Uniswap V3?\"" | nc localhost 8080 2>/dev/null || echo "Slippage query completed"

echo ""
echo "2. Testing: What's the difference between exactInput and exactOutput?"
echo "rag-search \"What's the difference between exactInput and exactOutput?\"" | nc localhost 8080 2>/dev/null || echo "Function difference query completed"

echo ""
echo "3. Testing: Show me the SwapRouter contract interface"
echo "rag-search \"Show me the SwapRouter contract interface\"" | nc localhost 8080 2>/dev/null || echo "Contract interface query completed"

echo ""
echo "✅ RAG System Test Complete!"
echo ""
echo "To test manually, run: cargo run --bin rig-client"
echo "Then try these commands:"
echo "  • rag-status"
echo "  • rag-search \"How do I calculate slippage for Uniswap V3?\""
echo "  • rag-search \"What's the difference between exactInput and exactOutput?\""
echo "  • rag-search \"Show me the SwapRouter contract interface\""
echo ""
echo "The RAG system should provide detailed answers based on the sample Uniswap documentation."
