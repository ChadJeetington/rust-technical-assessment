//! Blockchain MCP Server Implementation
//! 
//! This module implements the core blockchain functionality as MCP tools.
//! Following the PRD example exactly - using Cast struct directly from foundry.
//! 
//! Tools exposed:
//! - balance: Get ETH balance of an address (exact PRD example implementation)
//! - transfer: Send ETH between addresses using Cast::send
//! - is_contract_deployed: Check if contract code exists using Cast::code

use alloy_ens::NameOrAddress;
use alloy_network::AnyNetwork;
use alloy_primitives::{Address, U256, Bytes};
use alloy_provider::{Provider, ProviderBuilder, RootProvider};
use alloy_rpc_types::TransactionRequest;
use alloy_serde::WithOtherFields;
use cast::Cast;
use eyre::Result;
use num_traits::cast::ToPrimitive;
use rmcp::{
    ErrorData as McpError, ServerHandler,
    handler::server::{router::tool::ToolRouter, wrapper::Parameters},
    model::{CallToolResult, Content, ServerCapabilities, ServerInfo},
    schemars::JsonSchema,
    tool, tool_handler, tool_router,
};
use serde::{Deserialize, Serialize};
use std::{env, str::FromStr};
use tracing::{info, error};

/// Request structure for balance queries
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct BalanceRequest {
    #[schemars(description = "The address or ENS name to check balance for")]
    pub who: String,
}

/// Request structure for ETH transfers
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct TransferRequest {
    #[schemars(description = "Recipient address")]
    pub to: String,
    #[schemars(description = "Amount in ETH (e.g., '1.0')")]
    pub amount: String,
}

/// Request structure for contract deployment checks
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct ContractDeploymentRequest {
    #[schemars(description = "Contract address to check")]
    pub address: String,
}

/// Request structure for ERC-20 token balance queries
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct TokenBalanceRequest {
    #[schemars(description = "Token contract address (e.g., USDC: 0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48)")]
    pub token_address: String,
    #[schemars(description = "Account address to check balance for")]
    pub account_address: String,
}

/// Response structure for account information
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AccountInfo {
    #[schemars(description = "Account index (0-9)")]
    pub index: u32,
    #[schemars(description = "Public address")]
    pub address: String,
    #[schemars(description = "Private key (only included in get_private_keys)")]
    pub private_key: Option<String>,
}

/// Response structure for account listings
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct AccountListResponse {
    #[schemars(description = "List of available accounts")]
    pub accounts: Vec<AccountInfo>,
    #[schemars(description = "Total number of accounts")]
    pub total: u32,
}

/// Validated address information
#[derive(Debug, Clone)]
pub struct ValidatedAddress {
    pub address: String,
    pub resolved_address: Address,
    pub address_type: String,
}

/// Blockchain MCP Service - Following PRD Example Exactly
/// 
/// This matches the "MyMcp" struct from the PRD example, using Cast directly
#[derive(Clone)]
pub struct BlockchainService {
    /// Provider for blockchain connection (we'll create Cast on-demand)
    provider: RootProvider<AnyNetwork>,
    /// Alice's address (default sender from PRD)
    alice_address: Address,
    /// Bob's address (default recipient from PRD)
    bob_address: Address,
    /// Alice's private key for transactions
    alice_private_key: String,
    /// All available anvil accounts (addresses and private keys)
    anvil_accounts: Vec<AccountInfo>,
    /// Tool router for MCP
    tool_router: ToolRouter<Self>,
}

