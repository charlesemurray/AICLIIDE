# Development Guide

This guide covers how to develop and test the Amazon Q CLI locally using cargo.

## Prerequisites

- Rust toolchain (install via [rustup](https://rustup.rs/))
- Git

## Setup

1. Clone the repository:
```bash
git clone https://github.com/aws/amazon-q-developer-cli.git
cd amazon-q-developer-cli
```

2. Install Rust toolchain:
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup default stable
rustup toolchain install nightly
cargo install typos-cli
```

3. Set up cargo environment:
```bash
source ~/.cargo/env  # Add cargo to PATH
```

## Running Q CLI with Cargo

### Basic Usage
```bash
# Run Q CLI directly
cargo run --bin chat_cli

# Run with arguments (note the -- separator)
cargo run --bin chat_cli -- --help
cargo run --bin chat_cli -- --version
```

### Chat Interface
```bash
# Start interactive chat
cargo run --bin chat_cli -- chat

# Non-interactive chat
echo "Hello" | cargo run --bin chat_cli -- chat
```

### Skills System (Development)
```bash
# List available skills
cargo run --bin chat_cli -- skills list

# Create a new skill
cargo run --bin chat_cli -- skills create my-skill

# Run a skill
cargo run --bin chat_cli -- skills run calculator --params '{"a": 2, "b": 3, "op": "add"}'

# Get skill info
cargo run --bin chat_cli -- skills info calculator
```

### Other Commands
```bash
# Settings
cargo run --bin chat_cli -- settings list

# Version info
cargo run --bin chat_cli -- --version

# Help
cargo run --bin chat_cli -- --help
```

## Development Workflow

### 1. Make Changes
Edit source code in `crates/chat-cli/src/`

### 2. Test Changes
```bash
# Quick test - run the binary
cargo run --bin chat_cli -- --help

# Run unit tests
cargo test

# Run specific tests
cargo test skills::unit_tests
```

### 3. Build and Test
```bash
# Debug build (faster compilation)
cargo build --bin chat_cli

# Release build (optimized)
cargo build --bin chat_cli --release

# Test the built binary
./target/debug/chat_cli --help
./target/release/chat_cli --help
```

## Testing

### Unit Tests
```bash
cargo test                           # All tests
cargo test skills                    # Skills-related tests
cargo test skills::unit_tests        # Fast unit tests only
```

### Integration Tests
```bash
cargo test --test integration_tests
```

### Performance Testing
```bash
# Release build for performance testing
cargo build --bin chat_cli --release
time ./target/release/chat_cli skills list
```

## Debugging

### Debug Mode
```bash
# Run with debug output
RUST_LOG=debug cargo run --bin chat_cli -- skills list

# Run with trace output
RUST_LOG=trace cargo run --bin chat_cli -- skills list
```

### Using Built Binary
```bash
# Build once, run multiple times (faster for testing)
cargo build --bin chat_cli
./target/debug/chat_cli skills create test-skill
./target/debug/chat_cli skills list
./target/debug/chat_cli skills run test-skill
```

## Common Issues

### Cargo Not Found
```bash
# Add cargo to PATH
source ~/.cargo/env
```

### Compilation Errors
```bash
# Clean and rebuild
cargo clean
cargo build --bin chat_cli
```

### Skills Not Loading
```bash
# Check current directory has .rs files
ls *.rs

# Run with debug output
RUST_LOG=debug cargo run --bin chat_cli -- skills list
```

## Project Structure

- `crates/chat-cli/` - Main CLI application
- `crates/chat-cli/src/cli/skills/` - Skills system
- `crates/agent/` - Agent system  
- `scripts/` - Build and deployment scripts
- `docs/` - Technical documentation

## Comparison: Development vs Released CLI

| Command | Development (cargo) | Released CLI |
|---------|-------------------|--------------|
| Basic usage | `cargo run --bin chat_cli` | `q` |
| Chat | `cargo run --bin chat_cli -- chat` | `q chat` |
| Skills | `cargo run --bin chat_cli -- skills list` | Not available yet |
| Help | `cargo run --bin chat_cli -- --help` | `q --help` |

**Note**: The skills system is only available in the development version built with cargo, not in the released Q CLI yet.
