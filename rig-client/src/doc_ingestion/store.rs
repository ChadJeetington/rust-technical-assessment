use std::collections::HashMap;
use std::sync::RwLock;
use async_trait::async_trait;
use super::*;

/// Simple in-memory document store implementation
#[derive(Default)]
pub struct InMemoryDocStore {
    documents: RwLock<HashMap<String, ProcessedDocument>>,
}

#[async_trait]
impl DocumentStore for InMemoryDocStore {
    async fn store_document(&self, doc: ProcessedDocument) -> Result<(), IngestionError> {
        let mut docs = self.documents.write()
            .map_err(|_| IngestionError::StorageError("Failed to acquire write lock".to_string()))?;
        docs.insert(doc.metadata.title.clone(), doc);
        Ok(())
    }
    
    async fn get_document(&self, title: &str) -> Result<Option<ProcessedDocument>, IngestionError> {
        let docs = self.documents.read()
            .map_err(|_| IngestionError::StorageError("Failed to acquire read lock".to_string()))?;
        Ok(docs.get(title).cloned())
    }
    
    async fn list_documents(&self) -> Result<Vec<DocumentMetadata>, IngestionError> {
        let docs = self.documents.read()
            .map_err(|_| IngestionError::StorageError("Failed to acquire read lock".to_string()))?;
        Ok(docs.values().map(|doc| doc.metadata.clone()).collect())
    }
    
    async fn delete_document(&self, title: &str) -> Result<(), IngestionError> {
        let mut docs = self.documents.write()
            .map_err(|_| IngestionError::StorageError("Failed to acquire write lock".to_string()))?;
        docs.remove(title);
        Ok(())
    }
}