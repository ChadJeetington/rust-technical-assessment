use crate::agent::Agent;
use crate::rag::UniswapRagSystem;
use std::path::Path;

#[tokio::test]
async fn test_query_classification() {
    // Initialize test agent with RAG system
    let mut agent = Agent::new().await.expect("Failed to create agent");
    agent.initialize_rag_system(Some("../test_docs")).await.expect("Failed to initialize RAG");

    // Test cases for documentation queries
    let doc_queries = vec![
        // Conceptual understanding
        "How does the system handle slippage tolerance?",
        "Can you explain the relationship between pools and pairs?",
        "What is the purpose of the router contract?",
        
        // Technical documentation
        "Show me the documentation for exactInput",
        "What parameters does swapExactTokensForTokens accept?",
        "List all available configuration options for the pool",
        
        // Best practices
        "What's the recommended way to implement price feeds?",
        "Show me best practices for handling callbacks",
        "How should I structure my integration code?",
        
        // Troubleshooting
        "Why am I getting insufficient output amount?",
        "How do I fix the deadline exceeded error?",
        "What causes the K value to change?",
    ];

    // Test cases for operation queries
    let op_queries = vec![
        // Direct actions
        "Execute swap with 1000 USDC",
        "Deploy new pool contract",
        "Send 5 ETH to this address",
        
        // State queries
        "Get current pool reserves",
        "Check my token balance",
        "Fetch latest price",
        
        // Multi-step operations
        "Swap tokens through multiple pools",
        "Add liquidity to the ETH/USDC pool",
        "Bridge tokens to optimism",
    ];

    // Test documentation queries
    for query in doc_queries {
        let result = agent.is_documentation_query(query).await.expect("Classification failed");
        assert!(result, "Failed to classify documentation query: {}", query);
    }

    // Test operation queries
    for query in op_queries {
        let result = agent.is_documentation_query(query).await.expect("Classification failed");
        assert!(!result, "Incorrectly classified operation query as documentation: {}", query);
    }

    // Test edge cases and ambiguous queries
    let edge_cases = vec![
        // Mixed intent (should classify based on stronger signal)
        ("Show me how to execute a swap", true),  // Documentation-leaning
        ("Execute the function described in docs", false),  // Operation-leaning
        
        // Very short queries
        ("help", true),
        ("run", false),
        
        // Complex multi-intent queries
        ("I need to understand how pools work and then add liquidity", true),
        ("First deploy the contract then show me how to use it", false),
    ];

    for (query, expected) in edge_cases {
        let result = agent.is_documentation_query(query).await.expect("Classification failed");
        assert_eq!(result, expected, "Failed to correctly classify edge case: {}", query);
    }
}

#[tokio::test]
async fn test_query_preprocessing() {
    let agent = Agent::new().await.expect("Failed to create agent");

    // Test filler word removal
    assert_eq!(
        agent.preprocess_query("Could you please show me the documentation?"),
        "show me the docs?"
    );

    // Test normalization
    assert_eq!(
        agent.preprocess_query("I need examples of implementing functions"),
        "examples of implementation functions"
    );

    // Test whitespace handling
    assert_eq!(
        agent.preprocess_query("show   me   the    docs"),
        "show me the docs"
    );

    // Test case normalization
    assert_eq!(
        agent.preprocess_query("SHOW me THE docs"),
        "show me the docs"
    );

    // Test combined preprocessing
    assert_eq!(
        agent.preprocess_query("Could you please SHOW me some EXAMPLES of IMPLEMENTING functions?"),
        "show me some examples of implementation functions?"
    );
}
