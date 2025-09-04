//! RAG (Retrieval-Augmented Generation) system for Uniswap documentation and contract source code
//! 
//! This module provides:
//! 1. Document storage and management
//! 2. Vector embeddings using local embedding model
//! 3. Context integration for LLM responses
//! 4. Search functionality for Uniswap docs and contracts

use rig::{
    embeddings::EmbeddingsBuilder, 
    vector_store::{in_memory_store::InMemoryVectorStore, VectorStoreIndex},
    Embed,
};
use rig_fastembed::{Client as FastembedClient, FastembedModel};

use serde::{Deserialize, Serialize};
use std::path::Path;
use tracing::{debug, info, warn};
use crate::doc_ingestion::store::InMemoryDocStore;

/// Document structure for storing Uniswap documentation and contract code with semantic chunking
#[derive(rig::Embed, Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct UniswapDocument {
    /// Unique identifier for the document
    pub id: String,
    /// Document title or name
    pub title: String,
    /// Document type (docs, contract, interface, etc.)
    pub doc_type: DocumentType,
    /// The main content chunk to be embedded and searched
    #[embed]
    pub content: String,
    /// Semantic chunks for better context understanding
    #[embed]
    pub semantic_chunks: Vec<String>,
    /// Code examples if present
    #[embed]
    pub code_examples: Vec<String>,
    /// Function signatures if present
    #[embed]
    pub function_signatures: Vec<String>,
    /// Additional metadata
    pub metadata: DocumentMetadata,
    /// Parent document ID if this is a chunk
    pub parent_id: Option<String>,
    /// Chunk type if this is a semantic chunk
    pub chunk_type: Option<ChunkType>,
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
    Tutorial,
    Reference,
    Explanation,
}

/// Types of semantic chunks for better context understanding
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum ChunkType {
    /// Function or method documentation
    FunctionDoc,
    /// Code example
    Example,
    /// Interface definition
    Interface,
    /// Conceptual explanation
    Concept,
    /// Usage guide
    Usage,
    /// Error handling
    Error,
    /// Best practices
    BestPractice,
    /// Parameter description
    Parameter,
    /// Return value description
    ReturnValue,
    /// Security consideration
    Security,
}

/// Metadata for documents with enhanced context
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct DocumentMetadata {
    /// Source file path (if applicable)
    pub source_path: Option<String>,
    /// Version information with semantic versioning
    pub version: Option<String>,
    /// Tags for categorization
    pub tags: Vec<String>,
    /// Creation timestamp
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// Last updated timestamp
    pub updated_at: chrono::DateTime<chrono::Utc>,
    /// Source repository URL
    pub source_repo: Option<String>,
    /// Source commit hash
    pub source_commit: Option<String>,
    /// Related documents by ID
    pub related_docs: Vec<String>,
    /// Document dependencies (e.g., required interfaces, base contracts)
    pub dependencies: Vec<String>,
    /// Document status (draft, reviewed, published)
    pub status: DocumentStatus,
    /// Document authors
    pub authors: Vec<String>,
    /// Document reviewers
    pub reviewers: Vec<String>,
    /// Semantic version requirements
    pub version_requirements: Option<String>,
}

/// Document status for tracking lifecycle
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum DocumentStatus {
    Draft,
    InReview,
    Published,
    Archived,
    Deprecated,
}

impl std::fmt::Display for UniswapDocument {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Format document with all relevant context
        writeln!(f, "# {}", self.title)?;
        if let Some(chunk_type) = &self.chunk_type {
            writeln!(f, "Type: {:?}", chunk_type)?;
        }
        writeln!(f, "\n{}", self.content)?;
        
        // Include code examples if present
        if !self.code_examples.is_empty() {
            writeln!(f, "\nCode Examples:")?;
            for example in &self.code_examples {
                writeln!(f, "```solidity\n{}\n```", example)?;
            }
        }
        
        // Include function signatures if present
        if !self.function_signatures.is_empty() {
            writeln!(f, "\nFunction Signatures:")?;
            for sig in &self.function_signatures {
                writeln!(f, "```solidity\n{}\n```", sig)?;
            }
        }
        
