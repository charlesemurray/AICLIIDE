# Creation Workflow Enhancement Plan

## Problem Statement

Current creation workflows have fundamental UX issues:
1. **Wrong question order** - asks for implementation details before type selection
2. **Poor input methods** - text input for limited options instead of multiple choice
3. **No prompt building tools** - expects users to have complete prompts ready
4. **Poor guidance** - minimal examples and no step-by-step assistance

## Implementation Plan

### Phase 1: Fix Question Flow & Input Methods (Priority: Critical)

#### 1.1 Add Multiple Choice UI Methods
- [ ] Add `select_option()` method to TerminalUI trait:
  ```rust
  fn select_option(&mut self, prompt: &str, options: &[(&str, &str)]) -> Result<String>;
  // options: &[("key", "description")]
  ```
- [ ] Add `select_multiple()` for multi-select scenarios
- [ ] Add `confirm_with_default()` for yes/no with smart defaults

#### 1.2 Skill Creation Flow Fix
- [ ] **Skill Type Selection** (multiple choice):
  ```
  What type of skill do you want to create?
  1. Command Execution - Run shell commands and scripts
  2. AI Assistant - Chat-based conversational helper  
  3. Text Template - Generate text with variables
  4. Interactive Session - Long-running interpreter (Python, Node, etc.)
  ```
- [ ] **Type-specific questions** with appropriate input methods:
  - **Code Skills**: Text input for command
  - **Conversation Skills**: Multiple choice for role + prompt builder
  - **Template Skills**: Text input with variable detection
  - **Session Skills**: Multiple choice for interpreter

#### 1.3 Command Creation Flow Fix
- [ ] **Command Type Selection** (multiple choice):
  ```
  What type of command do you want to create?
  1. System Command - Execute a program or script
  2. Command Alias - Shortcut to existing command with preset args
  3. Multi-step Script - Series of commands executed in sequence
  ```

#### 1.4 Agent Creation Enhancement
- [ ] **Agent Role Selection** (multiple choice):
  ```
  What role should this agent have?
  1. Code Reviewer - Reviews and suggests improvements to code
  2. Documentation Writer - Creates and maintains documentation
  3. Domain Expert - Specialized knowledge in a specific area
  4. General Assistant - Flexible helper for various tasks
  5. Custom Role - I'll write my own prompt
  ```

### Phase 2: Prompt Building Tools (Priority: High)

#### 2.1 Interactive Prompt Builder
- [ ] **Role-based prompt generation**:
  ```
  Selected: Code Reviewer
  
  What programming languages should this reviewer focus on?
  □ Python    □ JavaScript    □ Rust    □ Go
  □ Java      □ TypeScript    □ C++     □ Other: ____
  
  What aspects should the reviewer prioritize?
  □ Security vulnerabilities    □ Performance optimization
  □ Code style and formatting   □ Architecture and design
  □ Testing coverage           □ Documentation quality
  ```

#### 2.2 Template Library with Multiple Choice
- [ ] **Template Selection**:
  ```
  Choose a starting template:
  1. Code Reviewer (Python/JS focus)
  2. Documentation Writer (Technical docs)
  3. API Helper (REST/GraphQL expert)
  4. DevOps Assistant (Infrastructure focus)
  5. Custom (Start from scratch)
  ```

### Phase 3: Enhanced UI Components (Priority: Medium)

#### 3.1 Smart Multiple Choice
- [ ] **Context-aware options** - show relevant choices based on project detection
- [ ] **Searchable lists** - for long option lists (e.g., programming languages)
- [ ] **Grouped options** - categorize related choices

#### 3.2 Progressive Disclosure
- [ ] **Beginner vs Expert modes**:
  - Beginner: Multiple choice for everything
  - Expert: Allow text input shortcuts for known values
- [ ] **Smart defaults** based on context and previous choices

#### 3.3 Validation and Feedback
- [ ] **Real-time validation** with helpful suggestions
- [ ] **Preview mode** - show what will be created before saving
- [ ] **Edit mode** - modify choices before finalizing

## UI Method Examples

### Current (Poor UX):
```rust
self.config.skill_type = ui.prompt_required("Skill type (code_inline|conversation|prompt_inline)")?;
```

### Improved (Multiple Choice):
```rust
let skill_type = ui.select_option(
    "What type of skill do you want to create?",
    &[
        ("command", "Command Execution - Run shell commands and scripts"),
        ("assistant", "AI Assistant - Chat-based conversational helper"),
        ("template", "Text Template - Generate text with variables"),
        ("session", "Interactive Session - Long-running interpreter"),
    ]
)?;
```

### Advanced (Context-Aware):
```rust
let languages = ui.select_multiple(
    "Which programming languages should this code reviewer focus on?",
    &context.detected_languages(), // Smart defaults from project
    true // allow_other
)?;
```

## Implementation Order

### Week 1: UI Foundation
1. Add multiple choice methods to TerminalUI trait
2. Implement terminal-native selection UI (arrow keys, numbers)
3. Add confirmation and multi-select methods

### Week 2: Flow Restructuring  
1. Fix skill creation question order with multiple choice
2. Add command type selection
3. Enhance agent role selection

### Week 3: Prompt Building
1. Create role-based prompt templates
2. Build interactive prompt builder with multiple choice
3. Integrate template selection

### Week 4: Polish and Advanced Features
1. Add context-aware smart defaults
2. Implement preview and edit modes
3. Comprehensive testing

## Success Criteria

- [ ] No typing required for predefined options
- [ ] Question flow feels natural (type first, then details)
- [ ] New users can create working skills in under 2 minutes
- [ ] Expert users can still work efficiently
- [ ] All choices have clear, helpful descriptions

## Files to Modify

### UI Enhancement
- `crates/chat-cli/src/cli/creation/ui.rs` - Add multiple choice methods
- `crates/chat-cli/src/cli/creation/types.rs` - Update TerminalUI trait

### Flow Fixes
- `crates/chat-cli/src/cli/creation/flows/skill.rs` - Restructure with multiple choice
- `crates/chat-cli/src/cli/creation/flows/command.rs` - Add command type selection
- `crates/chat-cli/src/cli/creation/flows/agent.rs` - Add role selection

### New Components
- `crates/chat-cli/src/cli/creation/prompt_builder.rs` - Interactive prompt building
- `crates/chat-cli/src/cli/creation/templates/` - Template library directory

Ready to start with the UI foundation?
