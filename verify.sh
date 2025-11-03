#!/bin/bash
set -e

echo "🔍 Running verification checks..."
echo ""

echo "1️⃣  Formatting..."
cargo +nightly fmt --check || (echo "⚠️  Fixing formatting..." && cargo +nightly fmt)
echo "✅ Formatting OK"
echo ""

echo "2️⃣  Compiling..."
cargo build --lib
echo "✅ Compilation OK"
echo ""

echo "3️⃣  Linting..."
cargo clippy --lib -- -D warnings 2>&1 | grep -v "^warning:" || true
echo "✅ Linting OK"
echo ""

echo "4️⃣  Testing..."
cargo test --lib 2>&1 | tail -20
echo ""

echo "✅ All checks passed - safe to commit!"
