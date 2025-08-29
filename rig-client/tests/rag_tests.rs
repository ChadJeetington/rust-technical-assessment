//! Rust tests for RAG system functionality
//! Tests the bonus section part 2 of the PRD

use rig_client::rag::{UniswapRagSystem, UniswapDocument, DocumentType, DocumentMetadata};

#[tokio::test]
async fn test_rag_system_initialization() {
    // Test that the RAG system can be initialized
    let rag_system = UniswapRagSystem::new().await;
    assert!(rag_system.is_ok(), "RAG system should initialize successfully");
}

#[tokio::test]
async fn test_sample_documentation_loading() {
    // Test that sample documentation can be loaded
    let mut rag_system = UniswapRagSystem::new().await.unwrap();
    let result = rag_system.add_sample_documentation().await;
    assert!(result.is_ok(), "Sample documentation should load successfully");
    
    // Check that documents were added
    assert!(rag_system.document_count() > 0, "Should have loaded sample documents");
}

#[tokio::test]
async fn test_document_search() {
    // Test that document search works
    let mut rag_system = UniswapRagSystem::new().await.unwrap();
    rag_system.add_sample_documentation().await.unwrap();
    
    // Test search for slippage information
    let results = rag_system.search("slippage calculation", 3).await;
    assert!(results.is_ok(), "Search should work");
    
    let results = results.unwrap();
    assert!(!results.is_empty(), "Should find relevant documents");
}

#[tokio::test]
async fn test_document_structure() {
    // Test that document structure works correctly
    let doc = UniswapDocument {
        id: "test_doc".to_string(),
        title: "Test Document".to_string(),
        doc_type: DocumentType::Documentation,
        content: "This is a test document about Uniswap V3 slippage calculation.".to_string(),
        metadata: DocumentMetadata {
            source_path: None,
            version: Some("V3".to_string()),
            tags: vec!["uniswap".to_string(), "v3".to_string(), "slippage".to_string()],
            created_at: chrono::Utc::now(),
        },
    };
    
    assert_eq!(doc.id, "test_doc");
    assert_eq!(doc.title, "Test Document");
    assert_eq!(doc.doc_type, DocumentType::Documentation);
    assert!(doc.content.contains("slippage"));
    assert!(doc.metadata.tags.contains(&"uniswap".to_string()));
}

#[tokio::test]
async fn test_rag_status() {
    // Test that RAG status works
    let mut rag_system = UniswapRagSystem::new().await.unwrap();
    rag_system.add_sample_documentation().await.unwrap();
    
    let status = rag_system.document_count();
    assert!(status > 0, "Should have documents loaded");
}

#[tokio::test]
async fn test_specific_queries() {
    // Test the specific queries mentioned in the PRD
    let mut rag_system = UniswapRagSystem::new().await.unwrap();
    rag_system.add_sample_documentation().await.unwrap();
    
    // Test query 1: "How do I calculate slippage for Uniswap V3?"
    let results1 = rag_system.search("How do I calculate slippage for Uniswap V3?", 3).await;
    assert!(results1.is_ok(), "Slippage query should work");
    assert!(!results1.unwrap().is_empty(), "Should find slippage documents");
    
    // Test query 2: "What's the difference between exactInput and exactOutput?"
    let results2 = rag_system.search("What's the difference between exactInput and exactOutput?", 3).await;
    assert!(results2.is_ok(), "Function difference query should work");
    assert!(!results2.unwrap().is_empty(), "Should find function documentation");
    
    // Test query 3: "Show me the SwapRouter contract interface"
    let results3 = rag_system.search("Show me the SwapRouter contract interface", 3).await;
    assert!(results3.is_ok(), "Contract interface query should work");
    assert!(!results3.unwrap().is_empty(), "Should find contract code");
}
