# Q CLI Skills System - Implementation Plan

## Overview
Implement Claude Skills-like functionality for Q CLI - interactive, stateful applications that extend Q's capabilities within the command-line interface.

## Core Concept
Q CLI Skills are interactive command-line applications that:
- Can be invoked from chat via `@skill_name` syntax
- Maintain state across interactions
- Can render interactive UI elements in terminal
- Integrate with existing Q CLI tools and agents

## Architecture

### Core Skill Trait
```rust
#[async_trait]
pub trait Skill: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    async fn execute(&self, params: serde_json::Value) -> Result<SkillResult>;
    async fn render_ui(&self) -> Result<SkillUI>;
    fn supports_interactive(&self) -> bool { false }
}

pub struct SkillResult {
    pub output: String,
    pub ui_updates: Option<Vec<UIUpdate>>,
    pub state_changes: Option<serde_json::Value>,
}
```

## Integration Points
1. **Chat Integration**: Skills invoked via `@skill_name` syntax
2. **Tool Reuse**: Skills can call existing tools (fs_read, use_aws, etc.)
3. **Agent Integration**: Agents can recommend and use skills
4. **CLI Commands**: `q skills` subcommand for management

## File Structure
```
crates/chat-cli/src/cli/
├── skills/                    # New skills module
│   ├── mod.rs                # Skill trait and core types
│   ├── registry.rs           # Skill discovery and management
│   ├── builtin/              # Built-in skills
│   │   ├── calculator.rs
│   │   ├── file_browser.rs
│   │   └── mod.rs
│   └── tests/                # Comprehensive test suite
└── chat/
    ├── skills_integration.rs  # Integration with chat system
    └── ...
```

## Implementation Phases
1. **Core Skill Interface** - Basic skill trait and execution
2. **Skill Registry** - Discovery and management
3. **Chat Integration** - @skill_name syntax support
4. **Built-in Skills** - Calculator, file browser examples
5. **CLI Commands** - Management interface

## Test-Driven Approach
Each feature will be implemented by:
1. Writing comprehensive tests first
2. Implementing minimal code to pass tests
3. Refactoring for integration with existing Q CLI structure
