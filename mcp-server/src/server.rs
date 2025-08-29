//! MCP Server HTTP Server Module
//! 
//! This module handles the HTTP server setup, MCP service configuration,
//! and server lifecycle management.

use anyhow::Result;
use rmcp::transport::streamable_http_server::{
    StreamableHttpService, session::local::LocalSessionManager,
};
use tracing::info;

use crate::combined_service::CombinedService;

/// Server configuration
#[derive(Debug, Clone)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub mcp_path: String,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            port: 8080,
            mcp_path: "/mcp".to_string(),
        }
    }
}

/// MCP Server instance
pub struct McpServer {
    config: ServerConfig,
    combined_service: CombinedService,
}

impl McpServer {
    /// Create a new MCP server instance
    pub async fn new(config: ServerConfig) -> Result<Self> {
        info!("🔧 Creating MCP server with config: {:?}", config);
        
        // Create combined service
        let combined_service = CombinedService::new().await
            .map_err(|e| anyhow::anyhow!("Failed to create combined service: {}", e))?;
        
        Ok(Self {
            config,
            combined_service,
        })
    }

    /// Start the HTTP server
    pub async fn start(self) -> Result<()> {
        let config = self.config.clone();
        let combined_service = self.combined_service;
        
        info!("🚀 Starting MCP Combined Server");
        info!("🌐 HTTP Server listening on http://{}:{}", config.host, config.port);
        info!("📡 Connecting to anvil network at 127.0.0.1:8545");
        info!("🔍 Brave Search API integration enabled");

        // Create StreamableHttpService with sync constructor
        let service = StreamableHttpService::new(
            move || Ok(combined_service.clone()),
            LocalSessionManager::default().into(),
            Default::default(),
        );

        // Create axum router with MCP service
        let router = axum::Router::new().nest_service(&config.mcp_path, service);
        let tcp_listener = tokio::net::TcpListener::bind(format!("{}:{}", config.host, config.port)).await?;
        
        info!("✅ MCP Combined Server ready on port {} - exposing blockchain and search tools", config.port);
        info!("🔗 RIG clients can connect to: http://{}:{}{}", config.host, config.port, config.mcp_path);

        // Start the axum server with graceful shutdown
        axum::serve(tcp_listener, router)
            .with_graceful_shutdown(async {
                tokio::signal::ctrl_c().await.unwrap();
                info!("🛑 MCP server shutting down...");
            })
            .await?;
        
        Ok(())
    }

    /// Start the server with custom shutdown signal
    pub async fn start_with_shutdown<F>(self, shutdown_signal: F) -> Result<()>
    where
        F: std::future::Future<Output = ()> + Send + 'static,
    {
        let config = self.config.clone();
        let combined_service = self.combined_service;
        
        info!("🚀 Starting MCP Combined Server");
        info!("🌐 HTTP Server listening on http://{}:{}", config.host, config.port);
        info!("📡 Connecting to anvil network at 127.0.0.1:8545");
        info!("🔍 Brave Search API integration enabled");

        // Create StreamableHttpService with sync constructor
        let service = StreamableHttpService::new(
            move || Ok(combined_service.clone()),
            LocalSessionManager::default().into(),
            Default::default(),
        );

        // Create axum router with MCP service
        let router = axum::Router::new().nest_service(&config.mcp_path, service);
        let tcp_listener = tokio::net::TcpListener::bind(format!("{}:{}", config.host, config.port)).await?;
        
        info!("✅ MCP Blockchain Server ready on port {} - exposing balance, transfer, and is_contract_deployed tools", config.port);
        info!("🔗 RIG clients can connect to: http://{}:{}{}", config.host, config.port, config.mcp_path);

        // Start the axum server with custom shutdown
        axum::serve(tcp_listener, router)
            .with_graceful_shutdown(async {
                shutdown_signal.await;
                info!("🛑 MCP server shutting down...");
            })
            .await?;
        
        Ok(())
    }
}

/// Initialize logging for the server
pub fn init_logging() {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env().add_directive(tracing::Level::INFO.into()))
        .with_writer(std::io::stderr)
        .with_ansi(false)
        .init();
}
