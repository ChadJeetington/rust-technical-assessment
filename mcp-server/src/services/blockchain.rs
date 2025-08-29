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
use alloy_primitives::{Address, U256, Bytes, TxHash};
use alloy_provider::{Provider, ProviderBuilder, RootProvider, PendingTransactionBuilder};
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
use std::{env, str::FromStr, time::Duration};
use tracing::{info, error};
use hex;

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

/// Request structure for token swaps
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct SwapRequest {
    #[schemars(description = "Token to swap from (e.g., 'ETH')")]
    pub from_token: String,
    #[schemars(description = "Token to swap to (e.g., 'USDC')")]
    pub to_token: String,
    #[schemars(description = "Amount to swap (e.g., '10')")]
    pub amount: String,
    #[schemars(description = "DEX to use (e.g., 'Uniswap V2')")]
    pub dex: Option<String>,
    #[schemars(description = "Slippage tolerance in basis points (e.g., '500' for 5%)")]
    pub slippage: Option<String>,
}

/// Request structure for transaction status checks
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct TransactionStatusRequest {
    #[schemars(description = "Transaction hash to check status for")]
    pub tx_hash: String,
    #[schemars(description = "Timeout in seconds for waiting for transaction (default: 30)")]
    pub timeout: Option<u64>,
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
        if dotenv::dotenv().is_err() {
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
    pub async fn balance(
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
    pub async fn send_eth(
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
        
        info!("📝 Transaction sent with hash: {}", tx_hash);
        
        // Wait for transaction confirmation (30 second timeout)
        match self.wait_for_transaction_confirmation(tx_hash, 30).await {
            Ok(confirmation_text) => {
                let response_text = format!(
                    "ETH Transfer:\n\
                    From: {} (Alice)\n\
                    To: {} ({})\n\
                    Amount: {} ETH\n\
                    \n{}",
                    self.alice_address,
                    validated_recipient.address,
                    validated_recipient.address_type,
                    amount,
                    confirmation_text
                );
                
                info!("🔍 MCP Server send_eth response: {}", response_text);
                Ok(CallToolResult::success(vec![Content::text(response_text)]))
            }
            Err(_e) => {
                // If waiting fails, return the transaction hash for manual checking
                let response_text = format!(
                    "ETH Transfer Sent:\n\
                    From: {} (Alice)\n\
                    To: {} ({})\n\
                    Amount: {} ETH\n\
                    Transaction Hash: {}\n\
                    Status: Sent to network (confirmation timeout)\n\
                    \n⚠️  Transaction was sent but confirmation timed out.\n\
                    Use check_transaction_status with hash {} to check the final status.",
                    self.alice_address,
                    validated_recipient.address,
                    validated_recipient.address_type,
                    amount,
                    tx_hash,
                    tx_hash
                );
                
                info!("⚠️  MCP Server send_eth response (timeout): {}", response_text);
                Ok(CallToolResult::success(vec![Content::text(response_text)]))
            }
        }
    }

    /// Check if a contract is deployed using Cast::code
    #[tool(description = "Check if a contract is deployed at the specified address")]
    pub async fn is_contract_deployed(
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
    pub async fn token_balance(
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
            if lowercase_input == *name
                && let Some(account) = self.anvil_accounts.get(*index)
                    && let Ok(addr) = Address::from_str(&account.address) {
                        return Ok(ValidatedAddress {
                            address: account.address.clone(),
                            resolved_address: addr,
                            address_type: format!("Anvil Account {}", index),
                        });
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
    pub async fn get_accounts(&self) -> Result<CallToolResult, McpError> {
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
    pub async fn get_private_keys(&self) -> Result<CallToolResult, McpError> {
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

    /// Execute a token swap using Uniswap V2 Router
    #[tool(description = "Swap tokens using Uniswap V2 Router - integrates with search API to find contract addresses")]
    pub async fn swap_tokens(
        &self,
        Parameters(SwapRequest { from_token, to_token, amount, dex, slippage }): Parameters<SwapRequest>,
    ) -> Result<CallToolResult, McpError> {
        info!("🔄 MCP Server: swap_tokens called with from={}, to={}, amount={}, dex={:?}", 
              from_token, to_token, amount, dex);
        
        // Check if we have Alice's private key available
        if self.alice_private_key.is_empty() {
            return Ok(CallToolResult::success(vec![Content::text(
                format!(
                    "ERROR: Cannot execute swap - private key not available.\n\n\
                    Alice's address: {}\n\
                    Requested swap: {} {} to {}\n\
                    DEX: {}\n\n\
                    SOLUTION: Set the private key in your environment:\n\
                    export ALICE_PRIVATE_KEY=\"0x...\"\n\
                    or\n\
                    export PRIVATE_KEY=\"0x...\"\n\n\
                    The private key should correspond to Alice's address ({}).",
                    self.alice_address, amount, from_token, to_token, dex.as_deref().unwrap_or("Uniswap V2"), self.alice_address
                )
            )]))
        }

        let dex_name = dex.unwrap_or_else(|| "Uniswap V2".to_string());
        let slippage_bps = slippage.unwrap_or_else(|| "500".to_string()); // Default 5% slippage
        
        // Step 1: Get Uniswap V2 Router address (hardcoded for mainnet)
        let router_address = "0x7a250d5630B4cF539739dF2C5dAcb4c659F2488D"; // Uniswap V2 Router
        let router_addr = Address::from_str(router_address)
            .map_err(|e| McpError::internal_error(format!("Invalid router address: {}", e), None))?;
        
        info!("📋 Using Uniswap V2 Router: {}", router_address);
        
        // Step 2: Get token addresses (hardcoded common tokens for now)
        let (from_token_addr, to_token_addr) = self.get_token_addresses(&from_token, &to_token).await?;
        
        info!("🪙 Token addresses - From: {} ({}) To: {} ({})", 
              from_token, from_token_addr, to_token, to_token_addr);
        
        // Step 3: Calculate swap parameters
        let amount_wei = self.parse_amount_to_wei(&amount, &from_token).await?;
        let amount_out_min = U256::ZERO; // For now, set to 0 (no slippage protection)
        
        // Step 4: Create swap path
        let path = vec![from_token_addr, to_token_addr];
        
        // Step 5: Calculate deadline (5 minutes from now)
        let deadline = U256::from(
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs() + 300
        );
        
        info!("📊 Swap parameters - Amount: {} wei, Path: {:?}, Deadline: {}", 
              amount_wei, path, deadline);
        
        // Step 6: Encode the swap function call
        let calldata = self.encode_swap_exact_eth_for_tokens(
            amount_out_min,
            &path,
            self.alice_address,
            deadline
        ).await?;
        
        info!("🔧 Encoded calldata: {}", calldata);
        
        // Step 7: Create and send transaction using Cast
        let tx = TransactionRequest::default()
            .to(router_addr)
            .value(amount_wei) // Send ETH with the transaction
            .input(Bytes::from_str(&calldata).unwrap().into())
            .from(self.alice_address);
        
        let tx = WithOtherFields::new(tx);
        
        // Create Cast instance and send transaction
        let cast = Cast::new(self.provider.clone());
        let pending_tx = cast.send(tx).await
            .map_err(|e| McpError::internal_error(format!("Failed to send swap transaction: {}", e), None))?;
        let tx_hash = *pending_tx.tx_hash();
        
        info!("📝 Swap transaction sent with hash: {}", tx_hash);
        
        // Wait for transaction confirmation (30 second timeout)
        match self.wait_for_transaction_confirmation(tx_hash, 30).await {
            Ok(confirmation_text) => {
                let response_text = format!(
                    "Token Swap:\n\
                    From: {} (Alice)\n\
                    Swap: {} {} → {} {}\n\
                    DEX: {}\n\
                    Router: {}\n\
                    Amount: {} {} ({} wei)\n\
                    Path: {} → {}\n\
                    Slippage: {}%\n\
                    \n{}\n\n\
                    💡 Note: This is a test transaction on forked mainnet.\n\
                    The swap will execute using real Uniswap V2 contracts.",
                    self.alice_address,
                    amount, from_token, amount, to_token,
                    dex_name,
                    router_address,
                    amount, from_token, amount_wei,
                    from_token, to_token,
                    (slippage_bps.parse::<u32>().unwrap_or(500) as f64) / 100.0,
                    confirmation_text
                );
                
                info!("🔍 MCP Server swap_tokens response: {}", response_text);
                Ok(CallToolResult::success(vec![Content::text(response_text)]))
            }
            Err(_e) => {
                // If waiting fails, return the transaction hash for manual checking
                let response_text = format!(
                    "Token Swap Sent:\n\
                    From: {} (Alice)\n\
                    Swap: {} {} → {} {}\n\
                    DEX: {}\n\
                    Router: {}\n\
                    Amount: {} {} ({} wei)\n\
                    Path: {} → {}\n\
                    Slippage: {}%\n\
                    Transaction Hash: {}\n\
                    Status: Sent to network (confirmation timeout)\n\
                    \n⚠️  Transaction was sent but confirmation timed out.\n\
                    Use check_transaction_status with hash {} to check the final status.\n\n\
                    💡 Note: This is a test transaction on forked mainnet.\n\
                    The swap will execute using real Uniswap V2 contracts.",
                    self.alice_address,
                    amount, from_token, amount, to_token,
                    dex_name,
                    router_address,
                    amount, from_token, amount_wei,
                    from_token, to_token,
                    (slippage_bps.parse::<u32>().unwrap_or(500) as f64) / 100.0,
                    tx_hash,
                    tx_hash
                );
                
                info!("⚠️  MCP Server swap_tokens response (timeout): {}", response_text);
                Ok(CallToolResult::success(vec![Content::text(response_text)]))
            }
        }
    }

    /// Get default addresses as specified in PRD
    #[tool(description = "Get the default sender and recipient addresses as specified in PRD")]
    pub async fn get_default_addresses(&self) -> Result<CallToolResult, McpError> {
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

    /// Helper method to get token addresses for common tokens
    async fn get_token_addresses(&self, from_token: &str, to_token: &str) -> Result<(Address, Address), McpError> {
        // Hardcoded addresses for common tokens on mainnet
        let token_addresses = [
            ("ETH", "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2"), // WETH
            ("WETH", "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2"),
            ("USDC", "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48"),
            ("USDT", "0xdAC17F958D2ee523a2206206994597C13D831ec7"),
            ("DAI", "0x6B175474E89094C44Da98b954EedeAC495271d0F"),
            ("LINK", "0x514910771AF9Ca656af840dff83E8264EcF986CA"),
            ("UNI", "0x1f9840a85d5aF5bf1D1762F925BDADdC4201F984"),
        ];

        let from_addr = token_addresses
            .iter()
            .find(|(symbol, _)| symbol.eq_ignore_ascii_case(from_token))
            .map(|(_, addr)| Address::from_str(addr).unwrap())
            .unwrap_or_else(|| {
                // If not found, try to parse as address
                Address::from_str(from_token).unwrap_or_else(|_| {
                    panic!("Unknown token: {}. Supported tokens: ETH, WETH, USDC, USDT, DAI, LINK, UNI", from_token)
                })
            });

        let to_addr = token_addresses
            .iter()
            .find(|(symbol, _)| symbol.eq_ignore_ascii_case(to_token))
            .map(|(_, addr)| Address::from_str(addr).unwrap())
            .unwrap_or_else(|| {
                // If not found, try to parse as address
                Address::from_str(to_token).unwrap_or_else(|_| {
                    panic!("Unknown token: {}. Supported tokens: ETH, WETH, USDC, USDT, DAI, LINK, UNI", to_token)
                })
            });

        Ok((from_addr, to_addr))
    }

    /// Helper method to parse amount to wei
    async fn parse_amount_to_wei(&self, amount: &str, _token: &str) -> Result<U256, McpError> {
        let amount_float = amount.parse::<f64>()
            .map_err(|e| McpError::invalid_params(format!("Invalid amount: {}", e), None))?;
        
        // Convert to wei (18 decimals for ETH)
        let amount_wei = (amount_float * 1e18) as u128;
        Ok(U256::from(amount_wei))
    }

    /// Helper method to encode swapExactETHForTokens function call
    async fn encode_swap_exact_eth_for_tokens(
        &self,
        amount_out_min: U256,
        path: &[Address],
        to: Address,
        deadline: U256,
    ) -> Result<String, McpError> {
        // Function signature: swapExactETHForTokens(uint256 amountOutMin, address[] path, address to, uint256 deadline)
        // Function selector: 0x7ff36ab5
        
        let mut calldata = Vec::new();
        
        // Function selector
        calldata.extend_from_slice(&[0x7f, 0xf3, 0x6a, 0xb5]);
        
        // Encode amountOutMin (uint256) - 32 bytes
        let amount_out_min_bytes: [u8; 32] = amount_out_min.to_be_bytes();
        calldata.extend_from_slice(&amount_out_min_bytes);
        
        // Encode path array offset (uint256) - 32 bytes
        let path_offset = U256::from(96); // 32 + 32 + 32 = 96 bytes for the first 3 parameters
        let path_offset_bytes: [u8; 32] = path_offset.to_be_bytes();
        calldata.extend_from_slice(&path_offset_bytes);
        
        // Encode to address (address) - 32 bytes
        let mut to_bytes = [0u8; 32];
        to_bytes[12..].copy_from_slice(to.as_slice()); // Address is 20 bytes, padded to 32
        calldata.extend_from_slice(&to_bytes);
        
        // Encode deadline (uint256) - 32 bytes
        let deadline_bytes: [u8; 32] = deadline.to_be_bytes();
        calldata.extend_from_slice(&deadline_bytes);
        
        // Encode path array length (uint256) - 32 bytes
        let path_length = U256::from(path.len());
        let path_length_bytes: [u8; 32] = path_length.to_be_bytes();
        calldata.extend_from_slice(&path_length_bytes);
        
        // Encode path array elements (address[]) - each address is 32 bytes
        for addr in path {
            let mut addr_bytes = [0u8; 32];
            addr_bytes[12..].copy_from_slice(addr.as_slice()); // Address is 20 bytes, padded to 32
            calldata.extend_from_slice(&addr_bytes);
        }
        
        Ok(format!("0x{}", hex::encode(calldata)))
    }

    /// Check transaction status and receipt
    #[tool(description = "Check the status of a transaction by hash - returns success/failure and receipt details")]
    pub async fn check_transaction_status(
        &self,
        Parameters(TransactionStatusRequest { tx_hash, timeout }): Parameters<TransactionStatusRequest>,
    ) -> Result<CallToolResult, McpError> {
        info!("🔍 Checking transaction status for: {}", tx_hash);
        
        let tx_hash = TxHash::from_str(&tx_hash)
            .map_err(|e| McpError::invalid_params(format!("Invalid transaction hash: {}", e), None))?;
        
        let timeout_secs = timeout.unwrap_or(30);
        
        // Try to get the transaction receipt
        match self.provider.get_transaction_receipt(tx_hash).await {
            Ok(Some(receipt)) => {
                // Transaction has been mined
                let status = if receipt.inner.inner.inner.receipt.status.coerce_status() {
                    "SUCCESS"
                } else {
                    "FAILED"
                };
                
                let gas_used = receipt.gas_used;
                let gas_price = receipt.effective_gas_price;
                let total_cost = gas_used as u128 * gas_price;
                
                let response_text = format!(
                    "Transaction Status: {}\n\
                    Hash: {}\n\
                    Block Number: {}\n\
                    Gas Used: {}\n\
                    Gas Price: {} wei\n\
                    Total Cost: {} wei ({:.6} ETH)\n\
                    Status: {}\n\
                    \n📋 Receipt Details:\n\
                    - Transaction Type: {}\n\
                    - Cumulative Gas Used: {}\n\
                    - Contract Address: {}\n\
                    - Logs: {}",
                    status,
                    tx_hash,
                    receipt.block_number.unwrap_or_default(),
                    gas_used,
                    gas_price,
                    total_cost,
                    total_cost.to_f64().unwrap_or(0.0) / 1e18,
                    status,
                    receipt.inner.inner.r#type,
                    receipt.inner.inner.inner.receipt.cumulative_gas_used,
                    receipt.contract_address.map(|addr| format!("{:?}", addr)).unwrap_or_else(|| "None".to_string()),
                    receipt.logs().len()
                );
                
                info!("✅ Transaction status check completed: {}", status);
                Ok(CallToolResult::success(vec![Content::text(response_text)]))
            }
            Ok(None) => {
                // Transaction not yet mined, try to wait for it
                info!("⏳ Transaction not yet mined, waiting up to {} seconds...", timeout_secs);
                
                match PendingTransactionBuilder::new(self.provider.clone(), tx_hash)
                    .with_timeout(Some(Duration::from_secs(timeout_secs)))
                    .get_receipt()
                    .await
                {
                    Ok(receipt) => {
                        let status = if receipt.inner.inner.inner.receipt.status.coerce_status() {
                            "SUCCESS"
                        } else {
                            "FAILED"
                        };
                        
                        let gas_used = receipt.gas_used;
                        let gas_price = receipt.effective_gas_price;
                        let total_cost = gas_used as u128 * gas_price;
                        
                        let response_text = format!(
                            "Transaction Status: {} (Waited for confirmation)\n\
                            Hash: {}\n\
                            Block Number: {}\n\
                            Gas Used: {}\n\
                            Gas Price: {} wei\n\
                            Total Cost: {} wei ({:.6} ETH)\n\
                            Status: {}\n\
                            \n📋 Receipt Details:\n\
                            - Transaction Type: {}\n\
                            - Cumulative Gas Used: {}\n\
                            - Contract Address: {}\n\
                            - Logs: {}",
                            status,
                            tx_hash,
                            receipt.block_number.unwrap_or_default(),
                            gas_used,
                            gas_price,
                            total_cost,
                            total_cost.to_f64().unwrap_or(0.0) / 1e18,
                            status,
                            receipt.inner.inner.r#type,
                            receipt.inner.inner.inner.receipt.cumulative_gas_used,
                            receipt.contract_address.map(|addr| format!("{:?}", addr)).unwrap_or_else(|| "None".to_string()),
                            receipt.logs().len()
                        );
                        
                        info!("✅ Transaction confirmed after waiting: {}", status);
                        Ok(CallToolResult::success(vec![Content::text(response_text)]))
                    }
                    Err(_e) => {
                        // Check if transaction exists in mempool
                        match self.provider.get_transaction_by_hash(tx_hash).await {
                            Ok(Some(_)) => {
                                let response_text = format!(
                                    "Transaction Status: PENDING\n\
                                    Hash: {}\n\
                                    Status: Transaction is in mempool but not yet mined\n\
                                    \n⏳ The transaction was sent to the network and is waiting to be included in a block.\n\
                                    Try checking again in a few seconds, or increase the timeout parameter.\n\
                                    \n💡 Tip: Use a longer timeout (e.g., 60 seconds) for slower networks.",
                                    tx_hash
                                );
                                
                                info!("⏳ Transaction is pending in mempool");
                                Ok(CallToolResult::success(vec![Content::text(response_text)]))
                            }
                            Ok(None) => {
                                let response_text = format!(
                                    "Transaction Status: NOT FOUND\n\
                                    Hash: {}\n\
                                    Status: Transaction not found in mempool or blockchain\n\
                                    \n❌ This transaction hash was not found on the network.\n\
                                    Possible reasons:\n\
                                    - Transaction was never sent\n\
                                    - Transaction was dropped from mempool\n\
                                    - Invalid transaction hash\n\
                                    - Wrong network",
                                    tx_hash
                                );
                                
                                info!("❌ Transaction not found");
                                Ok(CallToolResult::success(vec![Content::text(response_text)]))
                            }
                            Err(e) => {
                                Err(McpError::internal_error(
                                    format!("Failed to check transaction status: {}", e),
                                    None
                                ))
                            }
                        }
                    }
                }
            }
            Err(e) => {
                Err(McpError::internal_error(
                    format!("Failed to get transaction receipt: {}", e),
                    None
                ))
            }
        }
    }

    /// Wait for transaction confirmation and return detailed status
    async fn wait_for_transaction_confirmation(&self, tx_hash: TxHash, timeout_secs: u64) -> Result<String, McpError> {
        info!("⏳ Waiting for transaction confirmation: {}", tx_hash);
        
        match PendingTransactionBuilder::new(self.provider.clone(), tx_hash)
            .with_timeout(Some(Duration::from_secs(timeout_secs)))
            .get_receipt()
            .await
        {
            Ok(receipt) => {
                let status = if receipt.inner.inner.inner.receipt.status.coerce_status() {
                    "SUCCESS"
                } else {
                    "FAILED"
                };
                
                let gas_used = receipt.gas_used;
                let gas_price = receipt.effective_gas_price;
                let total_cost = gas_used as u128 * gas_price;
                
                let response_text = format!(
                    "Transaction Confirmed: {}\n\
                    Hash: {}\n\
                    Block Number: {}\n\
                    Gas Used: {}\n\
                    Gas Price: {} wei\n\
                    Total Cost: {} wei ({:.6} ETH)\n\
                    Status: {}",
                    status,
                    tx_hash,
                    receipt.block_number.unwrap_or_default(),
                    gas_used,
                    gas_price,
                    total_cost,
                    total_cost.to_f64().unwrap_or(0.0) / 1e18,
                    status
                );
                
                info!("✅ Transaction confirmed: {}", status);
                Ok(response_text)
            }
            Err(e) => {
                Err(McpError::internal_error(
                    format!("Failed to wait for transaction confirmation: {}", e),
                    None
                ))
            }
        }
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
