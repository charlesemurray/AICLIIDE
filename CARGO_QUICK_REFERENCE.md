# Cargo Quick Reference for Q CLI Development

## Essential Commands

### Setup
```bash
source ~/.cargo/env                    # Add cargo to PATH
```

### Basic Development
```bash
cargo run --bin chat_cli               # Run Q CLI
cargo run --bin chat_cli -- --help     # Show help
cargo test                             # Run all tests
cargo test skills::unit_tests          # Run fast unit tests
```

### Skills Development
```bash
cargo run --bin chat_cli -- skills list                           # List skills
cargo run --bin chat_cli -- skills create my-skill                # Create skill
cargo run --bin chat_cli -- skills run calculator --params '{...}' # Run skill
```

### Build & Test
```bash
cargo build --bin chat_cli              # Debug build
cargo build --bin chat_cli --release    # Release build
./target/debug/chat_cli skills list     # Test built binary
```

### Debugging
```bash
RUST_LOG=debug cargo run --bin chat_cli -- skills list   # Debug output
cargo clean && cargo build --bin chat_cli                # Clean rebuild
```

## Key Differences from Released CLI

| Task | Development | Released |
|------|-------------|----------|
| Run CLI | `cargo run --bin chat_cli` | `q` |
| Skills | `cargo run --bin chat_cli -- skills list` | Not available |
| Chat | `cargo run --bin chat_cli -- chat` | `q chat` |

## File Locations
- Source: `crates/chat-cli/src/`
- Binary: `target/debug/chat_cli` or `target/release/chat_cli`
- Tests: `crates/chat-cli/src/cli/skills/tests/`
