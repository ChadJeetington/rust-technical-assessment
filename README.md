# Rust Technical Assessment - AI Agent System

**Position**: Rust Engineer  
**Assessment**: AI-Powered Ethereum Blockchain Agent

## 🎯 Project Overview

This project implements an AI agent system that enables natural language interaction with the Ethereum blockchain. The system demonstrates advanced Rust development skills, AI integration capabilities, and blockchain expertise through a two-component architecture leveraging cutting-edge technologies.

## 🏗️ System Architecture

```
             ┌─────────────────┐    MCP Protocol    ┌──────────────────┐
             │   RIG Agent     │◄──────────────────►│   MCP Server     │
             │   (Client)      │                    │                  │
             ├─────────────────┤                    ├──────────────────┤
User   ◄───► │ • CLI REPL      │                    │ • Foundry - Cast │
Claude ◄───► │ • LLM API Key   │                    │ • Tx Generation  │
             │ • User Input    │                    │ • State Fork     │
             │ • Response      │                    │ • Anthropic SDK  │
             └─────────────────┘                    └──────────────────┘
                      │                                       │
                      └───────────────┐           ┌───────────┘
                                      │           │
                                 ┌────▼───────────▼──────┐
                                 │   Forked Ethereum     │
                                 │     Test Network      │
                                 │   (via Foundry)       │
                                 └───────────────────────┘
```

## 🚀 Core Components

### AI Agent Client (RIG Framework)
- **Technology**: RIG (Rust AI agent framework)
- **Interface**: CLI REPL with natural language processing
- **AI Integration**: Claude API for intelligent command interpretation
- **Capabilities**: Processes user intent and orchestrates blockchain operations

### MCP Server (Model Context Protocol)
- **Technology**: Anthropic Rust SDK for MCP implementation
- **Purpose**: Exposes Foundry blockchain tools as standardized MCP tools
- **Integration**: Direct integration with Foundry's cast functionality
- **Network**: Operates on forked Ethereum mainnet for safe testing

## 💡 Key Features & Capabilities

### Core Functionality (Required)
The system handles sophisticated natural language commands:

```bash
# Natural Language Blockchain Operations
> send 1 ETH from Alice to Bob
> How much USDC does Alice have?
> Is Uniswap V2 Router (0x7a250d5630B4cF539739dF2C5dAcb4c659F2488D) deployed?
```

**Technical Implementation**:
- ✅ Natural language parsing and intent recognition
- ✅ Automatic sender identification (defaults to account 0)
- ✅ Address validation and ENS resolution
- ✅ Transaction generation using Foundry toolchain
- ✅ Execution on forked Ethereum network
- ✅ Transaction confirmation and hash reporting

### Advanced Features (Bonus)

#### 🌐 Server-side External API Integration
- **APIs**: Brave Search, DefiLlama, 0x Protocol
- **Capability**: Real-time data integration for DeFi operations
- **Example**: Complex Uniswap swaps with live price data

```bash
> Use Uniswap V2 Router to swap 10 ETH for USDC on Alice's account
```

#### 🧠 Client-side RAG System
- **Technology**: Retrieval-Augmented Generation with local embeddings
- **Data Sources**: Uniswap V2/V3 documentation and contract source code
- **Capability**: Contextual documentation assistance

```bash
> How do I calculate slippage for Uniswap V3?
> What's the difference between exactInput and exactOutput?
> Show me the SwapRouter contract interface
```

## 🛠️ Technology Stack

| Component | Technology | Purpose |
|-----------|------------|---------|
| **Client Framework** | RIG (Rust AI agent framework) | AI agent orchestration |
| **Server Framework** | Anthropic Rust SDK | MCP protocol implementation |
| **Blockchain Tools** | Foundry (forge, cast, anvil) | Ethereum interaction |
| **Language** | Rust | High-performance system development |
| **AI Provider** | Claude API | Natural language processing |
| **Interface** | CLI REPL | User interaction |
| **Network** | Forked Ethereum Mainnet | Safe testing environment |

## 🔧 Development Environment

### Test Network Configuration
```bash
anvil --fork-url https://eth-mainnet.g.alchemy.com/v2/4UjEl1ULr2lQYsGR5n7gGKd3pzgAzxKs
```

**Test Accounts**:
- **Alice**: `0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266` (10,000 ETH)
- **Bob**: `0x70997970C51812dc3A010C7d01b50e0d17dc79C8` (10,000 ETH)
- **Network**: `127.0.0.1:8545`

### Prerequisites
- ✅ **Foundry v1.3.2-stable** (forge, cast, anvil)
- ✅ Anthropic API key configuration
- ✅ RIG framework dependencies
- ✅ **Rust 1.85+ with Edition 2024**

## 📋 Implementation Strategy

This project follows a methodical, AI-assisted development approach:

1. **Environment Setup**: Foundry network, API keys, dependencies
2. **Core MCP Server**: Blockchain tool exposure via MCP protocol
3. **RIG Client Development**: CLI interface with Claude integration
4. **Integration Testing**: End-to-end natural language workflows
5. **Advanced Features**: External APIs and RAG system implementation

## 🎯 Assessment Criteria Alignment

### Technical Excellence
- **Rust Proficiency**: Advanced async/await patterns, error handling, and idiomatic code
- **Blockchain Integration**: Direct Foundry integration with proper transaction handling
- **AI Integration**: Sophisticated natural language processing and tool orchestration
- **Protocol Implementation**: Standards-compliant MCP server development

### Innovation & AI Usage
- **"Vibe Coding"**: AI-assisted developmental approach for quick initial project structure and implementation
- **Modern Architecture**: Cutting-edge MCP protocol implementation
- **Intelligent Design**: Context-aware natural language processing
- **Rapid Iteration**: AI-powered development acceleration

## 📁 Project Structure

```
rust-technical-assessment/
├── .cursorrules              # AI development context
├── PRD.md                   # Product Requirements Document
├── IMPLEMENTATION.md        # Step-by-step development guide
├── README.md               # This file
├── mcp-server/             # Anthropic MCP server implementation
└── rig-client/             # RIG framework AI agent client
```

## 🚀 Getting Started

1. **Review Documentation**: Start with `PRD.md` for complete requirements
2. **Follow Implementation Guide**: Use `IMPLEMENTATION.md` for step-by-step development
3. **Environment Setup**: Configure Foundry, API keys, and dependencies
4. **Incremental Development**: Build and test each component systematically

---

**Note**: This assessment showcases modern Rust development practices, advanced AI integration, and sophisticated blockchain interaction patterns. The implementation demonstrates both technical depth and innovative problem-solving approaches expected in cutting-edge AI/blockchain development.