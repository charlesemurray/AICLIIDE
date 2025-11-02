#!/bin/bash

# Test script for auto-approve functionality
echo "Testing Q CLI Auto-Approve System"
echo "=================================="

# Build the project first
echo "Building Q CLI..."
cargo build --bin chat_cli --release

if [ $? -ne 0 ]; then
    echo "❌ Build failed"
    exit 1
fi

echo "✅ Build successful"
echo ""

# Test 1: Help text shows new options
echo "Test 1: Checking help text includes new options"
./target/release/chat_cli --help | grep -E "(auto-approve|batch-mode)"

if [ $? -eq 0 ]; then
    echo "✅ Help text includes new options"
else
    echo "❌ Help text missing new options"
fi

echo ""

# Test 2: CLI accepts new arguments
echo "Test 2: Testing CLI argument parsing"
echo "Testing --auto-approve 5..."
./target/release/chat_cli --auto-approve 5 --no-interactive "echo test" 2>/dev/null

if [ $? -eq 0 ]; then
    echo "✅ --auto-approve argument accepted"
else
    echo "❌ --auto-approve argument failed"
fi

echo "Testing --batch-mode..."
./target/release/chat_cli --batch-mode --no-interactive "echo test" 2>/dev/null

if [ $? -eq 0 ]; then
    echo "✅ --batch-mode argument accepted"
else
    echo "❌ --batch-mode argument failed"
fi

echo ""
echo "🎉 Auto-approve system ready for testing!"
echo ""
echo "Usage examples:"
echo "  q chat --auto-approve 10 'create 10 files'"
echo "  q chat --batch-mode 'implement the plan'"
echo "  q chat 'start work' # then type 'auto 5' or 'batch'"
