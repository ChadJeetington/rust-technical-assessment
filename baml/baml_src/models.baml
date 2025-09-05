// Shared data models for the blockchain agent

// Intent cluster for query classification
class IntentCluster {
  name string
  examples string[]
}

// Response type for blockchain operations
class BlockchainResponse {
  success bool
  response string
  error string?
  transaction_hash string?
}

// Response type for general questions
class GeneralResponse {
  response string
  category string
}

// Document structure for storing Uniswap documentation
class UniswapDocument {
  id string
  title string
  doc_type DocumentType
  content string
  semantic_chunks string[]
  code_examples string[]
  function_signatures string[]
  metadata DocumentMetadata
  parent_id string?
  chunk_type ChunkType?
}

// Types of documents
enum DocumentType {
  Documentation
  ContractCode
  Interface
  Guide
  Example
  FAQ
  Tutorial
  Reference
  Explanation
}

// Types of semantic chunks
enum ChunkType {
  FunctionDoc
  Example
  Interface
  Concept
  Usage
  Error
  BestPractice
  Parameter
  ReturnValue
  Security
}

// Document metadata
class DocumentMetadata {
  source_path string?
  version string?
  tags string[]
  created_at string
  updated_at string
  source_repo string?
  source_commit string?
  related_docs string[]
  dependencies string[]
  status DocumentStatus
  authors string[]
  reviewers string[]
  version_requirements string?
}

// Document status
enum DocumentStatus {
  Draft
  InReview
  Published
  Archived
  Deprecated
}

// Search result type
class SearchResult {
  documents UniswapDocument[]
  scores float[]
}

// Query analysis result
class QueryAnalysis {
  is_doc_query bool
  confidence float
  intent string
}

// Document source information
class DocumentSource {
  location string
  doc_type string
  metadata SourceMetadata
}

// Source metadata
class SourceMetadata {
  version string?
  last_updated string?
  format string
  encoding string
  tags string[]
}

// Chunk processing result
class ChunkResult {
  chunks string[]
  examples string[]
  signatures string[]
}

// Document processing result
class ProcessingResult {
  success bool
  processed_content string
  validation_errors string[]
  extracted_metadata DocumentMetadata
}