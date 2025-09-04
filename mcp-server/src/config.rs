use std::env;
use tracing::info;

/// Configuration for blockchain service
#[derive(Clone, Debug)]
pub struct BlockchainConfig {
    /// Default slippage in basis points (e.g., 500 = 5%)
    pub default_slippage_bps: String,
    /// Default transaction deadline in seconds
    pub default_deadline_secs: u64,
    /// RPC URL for blockchain connection
    pub rpc_url: String,
    /// Alice's private key for transactions
    pub alice_private_key: String,
}

impl BlockchainConfig {
    pub fn from_env() -> Self {
        // Load environment variables from .env file if it exists
        if dotenv::dotenv().is_err() {
            info!("No .env file found, using system environment variables only");
        } else {
            info!("Loaded environment variables from .env file");
        }

        // Load configurable business logic values from environment
        let default_slippage_bps = env::var("DEFAULT_SLIPPAGE_BPS")
            .unwrap_or_else(|_| {
                info!("⚠️  No DEFAULT_SLIPPAGE_BPS found in environment, using default: 500 (5%)");
                "500".to_string()
            });

        let default_deadline_secs = env::var("DEFAULT_DEADLINE_SECS")
            .unwrap_or_else(|_| {
                info!("⚠️  No DEFAULT_DEADLINE_SECS found in environment, using default: 300 (5 minutes)");
                "300".to_string()
            })
            .parse::<u64>()
            .unwrap_or(300);

        let rpc_url = env::var("RPC_URL")
            .unwrap_or_else(|_| {
                info!("⚠️  No RPC_URL found in environment, using default: http://127.0.0.1:8545");
                "http://127.0.0.1:8545".to_string()
            });

        let alice_private_key = env::var("ALICE_PRIVATE_KEY")
            .or_else(|_| env::var("PRIVATE_KEY"))
            .unwrap_or_else(|_| {
                info!("⚠️  No ALICE_PRIVATE_KEY or PRIVATE_KEY found in environment");
                info!("    Transactions will not be possible without a private key");
                String::new()
            });

        info!("⚙️  Configuration loaded:");
        info!("    • Default slippage: {}bps ({}%)", 
            default_slippage_bps, 
            default_slippage_bps.parse::<f64>().unwrap_or(500.0) / 100.0
        );
        info!("    • Default deadline: {}s ({}min)", 
            default_deadline_secs, 
            default_deadline_secs / 60
        );
        info!("    • RPC URL: {}", rpc_url);
        info!("    • Private key: {}", 
            if alice_private_key.is_empty() { "Not set" } else { "Set" }
        );

        Self {
            default_slippage_bps,
            default_deadline_secs,
            rpc_url,
            alice_private_key,
        }
    }
}
