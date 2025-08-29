//! Error types for the RIG client

use thiserror::Error;

/// Errors that can occur in the RIG client
#[derive(Error, Debug)]
pub enum ClientError {
    #[error("MCP server connection failed: {0}")]
    McpConnection(String),

    #[error("Claude API error: {0}")]
    ClaudeApi(String),

    #[error("Missing environment variable: {0}")]
    MissingEnvVar(String),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("CLI error: {0}")]
    Cli(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON serialization error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Anyhow error: {0}")]
    Anyhow(#[from] anyhow::Error),

    #[error("RAG system error: {0}")]
    RagError(String),

    #[error("Embedding error: {0}")]
    EmbeddingError(#[from] rig::embeddings::EmbedError),

    #[error("Vector store error: {0}")]
    VectorStoreError(#[from] rig::vector_store::VectorStoreError),
}

impl From<rmcp::ErrorData> for ClientError {
    fn from(err: rmcp::ErrorData) -> Self {
        ClientError::McpConnection(err.to_string())
    }
}