#[tool_router]
impl BlockchainService {
    /// Create a new blockchain service instance
    pub async fn new() -> Result<Self> {
        // Load environment variables from .env file if it exists
        if let Err(_) = dotenv::dotenv() {
            info!("No .env file found, using system environment variables only");
        } else {
            info!("Loaded environment variables from .env file");
        }
        
        let rpc_url = env::var("ANVIL_RPC_URL").unwrap_or_else(|_| "http://127.0.0.1:8545".to_string());
        
        // Create provider connection to anvil
        let provider = ProviderBuilder::<_, _, AnyNetwork>::default()
            .connect(&rpc_url)
            .await?;

        // Dynamically get all accounts from the anvil node
        let available_addresses = provider.get_accounts().await
            .map_err(|e| eyre::eyre!("Failed to get accounts from anvil: {}", e))?;

        if available_addresses.is_empty() {
            return Err(eyre::eyre!("No accounts available from anvil node"));
        }

        // PRD requirement: Default sender is account 0 (first account from anvil)
        let alice_address = available_addresses[0]; // Account 0 - default sender
        
        // PRD requirement: Bob is account 1 (second account from anvil)
        let bob_address = if available_addresses.len() > 1 {
            available_addresses[1] // Account 1 - default recipient
        } else {
            return Err(eyre::eyre!("Need at least 2 accounts from anvil for Alice and Bob"));
        };

        // Load accounts dynamically - addresses only, no private keys from RPC
        let anvil_accounts = Self::load_anvil_accounts(&available_addresses).await?;
        
        // Load Alice's private key from environment variable for transaction signing
        let alice_private_key = env::var("ALICE_PRIVATE_KEY")
            .or_else(|_| env::var("PRIVATE_KEY"))
            .unwrap_or_else(|_| {
                info!("⚠️  No ALICE_PRIVATE_KEY or PRIVATE_KEY found in environment");
                info!("    Transactions will not be possible without a private key");
                String::new()
            });

        info!("🔗 Blockchain service configured for anvil network at {}", rpc_url);
        info!("👤 Alice (Account 0): {} (default sender per PRD)", alice_address);
        info!("👤 Bob (Account 1): {} (default recipient per PRD)", bob_address);
        info!("📊 Loaded {} accounts from anvil", anvil_accounts.len());
        if !alice_private_key.is_empty() {
            info!("🔑 Alice's private key loaded for transaction signing");
        } else {
            info!("⚠️  Alice's private key not available - transactions disabled");
        }

        Ok(Self {
            provider,
            alice_address,
            bob_address,
            alice_private_key,
            anvil_accounts,
            tool_router: Self::tool_router(),
        })
    }

    /// Load anvil accounts dynamically - addresses only from eth_accounts RPC
    async fn load_anvil_accounts(addresses: &[Address]) -> Result<Vec<AccountInfo>> {
        let mut accounts = Vec::new();
        
        for (index, &address) in addresses.iter().enumerate() {
            let address_str = format!("{:?}", address);
            
            // No private keys available via RPC - this is by design for security
            accounts.push(AccountInfo {
                index: index as u32,
                address: address_str,
                private_key: None, // Private keys not exposed via RPC
            });
        }

        Ok(accounts)
    }

    /// Get the balance of an account in wei - Following PRD Example Pattern
    #[tool(description = "Get the balance of an account in wei")]
    async fn balance(
        &self,
        Parameters(BalanceRequest { who }): Parameters<BalanceRequest>,
    ) -> Result<CallToolResult, McpError> {
        let who_clone = who.clone();
        let address = NameOrAddress::from(who)
            .resolve(&self.provider)
            .await
            .unwrap();
        let balance = self.provider.get_balance(address).await.unwrap();

        // Convert wei to ETH for better readability
        let balance_eth = balance.to_f64().unwrap_or(0.0) / 1e18;
        
        let response_text = format!(
            "ETH Balance Query:\n\
            Account: {} (resolved to {})\n\
            Balance: {:.6} ETH ({} wei)",
            who_clone, address, balance_eth, balance
        );

        Ok(CallToolResult::success(vec![Content::text(response_text)]))
    }

