//! RAG (Retrieval-Augmented Generation) system for Uniswap documentation and contract source code
//! 
//! This module provides:
//! 1. Document storage and management
//! 2. Vector embeddings using local embedding model
//! 3. Context integration for LLM responses
//! 4. Search functionality for Uniswap docs and contracts

use rig::{
    embeddings::EmbeddingsBuilder, vector_store::{in_memory_store::InMemoryVectorStore, VectorStoreIndex},
    Embed,
};
use rig_fastembed::{Client as FastembedClient, FastembedModel};

use serde::{Deserialize, Serialize};
use std::path::Path;
use tracing::{debug, error, info, warn};
use uuid::Uuid;
use walkdir::WalkDir;

/// Document structure for storing Uniswap documentation and contract code
#[derive(rig::Embed, Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct UniswapDocument {
    /// Unique identifier for the document
    pub id: String,
    /// Document title or name
    pub title: String,
    /// Document type (docs, contract, interface, etc.)
    pub doc_type: DocumentType,
    /// The actual content to be embedded and searched
    #[embed]
    pub content: String,
    /// Additional metadata
    pub metadata: DocumentMetadata,
}

/// Types of documents we can store
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum DocumentType {
    Documentation,
    ContractCode,
    Interface,
    Guide,
    Example,
    FAQ,
}

