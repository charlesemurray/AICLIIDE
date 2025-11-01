#!/bin/bash

# Quick test runner for Amazon Q CLI
# Usage: ./scripts/quick_test.sh [test_type]

set -e

PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$PROJECT_ROOT"

echo "🧪 Amazon Q CLI Test Runner"
echo "=========================="

# Function to run tests with timing
run_test() {
    local test_name="$1"
    local test_command="$2"
    
    echo "Running $test_name..."
    start_time=$(date +%s)
    
    if eval "$test_command"; then
        end_time=$(date +%s)
        duration=$((end_time - start_time))
        echo "✅ $test_name completed in ${duration}s"
        return 0
    else
        end_time=$(date +%s)
        duration=$((end_time - start_time))
        echo "❌ $test_name failed after ${duration}s"
        return 1
    fi
}

# Parse command line arguments
TEST_TYPE="${1:-all}"

case "$TEST_TYPE" in
    "unit")
        echo "Running unit tests only..."
        run_test "Unit Tests" "cargo test --lib --workspace --all-features"
        ;;
    "integration")
        echo "Running integration tests only..."
        run_test "Integration Tests" "cargo test --test '*' --workspace --all-features"
        ;;
    "docs")
        echo "Running documentation tests..."
        run_test "Documentation Tests" "cargo test --doc --workspace"
        ;;
    "lint")
        echo "Running linting checks..."
        run_test "Clippy" "cargo clippy --workspace --all-features -- -D warnings"
        run_test "Format Check" "cargo +nightly fmt --check"
        ;;
    "coverage")
        echo "Running tests with coverage..."
        if ! command -v cargo-tarpaulin &> /dev/null; then
            echo "Installing cargo-tarpaulin..."
            cargo install cargo-tarpaulin
        fi
        run_test "Coverage Analysis" "cargo tarpaulin --workspace --all-features --out Stdout"
        ;;
    "all"|*)
        echo "Running all tests and checks..."
        
        # Unit tests
        run_test "Unit Tests" "cargo test --lib --workspace --all-features" || exit 1
        
        # Integration tests
        run_test "Integration Tests" "cargo test --test '*' --workspace --all-features" || exit 1
        
        # Documentation tests
        run_test "Documentation Tests" "cargo test --doc --workspace" || exit 1
        
        # Linting
        run_test "Clippy" "cargo clippy --workspace --all-features -- -D warnings" || exit 1
        run_test "Format Check" "cargo +nightly fmt --check" || exit 1
        
        echo ""
        echo "🎉 All tests and checks passed!"
        ;;
esac

echo ""
echo "Test run completed at $(date)"
