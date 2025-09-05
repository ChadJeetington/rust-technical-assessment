// Test cases for all functions

test process_blockchain_command {
  functions [ProcessBlockchainCommand]
  args {
    command "send 1 ETH to Bob"
  }
}

test handle_general_question {
  functions [HandleGeneralQuestion]
  args {
    input "What tools do you have access to?"
  }
}

test search_uniswap_docs {
  functions [SearchUniswapDocs]
  args {
    query "How does Uniswap V2 work?"
    limit 3
  }
}

test is_documentation_query {
  functions [IsDocumentationQuery]
  args {
    input "Explain how Uniswap slippage works"
  }
}

test create_semantic_chunks {
  functions [CreateSemanticChunks]
  args {
    content #"
      # Uniswap V2 Router

      The Uniswap V2 Router is the main contract for executing swaps.

      ```solidity
      function swapExactTokensForTokens(
        uint amountIn,
        uint amountOutMin,
        address[] calldata path,
        address to,
        uint deadline
      ) external returns (uint[] memory amounts);
      ```

      ## Usage Guide
      To perform a swap, you need to:
      1. Approve tokens
      2. Calculate amounts
      3. Execute swap
    "#
  }
}

test process_document {
  functions [ProcessDocument]
  args {
    content "# Example Document\nThis is a test document..."
    metadata {
      version "1.0.0"
      format "markdown"
      encoding "utf-8"
      tags ["test", "example"]
    }
  }
}