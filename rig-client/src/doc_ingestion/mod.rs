pub mod sources;
pub mod store;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Sha256, Digest};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum IngestionError {
    #[error("Failed to fetch document: {0}")]
    FetchError(String),
    #[error("Failed to process document: {0}")]
    ProcessingError(String),
    #[error("Document validation failed: {0}")]
    ValidationError(String),
    #[error("Storage error: {0}")]
    StorageError(String),
}

/// Represents a source of documentation (Git repo, local filesystem, S3, etc.)
#[async_trait]
pub trait DocumentSource: Send + Sync {
    /// Fetch documents from the source
    async fn fetch_documents(&self) -> Result<Vec<RawDocument>, IngestionError>;
    
    /// Check if source has updates
    async fn has_updates(&self) -> Result<bool, IngestionError>;
    
    /// Get source metadata
    fn get_metadata(&self) -> DocumentSourceMetadata;
}

/// Raw document before processing
#[derive(Debug, Clone)]
pub struct RawDocument {
    pub content: Vec<u8>,
    pub metadata: DocumentMetadata,
    pub checksum: String,
}

impl RawDocument {
    pub fn new(content: Vec<u8>, metadata: DocumentMetadata) -> Self {
        let mut hasher = Sha256::new();
        hasher.update(&content);
        let checksum = format!("{:x}", hasher.finalize());
        
        Self {
            content,
            metadata,
            checksum,
        }
    }
    
    pub fn validate(&self) -> Result<(), IngestionError> {
        // Validate document integrity and format
        if self.content.is_empty() {
            return Err(IngestionError::ValidationError("Empty document".to_string()));
        }
        
        // Additional validation based on document type
        match self.metadata.doc_type {
            DocumentType::Solidity => self.validate_solidity(),
            DocumentType::Markdown => self.validate_markdown(),
            DocumentType::JSON => self.validate_json(),
            DocumentType::Other(_) => Ok(()) // No specific validation for other types
        }
    }
    
    fn validate_solidity(&self) -> Result<(), IngestionError> {
        let content = String::from_utf8_lossy(&self.content);
        
        // Basic Solidity validation
        if !content.contains("pragma solidity") && !content.contains("contract ") && !content.contains("interface ") {
            return Err(IngestionError::ValidationError("Not a valid Solidity file".to_string()));
        }

        // Check for basic structure
        let has_valid_structure = content.contains('{') && content.contains('}');
        if !has_valid_structure {
            return Err(IngestionError::ValidationError("Invalid Solidity structure".to_string()));
        }

        Ok(())
    }
    
    fn validate_markdown(&self) -> Result<(), IngestionError> {
        let content = String::from_utf8_lossy(&self.content);
        
        // Check for common markdown elements
        let has_structure = content.contains('#') || // Headers
            content.contains('-') || // Lists
            content.contains("```") || // Code blocks
            content.contains('*') || // Emphasis
            content.contains('[') || // Links
            content.contains('|'); // Tables
            
        if !has_structure {
            return Err(IngestionError::ValidationError("Invalid markdown structure".to_string()));
        }
        
        Ok(())
    }