        Ok(())
    }
}

impl DocumentMetadata {
    /// Create a new DocumentMetadata with default values
    pub fn new(source_path: Option<String>, version: Option<String>, tags: Vec<String>) -> Self {
        let now = chrono::Utc::now();
        Self {
            source_path,
            version,
            tags,
            created_at: now,
            updated_at: now,
            source_repo: None,
            source_commit: None,
            related_docs: Vec::new(),
            dependencies: Vec::new(),
            status: DocumentStatus::Published,
            authors: Vec::new(),
            reviewers: Vec::new(),
            version_requirements: None,
        }
    }
}

impl UniswapDocument {
    /// Create a new UniswapDocument with default values
    pub fn new(id: String, title: String, doc_type: DocumentType, content: String, metadata: DocumentMetadata) -> Self {
        let mut doc = Self {
            id,
            title,
            doc_type,
            content,
            semantic_chunks: Vec::new(),
            code_examples: Vec::new(),
            function_signatures: Vec::new(),
            metadata,
            parent_id: None,
            chunk_type: None,
        };
        doc.create_semantic_chunks();
        doc
    }

    /// Get content for RAG context
    pub fn get_content(&self) -> String {
        let mut context = String::new();
        
        // Add title and metadata
        context.push_str(&format!("# {}\n", self.title));
        if let Some(v) = &self.metadata.version {
            context.push_str(&format!("Version: {}\n", v));
        }
        
        // Add main content
        context.push_str("\n");
        context.push_str(&self.content);
        
        // Add semantic chunks if present
        if !self.semantic_chunks.is_empty() {
            context.push_str("\n\nAdditional Context:\n");
            for chunk in &self.semantic_chunks {
                context.push_str(&format!("- {}\n", chunk));
            }
        }
        
        // Add code examples if present
        if !self.code_examples.is_empty() {
            context.push_str("\n\nCode Examples:\n");
            for example in &self.code_examples {
                context.push_str("```solidity\n");
                context.push_str(example);
                context.push_str("\n```\n");
            }
        }
        
        // Add function signatures if present
        if !self.function_signatures.is_empty() {
            context.push_str("\n\nFunction Signatures:\n");
            for sig in &self.function_signatures {
                context.push_str("```solidity\n");
                context.push_str(sig);
                context.push_str("\n```\n");
            }
        }
        
        context
    }
    
    /// Create semantic chunks from content
    pub fn create_semantic_chunks(&mut self) {
        let mut chunks = Vec::new();
        let mut examples = Vec::new();
        let mut signatures = Vec::new();
        
        // Split content into sections based on markdown headers
        let sections: Vec<&str> = self.content.split("\n#").collect();
        
        for section in sections {
            // Extract code examples
            if section.contains("```solidity") {
                let code = section
                    .split("```solidity")
                    .nth(1)
                    .and_then(|s| s.split("```").next())
                    .unwrap_or("");
                if !code.is_empty() {
                    examples.push(code.trim().to_string());
                }
            }
            
            // Extract function signatures
            if section.contains("function ") {
                let sig = section
                    .lines()
                    .find(|l| l.contains("function "))
                    .unwrap_or("")
                    .trim();
                if !sig.is_empty() {
                    signatures.push(sig.to_string());
                }
            }
            
            // Create semantic chunks based on content
            let chunk = section.trim();
            if !chunk.is_empty() {
                chunks.push(chunk.to_string());
            }
        }
        
        self.semantic_chunks = chunks;
        self.code_examples = examples;
        self.function_signatures = signatures;
    }
}

/// RAG system for Uniswap documentation and contracts
pub struct UniswapRagSystem {
    /// Vector store index for similarity search
    index: InMemoryVectorStore<UniswapDocument>,
    /// Fastembed client for local embeddings
    embedding_client: FastembedClient,
    /// Document count for monitoring
    document_count: usize,
    /// Document ingestion pipeline
    ingestion_pipeline: Option<crate::doc_ingestion::DocumentIngestionPipeline>,
}

