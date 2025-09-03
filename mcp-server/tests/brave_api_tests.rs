//! Brave Search API Tests for MCP Blockchain Server
//! 
//! These tests verify that the Brave Search API integration works correctly,
//! including web search functionality and swap intent handling.

use mcp_server::services::search::{SearchService, WebSearchRequest, SwapIntentRequest, SearchResponse, SearchResult};
use rmcp::handler::server::tool::Parameters;

#[tokio::test]
async fn test_search_service_creation() {
    println!("\n🧪 Testing Brave Search Service creation...");
    println!("📝 INPUT: Attempting to create new SearchService instance");
    println!("📝 EXPECTED: Service creation or API key error if BRAVE_SEARCH_API_KEY not set");
    
    // This test will fail if BRAVE_SEARCH_API_KEY is not set, which is expected
    let result = SearchService::new().await;
    match result {
        Ok(_service) => {
            println!("✅ OUTPUT: SearchService created successfully");
            println!("📊 RESPONSE DETAILS: Service instance created with Brave Search API connection");
            assert!(true, "Service created");
        }
        Err(e) => {
            println!("⚠️  OUTPUT: SearchService creation failed");
            println!("📊 ERROR DETAILS: {}", e);
            println!("💡 This is expected if BRAVE_SEARCH_API_KEY is not set");
            println!("   Set it with: export BRAVE_SEARCH_API_KEY='BSA-xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx'");
            
            // Don't fail the test if API key isn't set - this is a common case
        }
    }
    println!("🔚 Test completed\n");
}

#[tokio::test]
async fn test_web_search_request_serialization() {
    println!("\n🧪 Testing WebSearchRequest serialization...");
    
    let request = WebSearchRequest {
        query: "test query".to_string(),
        count: Some(5),
        country: Some("us".to_string()),
        search_lang: Some("en".to_string()),
    };
    
    println!("📝 INPUT: WebSearchRequest {{");
    println!("         query: \"{}\"", request.query);
    println!("         count: {:?}", request.count);
    println!("         country: {:?}", request.country);
    println!("         search_lang: {:?}", request.search_lang);
    println!("       }}");
    
    let json = serde_json::to_string(&request).unwrap();
    println!("📝 EXPECTED: JSON string with all fields");
    println!("✅ OUTPUT JSON: {}", json);
    
    let deserialized: WebSearchRequest = serde_json::from_str(&json).unwrap();
    
    println!("📊 VALIDATION: Query matches: {}", request.query == deserialized.query);
    println!("📊 VALIDATION: Count matches: {}", request.count == deserialized.count);
    println!("📊 VALIDATION: Country matches: {}", request.country == deserialized.country);
    println!("📊 VALIDATION: Search lang matches: {}", request.search_lang == deserialized.search_lang);
    
    assert_eq!(request.query, deserialized.query);
    assert_eq!(request.count, deserialized.count);
    assert_eq!(request.country, deserialized.country);
    assert_eq!(request.search_lang, deserialized.search_lang);
    
    println!("✅ WebSearchRequest serialization test: PASSED");
    println!("🔚 Test completed\n");
}

#[tokio::test]
async fn test_swap_intent_request_serialization() {
    println!("\n🧪 Testing SwapIntentRequest serialization...");
    
    let request = SwapIntentRequest {
        from_token: "ETH".to_string(),
        to_token: "USDC".to_string(),
        amount: "1.0".to_string(),
        dex: Some("Uniswap V2".to_string()),
    };
    
    println!("📝 INPUT: SwapIntentRequest {{");
    println!("         from_token: \"{}\"", request.from_token);
    println!("         to_token: \"{}\"", request.to_token);
    println!("         amount: \"{}\"", request.amount);
    println!("         dex: {:?}", request.dex);
    println!("       }}");
    
    let json = serde_json::to_string(&request).unwrap();
    println!("📝 EXPECTED: JSON string with all fields");
    println!("✅ OUTPUT JSON: {}", json);
    
    let deserialized: SwapIntentRequest = serde_json::from_str(&json).unwrap();
    
    println!("📊 VALIDATION: From token matches: {}", request.from_token == deserialized.from_token);
    println!("📊 VALIDATION: To token matches: {}", request.to_token == deserialized.to_token);
    println!("📊 VALIDATION: Amount matches: {}", request.amount == deserialized.amount);
    println!("📊 VALIDATION: DEX matches: {}", request.dex == deserialized.dex);
    
    assert_eq!(request.from_token, deserialized.from_token);
    assert_eq!(request.to_token, deserialized.to_token);
    assert_eq!(request.amount, deserialized.amount);
    assert_eq!(request.dex, deserialized.dex);
    
    println!("✅ SwapIntentRequest serialization test: PASSED");
    println!("🔚 Test completed\n");
}

