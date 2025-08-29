//! Configuration management for the RIG client

use clap::Parser;
use std::env;

/// CLI arguments and configuration
#[derive(Parser, Debug)]
#[command(name = "rig-client")]
#[command(about = "AI Agent for Ethereum blockchain interaction via natural language")]
#[command(version)]
pub struct Config {
    /// Enable verbose logging
    #[arg(short, long)]
    pub verbose: bool,
    
    /// MCP server URL (default: local)
    #[arg(long, default_value = "http://127.0.0.1:8080/mcp")]
    pub mcp_server: String,
}

impl Config {
    /// Create a new configuration from CLI arguments
    pub fn new() -> Self {
        Self::parse()
    }

    /// Get the log level based on verbose flag
    pub fn log_level(&self) -> tracing::Level {
        if self.verbose {
            tracing::Level::DEBUG
        } else {
            tracing::Level::INFO
        }
    }

    /// Get the Claude API key from environment
    pub fn anthropic_api_key(&self) -> crate::Result<String> {
        env::var("ANTHROPIC_API_KEY")
            .map_err(|_| crate::ClientError::MissingEnvVar("ANTHROPIC_API_KEY".to_string()))
    }
}
