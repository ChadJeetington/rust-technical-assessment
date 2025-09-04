use crate::doc_ingestion::{
    DocumentSource, DocumentMetadata, DocumentSourceMetadata,
    DocumentType, RawDocument, IngestionError,
};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tokio::fs;
use tracing::{info, warn};
use walkdir::WalkDir;

/// Uniswap documentation source that reads from pre-processed files
pub struct UniswapDocSource {
    /// Documentation version mapping
    versions: Vec<UniswapVersion>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct UniswapVersion {
    version: String,
    contracts_path: PathBuf,
    docs_path: PathBuf,
    metadata: DocumentSourceMetadata,
}

impl UniswapDocSource {
    pub fn new(base_dir: PathBuf) -> Self {
        // Define known Uniswap versions and their documentation paths
        let versions = vec![
            UniswapVersion {
                version: "v2".to_string(),
                contracts_path: base_dir.join("v2/contracts"),
                docs_path: base_dir.join("v2/docs"),
                metadata: DocumentSourceMetadata {
                    source_type: "uniswap".to_string(),
                    location: "https://docs.uniswap.org/contracts/v2/overview".to_string(),
                    version: Some("v2".to_string()),
                },
            },
            UniswapVersion {
                version: "v3".to_string(),
                contracts_path: base_dir.join("v3/contracts"),
                docs_path: base_dir.join("v3/docs"),
                metadata: DocumentSourceMetadata {
                    source_type: "uniswap".to_string(),
                    location: "https://docs.uniswap.org/contracts/v3/overview".to_string(),
                    version: Some("v3".to_string()),
                },
            },
        ];

        Self { versions }
    }
}

#[async_trait]
impl DocumentSource for UniswapDocSource {
    async fn fetch_documents(&self) -> Result<Vec<RawDocument>, IngestionError> {
        let mut documents = Vec::new();
        
        info!("🔍 Loading Uniswap documentation from processed files...");
        
        for version in &self.versions {
            info!("📚 Processing Uniswap {} documentation...", version.version);
            
            info!("   Checking contracts path: {}", version.contracts_path.display());
            if version.contracts_path.exists() {
                info!("   Found contracts directory");
                
                for entry in WalkDir::new(&version.contracts_path)
                    .into_iter()
                    .filter_map(Result::ok)
                    .filter(|e| e.file_type().is_file())
                {
                    let path = entry.path();
                    if let Some(ext) = path.extension() {
                        if ext == "sol" {
                            info!("   Processing contract: {}", path.display());
                            match fs::read(path).await {
                                Ok(content) => {
                                    let doc = RawDocument::new(
                                        content,
                                        DocumentMetadata {
                                            title: path.file_name()
                                                .and_then(|n| n.to_str())
                                                .unwrap_or("Unknown")
                                                .to_string(),
                                            doc_type: DocumentType::Solidity,
                                            version: Some(version.version.clone()),
                                            created_at: chrono::Utc::now(),
                                            updated_at: chrono::Utc::now(),
                                            source: version.metadata.clone(),
                                            tags: vec![
                                                version.version.clone(),
                                                "contract".to_string(),
                                                path.strip_prefix(&version.contracts_path)
                                                    .unwrap_or(path)
                                                    .to_string_lossy()
                                                    .to_string(),
                                            ],
                                        },
                                    );
                                    documents.push(doc);
                                }
                                Err(e) => warn!("   ⚠️ Failed to read file: {}", e),
                            }
                        }
                    }
                }
            } else {
                warn!("   ⚠️ Contracts directory not found");
            }
            
            info!("   Checking docs path: {}", version.docs_path.display());
            if version.docs_path.exists() {
                info!("   Found docs directory");
                
                for entry in WalkDir::new(&version.docs_path)
                    .into_iter()
                    .filter_map(Result::ok)
                    .filter(|e| e.file_type().is_file())
                {
                    let path = entry.path();
                    if let Some(ext) = path.extension() {
                        if matches!(ext.to_str(), Some("md") | Some("mdx")) {
                            info!("   Processing doc: {}", path.display());
                            match fs::read(path).await {
                                Ok(content) => {
                                    let doc = RawDocument::new(
                                        content,
                                        DocumentMetadata {
                                            title: path.file_name()
                                                .and_then(|n| n.to_str())
                                                .unwrap_or("Unknown")
                                                .to_string(),
                                            doc_type: DocumentType::Markdown,
                                            version: Some(version.version.clone()),
                                            created_at: chrono::Utc::now(),
                                            updated_at: chrono::Utc::now(),
                                            source: version.metadata.clone(),
                                            tags: vec![
                                                version.version.clone(),
                                                "documentation".to_string(),
                                                path.strip_prefix(&version.docs_path)
                                                    .unwrap_or(path)
                                                    .to_string_lossy()
                                                    .to_string(),
                                            ],
                                        },
                                    );
                                    documents.push(doc);
                                }
                                Err(e) => warn!("   ⚠️ Failed to read file: {}", e),
                            }
                        }
                    }
                }
            } else {
                warn!("   ⚠️ Docs directory not found");
            }
        }
        
        info!("✅ Loaded {} documents", documents.len());
        Ok(documents)
    }
    
    async fn has_updates(&self) -> Result<bool, IngestionError> {
        // In a real implementation, this would check for updates in the processed documentation
        // For now, always return true to process documents
        Ok(true)
    }
    
    fn get_metadata(&self) -> DocumentSourceMetadata {
        DocumentSourceMetadata {
            source_type: "uniswap".to_string(),
            location: "https://docs.uniswap.org".to_string(),
            version: Some("latest".to_string()),
        }
    }
}