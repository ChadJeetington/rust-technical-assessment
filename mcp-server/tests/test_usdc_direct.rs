use alloy_primitives::{Address, U256, Bytes};
use alloy_provider::{Provider, ProviderBuilder};
use alloy_network::AnyNetwork;
use alloy_rpc_types::TransactionRequest;
use alloy_serde::WithOtherFields;
use std::str::FromStr;

#[tokio::test]
async fn test_usdc_balance_direct() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 Testing USDC balance directly...");
    
    // Connect to anvil
    let provider = ProviderBuilder::<_, _, AnyNetwork>::default()
        .connect("http://127.0.0.1:8545")
        .await?;
    
    // USDC address
    let usdc_address = "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48";
    let alice_address = "0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266";
    
    let token_addr = Address::from_str(usdc_address)?;
    let account_addr = Address::from_str(alice_address)?;
    
    println!("📝 USDC Address: {}", token_addr);
    println!("📝 Alice Address: {}", account_addr);
    
    // Test balanceOf call
    println!("\n🔍 Testing balanceOf call...");
    let mut call_data = Vec::new();
    call_data.extend_from_slice(&[0x70, 0xa0, 0x82, 0x31]); // balanceOf selector
    call_data.extend_from_slice(&[0; 12]); // Pad to 32 bytes
    call_data.extend_from_slice(account_addr.as_slice()); // Account address (20 bytes)
    
    let call_request = TransactionRequest::default()
        .to(token_addr)
        .input(Bytes::from(call_data).into());
    
    let result = provider.call(WithOtherFields::new(call_request)).await?;
    println!("✅ balanceOf result: 0x{}", hex::encode(&result));
    
    // Decode the result
    let balance = if result.len() >= 32 {
        U256::from_be_slice(&result[result.len()-32..])
    } else {
        U256::ZERO
    };
    println!("📊 Decoded balance: {}", balance);
    
    // Test symbol call
    println!("\n🔍 Testing symbol call...");
    let symbol_call = TransactionRequest::default()
        .to(token_addr)
        .input(Bytes::from([0x95, 0xd8, 0x9b, 0x41]).into());
    
    let symbol_result = provider.call(WithOtherFields::new(symbol_call)).await?;
    println!("✅ symbol result: 0x{}", hex::encode(&symbol_result));
    
    // Decode symbol
    let symbol = if symbol_result.len() > 64 {
        let length = u32::from_be_bytes([symbol_result[60], symbol_result[61], symbol_result[62], symbol_result[63]]) as usize;
        if symbol_result.len() >= 64 + length {
            String::from_utf8(symbol_result[64..64+length].to_vec()).unwrap_or_else(|_| "UNKNOWN".to_string())
        } else {
            "UNKNOWN".to_string()
        }
    } else {
        "UNKNOWN".to_string()
    };
    println!("📊 Decoded symbol: {}", symbol);
    
    // Test decimals call
    println!("\n🔍 Testing decimals call...");
    let decimals_call = TransactionRequest::default()
        .to(token_addr)
        .input(Bytes::from([0x31, 0x3c, 0xe5, 0x67]).into());
    
    let decimals_result = provider.call(WithOtherFields::new(decimals_call)).await?;
    println!("✅ decimals result: 0x{}", hex::encode(&decimals_result));
    
    // Decode decimals
    let decimals = if decimals_result.len() >= 32 {
        decimals_result[31]
    } else {
        18
    };
    println!("📊 Decoded decimals: {}", decimals);
    
    // Format balance
    let formatted_balance = if decimals > 0 {
        let divisor = U256::from(10).pow(U256::from(decimals));
        let whole = balance / divisor;
        let fraction = balance % divisor;
        format!("{}.{:0width$} {}", whole, fraction, symbol, width = decimals as usize)
    } else {
        format!("{} {}", balance, symbol)
    };
    
    println!("\n🎉 Final Result:");
    println!("Token Balance:");
    println!("Account: {}", alice_address);
    println!("Token: {} ({})", usdc_address, symbol);
    println!("Balance: {} (raw: {})", formatted_balance, balance);
    
    // Assert that we got a valid response
    assert!(symbol == "USDC", "Expected USDC symbol, got: {}", symbol);
    assert_eq!(decimals, 6, "Expected 6 decimals for USDC, got: {}", decimals);
    
    Ok(())
}