    fn validate_json(&self) -> Result<(), IngestionError> {
        let content = String::from_utf8_lossy(&self.content);
        
        // Try to parse as JSON
        match serde_json::from_str::<serde_json::Value>(&content) {
            Ok(_) => Ok(()),
            Err(e) => {
                // Only fail for clearly invalid JSON
                if content.trim().is_empty() || 
                   (!content.contains('{') && !content.contains('[')) {
                    Err(IngestionError::ValidationError(format!("Invalid JSON: {}", e)))
                } else {
                    // Be lenient for partial JSON files
                    Ok(())
                }
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentMetadata {
    pub title: String,
    pub doc_type: DocumentType,
    pub version: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub source: DocumentSourceMetadata,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentSourceMetadata {
    pub source_type: String,
    pub location: String,
    pub version: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DocumentType {
    Solidity,
    Markdown,
    JSON,
    Other(String),
}

impl std::fmt::Display for DocumentType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DocumentType::Solidity => write!(f, "Solidity"),
            DocumentType::Markdown => write!(f, "Markdown"),
            DocumentType::JSON => write!(f, "JSON"),
            DocumentType::Other(s) => write!(f, "Other({})", s),
        }
    }
}

/// Processes raw documents into indexed documents
#[async_trait]
pub trait DocumentProcessor: Send + Sync {
    async fn process(&self, doc: RawDocument) -> Result<ProcessedDocument, IngestionError>;
}

/// Default document processor implementation
pub struct DefaultDocumentProcessor;

#[async_trait]
impl DocumentProcessor for DefaultDocumentProcessor {
    async fn process(&self, doc: RawDocument) -> Result<ProcessedDocument, IngestionError> {
        // Validate document
        doc.validate()?;
        
        // Convert content to string
        let content = String::from_utf8(doc.content)
            .map_err(|e| IngestionError::ProcessingError(format!("Invalid UTF-8: {}", e)))?;
        
        // Create semantic chunks based on document type
        let chunks = match doc.metadata.doc_type {
            DocumentType::Solidity => self.chunk_solidity(&content),
            DocumentType::Markdown => self.chunk_markdown(&content),
            _ => vec![content.clone()],
        };
        
        Ok(ProcessedDocument {
            content,
            chunks,
            metadata: doc.metadata,
            checksum: doc.checksum,
        })
    }
}

impl DefaultDocumentProcessor {
    fn chunk_solidity(&self, content: &str) -> Vec<String> {
        let mut chunks = Vec::new();
        
        // Split by contract definitions
        for line in content.lines() {
            if line.contains("contract ") || line.contains("interface ") || line.contains("library ") {
                chunks.push(line.to_string());
            }
            // Extract function definitions
            if line.contains("function ") {
                chunks.push(line.to_string());
            }
        }
        
        if chunks.is_empty() {
            chunks.push(content.to_string());
        }
        
        chunks
    }
    
    fn chunk_markdown(&self, content: &str) -> Vec<String> {
        let mut chunks = Vec::new();
        let mut current_chunk = String::new();
        
        for line in content.lines() {
            // Start new chunk on headers
            if line.starts_with('#') {
                if !current_chunk.is_empty() {
                    chunks.push(current_chunk.trim().to_string());
                }
                current_chunk = line.to_string();
            } else {
                current_chunk.push('\n');
                current_chunk.push_str(line);
            }
        }
        
        // Add final chunk
        if !current_chunk.is_empty() {
            chunks.push(current_chunk.trim().to_string());
        }
        
        if chunks.is_empty() {
            chunks.push(content.to_string());
        }
        
        chunks
    }
}

/// Processed document ready for indexing
#[derive(Debug, Clone)]
pub struct ProcessedDocument {
    pub content: String,
    pub chunks: Vec<String>,
    pub metadata: DocumentMetadata,
    pub checksum: String,
}

/// Document ingestion orchestrator
pub struct DocumentIngestionPipeline {
    sources: Vec<Box<dyn DocumentSource>>,
    processor: Box<dyn DocumentProcessor>,
    store: Box<dyn DocumentStore>,
}

impl DocumentIngestionPipeline {
    pub fn new(
        sources: Vec<Box<dyn DocumentSource>>,
        processor: Box<dyn DocumentProcessor>,
        store: Box<dyn DocumentStore>,
    ) -> Self {
        Self {
            sources,
            processor,
            store,
        }
    }
    
    /// Get a reference to the document store
    pub fn get_store(&self) -> &Box<dyn DocumentStore> {
        &self.store
    }

    pub async fn run(&self) -> Result<IngestionStats, IngestionError> {
        let mut stats = IngestionStats::default();
        
        for source in &self.sources {
            // Check for updates
            if !source.has_updates().await? {
                continue;
            }
            
            // Fetch documents
            let raw_docs = source.fetch_documents().await?;
            stats.total_documents += raw_docs.len();
            
            // Process and store documents
            for doc in raw_docs {
                match self.process_and_store(doc).await {
                    Ok(_) => stats.successful_documents += 1,
                    Err(e) => {
                        stats.failed_documents += 1;
                        stats.errors.push(e.to_string());
                    }
                }
            }
        }
        
        Ok(stats)
    }
    
    async fn process_and_store(&self, doc: RawDocument) -> Result<(), IngestionError> {
        // Process document
        let processed = self.processor.process(doc).await?;
        
        // Store document
        self.store.store_document(processed).await?;
        
        Ok(())
    }
}

#[derive(Debug, Default)]
pub struct IngestionStats {
    pub total_documents: usize,
    pub successful_documents: usize,
    pub failed_documents: usize,
    pub errors: Vec<String>,
}

/// Persistent storage for documents and embeddings
#[async_trait]
pub trait DocumentStore: Send + Sync {
    async fn store_document(&self, doc: ProcessedDocument) -> Result<(), IngestionError>;
    async fn get_document(&self, checksum: &str) -> Result<Option<ProcessedDocument>, IngestionError>;
    async fn list_documents(&self) -> Result<Vec<DocumentMetadata>, IngestionError>;
    async fn delete_document(&self, checksum: &str) -> Result<(), IngestionError>;
}