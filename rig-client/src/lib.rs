//! RIG AI Agent Client Library for Ethereum Blockchain Interaction
//! 
//! This library provides the core functionality for an AI agent that can interact
//! with the Ethereum blockchain using natural language commands.

pub mod agent;
pub mod cli;
pub mod config;
pub mod error;
pub mod rag;

pub use agent::BlockchainAgent;
pub use cli::Repl;
pub use config::Config;
pub use error::ClientError;

/// Re-export common types for convenience
pub type Result<T> = std::result::Result<T, ClientError>;
