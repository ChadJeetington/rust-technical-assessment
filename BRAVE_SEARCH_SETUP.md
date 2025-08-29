# Brave Search API Integration Setup

This document explains how to set up the Brave Search API integration for the MCP server.

## Overview

The Brave Search API integration provides web search capabilities to the MCP server, allowing it to:
- Search the web for current information
- Get token prices and market data
- Find contract addresses and documentation

## Setup Steps

### 1. Get a Brave Search API Key

1. Go to [Brave Search API](https://brave.com/search/api/)
2. Sign up for an account
3. Get your API key from the dashboard
4. The API key will look like: `BSA-xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx`

### 2. Set Environment Variable

Set your Brave Search API key as an environment variable:

```bash
export BRAVE_SEARCH_API_KEY="BSA-xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx"
```

Or add it to your `.env` file:

```bash
echo "BRAVE_SEARCH_API_KEY=BSA-xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx" >> .env
```

### 3. Available Tools

The MCP server now exposes these additional tools:

#### Web Search
- **Tool**: `web_search`
- **Description**: Search the web using Brave Search API
- **Parameters**:
  - `query`: Search query string
  - `count`: Number of results (default: 10)
  - `country`: Country code (default: 'us')
  - `search_lang`: Search language (default: 'en')

#### Token Price Search
- **Tool**: `get_token_price`
- **Description**: Get current token price information
- **Parameters**:
  - `token`: Token symbol (e.g., 'USDC', 'ETH')
  - `base_currency`: Base currency (default: 'USD')

#### Contract Information Search
- **Tool**: `get_contract_info`
- **Description**: Search for smart contract information
- **Parameters**:
  - `contract`: Contract name or address (e.g., 'Uniswap V2 Router')
  - `network`: Network (default: 'ethereum')

### 4. Example Usage

Once the server is running, you can use these tools through the RIG client:

```bash
# Search for Uniswap V2 Router contract
> Search for Uniswap V2 Router contract address on Ethereum

# Get current ETH price
> What is the current price of ETH in USD?

# Find contract documentation
> Search for Uniswap V2 Router documentation
```

### 5. Testing the Integration

To test that the Brave Search API is working:

1. Start the MCP server:
   ```bash
   cd mcp-server
   cargo run
   ```

2. The server should log:
   ```
   🔍 Creating Brave Search service
   🔍 Brave Search API integration enabled
   ```

3. If the API key is missing, you'll see an error:
   ```
   BRAVE_SEARCH_API_KEY environment variable not set
   ```

### 6. Error Handling

The integration includes proper error handling for:
- Missing API key
- Network errors
- Invalid API responses
- Rate limiting

All errors are logged and returned as proper MCP error responses.

## Architecture

The Brave Search integration follows the same pattern as the blockchain service:

```
┌─────────────────┐    MCP Protocol    ┌──────────────────┐
│   RIG Agent     │◄──────────────────►│   MCP Server     │
│   (Client)      │                    │                  │
└─────────────────┘                    ├──────────────────┤
                                       │ • Blockchain     │
                                       │ • Brave Search   │
                                       │ • Combined       │
                                       └──────────────────┘
```

The search service is integrated into the combined service, which provides both blockchain and search capabilities through a single MCP server.
