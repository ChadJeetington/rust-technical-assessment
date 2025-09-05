// Blockchain Agent - Integrates Claude AI with MCP server for blockchain operations

// Process blockchain command
function ProcessBlockchainCommand(command: string) -> BlockchainResponse {
  client CustomHaiku
  prompt #"
    Process this blockchain command:
    {{ command }}

    Available operations:
    - Send ETH between addresses
    - Check token balances
    - Verify contract deployment
    - Get account information

    Default addresses:
    - Alice (Account 0) is the default sender
    - Bob (Account 1) is the default recipient

    {{ ctx.output_format }}
  "#
}

// Handle general questions without tool calling
function HandleGeneralQuestion(input: string) -> GeneralResponse {
  client CustomHaiku
  prompt #"
    Answer this general question about the blockchain agent:
    {{ input }}

    Available capabilities:
    - Blockchain operations (ETH transfers, balance checks)
    - Documentation search (Uniswap docs and contracts)
    - Web search for current information
    - Natural language processing

    Key features:
    - Default accounts (Alice and Bob)
    - RAG system for Uniswap
    - MCP tool integration
    - Local embeddings

    {{ ctx.output_format }}
  "#
}