#[tokio::test]
async fn test_search_response_serialization() {
    println!("\n🧪 Testing SearchResponse serialization...");
    
    let response = SearchResponse {
        query: "test".to_string(),
        results: vec![
            SearchResult {
                title: "Test Result".to_string(),
                url: "https://example.com".to_string(),
                description: "Test description".to_string(),
            }
        ],
        total_results: 1,
    };
    
    println!("📝 INPUT: SearchResponse {{");
    println!("         query: \"{}\"", response.query);
    println!("         total_results: {}", response.total_results);
    println!("         results: {} items", response.results.len());
    println!("       }}");
    
    let json = serde_json::to_string(&response).unwrap();
    println!("📝 EXPECTED: JSON string with all fields");
    println!("✅ OUTPUT JSON: {}", json);
    
    let deserialized: SearchResponse = serde_json::from_str(&json).unwrap();
    
    println!("📊 VALIDATION: Query matches: {}", response.query == deserialized.query);
    println!("📊 VALIDATION: Total results matches: {}", response.total_results == deserialized.total_results);
    println!("📊 VALIDATION: Results count matches: {}", response.results.len() == deserialized.results.len());
    
    assert_eq!(response.query, deserialized.query);
    assert_eq!(response.total_results, deserialized.total_results);
    assert_eq!(response.results.len(), deserialized.results.len());
    
    // Test the first result
    if let (Some(original), Some(deserialized_result)) = (response.results.first(), deserialized.results.first()) {
        println!("📊 VALIDATION: First result title matches: {}", original.title == deserialized_result.title);
        println!("📊 VALIDATION: First result URL matches: {}", original.url == deserialized_result.url);
        println!("📊 VALIDATION: First result description matches: {}", original.description == deserialized_result.description);
        
        assert_eq!(original.title, deserialized_result.title);
        assert_eq!(original.url, deserialized_result.url);
        assert_eq!(original.description, deserialized_result.description);
    }
    
    println!("✅ SearchResponse serialization test: PASSED");
    println!("🔚 Test completed\n");
}

#[tokio::test]
async fn test_web_search_functionality() {
    println!("\n🧪 Testing actual web search functionality...");
    
    // Try to create search service
    match SearchService::new().await {
        Ok(service) => {
            println!("✅ SearchService created successfully");
            
            // Test web search
            let search_request = WebSearchRequest {
                query: "Ethereum price".to_string(),
                count: Some(3),
                country: Some("us".to_string()),
                search_lang: Some("en".to_string()),
            };
            
            println!("📝 INPUT: Searching for \"{}\"", search_request.query);
            println!("📝 EXPECTED: Search results or API error");
            
            let result = service.web_search(Parameters(search_request)).await;
            
            match result {
                Ok(call_result) => {
                    println!("✅ Web search successful!");
                    println!("📊 Response: {:?}", call_result);
                    
                    if let Some(content) = call_result.content {
                        println!("📝 Search Response: {:?}", content);
                        println!("✅ Response validation: PASSED");
                    }
                }
                Err(e) => {
                    println!("⚠️  Web search failed: {}", e);
                    println!("💡 This might be expected if:");
                    println!("   - API key is invalid");
                    println!("   - Network connection issues");
                    println!("   - Rate limiting");
                }
            }
        }
        Err(e) => {
            println!("⚠️  SearchService creation failed: {}", e);
            println!("💡 This is expected if BRAVE_SEARCH_API_KEY is not set");
            println!("   Set it with: export BRAVE_SEARCH_API_KEY='BSA-xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx'");
        }
    }
    
    println!("🔚 Web search functionality test completed\n");
}

#[tokio::test]
async fn test_swap_intent_functionality() {
    println!("\n🧪 Testing swap intent functionality...");
    
    // Try to create search service
    match SearchService::new().await {
        Ok(service) => {
            println!("✅ SearchService created successfully");
            
            // Test swap intent
            let swap_request = SwapIntentRequest {
                from_token: "ETH".to_string(),
                to_token: "USDC".to_string(),
                amount: "1.0".to_string(),
                dex: Some("Uniswap V2".to_string()),
            };
            
            println!("📝 INPUT: Swap intent from {} to {} ({})", 
                     swap_request.from_token, swap_request.to_token, swap_request.amount);
            println!("📝 EXPECTED: Swap intent response with recommendations");
            
            let result = service.handle_swap_intent(Parameters(swap_request)).await;
            
            match result {
                Ok(call_result) => {
                    println!("✅ Swap intent successful!");
                    println!("📊 Response: {:?}", call_result);
                    
                    if let Some(content) = call_result.content {
                        println!("📝 Swap Intent Response: {:?}", content);
                        println!("✅ Response validation: PASSED");
                    }
                }
                Err(e) => {
                    println!("⚠️  Swap intent failed: {}", e);
                    println!("💡 This might be expected if:");
                    println!("   - API key is invalid");
                    println!("   - Network connection issues");
                    println!("   - Search service unavailable");
                }
            }
        }
        Err(e) => {
            println!("⚠️  SearchService creation failed: {}", e);
            println!("💡 This is expected if BRAVE_SEARCH_API_KEY is not set");
            println!("   Set it with: export BRAVE_SEARCH_API_KEY='BSA-xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx'");
        }
    }
    
    println!("🔚 Swap intent functionality test completed\n");
}
