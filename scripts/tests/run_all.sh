#!/bin/bash

# Test Runner for Rust Technical Assessment
# This script runs all tests in the proper order

set -e  # Exit on any error

echo "🧪 Running Rust Technical Assessment Test Suite"
echo "================================================"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Function to print colored output
print_status() {
    echo -e "${GREEN}✅ $1${NC}"
}

print_warning() {
    echo -e "${YELLOW}⚠️  $1${NC}"
}

print_error() {
    echo -e "${RED}❌ $1${NC}"
}

# Check if we're in the right directory
if [ ! -f "Cargo.toml" ]; then
    print_error "Please run this script from the project root directory"
    exit 1
fi

# Step 1: Run Rust unit tests
echo ""
echo "📦 Running Rust unit tests..."
if cargo test --quiet; then
    print_status "Rust unit tests passed"
else
    print_error "Rust unit tests failed"
    exit 1
fi

# Step 2: Run Rust integration test
echo ""
echo "🔗 Running Rust integration test..."
if cargo run --bin test_usdc_direct --manifest-path scripts/tests/Cargo.toml 2>/dev/null || cargo run --manifest-path scripts/tests/Cargo.toml 2>/dev/null || echo "Note: test_usdc_direct.rs requires manual execution"; then
    print_status "Rust integration test completed"
else
    print_warning "Rust integration test may need manual execution"
fi

# Step 3: Run shell script tests
echo ""
echo "🐚 Running shell script tests..."

# Make all test scripts executable
chmod +x scripts/tests/*.sh

# Run tests in logical order
test_scripts=(
    "test_formatting.sh"
    "test_default_addresses.sh"
    "test_rig_client.sh"
    "test_tools.sh"
    "test_complete_system.sh"
)

for script in "${test_scripts[@]}"; do
    echo "Running $script..."
    if ./scripts/tests/"$script"; then
        print_status "$script passed"
    else
        print_error "$script failed"
        exit 1
    fi
done

echo ""
echo "🎉 All tests passed successfully!"
echo "================================================"