impl UniswapRagSystem {
    /// Create a new RAG system with local embedding model and optional configuration
    pub async fn new() -> crate::Result<Self> {
        info!("🔧 Initializing Uniswap RAG System with local embeddings");
        
        // Initialize Fastembed client for local embeddings
        let embedding_client = FastembedClient::new();
        let _embedding_model = embedding_client.embedding_model(&FastembedModel::AllMiniLML6V2Q);
        
        // Create empty vector store with optimized settings
        let vector_store = InMemoryVectorStore::<UniswapDocument>::from_documents(vec![]);
        
        // Initialize document ingestion pipeline
        let doc_source = crate::doc_ingestion::sources::uniswap::UniswapDocSource::new(
            std::path::PathBuf::from("../docs/uniswap")
        );
        
        let processor = crate::doc_ingestion::DefaultDocumentProcessor;
        let doc_store = InMemoryDocStore::default();
        
        let pipeline = crate::doc_ingestion::DocumentIngestionPipeline::new(
            vec![Box::new(doc_source)],
            Box::new(processor),
            Box::new(doc_store)
        );
        
        info!("✅ RAG System initialized with local embedding model and document pipeline");
        
        // Create the RAG system
        let mut rag = Self {
            index: vector_store,
            embedding_client,
            document_count: 0,
            ingestion_pipeline: Some(pipeline),
        };

        // Load documents immediately
        rag.load_documentation(&std::path::Path::new("")).await?;
        
        Ok(rag)
    }
    