    /// Send ETH from Alice to another address using Cast::send
    #[tool(description = "Send ETH from Alice to another address - NOTE: Requires private key access")]
    async fn send_eth(
        &self,
        Parameters(TransferRequest { to, amount }): Parameters<TransferRequest>,
    ) -> Result<CallToolResult, McpError> {
        info!("🚀 MCP Server: send_eth called with to={}, amount={}", to, amount);
        // Step 1: Validate recipient address (PRD requirement)
        let validated_recipient = self.validate_recipient_address(&to).await?;
        
        // Check if we have Alice's private key available from environment
        if self.alice_private_key.is_empty() {
            return Ok(CallToolResult::success(vec![Content::text(
                format!(
                    "ERROR: Cannot send transaction - private key not available.\n\n\
                    Alice's address: {}\n\
                    Validated recipient: {} ({})\n\
                    Requested transfer: {} ETH\n\n\
                    SOLUTION: Set the private key in your environment:\n\
                    export ALICE_PRIVATE_KEY=\"0x...\"\n\
                    or\n\
                    export PRIVATE_KEY=\"0x...\"\n\n\
                    The private key should correspond to Alice's address ({}).\n\
                    Accounts are loaded dynamically from anvil, but private keys must be\n\
                    provided via environment variables for security.",
                    self.alice_address, validated_recipient.address, validated_recipient.address_type, amount, self.alice_address
                )
            )]))
        }

        let to_address = validated_recipient.resolved_address;
        
        // Parse amount to wei
        let amount_wei = U256::from_str(&format!("{}000000000000000000", amount.replace(".", "")))
            .unwrap();
        
        // Create transaction request
        let tx = TransactionRequest::default()
            .to(to_address)
            .value(amount_wei)
            .from(self.alice_address);
        
        let tx = WithOtherFields::new(tx);
        
        // Create Cast instance and send transaction
        let cast = Cast::new(self.provider.clone());
        let pending_tx = cast.send(tx).await.unwrap();
        let tx_hash = *pending_tx.tx_hash();
        
        let response_text = format!(
            "ETH Transfer Successful:\n\
            From: {} (Alice)\n\
            To: {} ({})\n\
            Amount: {} ETH\n\
            Transaction Hash: {}\n\
            Status: Sent to network",
            self.alice_address,
            validated_recipient.address,
            validated_recipient.address_type,
            amount,
            tx_hash
        );
        
        info!("🔍 MCP Server send_eth response: {}", response_text);
        info!("📝 Transaction hash: {}", tx_hash);
        
        Ok(CallToolResult::success(vec![Content::text(response_text)]))
    }

    /// Check if a contract is deployed using Cast::code
    #[tool(description = "Check if a contract is deployed at the specified address")]
    async fn is_contract_deployed(
        &self,
        Parameters(ContractDeploymentRequest { address }): Parameters<ContractDeploymentRequest>,
    ) -> Result<CallToolResult, McpError> {
        // Validate the contract address
        let validated_address = self.validate_recipient_address(&address).await?;
        let addr = validated_address.resolved_address;
        
        // Create Cast instance and check if there's code at the address
        let cast = Cast::new(self.provider.clone());
        let code = cast.code(addr, None, false).await.unwrap();
        
        // Contract is deployed if code is not "0x" (empty)
        let is_deployed = !code.is_empty() && code != "0x";
        
        Ok(CallToolResult::success(vec![Content::text(format!(
            "Contract Deployment Check:\n\
            Input: {} ({})\n\
            Resolved Address: {}\n\
            Status: {}\n\
            Code Length: {} bytes",
            validated_address.address,
            validated_address.address_type,
            validated_address.resolved_address,
            if is_deployed { "DEPLOYED" } else { "NOT DEPLOYED" },
            if code.len() > 2 { (code.len() - 2) / 2 } else { 0 } // Remove 0x prefix and convert hex to bytes
        ))]))
    }

