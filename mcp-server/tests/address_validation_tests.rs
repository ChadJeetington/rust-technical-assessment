//! Address Validation Tests for MCP Blockchain Server
//! 
//! These tests verify that addresses from the PRD and anvil account data
//! are properly validated and formatted.

use alloy_primitives::Address;
use std::str::FromStr;
use mcp_server::services::blockchain::AccountInfo;

#[test]
fn test_prd_addresses() {
    println!("\n🧪 Testing PRD addresses are valid...");
    
    // Test Alice's address (account 0)
    println!("\n📋 Test 1: Alice's Address Validation");
    let alice = "0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266";
    println!("📝 INPUT STRING: \"{}\"", alice);
    println!("📝 EXPECTED: Valid Ethereum address (42 chars, starts with 0x)");
    let alice_addr = Address::from_str(alice).unwrap();
    println!("✅ OUTPUT ADDRESS: {}", alice_addr);
    println!("📊 VALIDATION: Address length: {} chars", alice.len());
    println!("📊 VALIDATION: Starts with 0x: {}", alice.starts_with("0x"));
    
    // Test Bob's address (account 1)  
    println!("\n📋 Test 2: Bob's Address Validation");
    let bob = "0x70997970C51812dc3A010C7d01b50e0d17dc79C8";
    println!("📝 INPUT STRING: \"{}\"", bob);
    println!("📝 EXPECTED: Valid Ethereum address (42 chars, starts with 0x)");
    let bob_addr = Address::from_str(bob).unwrap();
    println!("✅ OUTPUT ADDRESS: {}", bob_addr);
    println!("📊 VALIDATION: Address length: {} chars", bob.len());
    println!("📊 VALIDATION: Starts with 0x: {}", bob.starts_with("0x"));
    
    // Test Uniswap V2 Router address from PRD
    println!("\n📋 Test 3: Uniswap V2 Router Address Validation");
    let router = "0x7a250d5630B4cF539739dF2C5dAcb4c659F2488D";
    println!("📝 INPUT STRING: \"{}\"", router);
    println!("📝 EXPECTED: Valid Ethereum contract address (42 chars, starts with 0x)");
    let router_addr = Address::from_str(router).unwrap();
    println!("✅ OUTPUT ADDRESS: {}", router_addr);
    println!("📊 VALIDATION: Address length: {} chars", router.len());
    println!("📊 VALIDATION: Starts with 0x: {}", router.starts_with("0x"));
    
    println!("🔚 PRD address validation tests completed\n");
}

#[test]
fn test_anvil_accounts_data() {
    println!("\n🧪 Testing anvil accounts data validation (dynamic loading compatible)...");
    
    // Test data for known anvil default accounts (what we expect to see in default setup)
    let expected_default_accounts = vec![
        ("0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266", "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80"),
        ("0x70997970C51812dc3A010C7d01b50e0d17dc79C8", "0x59c6995e998f97a5a0044966f0945389dc9e86dae88c7a8412f4603b6b78690d"),
        ("0x3C44CdDdB6a900fa2b585dd299e03d12FA4293BC", "0x5de4111afa1a4b94908f83103eb1f1706367c2e68ca870fc3fb9a804cdab365a"),
        ("0x90F79bf6EB2c4f870365E785982E1f101E93b906", "0x7c852118294e51e653712a81e05800f419141751be58f605c371e15141b007a6"),
        ("0x15d34AAf54267DB7D7c367839AAf71A00a2C6A65", "0x47e179ec197488593b187f80a00eb0da91f1b9d0b13f8733639f19c30a34926a"),
        ("0x9965507D1a55bcC2695C58ba16FB37d819B0A4dc", "0x8b3a350cf5c34c9194ca85829a2df0ec3153be0318b5e2d3348e872092edffba"),
        ("0x976EA74026E726554dB657fA54763abd0C3a0aa9", "0x92db14e403b83dfe3df233f83dfa3a0d7096f21ca9b0d6d6b8d88b2b4ec1564e"),
        ("0x14dC79964da2C08b23698B3D3cc7Ca32193d9955", "0x4bbbf85ce3377467afe5d46f804f221813b2bb87f24d81f60f1fcdbf7cbf4356"),
        ("0x23618e81E3f5cdF7f54C3d65f7FBc0aBf5B21E8f", "0xdbda1821b80551c9d65939329250298aa3472ba22feea921c0cf5d620ea67b97"),
        ("0xa0Ee7A142d267C1f36714E4a8F75612F20a79720", "0x2a871d0798f97d79848a013d4936a73bf4cc922c825d33c1cf7073dff6d409c6"),
    ];
    
    println!("📝 INPUT: {} expected default anvil accounts", expected_default_accounts.len());
    println!("📝 EXPECTED: All addresses and private keys should be valid format");
    println!("📝 NOTE: This tests the known account format that dynamic loading should recognize");
    
    for (index, (address, private_key)) in expected_default_accounts.iter().enumerate() {
        println!("\n📋 Test Account {}: Address and Private Key Validation", index);
        println!("📝 ADDRESS: {}", address);
        println!("📝 PRIVATE_KEY: {}...", &private_key[..10]); // Only show first 10 chars for security
        
        // Validate address format
        let addr_result = Address::from_str(address);
        assert!(addr_result.is_ok(), "Account {} address should be valid: {}", index, address);
        println!("✅ Address validation: PASSED");
        
        // Validate private key format (should be 66 chars, start with 0x)
        assert!(private_key.len() == 66, "Account {} private key should be 66 chars: {}", index, private_key.len());
        assert!(private_key.starts_with("0x"), "Account {} private key should start with 0x: {}", index, private_key);
        println!("✅ Private key format validation: PASSED");
        
        // Test AccountInfo creation (as dynamic loading would create)
        let account_info = AccountInfo {
            index: index as u32,
            address: address.to_string(),
            private_key: Some(private_key.to_string()),
        };
        
        // Test serialization
        let json = serde_json::to_string(&account_info).unwrap();
        assert!(json.contains("index"));
        assert!(json.contains("address"));
        assert!(json.contains("private_key"));
        println!("✅ AccountInfo serialization: PASSED");
    }
    
    // Test AccountInfo with unknown account (no private key)
    println!("\n📋 Test Unknown Account: Address without known private key");
    let unknown_account = AccountInfo {
        index: 999,
        address: "0x1234567890123456789012345678901234567890".to_string(),
        private_key: None,
    };
    println!("📝 INPUT: Unknown address with no private key");
    let json = serde_json::to_string(&unknown_account).unwrap();
    println!("📝 EXPECTED: JSON with null private_key field");
    println!("✅ OUTPUT JSON: {}", json);
    assert!(json.contains("\"private_key\":null"));
    println!("✅ Unknown account handling: PASSED");
    
    println!("\n📊 SUMMARY: All {} default accounts + unknown account validation completed", expected_default_accounts.len());
    println!("🔚 Dynamic anvil accounts data validation tests completed\n");
}