    /// Load and index Uniswap documentation using the ingestion pipeline
    pub async fn load_documentation(&mut self, _docs_path: &Path) -> crate::Result<()> {
        info!("📚 Loading Uniswap documentation using ingestion pipeline");
        
        if let Some(pipeline) = &self.ingestion_pipeline {
            // Run the ingestion pipeline
            let stats = pipeline.run().await
                .map_err(|e| crate::ClientError::RagError(format!("Document ingestion failed: {}", e)))?;
            
            info!("📊 Document ingestion stats:");
            info!("   Total documents: {}", stats.total_documents);
            info!("   Successfully processed: {}", stats.successful_documents);
            info!("   Failed: {}", stats.failed_documents);
            
            if !stats.errors.is_empty() {
                warn!("⚠️ Ingestion errors:");
                for error in &stats.errors {
                    warn!("   - {}", error);
                }
            }
            
            // Convert processed documents to UniswapDocuments and index them
            let mut documents = Vec::new();
            
            // Run the pipeline to process documents
            let stats = pipeline.run().await
                .map_err(|e| crate::ClientError::RagError(format!("Failed to process documents: {}", e)))?;
            
            info!("📊 Document ingestion stats:");
            info!("   Total documents: {}", stats.total_documents);
            info!("   Successfully processed: {}", stats.successful_documents);
            info!("   Failed: {}", stats.failed_documents);
            
            if !stats.errors.is_empty() {
                warn!("⚠️ Ingestion errors:");
                for error in &stats.errors {
                    warn!("   - {}", error);
                }
            }
            
            if stats.successful_documents == 0 {
                warn!("⚠️ No documents were successfully processed");
                return Ok(());
            }
            
            // Get all documents from the store
            let store = pipeline.get_store();
            let doc_list = store.list_documents().await
                .map_err(|e| crate::ClientError::RagError(format!("Failed to list documents: {}", e)))?;
            
            for metadata in doc_list {
                // Use title as a simple key since we don't have checksum in metadata
                if let Ok(Some(doc)) = store.get_document(&metadata.title).await {
                    let uniswap_doc = UniswapDocument {
                        id: doc.checksum.clone(),
                        title: doc.metadata.title,
                        doc_type: match doc.metadata.doc_type {
                            crate::doc_ingestion::DocumentType::Solidity => DocumentType::ContractCode,
                            crate::doc_ingestion::DocumentType::Markdown => DocumentType::Documentation,
                            crate::doc_ingestion::DocumentType::JSON => DocumentType::Interface,
                            _ => DocumentType::Documentation,
                        },
                        content: doc.content,
                        semantic_chunks: doc.chunks,
                        code_examples: Vec::new(), // Will be populated by create_semantic_chunks
                        function_signatures: Vec::new(), // Will be populated by create_semantic_chunks
                        metadata: DocumentMetadata::new(
                            Some(doc.metadata.source.location),
                            doc.metadata.version.clone(),
                            doc.metadata.tags,
                        ),
                        parent_id: None,
                        chunk_type: None,
                    };
                    documents.push(uniswap_doc);
                }
            }
            
            if documents.is_empty() {
                warn!("⚠️ No documents were successfully processed");
                return Ok(());
            }
            
            // Index the documents
            self.index_documents(documents).await?;
            
            info!("✅ Successfully loaded and indexed {} documents", self.document_count);
        } else {
            warn!("⚠️ No ingestion pipeline available");
        }
        
        Ok(())
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
    
    /// Get all documents for agentic RAG integration
    pub async fn get_all_documents(&self) -> crate::Result<Vec<UniswapDocument>> {
        // Return all documents from the vector store
        let mut docs = Vec::new();
        let embedding_model = self.embedding_client.embedding_model(&FastembedModel::AllMiniLML6V2Q);
        let index = self.index.clone().index(embedding_model);
        
        // Get all documents from the vector store
        let req = rig::vector_store::request::VectorSearchRequest::builder()
            .query("") // Empty query to get all documents
            .samples(self.document_count as u64)
            .build()
            .map_err(|e| crate::ClientError::RagError(format!("Failed to build search request: {}", e)))?;
        
        let results = index
            .top_n::<UniswapDocument>(req)
            .await
            .map_err(|e| crate::ClientError::RagError(format!("Failed to get documents: {}", e)))?;
        
        for (_, _, doc) in results {
            docs.push(doc);
        }
        
        Ok(docs)
    }

    /// Search against a set of example queries to determine query type
    pub async fn search_examples(&self, query: &str, examples: &[&str]) -> crate::Result<Vec<(f64, String, String)>> {
        debug!("🔍 Comparing query '{}' against {} examples", query, examples.len());
        
        // Create a temporary vector store for the examples
        #[derive(rig::Embed, Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
        struct QueryDoc {
            #[embed]
            text: String,
        }
        
        // Create embeddings for examples
        let embedding_model = self.embedding_client.embedding_model(&FastembedModel::AllMiniLML6V2Q);
        let mut builder = EmbeddingsBuilder::new(embedding_model.clone());
        
        // Add examples
        let example_docs: Vec<QueryDoc> = examples.iter()
            .map(|&text| QueryDoc { text: text.to_string() })
            .collect();
            
        for doc in example_docs.iter() {
            builder = builder.document(doc.clone())
                .map_err(|e| crate::ClientError::RagError(format!("Failed to add example: {}", e)))?;
        }
        
        // Build embeddings and create vector store
        let embeddings = builder.build()
            .await
            .map_err(|e| crate::ClientError::RagError(format!("Failed to build embeddings: {}", e)))?;
            
        let vector_store = InMemoryVectorStore::from_documents_with_id_f(embeddings, |doc| doc.text.clone());
        let index = vector_store.index(embedding_model);
        
        // Search for similar examples
        let req = rig::vector_store::request::VectorSearchRequest::builder()
            .query(query)
            .samples(examples.len() as u64)
            .build()
            .map_err(|e| crate::ClientError::RagError(format!("Failed to build search request: {}", e)))?;
            
        let results = index
            .top_n::<QueryDoc>(req)
            .await
            .map_err(|e| crate::ClientError::RagError(format!("Search failed: {}", e)))?;
            
        debug!("📋 Found {} matching examples", results.len());
            
        // Convert results to expected format
        let formatted_results = results.into_iter()
            .map(|(score, _, doc)| (score, format!("example_{}", examples.iter().position(|&e| e == doc.text).unwrap_or(0)), doc.text))
            .collect();
            
        Ok(formatted_results)
    }

}