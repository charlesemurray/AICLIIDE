#!/bin/bash

# User Acceptance Test Runner for Skill Creation Assistant

set -e

echo "Running User Acceptance Tests for Skill Creation Assistant..."

# Run UAT tests
cargo test --test user_acceptance -- --nocapture

echo "UAT Results Summary:"
echo "- UAT-001: Developer creates first command skill"
echo "- UAT-002: Data scientist creates Python REPL"  
echo "- UAT-003: Technical writer creates documentation assistant"
echo "- UAT-004: DevOps engineer creates infrastructure template"
echo "- UAT-005: User handles creation errors gracefully"

echo "All User Acceptance Tests completed successfully!"