    /// Get ERC-20 token balance for an account
    #[tool(description = "Get ERC-20 token balance (e.g., USDC) for an account")]
    async fn token_balance(
        &self,
        Parameters(TokenBalanceRequest { token_address, account_address }): Parameters<TokenBalanceRequest>,
    ) -> Result<CallToolResult, McpError> {
        info!("🔍 Starting token balance query for token: {}, account: {}", token_address, account_address);
        
        let token_addr = Address::from_str(&token_address)
            .map_err(|e| {
                error!("❌ Invalid token address: {}", e);
                McpError::invalid_params(format!("Invalid token address: {}", e), None)
            })?;
        let account_addr = Address::from_str(&account_address)
            .map_err(|e| {
                error!("❌ Invalid account address: {}", e);
                McpError::invalid_params(format!("Invalid account address: {}", e), None)
            })?;
        
        info!("✅ Address validation passed");
        
        // ERC-20 balanceOf(address) function selector: 0x70a08231
        // Encode the function call: balanceOf(account_address)
        let mut call_data = Vec::new();
        call_data.extend_from_slice(&[0x70, 0xa0, 0x82, 0x31]); // balanceOf selector
        call_data.extend_from_slice(&[0; 12]); // Pad to 32 bytes
        call_data.extend_from_slice(account_addr.as_slice()); // Account address (20 bytes)
        
        let call_request = TransactionRequest::default()
            .to(token_addr)
            .input(Bytes::from(call_data).into());
        
        info!("📞 Making balanceOf call to token contract...");
        
        // Make the call
        let result = self.provider.call(WithOtherFields::new(call_request)).await
            .map_err(|e| {
                error!("❌ Failed to call token contract: {}", e);
                McpError::internal_error(format!("Failed to call token contract: {}", e), None)
            })?;
        
        info!("✅ balanceOf call successful, result length: {}", result.len());
        
        // Decode the result (U256 balance)
        let balance = if result.len() >= 32 {
            U256::from_be_slice(&result[result.len()-32..])
        } else {
            U256::ZERO
        };
        
        info!("📊 Decoded balance: {}", balance);
        
        // Try to get token symbol and decimals for better formatting
        info!("🔍 Getting token info (symbol and decimals)...");
        let (symbol, decimals) = self.get_token_info(&token_addr).await;
        info!("✅ Token info: symbol={}, decimals={}", symbol, decimals);
        
        let formatted_balance = if decimals > 0 {
            let divisor = U256::from(10).pow(U256::from(decimals));
            let whole = balance / divisor;
            let fraction = balance % divisor;
            format!("{}.{:0width$} {}", whole, fraction, symbol, width = decimals as usize)
        } else {
            format!("{} {}", balance, symbol)
        };
        
        let response_text = format!(
            "Token Balance:\nAccount: {}\nToken: {} ({})\nBalance: {} (raw: {})",
            account_address, token_address, symbol, formatted_balance, balance
        );
        
        info!("✅ Token balance query completed successfully");
        info!("📝 Response: {}", response_text);
        
        Ok(CallToolResult::success(vec![Content::text(response_text)]))
    }

    /// Helper function to get token symbol and decimals
    async fn get_token_info(&self, token_addr: &Address) -> (String, u8) {
        info!("🔍 Getting token info for address: {}", token_addr);
        
        // Try to get symbol - ERC-20 symbol() function selector: 0x95d89b41
        let symbol_call = TransactionRequest::default()
            .to(*token_addr)
            .input(Bytes::from([0x95, 0xd8, 0x9b, 0x41]).into());
        
        let symbol = if let Ok(result) = self.provider.call(WithOtherFields::new(symbol_call)).await {
            info!("✅ Symbol call successful, result length: {}", result.len());
            // Decode string (skip first 64 bytes for offset and length, then read the string)
            if result.len() > 64 {
                let length = u32::from_be_bytes([result[60], result[61], result[62], result[63]]) as usize;
                if result.len() >= 64 + length {
                    let symbol_str = String::from_utf8(result[64..64+length].to_vec()).unwrap_or("UNKNOWN".to_string());
                    info!("📊 Decoded symbol: {}", symbol_str);
                    symbol_str
                } else {
                    info!("⚠️  Symbol result too short for decoding");
                    "UNKNOWN".to_string()
                }
            } else {
                info!("⚠️  Symbol result too short");
                "UNKNOWN".to_string()
            }
        } else {
            info!("⚠️  Symbol call failed");
            "UNKNOWN".to_string()
        };
        
        // Try to get decimals - ERC-20 decimals() function selector: 0x313ce567
        let decimals_call = TransactionRequest::default()
            .to(*token_addr)
            .input(Bytes::from([0x31, 0x3c, 0xe5, 0x67]).into());
        
        let decimals = if let Ok(result) = self.provider.call(WithOtherFields::new(decimals_call)).await {
            info!("✅ Decimals call successful, result length: {}", result.len());
            if result.len() >= 32 {
                let decimals_val = result[31]; // Last byte should contain decimals for most tokens
                info!("📊 Decoded decimals: {}", decimals_val);
                decimals_val
            } else {
                info!("⚠️  Decimals result too short, defaulting to 18");
                18 // Default to 18 decimals
            }
        } else {
            info!("⚠️  Decimals call failed, defaulting to 18");
            18 // Default to 18 decimals
        };
        
        info!("✅ Token info complete: symbol={}, decimals={}", symbol, decimals);
        (symbol, decimals)
    }

