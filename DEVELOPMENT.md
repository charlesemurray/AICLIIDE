# Development Guide

## Project Structure

### Crate Organization
```
crates/
├── chat-cli/              # Main CLI application
│   └── src/
│       ├── cli/           # CLI commands and subcommands
│       │   ├── chat/      # Chat command implementation
│       │   ├── agent/     # Agent command implementation
│       │   └── mcp.rs     # MCP command
│       ├── auth/          # Authentication (Builder ID, IAM Identity Center)
│       ├── database/      # SQLite database and settings
│       ├── api_client/    # API client for Q Developer services
│       ├── util/          # Utility functions
│       ├── os/            # OS-specific functionality
│       ├── telemetry/     # Telemetry and metrics
│       ├── theme/         # UI theming
│       └── mcp_client/    # Model Context Protocol client
├── agent/                 # Agent framework
├── chat-cli-ui/          # UI components
├── semantic-search-client/ # Semantic search functionality
└── amzn-*-client/        # Generated API clients
```

### Module Organization Patterns

**Simple features:** Single file in appropriate directory
```rust
// crates/chat-cli/src/util/my_feature.rs
pub fn my_function() -> Result<()> {
    // implementation
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_my_function() {
        // test implementation
    }
}
```

**Complex features:** Directory with mod.rs + submodules
```
cli/my_feature/
├── mod.rs           # Public API and main logic
├── parser.rs        # Parsing logic
└── handler.rs       # Handler implementation
```

## Building

### Build the CLI
```bash
cargo build --bin chat_cli
```

### Build with optimizations (release)
```bash
cargo build --bin chat_cli --release
```

### Run without building
```bash
cargo run --bin chat_cli
```

### Run with subcommand
```bash
cargo run --bin chat_cli -- <subcommand>
# Examples:
cargo run --bin chat_cli -- login
cargo run --bin chat_cli -- chat "hello"
cargo run --bin chat_cli -- --help
```

## Testing

### Run all tests
```bash
cargo test
```

### Run tests for specific crate
```bash
cargo test -p chat_cli
```

### Run specific test
```bash
cargo test test_name
```

### Run tests with output
```bash
cargo test -- --nocapture
```

### Run ignored tests (require auth/CI)
```bash
cargo test -- --ignored
```

## Testing Conventions

### Inline Tests
Tests are co-located with implementation using `#[cfg(test)]`:

```rust
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add() {
        assert_eq!(add(2, 2), 4);
    }
}
```

### Snapshot Testing
Uses `insta` crate for snapshot tests:

```rust
#[test]
fn test_output_format() {
    let output = format_output("test");
    insta::assert_snapshot!(output, @"expected output");
}
```

### Async Tests
```rust
#[tokio::test]
async fn test_async_function() {
    let result = async_function().await;
    assert!(result.is_ok());
}
```

### Test Utilities
- Mock database: `Database::new().await` (returns test DB in test mode)
- Test tokens: `BuilderIdToken::test()` (available in test builds)
- Test context: Use `cfg!(test)` for test-specific behavior

## Code Quality

### Format code
```bash
cargo +nightly fmt
```

### Run lints
```bash
cargo clippy
```

### Check for typos
```bash
typos
```

### Run all checks
```bash
cargo +nightly fmt && cargo clippy && cargo test
```

## Adding New Features

### 1. Determine Location
- **CLI command:** `crates/chat-cli/src/cli/`
- **Utility function:** `crates/chat-cli/src/util/`
- **API integration:** `crates/chat-cli/src/api_client/`
- **Database feature:** `crates/chat-cli/src/database/`

### 2. Write Tests First (TDD)
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_feature() {
        // Arrange
        let input = "test";
        
        // Act
        let result = new_feature(input);
        
        // Assert
        assert_eq!(result, expected);
    }
}
```

### 3. Implement Feature
Write minimal code to pass tests.

### 4. Add to Module Tree
Update `mod.rs` to expose new module:
```rust
pub mod my_feature;
```

### 5. Integrate with CLI (if applicable)
Add to command structure in `cli/mod.rs` or appropriate subcommand.

## Authentication

### Internal Amazon Users
- Start URL: `https://amzn.awsapps.com/start`
- Supports Midway for private specs
- Check with: `BuilderIdToken::is_amzn_user()`

### Public Users
- Start URL: `https://view.awsapps.com/start`
- Builder ID authentication

## Database

### Migrations
Located in: `crates/chat-cli/src/database/sqlite_migrations/`

Format: `NNN_description.sql`

### Adding Migration
1. Create new file with next number
2. Write SQL migration
3. Migrations run automatically on startup

## Common Patterns

### Error Handling
```rust
use eyre::Result;

pub fn my_function() -> Result<String> {
    Ok("success".to_string())
}
```

### Async Functions
```rust
pub async fn fetch_data(client: &Client) -> Result<Data> {
    let response = client.get().await?;
    Ok(response)
}
```

### Configuration
Settings stored in database via `Database::settings()`:
```rust
let settings = database.settings();
let value = settings.get("key").await?;
```

## Debugging

### Enable verbose logging
```bash
cargo run --bin chat_cli -- -vvv chat "test"
```

### Check logs location
Logs are written to system temp directory and displayed with verbose flags.

## Resources

- Main README: [README.md](README.md)
- Contributing: [CONTRIBUTING.md](CONTRIBUTING.md)
- Security: [SECURITY.md](SECURITY.md)
