# Testing Guide

This document describes the testing strategy and organization for the Rust Technical Assessment project.

## Test Organization

### Unit Tests
- **Location**: Inline with source code using `#[cfg(test)]` modules
- **Purpose**: Test individual functions and modules
- **Running**: `cargo test`

### Integration Tests
- **Location**: `scripts/tests/` (Rust files) and `tests/integration/` (if any)
- **Purpose**: Test complete workflows and cross-module interactions
- **Running**: `cargo run --bin test_name --manifest-path scripts/tests/Cargo.toml`

### Shell Script Tests
- **Location**: `scripts/tests/`
- **Purpose**: End-to-end testing, environment setup, and external tool integration
- **Running**: Execute individual scripts or run `./scripts/tests/run_all.sh`

## Running Tests

### Rust Tests
```bash
# Run all Rust tests
cargo test

# Run specific test
cargo test test_name

# Run with output
cargo test -- --nocapture

# Run integration tests only
cargo run --bin test_usdc_direct --manifest-path scripts/tests/Cargo.toml
```

### Shell Script Tests
```bash
# Run individual script tests
./scripts/tests/test_default_addresses.sh
./scripts/tests/test_formatting.sh
./scripts/tests/test_complete_system.sh
./scripts/tests/test_rig_client.sh
./scripts/tests/test_tools.sh
```

## Test Categories

### 1. Unit Tests (`src/` modules)
- Fast, isolated tests
- Test individual functions and data structures
- Use `#[cfg(test)]` modules within source files

### 2. Integration Tests (`scripts/tests/` - Rust files)
- Test complete workflows
- Verify module interactions
- Test external service integrations
- Standalone Rust executables for complex testing scenarios

### 3. End-to-End Tests (`scripts/tests/`)
- Full system testing
- Environment validation
- External tool integration
- Performance and stress testing

## Best Practices

1. **Keep tests close to code**: Unit tests should be in the same file as the code they test
2. **Use descriptive test names**: Test names should clearly describe what they're testing
3. **Test both success and failure cases**: Ensure error handling is properly tested
4. **Use fixtures for complex data**: Store test data in `tests/fixtures/`
5. **Clean up after tests**: Ensure tests don't leave side effects
6. **Run tests before committing**: Always run the full test suite before pushing changes

## CI/CD Integration

Tests are automatically run in CI/CD pipelines:
- Unit and integration tests run on every commit
- Shell script tests run on pull requests
- Performance tests run on release candidates

## Troubleshooting

### Common Issues
1. **Test logs**: Check `target/test-output/` for detailed logs
2. **Environment issues**: Ensure all dependencies are installed
3. **Permission issues**: Make shell scripts executable with `chmod +x`

### Debugging Tests
```bash
# Run with verbose output
cargo test -- --nocapture --test-threads=1

# Run specific test with debug output
RUST_LOG=debug cargo test test_name
```