    /// Validate recipient address - PRD requirement for address validation
    async fn validate_recipient_address(&self, address_input: &str) -> Result<ValidatedAddress, McpError> {
        let trimmed_input = address_input.trim();
        
        // Step 1: Check if it's a valid Ethereum address format
        if let Ok(eth_address) = Address::from_str(trimmed_input) {
            // Valid hex address format
            return Ok(ValidatedAddress {
                address: trimmed_input.to_string(),
                resolved_address: eth_address,
                address_type: "Ethereum Address".to_string(),
            });
        }
        
        // Step 2: Check if it's an ENS name and try to resolve it
        if trimmed_input.ends_with(".eth") || trimmed_input.contains('.') {
            match NameOrAddress::from(trimmed_input.to_string()).resolve(&self.provider).await {
                Ok(resolved_address) => {
                    return Ok(ValidatedAddress {
                        address: trimmed_input.to_string(),
                        resolved_address,
                        address_type: "ENS Name (resolved)".to_string(),
                    });
                }
                Err(e) => {
                    return Err(McpError::invalid_params(
                        format!("Failed to resolve ENS name '{}': {}", trimmed_input, e),
                        None
                    ));
                }
            }
        }
        
        // Step 3: Check if it's a known account name (Alice, Bob, etc.)
        let lowercase_input = trimmed_input.to_lowercase();
        
        // Handle Alice and Bob specifically (PRD requirement)
        if lowercase_input == "alice" {
            return Ok(ValidatedAddress {
                address: format!("{:?}", self.alice_address),
                resolved_address: self.alice_address,
                address_type: "Alice (Account 0 - Default Sender)".to_string(),
            });
        }
        
        if lowercase_input == "bob" {
            return Ok(ValidatedAddress {
                address: format!("{:?}", self.bob_address),
                resolved_address: self.bob_address,
                address_type: "Bob (Account 1 - Default Recipient)".to_string(),
            });
        }
        
        // Handle numbered accounts
        let known_accounts = [
            ("account0", 0),
            ("account1", 1),
            ("account2", 2),
            ("account3", 3),
            ("account4", 4),
            ("account5", 5),
            ("account6", 6),
            ("account7", 7),
            ("account8", 8),
            ("account9", 9),
        ];
        
        for (name, index) in known_accounts.iter() {
            if lowercase_input == *name {
                if let Some(account) = self.anvil_accounts.get(*index) {
                    if let Ok(addr) = Address::from_str(&account.address) {
                        return Ok(ValidatedAddress {
                            address: account.address.clone(),
                            resolved_address: addr,
                            address_type: format!("Anvil Account {}", index),
                        });
                    }
                }
            }
        }
        
        // Step 4: If nothing matches, return validation error
        Err(McpError::invalid_params(
            format!(
                "Invalid recipient address: '{}'\n\n\
                Valid formats:\n\
                - Ethereum address: 0x742d35Cc6634C0532925a3b8D8C9C0C4e8C6C85b\n\
                - ENS name: vitalik.eth\n\
                - Known accounts: alice, bob, account0, account1, etc.\n\n\
                Please provide a valid recipient address.",
                trimmed_input
            ),
            None
        ))
    }