/// Metadata for documents
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct DocumentMetadata {
    /// Source file path (if applicable)
    pub source_path: Option<String>,
    /// Version information
    pub version: Option<String>,
    /// Tags for categorization
    pub tags: Vec<String>,
    /// Creation timestamp
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// RAG system for Uniswap documentation and contracts
pub struct UniswapRagSystem {
    /// Vector store index for similarity search
    index: InMemoryVectorStore<UniswapDocument>,
    /// Fastembed client for local embeddings
    embedding_client: FastembedClient,
    /// Document count for monitoring
    document_count: usize,
}

impl UniswapRagSystem {
    /// Create a new RAG system with local embedding model
    pub async fn new() -> crate::Result<Self> {
        info!("🔧 Initializing Uniswap RAG System with local embeddings");
        
        // Initialize Fastembed client for local embeddings
        let embedding_client = FastembedClient::new();
        let _embedding_model = embedding_client.embedding_model(&FastembedModel::AllMiniLML6V2Q);
        
        // Create empty vector store - we'll populate it later
        let vector_store = InMemoryVectorStore::<UniswapDocument>::from_documents(vec![]);
        
        info!("✅ RAG System initialized with local embedding model");
        
        Ok(Self {
            index: vector_store,
            embedding_client,
            document_count: 0,
        })
    }
    
    /// Load and index Uniswap documentation from a directory
    pub async fn load_documentation(&mut self, docs_path: &Path) -> crate::Result<()> {
        info!("📚 Loading Uniswap documentation from: {}", docs_path.display());
        
        if !docs_path.exists() {
            warn!("⚠️ Documentation path does not exist: {}", docs_path.display());
            return Ok(());
        }
        
        let mut documents = Vec::new();
        
        // Walk through all files in the documentation directory
        for entry in WalkDir::new(docs_path)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
        {
            let path = entry.path();
            
            // Only process text files
            if let Some(ext) = path.extension() {
                if !matches!(ext.to_str(), Some("md") | Some("txt") | Some("sol") | Some("json")) {
                    continue;
                }
            }
            
            match self.load_document_from_file(path).await {
                Ok(doc) => {
                    documents.push(doc);
                    debug!("📄 Loaded document: {}", path.display());
                }
                Err(e) => {
                    error!("❌ Failed to load document {}: {}", path.display(), e);
                }
            }
        }
        
        if documents.is_empty() {
            warn!("⚠️ No documents found in: {}", docs_path.display());
            return Ok(());
        }
        
        // Index the documents
        self.index_documents(documents).await?;
        
        info!("✅ Loaded and indexed {} documentation files", self.document_count);
        Ok(())
    }
    
    /// Load a single document from a file
    async fn load_document_from_file(&self, path: &Path) -> crate::Result<UniswapDocument> {
        let content = tokio::fs::read_to_string(path).await
            .map_err(|e| crate::ClientError::RagError(format!("Failed to read file {}: {}", path.display(), e)))?;
        
        let doc_type = self.infer_document_type(path);
        let title = path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("Unknown")
            .to_string();
        
        let metadata = DocumentMetadata {
            source_path: Some(path.to_string_lossy().to_string()),
            version: None,
            tags: self.extract_tags(&content, path),
            created_at: chrono::Utc::now(),
        };
        
        Ok(UniswapDocument {
            id: Uuid::new_v4().to_string(),
            title,
            doc_type,
            content,
            metadata,
        })
    }
    
    /// Infer document type from file path and content
    fn infer_document_type(&self, path: &Path) -> DocumentType {
        if let Some(ext) = path.extension() {
            match ext.to_str() {
                Some("sol") => DocumentType::ContractCode,
                Some("md") => DocumentType::Documentation,
                Some("json") => DocumentType::Interface,
                _ => DocumentType::Documentation,
            }
        } else {
            DocumentType::Documentation
        }
    }
    
    /// Extract tags from document content and path
    fn extract_tags(&self, content: &str, path: &Path) -> Vec<String> {
        let mut tags = Vec::new();
        
        // Add tags based on file path
        let path_str = path.to_string_lossy().to_lowercase();
        if path_str.contains("uniswap") {
            tags.push("uniswap".to_string());
        }
        if path_str.contains("v2") {
            tags.push("v2".to_string());
        }
        if path_str.contains("v3") {
            tags.push("v3".to_string());
        }
        if path_str.contains("router") {
            tags.push("router".to_string());
        }
        if path_str.contains("factory") {
            tags.push("factory".to_string());
        }
        if path_str.contains("pair") {
            tags.push("pair".to_string());
        }
        
        // Add tags based on content
        let content_lower = content.to_lowercase();
        if content_lower.contains("swap") {
            tags.push("swap".to_string());
        }
        if content_lower.contains("liquidity") {
            tags.push("liquidity".to_string());
        }
        if content_lower.contains("slippage") {
            tags.push("slippage".to_string());
        }
        if content_lower.contains("exactinput") || content_lower.contains("exact_input") {
            tags.push("exactinput".to_string());
        }
        if content_lower.contains("exactoutput") || content_lower.contains("exact_output") {
            tags.push("exactoutput".to_string());
        }
        
        tags
    }
    
    /// Index documents in the vector store
    async fn index_documents(&mut self, documents: Vec<UniswapDocument>) -> crate::Result<()> {
        info!("🔍 Indexing {} documents in vector store", documents.len());
        
        let embedding_model = self.embedding_client.embedding_model(&FastembedModel::AllMiniLML6V2Q);
        
        // Create embeddings for all documents using the documents method
        let embeddings = EmbeddingsBuilder::new(embedding_model.clone())
            .documents(documents.clone())
            .map_err(|e| crate::ClientError::RagError(format!("Failed to add documents: {}", e)))?
            .build()
            .await
            .map_err(|e| crate::ClientError::RagError(format!("Failed to build embeddings: {}", e)))?;
        
        // Create new vector store with embeddings using from_documents_with_id_f
        let vector_store = InMemoryVectorStore::from_documents_with_id_f(embeddings, |doc| doc.id.clone());
        self.index = vector_store;
        
        self.document_count = documents.len();
        info!("✅ Successfully indexed {} documents", self.document_count);
        
        Ok(())
    }
    
    /// Search for relevant documents based on query
    pub async fn search(&self, query: &str, limit: usize) -> crate::Result<Vec<(f64, String, UniswapDocument)>> {
        debug!("🔍 Searching for: '{}' (limit: {})", query, limit);
        
        let embedding_model = self.embedding_client.embedding_model(&FastembedModel::AllMiniLML6V2Q);
        let index = self.index.clone().index(embedding_model);
        
        let req = rig::vector_store::request::VectorSearchRequest::builder()
            .query(query)
            .samples(limit as u64)
            .build()
            .map_err(|e| crate::ClientError::RagError(format!("Failed to build search request: {}", e)))?;
        
        let results = index
            .top_n::<UniswapDocument>(req)
            .await
            .map_err(|e| crate::ClientError::RagError(format!("Search failed: {}", e)))?;
        
        debug!("📋 Found {} relevant documents", results.len());
        Ok(results)
    }
    
    /// Get document count
    pub fn document_count(&self) -> usize {
        self.document_count
    }
    
    /// Add sample Uniswap documentation if no external docs are available
    pub async fn add_sample_documentation(&mut self) -> crate::Result<()> {
        info!("📚 Adding sample Uniswap documentation");
        
        let sample_docs = vec![
            UniswapDocument {
                id: Uuid::new_v4().to_string(),
                title: "Uniswap V2 Router Interface".to_string(),
                doc_type: DocumentType::Interface,
                content: r#"
# Uniswap V2 Router Interface

## swapExactETHForTokens
Swaps an exact amount of ETH for as many output tokens as possible, along the route determined by the path.

```solidity
function swapExactETHForTokens(
    uint amountOutMin,
    address[] calldata path,
    address to,
    uint deadline
) external payable returns (uint[] memory amounts);
```

**Parameters:**
- `amountOutMin`: The minimum amount of output tokens that must be received for the transaction not to revert
- `path`: An array of token addresses. This function will swap ETH for the first token in the path, then swap that token for the second token, and so on
- `to`: Recipient of the output tokens
- `deadline`: Unix timestamp after which the transaction will revert

**Returns:**
- `amounts`: The input token amount and all subsequent output token amounts

## swapExactTokensForETH
Swaps an exact amount of input tokens for as much ETH as possible, along the route determined by the path.

```solidity
function swapExactTokensForETH(
    uint amountIn,
    uint amountOutMin,
    address[] calldata path,
    address to,
    uint deadline
) external returns (uint[] memory amounts);
```

## swapExactTokensForTokens
Swaps an exact amount of input tokens for as many output tokens as possible, along the route determined by the path.

```solidity
function swapExactTokensForTokens(
    uint amountIn,
    uint amountOutMin,
    address[] calldata path,
    address to,
    uint deadline
) external returns (uint[] memory amounts);
```
"#.to_string(),
                metadata: DocumentMetadata {
                    source_path: None,
                    version: Some("V2".to_string()),
                    tags: vec!["uniswap".to_string(), "v2".to_string(), "router".to_string(), "swap".to_string()],
                    created_at: chrono::Utc::now(),
                },
            },
            UniswapDocument {
                id: Uuid::new_v4().to_string(),
                title: "Uniswap V3 Router Interface".to_string(),
                doc_type: DocumentType::Interface,
                content: r#"
# Uniswap V3 Router Interface

## exactInput
Swaps an exact amount of input tokens for as many output tokens as possible, along the route determined by the path.

```solidity
function exactInput(ExactInputParams calldata params) external payable returns (uint256 amountOut);
```

**Parameters:**
- `params`: Struct containing:
  - `path`: The encoded path to swap along
  - `recipient`: The destination address of the output tokens
  - `deadline`: Unix timestamp after which the transaction will revert
  - `amountIn`: The amount of input tokens to send
  - `amountOutMinimum`: The minimum amount of output tokens that must be received for the transaction not to revert

## exactOutput
Swaps as few input tokens as possible for an exact amount of output tokens, along the route determined by the path.

```solidity
function exactOutput(ExactOutputParams calldata params) external payable returns (uint256 amountIn);
```

**Parameters:**
- `params`: Struct containing:
  - `path`: The encoded path to swap along
  - `recipient`: The destination address of the output tokens
  - `deadline`: Unix timestamp after which the transaction will revert
  - `amountOut`: The amount of output tokens to receive
  - `amountInMaximum`: The maximum amount of input tokens that can be required before the transaction reverts

## Key Differences from V2:
1. **Concentrated Liquidity**: V3 allows liquidity providers to concentrate their capital within custom price ranges
2. **Multiple Fee Tiers**: V3 supports multiple fee tiers (0.05%, 0.3%, 1%)
3. **Non-fungible Liquidity Positions**: Liquidity positions are represented as NFTs
4. **Improved Price Oracle**: V3 includes a built-in price oracle with improved accuracy
"#.to_string(),
                metadata: DocumentMetadata {
                    source_path: None,
                    version: Some("V3".to_string()),
                    tags: vec!["uniswap".to_string(), "v3".to_string(), "router".to_string(), "exactinput".to_string(), "exactoutput".to_string()],
                    created_at: chrono::Utc::now(),
                },
            },
            UniswapDocument {
                id: Uuid::new_v4().to_string(),
                title: "Slippage Calculation Guide".to_string(),
                doc_type: DocumentType::Guide,
                content: r#"
# Slippage Calculation for Uniswap

## What is Slippage?
Slippage is the difference between the expected price of a trade and the actual executed price. It occurs due to price movement between when a transaction is submitted and when it's executed.

## Calculating Slippage

### For Uniswap V2:
```solidity
// Calculate minimum output amount based on slippage tolerance
uint256 amountOutMin = amountOut * (10000 - slippageTolerance) / 10000;

// Example: 1% slippage tolerance
uint256 slippageTolerance = 100; // 1% = 100 basis points
uint256 expectedOutput = 1000 * 10**18; // 1000 tokens
uint256 minOutput = expectedOutput * (10000 - 100) / 10000; // 990 tokens
```

### For Uniswap V3:
```solidity
// V3 uses the same principle but with more precise calculations
// due to concentrated liquidity

// For exactInput swaps:
uint256 amountOutMinimum = expectedAmountOut * (10000 - slippageTolerance) / 10000;

// For exactOutput swaps:
uint256 amountInMaximum = expectedAmountIn * (10000 + slippageTolerance) / 10000;
```

## Best Practices:
1. **Set reasonable slippage**: 0.5% to 2% for stable pairs, 5% to 10% for volatile pairs
2. **Use deadline**: Always set a deadline to prevent stale transactions
3. **Monitor gas prices**: High gas prices can increase slippage
4. **Consider MEV**: Front-running can cause additional slippage

## Example Implementation:
```solidity
function swapWithSlippageProtection(
    address tokenIn,
    address tokenOut,
    uint256 amountIn,
    uint256 slippageTolerance // in basis points (100 = 1%)
) external {
    // Calculate minimum output
    uint256 expectedOutput = getExpectedOutput(tokenIn, tokenOut, amountIn);
    uint256 minOutput = expectedOutput * (10000 - slippageTolerance) / 10000;
    
    // Perform swap
    IUniswapV2Router02(router).swapExactTokensForTokens(
        amountIn,
        minOutput,
        getPath(tokenIn, tokenOut),
        msg.sender,
        block.timestamp + 300 // 5 minute deadline
    );
}
```
"#.to_string(),
                metadata: DocumentMetadata {
                    source_path: None,
                    version: None,
                    tags: vec!["uniswap".to_string(), "slippage".to_string(), "guide".to_string(), "v2".to_string(), "v3".to_string()],
                    created_at: chrono::Utc::now(),
                },
            },
            UniswapDocument {
                id: Uuid::new_v4().to_string(),
                title: "SwapRouter Contract Code".to_string(),
                doc_type: DocumentType::ContractCode,
                content: r#"
// SPDX-License-Identifier: GPL-2.0-or-later
pragma solidity >=0.7.5;
pragma abicoder v2;

import '@uniswap/v3-core/contracts/interfaces/callback/IUniswapV3SwapCallback.sol';
import '@uniswap/v3-core/contracts/interfaces/IUniswapV3Pool.sol';
import '@uniswap/v3-core/contracts/libraries/SafeCast.sol';

import '../interfaces/IUniswapV3SwapRouter.sol';
import '../base/PeripheryImmutableState.sol';
import '../base/PeripheryValidation.sol';
import '../libraries/Path.sol';
import '../libraries/PoolAddress.sol';
import '../libraries/CallbackValidation.sol';
import '../libraries/SafeERC20Namer.sol';

/// @title Uniswap V3 Swap Router
/// @notice Router for stateless execution of swaps against Uniswap V3
contract SwapRouter is IUniswapV3SwapRouter, PeripheryImmutableState, PeripheryValidation {
    using Path for bytes;
    using SafeCast for uint256;

    /// @dev Used as the placeholder value for amountInCached, because the computed amount in for an exact output swap
    /// can never actually be this value
    uint256 private constant DEFAULT_AMOUNT_IN_CACHED = type(uint256).max;

    /// @dev Transient storage variable used for returning the computed amount in for an exact output swap.
    uint256 private amountInCached = DEFAULT_AMOUNT_IN_CACHED;

    constructor(address _factory, address _WETH9) PeripheryImmutableState(_factory, _WETH9) {}

    /// @dev Returns the pool for the given token pair and fee. The pool contract may or may not exist.
    function getPool(
        address tokenA,
        address tokenB,
        uint24 fee
    ) private view returns (IUniswapV3Pool) {
        return IUniswapV3Pool(PoolAddress.computeAddress(factory, PoolAddress.getPoolKey(tokenA, tokenB, fee)));
    }

    struct SwapCallbackData {
        bytes path;
        address payer;
    }

    /// @inheritdoc IUniswapV3SwapRouter
    function exactInput(ExactInputParams calldata params)
        external
        payable
        override
        returns (uint256 amountOut)
    {
        amountOut = exactInputInternal(
            params.amountIn,
            params.amountOutMinimum,
            params.path,
            params.recipient,
            params.deadline
        );
    }

    /// @inheritdoc IUniswapV3SwapRouter
    function exactOutput(ExactOutputParams calldata params)
        external
        payable
        override
        returns (uint256 amountIn)
    {
        amountIn = exactOutputInternal(
            params.amountOut,
            params.amountInMaximum,
            params.path,
            params.recipient,
            params.deadline
        );
    }

    /// @dev Performs a single exact input swap
    function exactInputInternal(
        uint256 amountIn,
        uint256 amountOutMinimum,
        bytes calldata path,
        address recipient,
        uint256 deadline
    ) private returns (uint256 amountOut) {
        // it's okay that the payer is fixed to msg.sender here, as they're only paying for the "final" exact input swap, which happens first, and subsequent swaps are paid for within nested callback frames
        exactInputInternal(amountIn, 0, amountOutMinimum, path, recipient, deadline);
    }

    /// @dev Performs a single exact output swap
    function exactOutputInternal(
        uint256 amountOut,
        uint256 amountInMaximum,
        bytes calldata path,
        address recipient,
        uint256 deadline
    ) private returns (uint256 amountIn) {
        // it's okay that the payer is fixed to msg.sender here, as they're only paying for the "final" exact output swap, which happens first, and subsequent swaps are paid for within nested callback frames
        exactOutputInternal(amountOut, amountInMaximum, 0, path, recipient, deadline);
    }
}
"#.to_string(),
                metadata: DocumentMetadata {
                    source_path: None,
                    version: Some("V3".to_string()),
                    tags: vec!["uniswap".to_string(), "v3".to_string(), "contract".to_string(), "router".to_string(), "swap".to_string()],
                    created_at: chrono::Utc::now(),
                },
            },
        ];
        
        self.index_documents(sample_docs).await?;
        info!("✅ Added {} sample documents", self.document_count);
        
        Ok(())
    }
}
