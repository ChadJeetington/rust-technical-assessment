// Document ingestion and processing system

// Create semantic chunks from content
function CreateSemanticChunks(content: string) -> ChunkResult {
  client CustomHaiku
  prompt #"
    Create semantic chunks from this content:
    {{ content }}

    Extract:
    1. Semantic chunks (sections of related content)
    2. Code examples (Solidity code blocks)
    3. Function signatures

    {{ ctx.output_format }}
  "#
}

// Process and validate document
function ProcessDocument(content: string, metadata: SourceMetadata) -> ProcessingResult {
  client CustomHaiku
  prompt #"
    Process and validate this document:
    {{ content }}

    Document metadata:
    {{ metadata | json }}

    Validation rules:
    1. Check for complete code blocks
    2. Validate function signatures
    3. Ensure proper formatting
    4. Extract semantic sections

    {{ ctx.output_format }}
  "#
}