    /// Get list of all available anvil accounts (addresses only)
    #[tool(description = "Get list of all available anvil accounts with their addresses")]
    async fn get_accounts(&self) -> Result<CallToolResult, McpError> {
        // Create account list without private keys for security
        let accounts: Vec<AccountInfo> = self.anvil_accounts
            .iter()
            .map(|acc| AccountInfo {
                index: acc.index,
                address: acc.address.clone(),
                private_key: None, // Don't expose private keys in this method
            })
            .collect();

        let response = AccountListResponse {
            total: accounts.len() as u32,
            accounts,
        };

        let json_response = serde_json::to_string_pretty(&response).unwrap();
        
        Ok(CallToolResult::success(vec![Content::text(json_response)]))
    }

    /// Get list of all available anvil accounts with private key status
    #[tool(description = "Get list of all available anvil accounts - Private keys loaded from environment")]
    async fn get_private_keys(&self) -> Result<CallToolResult, McpError> {
        // Clone accounts and add private key info where available
        let mut accounts_with_keys = self.anvil_accounts.clone();
        
        // Only Alice (first account) has private key from environment
        if !accounts_with_keys.is_empty() && !self.alice_private_key.is_empty() {
            accounts_with_keys[0].private_key = Some(format!("{}...", &self.alice_private_key[..10])); // Show only first 10 chars
        }

        let response = AccountListResponse {
            total: accounts_with_keys.len() as u32,
            accounts: accounts_with_keys,
        };

        let json_response = serde_json::to_string_pretty(&response).unwrap();
        
        // Add explanatory note about private key management
        let explanation = format!(
            "\n\nNOTE: Account addresses loaded dynamically from anvil via eth_accounts RPC.\n\
            Private key for Alice (account 0) loaded from environment variable.\n\
            Environment variables checked: ALICE_PRIVATE_KEY, PRIVATE_KEY\n\
            Private key available for transactions: {}\n\n\
            Other accounts would need their private keys provided via additional\n\
            environment variables to enable transactions from those addresses.",
            if self.alice_private_key.is_empty() { "NO" } else { "YES" }
        );
        
        Ok(CallToolResult::success(vec![Content::text(format!("{}{}", json_response, explanation))]))
    }

    /// Get default addresses as specified in PRD
    #[tool(description = "Get the default sender and recipient addresses as specified in PRD")]
    async fn get_default_addresses(&self) -> Result<CallToolResult, McpError> {
        let response = format!(
            "Default Addresses (PRD Configuration):\n\n\
            👤 Alice (Account 0 - Default Sender):\n\
            Address: {}\n\
            Private Key: {}\n\
            Status: {}\n\n\
            👤 Bob (Account 1 - Default Recipient):\n\
            Address: {}\n\
            Private Key: Not available (for security)\n\n\
            📋 Usage:\n\
            • Alice (Account 0) is the default sender for all transactions\n\
            • Bob (Account 1) is the default recipient when not specified\n\
            • Addresses are dynamically loaded from anvil (PRD requirement)\n\
            • Alice's private key must be set in environment for transactions\n\n\
            🔧 Configuration:\n\
            • Alice: Account 0 from anvil (default sender)\n\
            • Bob: Account 1 from anvil (default recipient)\n\
            • ALICE_PRIVATE_KEY: [set in .env file]\n\n\
            💡 Example Commands:\n\
            • \"send 1 ETH from Alice to Bob\"\n\
            • \"send 0.5 ETH to Bob\" (Alice is default sender)\n\
            • \"How much ETH does Alice have?\"\n\n\
            📊 Anvil Accounts Loaded: {}",
            self.alice_address,
            if self.alice_private_key.is_empty() { "NOT SET" } else { "SET" },
            if self.alice_private_key.is_empty() { "❌ Transactions disabled" } else { "✅ Transactions enabled" },
            self.bob_address,
            self.anvil_accounts.len()
        );
        
        Ok(CallToolResult::success(vec![Content::text(response)]))
    }
}

/// Implement the MCP ServerHandler trait
#[tool_handler]
impl ServerHandler for BlockchainService {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            instructions: Some(
                "Ethereum blockchain operations server. Provides tools for balance queries, ETH transfers, and contract deployment checks using Foundry cast commands.".to_string()
            ),
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            ..Default::default()
        }
    }
}
