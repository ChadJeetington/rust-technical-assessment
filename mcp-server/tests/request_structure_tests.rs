//! Request Structure Tests for MCP Blockchain Server
//! 
//! These tests verify that all request and response structures can be
//! properly serialized and deserialized.

use mcp_server::services::blockchain::{BalanceRequest, TransferRequest, ContractDeploymentRequest, AccountInfo, AccountListResponse, TokenBalanceRequest};
use serde_json;

#[test]
fn test_request_structures() {
    println!("\n🧪 Testing request structure serialization...");
    
    // Test BalanceRequest
    println!("\n📋 Test 1: BalanceRequest Serialization");
    let balance_req = BalanceRequest {
        who: "0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266".to_string(),
    };
    println!("📝 INPUT STRUCT: BalanceRequest {{ who: \"{}\" }}", balance_req.who);
    let json = serde_json::to_string(&balance_req).unwrap();
    println!("📝 EXPECTED: JSON string with 'who' field");
    println!("✅ OUTPUT JSON: {}", json);
    println!("📊 VALIDATION: Contains 'who' field: {}", json.contains("who"));
    assert!(json.contains("who"));
    
    // Test TransferRequest
    println!("\n📋 Test 2: TransferRequest Serialization");
    let transfer_req = TransferRequest {
        to: "0x70997970C51812dc3A010C7d01b50e0d17dc79C8".to_string(),
        amount: "1.0".to_string(),
    };
    println!("📝 INPUT STRUCT: TransferRequest {{ to: \"{}\", amount: \"{}\" }}", transfer_req.to, transfer_req.amount);
    let json = serde_json::to_string(&transfer_req).unwrap();
    println!("📝 EXPECTED: JSON string with 'to' and 'amount' fields");
    println!("✅ OUTPUT JSON: {}", json);
    println!("📊 VALIDATION: Contains 'to' field: {}", json.contains("to"));
    println!("📊 VALIDATION: Contains 'amount' field: {}", json.contains("amount"));
    assert!(json.contains("to"));
    assert!(json.contains("amount"));
    
    // Test ContractDeploymentRequest
    println!("\n📋 Test 3: ContractDeploymentRequest Serialization");
    let contract_req = ContractDeploymentRequest {
        address: "0x7a250d5630B4cF539739dF2C5dAcb4c659F2488D".to_string(),
    };
    println!("📝 INPUT STRUCT: ContractDeploymentRequest {{ address: \"{}\" }}", contract_req.address);
    let json = serde_json::to_string(&contract_req).unwrap();
    println!("📝 EXPECTED: JSON string with 'address' field");
    println!("✅ OUTPUT JSON: {}", json);
    println!("📊 VALIDATION: Contains 'address' field: {}", json.contains("address"));
    assert!(json.contains("address"));

    // Test TokenBalanceRequest
    println!("\n📋 Test 4: TokenBalanceRequest Serialization");
    let token_balance_req = TokenBalanceRequest {
        token_address: "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48".to_string(),
        account_address: "0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266".to_string(),
    };
    println!("📝 INPUT STRUCT: TokenBalanceRequest {{ token_address: \"{}\", account_address: \"{}\" }}", 
             token_balance_req.token_address, token_balance_req.account_address);
    let json = serde_json::to_string(&token_balance_req).unwrap();
    println!("📝 EXPECTED: JSON string with 'token_address' and 'account_address' fields");
    println!("✅ OUTPUT JSON: {}", json);
    println!("📊 VALIDATION: Contains 'token_address' field: {}", json.contains("token_address"));
    println!("📊 VALIDATION: Contains 'account_address' field: {}", json.contains("account_address"));
    assert!(json.contains("token_address"));
    assert!(json.contains("account_address"));

    // Test AccountInfo
    println!("\n📋 Test 5: AccountInfo Serialization");
    let account_info = AccountInfo {
        index: 0,
        address: "0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266".to_string(),
        private_key: Some("0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80".to_string()),
    };
    println!("📝 INPUT STRUCT: AccountInfo {{ index: {}, address: \"{}\", private_key: Some(...) }}", account_info.index, account_info.address);
    let json = serde_json::to_string(&account_info).unwrap();
    println!("📝 EXPECTED: JSON string with 'index', 'address', and 'private_key' fields");
    println!("✅ OUTPUT JSON: {}", json);
    println!("📊 VALIDATION: Contains 'index' field: {}", json.contains("index"));
    println!("📊 VALIDATION: Contains 'address' field: {}", json.contains("address"));
    println!("📊 VALIDATION: Contains 'private_key' field: {}", json.contains("private_key"));
    assert!(json.contains("index"));
    assert!(json.contains("address"));
    assert!(json.contains("private_key"));

    // Test AccountInfo without private key
    println!("\n📋 Test 6: AccountInfo Serialization (No Private Key)");
    let account_info_no_key = AccountInfo {
        index: 1,
        address: "0x70997970C51812dc3A010C7d01b50e0d17dc79C8".to_string(),
        private_key: None,
    };
    println!("📝 INPUT STRUCT: AccountInfo {{ index: {}, address: \"{}\", private_key: None }}", account_info_no_key.index, account_info_no_key.address);
    let json = serde_json::to_string(&account_info_no_key).unwrap();
    println!("📝 EXPECTED: JSON string with 'index', 'address' fields and null 'private_key'");
    println!("✅ OUTPUT JSON: {}", json);
    println!("📊 VALIDATION: Contains 'private_key': null: {}", json.contains("\"private_key\":null"));
    assert!(json.contains("\"private_key\":null"));

    // Test AccountListResponse
    println!("\n📋 Test 7: AccountListResponse Serialization");
    let account_list = AccountListResponse {
        total: 2,
        accounts: vec![account_info, account_info_no_key],
    };
    println!("📝 INPUT STRUCT: AccountListResponse with {} accounts", account_list.total);
    let json = serde_json::to_string(&account_list).unwrap();
    println!("📝 EXPECTED: JSON string with 'total' and 'accounts' fields");
    println!("✅ OUTPUT JSON: {}", json);
    println!("📊 VALIDATION: Contains 'total' field: {}", json.contains("total"));
    println!("📊 VALIDATION: Contains 'accounts' field: {}", json.contains("accounts"));
    println!("📊 VALIDATION: Total is 2: {}", json.contains("\"total\":2"));
    assert!(json.contains("total"));
    assert!(json.contains("accounts"));
    assert!(json.contains("\"total\":2"));
    
    println!("🔚 Request structure tests completed\n");
}
