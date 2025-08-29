# RAG Implementation for Uniswap Documentation (Bonus Part 2)

This document describes the implementation of the Retrieval-Augmented Generation (RAG) system for Uniswap documentation and contract source code, as specified in the PRD bonus section part 2.

## Overview

The RAG system provides:
1. **Document Storage**: Ingest and store Uniswap V2/V3 documentation, guides, and contract source code
2. **Vector Embeddings**: Use a local embedding model (Fastembed) for efficient similarity search
3. **Context Integration**: Provide relevant docs to the LLM for better responses

## Architecture

### Components

1. **UniswapRagSystem** (`src/rag.rs`)
   - Main RAG system implementation
   - Uses Fastembed for local embeddings
   - In-memory vector store for document storage
   - Search functionality for relevant documentation

2. **UniswapDocument** (`src/rag.rs`)
   - Document structure with metadata
   - Supports different document types (Documentation, ContractCode, Interface, Guide, etc.)
   - Automatic tagging based on content and file paths

3. **Integration with BlockchainAgent** (`src/agent.rs`)
   - RAG system integrated into the main agent
   - Enhanced system prompt with RAG capabilities
   - Search methods for documentation queries

## Features

### Document Types Supported
- **Documentation**: Markdown files, guides, tutorials
- **ContractCode**: Solidity contract source code
- **Interface**: ABI definitions and interface documentation
- **Guide**: How-to guides and best practices
- **Example**: Code examples and sample implementations
- **FAQ**: Frequently asked questions

### Automatic Tagging
The system automatically tags documents based on:
- File path analysis (e.g., "uniswap", "v2", "v3", "router")
- Content analysis (e.g., "swap", "liquidity", "slippage", "exactinput", "exactoutput")

### Sample Documentation Included
The system comes with comprehensive sample documentation covering:

1. **Uniswap V2 Router Interface**
   - `swapExactETHForTokens` function documentation
   - `swapExactTokensForETH` function documentation
   - `swapExactTokensForTokens` function documentation
   - Parameter descriptions and return values

2. **Uniswap V3 Router Interface**
   - `exactInput` function documentation
   - `exactOutput` function documentation
   - Key differences from V2
   - Concentrated liquidity concepts

3. **Slippage Calculation Guide**
   - What is slippage and why it matters
   - Calculation methods for V2 and V3
   - Best practices and examples
   - Implementation code examples

4. **SwapRouter Contract Code**
   - Actual Solidity contract source code
   - Function implementations
   - Import statements and dependencies

## Usage

### CLI Commands

The RAG system is accessible through the CLI with these commands:

```bash
# Initialize RAG system (done automatically on startup)
rag-init [optional_path_to_docs]

# Search for relevant documentation
rag-search "How do I calculate slippage for Uniswap V3?"

# Check RAG system status
rag-status
```

### Example Queries

The system can answer complex queries like:

1. **"How do I calculate slippage for Uniswap V3?"**
   - Returns detailed slippage calculation guide
   - Includes code examples for both V2 and V3
   - Explains best practices and considerations

2. **"What's the difference between exactInput and exactOutput?"**
   - Compares the two V3 functions
   - Explains when to use each
   - Provides parameter differences

3. **"Show me the SwapRouter contract interface"**
   - Returns actual contract source code
   - Shows function signatures and implementations
   - Includes import statements and dependencies

## Technical Implementation

### Dependencies
```toml
# RAG and Vector Store support
rig-fastembed = "0.2.9"
rig-core = { version = "0.18.2", features = ["rmcp", "derive"] }

# File system operations for document loading
walkdir = "2.4"

# UUID generation for document IDs
uuid = { version = "1.6", features = ["v4"] }

# Date and time handling
chrono = { version = "0.4", features = ["serde"] }
```

### Key Classes and Methods

#### UniswapRagSystem
```rust
pub struct UniswapRagSystem {
    index: InMemoryVectorStore<UniswapDocument>,
    embedding_client: FastembedClient,
    document_count: usize,
}

impl UniswapRagSystem {
    pub async fn new() -> Result<Self>
    pub async fn load_documentation(&mut self, docs_path: &Path) -> Result<()>
    pub async fn search(&self, query: &str, limit: usize) -> Result<Vec<(f64, String, UniswapDocument)>>
    pub async fn add_sample_documentation(&mut self) -> Result<()>
}
```

#### UniswapDocument
```rust
#[derive(rig::Embed, Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct UniswapDocument {
    pub id: String,
    pub title: String,
    pub doc_type: DocumentType,
    #[embed]
    pub content: String,
    pub metadata: DocumentMetadata,
}
```

### Vector Store Integration

The system uses RIG's `InMemoryVectorStore` with Fastembed embeddings:

1. **Embedding Generation**: Uses `FastembedModel::AllMiniLML6V2Q` for local embeddings
2. **Vector Storage**: In-memory storage with automatic ID generation
3. **Similarity Search**: Cosine similarity search with configurable limits
4. **Document Retrieval**: Returns ranked results with relevance scores

## Testing

### Manual Testing
```bash
# Start the client
cargo run --bin rig-client

# Test RAG functionality
rag-status
rag-search "How do I calculate slippage for Uniswap V3?"
rag-search "What's the difference between exactInput and exactOutput?"
rag-search "Show me the SwapRouter contract interface"
```

### Automated Testing
```bash
# Run the test script
./test_rag.sh
```

## Performance Considerations

1. **Local Embeddings**: Fastembed provides fast, local embedding generation
2. **In-Memory Storage**: Suitable for small to medium document collections
3. **Efficient Search**: Vector similarity search with configurable result limits
4. **Automatic Initialization**: Sample documentation loaded on startup

## Future Enhancements

1. **Persistent Storage**: Add support for database-backed vector stores
2. **External Documentation**: Load from Uniswap GitHub repositories
3. **Real-time Updates**: Sync with latest documentation changes
4. **Advanced Filtering**: Filter by document type, version, or tags
5. **Caching**: Implement result caching for frequently searched queries

## Integration with PRD Requirements

This implementation satisfies all requirements from the PRD bonus section part 2:

✅ **Document Storage**: Comprehensive storage of Uniswap documentation and contract code  
✅ **Vector Embeddings**: Local embedding model using Fastembed  
✅ **Context Integration**: Relevant docs provided to LLM for better responses  
✅ **Example Queries**: All three example queries work correctly  
✅ **CLI Interface**: Easy-to-use command-line interface  
✅ **Sample Documentation**: Rich sample data covering V2, V3, and best practices  

The RAG system enhances the blockchain agent's capabilities by providing detailed, accurate information about Uniswap functionality, making it a powerful tool for developers working with DeFi protocols.
