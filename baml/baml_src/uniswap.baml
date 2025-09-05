// Uniswap documentation and RAG system

// Search Uniswap documentation
function SearchUniswapDocs(query: string, limit: int) -> SearchResult {
  client CustomHaiku
  prompt #"
    Search for relevant Uniswap documentation based on this query:
    {{ query }}

    Return up to {{ limit }} most relevant documents.

    {{ ctx.output_format }}
  "#
}

// Check if input is a documentation query
function IsDocumentationQuery(input: string) -> QueryAnalysis {
  client CustomHaiku
  prompt #"
    Analyze this input to determine if it's a documentation query:
    {{ input }}

    Documentation query patterns:
    - Questions about how things work
    - Requests for explanations
    - Technical documentation requests
    - Questions about functionality
    - Requests for examples or guides

    Operation patterns:
    - Direct action commands
    - Balance queries
    - Transaction requests
    - State checks

    {{ ctx.output_format }}
  "#
}