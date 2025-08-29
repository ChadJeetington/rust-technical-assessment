# RAG System Tests

This folder contains tests for the RAG (Retrieval-Augmented Generation) system implementation for Uniswap documentation.

## Test Files

### `rag_tests.rs`
Rust unit tests for the RAG system functionality. Tests include:

- **test_rag_system_initialization**: Verifies the RAG system can be initialized
- **test_sample_documentation_loading**: Tests loading of sample Uniswap documentation
- **test_document_search**: Tests document search functionality
- **test_document_structure**: Tests the UniswapDocument structure
- **test_rag_status**: Tests RAG system status reporting
- **test_specific_queries**: Tests the specific queries mentioned in the PRD

### `test_rag.sh`
Shell script for manual testing of the RAG system CLI commands.

## Running Tests

### Rust Tests
```bash
# Run all RAG tests
cargo test --test rag_tests

# Run specific test
cargo test --test rag_tests test_specific_queries

# Run tests with output
cargo test --test rag_tests -- --nocapture
```

### Manual Testing
```bash
# Make the script executable (if needed)
chmod +x tests/test_rag.sh

# Run the test script
./tests/test_rag.sh
```

## Test Coverage

The tests cover all the requirements from the PRD bonus section part 2:

✅ **Document Storage**: Tests document structure and metadata  
✅ **Vector Embeddings**: Tests embedding generation and storage  
✅ **Context Integration**: Tests search functionality  
✅ **Example Queries**: Tests the three specific queries from the PRD  
✅ **CLI Interface**: Tests command-line functionality  
✅ **Sample Documentation**: Tests loading and searching sample data  

## Expected Results

When running the tests, you should see:

1. **All Rust tests pass** (6/6 tests passing)
2. **Sample documentation loads successfully** (4+ documents)
3. **Search queries return relevant results** for:
   - Slippage calculation for Uniswap V3
   - Difference between exactInput and exactOutput
   - SwapRouter contract interface

## Manual Testing Commands

After starting the client with `cargo run --bin rig-client`, you can test:

```bash
# Check RAG system status
rag-status

# Search for slippage information
rag-search "How do I calculate slippage for Uniswap V3?"

# Search for function differences
rag-search "What's the difference between exactInput and exactOutput?"

# Search for contract interface
rag-search "Show me the SwapRouter contract interface"
```

The RAG system should provide detailed, accurate answers based on the sample Uniswap documentation.
