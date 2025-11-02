# Amazon Q CLI

## Installation

- **macOS**:
  - **DMG**: [Download now](https://desktop-release.q.us-east-1.amazonaws.com/latest/Amazon%20Q.dmg)
  - **HomeBrew**: ```brew install --cask amazon-q ```
- **Linux**:
  - [Ubuntu/Debian](https://docs.aws.amazon.com/amazonq/latest/qdeveloper-ug/command-line-installing.html#command-line-installing-ubuntu)
  - [AppImage](https://docs.aws.amazon.com/amazonq/latest/qdeveloper-ug/command-line-installing.html#command-line-installing-appimage)
  - [Alternative Linux builds](https://docs.aws.amazon.com/amazonq/latest/qdeveloper-ug/command-line-installing.html#command-line-installing-alternative-linux)

## Contributing

Thank you so much for considering to contribute to Amazon Q.

Before getting started, see our [contributing docs](CONTRIBUTING.md#security-issue-notifications).

### Prerequisites

- MacOS
  - Xcode 13 or later
  - Brew

#### 1. Clone repo

```shell
git clone https://github.com/aws/amazon-q-developer-cli.git
```

#### 2. Install the Rust toolchain using [Rustup](https://rustup.rs):

```shell
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup default stable
rustup toolchain install nightly
cargo install typos-cli
```

#### 3. Develop locally

See [DEVELOPMENT.md](DEVELOPMENT.md) for comprehensive development instructions.

Quick start:
- To compile and run: `cargo run --bin chat_cli`.
- To run tests: `cargo test`.
- To run lints: `cargo clippy`.
- To format rust files: `cargo +nightly fmt`.
- To run subcommands: `cargo run --bin chat_cli -- {subcommand}`.
  - Example: `cargo run --bin chat_cli -- skills list`

## Project Layout

- [`chat_cli`](crates/chat-cli/) - the `q` CLI, allows users to interface with Amazon Q Developer from
  the command line
- [`scripts/`](scripts/) - Contains ops and build related scripts
- [`crates/`](crates/) - Contains all rust crates
- [`docs/`](docs/) - Contains technical documentation

## Skills & Workflows

Amazon Q CLI now supports custom skills and workflows, enabling you to extend the agent's capabilities with reusable tools.

### What's New

- **Skills**: Create custom capabilities the agent can invoke through natural language
- **Workflows**: Chain multiple skills together for complex multi-step tasks
- **Natural Language**: Invoke skills and workflows conversationally
- **Type Safety**: Full schema validation for parameters and inputs
- **Error Handling**: Graceful error handling with clear messages

### Quick Example

Create a skill in `~/.q-skills/hello.json`:

```json
{
  "name": "hello",
  "description": "Greet a person by name",
  "parameters": [
    {
      "name": "name",
      "type": "string",
      "required": true
    }
  ],
  "implementation": {
    "type": "command",
    "command": "echo 'Hello, {{name}}!'"
  }
}
```

Then use it naturally:

```bash
q chat "Say hello to Alice"
```

The agent will automatically discover and use your skill.

### Built-in Skills

- **calculator**: Perform arithmetic operations
- More skills coming soon!

### Documentation

- [Quick Start Guide](docs/SKILLS_QUICKSTART.md) - Get started in 5 minutes
- [Full Integration Guide](docs/SKILLS_WORKFLOWS_INTEGRATION.md) - Complete documentation
- [API Reference](docs/SKILLS_WORKFLOWS_INTEGRATION.md#api-reference) - For developers

### Examples

See example skills in [`examples/skills/`](examples/skills/) and integration tests in `crates/chat-cli/tests/`:
- `skill_toolspec_integration.rs` - Skill integration examples
- `workflow_toolspec_integration.rs` - Workflow examples
- `natural_language_skill_invocation.rs` - Natural language usage
- `skill_workflow_error_handling.rs` - Error handling patterns

## Session Management

Amazon Q CLI automatically creates isolated workspaces for each conversation, keeping your analysis and planning documents separate from your code.

### Quick Start

```bash
# List active sessions
/sessions list

# Name a session
/sessions name abc123 "My Feature Work"

# View session history
/sessions history --search "authentication"

# Archive completed sessions
/sessions archive abc123
```

### Session Workspaces

Each session gets its own directory at `.amazonq/sessions/{conversation_id}/` for:
- Analysis documents
- Research notes
- Planning drafts
- Design documents

Your code stays in the repository, keeping it clean and organized.

### Documentation

- [User Guide](docs/SESSION_USER_GUIDE.md) - Complete command reference and best practices
- [Architecture](docs/SESSION_MANAGEMENT_DESIGN_V2.md) - Technical design details

## Security

For security related concerns, see [here](SECURITY.md).

## Licensing

This repo is dual licensed under MIT and Apache 2.0 licenses.

Those licenses can be found [here](LICENSE.MIT) and [here](LICENSE.APACHE).

“Amazon Web Services” and all related marks, including logos, graphic designs, and service names, are trademarks or trade dress of AWS in the U.S. and other countries. AWS’s trademarks and trade dress may not be used in connection with any product or service that is not AWS’s, in any manner that is likely to cause confusion among customers, or in any manner that disparages or discredits AWS.
