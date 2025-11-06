#!/bin/bash
# Standalone test for workflow validation

set -e

echo "=== Workflow Validation Tests ==="
echo

# Test 1: Valid workflow should pass
echo "Test 1: Valid workflow validation..."
cat > /tmp/test-valid-workflow.json << 'EOF'
{
  "name": "test-valid",
  "version": "1.0.0",
  "description": "Valid test workflow",
  "steps": [
    {
      "name": "step1",
      "tool": "calculator",
      "parameters": {"operation": "add", "a": 1, "b": 2}
    }
  ]
}
EOF

if cargo run --bin chat_cli -- workflows add /tmp/test-valid-workflow.json 2>&1 | grep -q "added successfully"; then
    echo "✅ PASS: Valid workflow accepted"
else
    echo "❌ FAIL: Valid workflow rejected"
    exit 1
fi

# Test 2: Cycle detection
echo
echo "Test 2: Cycle detection..."
cat > /tmp/test-cycle.json << 'EOF'
{
  "name": "test-cycle",
  "version": "1.0.0",
  "description": "Workflow with cycle",
  "steps": [
    {
      "name": "step1",
      "tool": "step1",
      "parameters": {}
    }
  ]
}
EOF

if cargo run --bin chat_cli -- workflows add /tmp/test-cycle.json 2>&1 | grep -q "cycle"; then
    echo "✅ PASS: Cycle detected and rejected"
else
    echo "❌ FAIL: Cycle not detected"
    exit 1
fi

# Test 3: Empty workflow
echo
echo "Test 3: Empty workflow validation..."
cat > /tmp/test-empty.json << 'EOF'
{
  "name": "test-empty",
  "version": "1.0.0",
  "description": "Empty workflow",
  "steps": []
}
EOF

if cargo run --bin chat_cli -- workflows add /tmp/test-empty.json 2>&1 | grep -q "at least one step"; then
    echo "✅ PASS: Empty workflow rejected"
else
    echo "❌ FAIL: Empty workflow not rejected"
    exit 1
fi

# Test 4: Invalid name
echo
echo "Test 4: Invalid name validation..."
cat > /tmp/test-invalid-name.json << 'EOF'
{
  "name": "invalid name with spaces",
  "version": "1.0.0",
  "description": "Invalid name",
  "steps": [
    {
      "name": "step1",
      "tool": "echo",
      "parameters": {}
    }
  ]
}
EOF

if cargo run --bin chat_cli -- workflows add /tmp/test-invalid-name.json 2>&1 | grep -q "alphanumeric"; then
    echo "✅ PASS: Invalid name rejected"
else
    echo "❌ FAIL: Invalid name not rejected"
    exit 1
fi

echo
echo "=== All validation tests passed! ==="
