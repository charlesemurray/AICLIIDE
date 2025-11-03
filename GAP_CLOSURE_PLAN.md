# Skills & Workflows - Gap Closure Plan

## Overview

**Goal**: Close critical gaps identified by senior engineer and UX designer assessments

**Timeline**: 30-60 hours total work
- **Phase 1 (Critical)**: 15-25 hours
- **Phase 2 (Important)**: 10-20 hours  
- **Phase 3 (Polish)**: 5-15 hours

---

## Phase 1: Critical Gaps (Must Have)

**Timeline**: 15-25 hours  
**Priority**: 🔴 Blocking production release

### 1.1: Natural Language Invocation Validation (6-8 hours)

**Goal**: Prove users can invoke skills through natural language

#### Step 1.1.1: Create Agent Mock (2h)
**Files to Create**:
- `crates/chat-cli/tests/helpers/mock_agent.rs`

**Implementation**:
```rust
pub struct MockAgent {
    available_tools: Vec<ToolSpec>,
}

impl MockAgent {
    pub fn with_skills(registry: &SkillRegistry) -> Self {
        let tools = registry.get_all_toolspecs();
        Self { available_tools: tools }
    }
    
    pub async fn process_input(&self, input: &str) -> AgentResponse {
        // Simple pattern matching for testing
        // "calculate 5 + 3" -> selects calculator tool
    }
}
```

**Tests**:
- Agent can discover skills
- Agent selects correct skill from natural language
- Agent invokes skill with correct parameters

**Validation**:
```bash
cargo test mock_agent
```

**Git Commit**: `test: add mock agent for natural language testing`

---

#### Step 1.1.2: Natural Language to Skill Test (2h)
**Files to Create**:
- `crates/chat-cli/tests/natural_language_invocation_e2e.rs`

**Implementation**:
```rust
#[tokio::test]
async fn test_user_invokes_skill_via_natural_language() {
    // Setup
    let registry = SkillRegistry::with_builtins();
    let agent = MockAgent::with_skills(&registry);
    
    // User input
    let input = "calculate 5 plus 3";
    
    // Agent processes
    let response = agent.process_input(input).await;
    
    // Verify
    assert!(response.used_tool("calculator"));
    assert!(response.result().contains("8"));
}
```

**Tests**:
- Simple calculation request
- Skill with parameters
- Skill not found scenario
- Ambiguous request handling

**Validation**:
```bash
cargo test natural_language_invocation_e2e
```

**Git Commit**: `test: add natural language to skill invocation tests`

---

#### Step 1.1.3: ChatSession Integration Test (2-4h)
**Files to Create**:
- `crates/chat-cli/tests/chat_session_skill_integration.rs`

**Implementation**:
```rust
#[tokio::test]
async fn test_skill_invocation_in_chat_session() {
    let mut os = Os::new().await.unwrap();
    let agents = get_test_agents(&os).await;
    let tool_manager = ToolManager::new_with_skills(&os).await.unwrap();
    
    // Simulate chat session with skill invocation
    let session = ChatSession::new(
        &mut os,
        "test_conv",
        agents,
        Some("calculate 10 + 5".to_string()),
        // ... other params
    ).await.unwrap();
    
    // Verify skill was invoked
    // Verify result returned to user
}
```

**Tests**:
- Skill invocation within chat
- Error handling in chat context
- Multiple skill invocations
- Skill + native tool usage

**Validation**:
```bash
cargo test chat_session_skill_integration
```

**Git Commit**: `test: add ChatSession skill integration tests`

---

### 1.2: User Feedback Mechanisms (4-6 hours)

**Goal**: Users know what's happening at each step

#### Step 1.2.1: Skill Loading Feedback (2h)
**Files to Modify**:
- `crates/chat-cli/src/cli/chat/skill_registry.rs`
- `crates/chat-cli/src/cli/chat/tool_manager.rs`

**Implementation**:
```rust
// In SkillRegistry::load_from_directory
pub async fn load_from_directory(&mut self, path: &Path) -> Result<LoadingSummary> {
    let mut summary = LoadingSummary::new();
    
    // Load skills
    for entry in entries {
        match self.load_skill(&entry).await {
            Ok(skill) => {
                summary.add_success(&skill.name);
                println!("✓ Loaded skill: {}", skill.name);
            }
            Err(e) => {
                summary.add_error(&entry, e);
                eprintln!("✗ Failed to load {}: {}", entry, e);
            }
        }
    }
    
    summary.print_summary();
    Ok(summary)
}
```

**Features**:
- Print skill loading progress
- Show success/failure for each skill
- Summary at end
- Clear error messages

**Validation**:
```bash
cargo run --bin chat_cli
# Should see: "✓ Loaded skill: calculator"
```

**Git Commit**: `feat: add skill loading feedback`

---

#### Step 1.2.2: Skill Execution Feedback (2-4h)
**Files to Modify**:
- `crates/chat-cli/src/cli/chat/tools/skill_tool.rs`

**Implementation**:
```rust
pub async fn invoke(&self, registry: &SkillRegistry, stdout: &mut impl Write) -> Result<InvokeOutput> {
    // Show what's happening
    writeln!(stdout, "🔧 Executing skill: {}", self.skill_name)?;
    
    let start = Instant::now();
    let result = self.execute_skill(registry).await?;
    let duration = start.elapsed();
    
    writeln!(stdout, "✓ Skill completed in {:.2}s", duration.as_secs_f64())?;
    
    Ok(result)
}
```

**Features**:
- Show skill name being executed
- Show execution time
- Show success/failure
- Show result preview

**Validation**:
```bash
cargo test skill_execution_feedback
```

**Git Commit**: `feat: add skill execution feedback`

---

### 1.3: Error UX Redesign (4-6 hours)

**Goal**: User-friendly errors with actionable guidance

#### Step 1.3.1: Error Message Redesign (2-3h)
**Files to Modify**:
- `crates/chat-cli/src/cli/skills/toolspec_conversion.rs`
- `crates/chat-cli/src/cli/chat/tools/skill_tool.rs`

**Implementation**:
```rust
#[derive(Debug, thiserror::Error)]
pub enum SkillError {
    #[error("Skill '{name}' not found.\n\n\
             💡 Tip: Check available skills with: q skills list\n\
             💡 Tip: Make sure your skill file is in ~/.q-skills/")]
    NotFound { name: String },
    
    #[error("Skill '{name}' failed to execute: {reason}\n\n\
             💡 Tip: Check the skill definition in ~/.q-skills/{name}.json\n\
             💡 Tip: Try running the command manually: {command}")]
    ExecutionFailed { name: String, reason: String, command: String },
    
    #[error("Invalid skill parameter '{param}': {reason}\n\n\
             💡 Tip: Check the skill's parameter requirements\n\
             💡 Tip: Use: q skills info {name}")]
    InvalidParameter { param: String, reason: String, name: String },
}
```

**Features**:
- Plain English errors
- Actionable tips
- Recovery suggestions
- Relevant commands

**Validation**:
```bash
cargo test error_messages
```

**Git Commit**: `feat: redesign error messages for better UX`

---

#### Step 1.3.2: Error Recovery Paths (2-3h)
**Files to Create**:
- `crates/chat-cli/src/cli/skills/error_recovery.rs`

**Implementation**:
```rust
pub struct ErrorRecovery;

impl ErrorRecovery {
    pub fn suggest_fix(error: &SkillError) -> Vec<String> {
        match error {
            SkillError::NotFound { name } => vec![
                format!("Check if skill exists: ls ~/.q-skills/{}.json", name),
                "List available skills: q skills list".to_string(),
                "Create a new skill: q create skill".to_string(),
            ],
            // ... other error types
        }
    }
}
```

**Features**:
- Specific suggestions per error type
- Commands user can run
- Links to documentation
- Examples

**Validation**:
```bash
cargo test error_recovery
```

**Git Commit**: `feat: add error recovery suggestions`

---

### 1.4: Skill Discovery UX (3-5 hours)

**Goal**: Users can find and learn about skills

#### Step 1.4.1: Enhanced Skills List Command (2-3h)
**Files to Modify**:
- `crates/chat-cli/src/cli/skills_cli.rs`

**Implementation**:
```rust
pub async fn list_skills(registry: &SkillRegistry) -> Result<()> {
    let skills = registry.list_skills();
    
    if skills.is_empty() {
        println!("No skills found.");
        println!("\n💡 Create your first skill:");
        println!("   q create skill my-skill");
        return Ok(());
    }
    
    println!("Available Skills:\n");
    
    for skill in skills {
        println!("  📦 {}", skill.name);
        println!("     {}", skill.description.as_deref().unwrap_or("No description"));
        
        if let Some(params) = &skill.parameters {
            println!("     Parameters: {}", params.len());
        }
        
        println!();
    }
    
    println!("💡 Get details: q skills info <name>");
    println!("💡 Use in chat: 'use <skill-name> to do X'");
    
    Ok(())
}
```

**Features**:
- Clear skill listing
- Descriptions shown
- Parameter counts
- Usage hints
- Empty state guidance

**Validation**:
```bash
cargo run --bin chat_cli -- skills list
```

**Git Commit**: `feat: enhance skills list command UX`

---

#### Step 1.4.2: Skill Info Command (1-2h)
**Files to Modify**:
- `crates/chat-cli/src/cli/skills_cli.rs`

**Implementation**:
```rust
pub async fn show_skill_info(registry: &SkillRegistry, name: &str) -> Result<()> {
    let skill = registry.get_skill(name)
        .ok_or_else(|| eyre::eyre!("Skill '{}' not found", name))?;
    
    println!("Skill: {}", skill.name);
    println!("Description: {}", skill.description.as_deref().unwrap_or("None"));
    println!();
    
    if let Some(params) = &skill.parameters {
        println!("Parameters:");
        for param in params {
            println!("  • {} ({}){}", 
                param.name, 
                param.type_,
                if param.required { " - required" } else { "" }
            );
        }
        println!();
    }
    
    println!("Usage Example:");
    println!("  q chat \"use {} to do something\"", skill.name);
    
    Ok(())
}
```

**Features**:
- Detailed skill information
- Parameter details
- Usage examples
- Clear formatting

**Validation**:
```bash
cargo run --bin chat_cli -- skills info calculator
```

**Git Commit**: `feat: add skill info command`

---

## Phase 2: Important Gaps (Should Have)

**Timeline**: 10-20 hours  
**Priority**: 🟡 Important for GA

### 2.1: User Testing & Validation (8-12 hours)

#### Step 2.1.1: User Testing Protocol (2h)
**Files to Create**:
- `docs/USER_TESTING_PROTOCOL.md`

**Content**:
- Test scenarios
- Success criteria
- Observation checklist
- Feedback collection form

**Git Commit**: `docs: add user testing protocol`

---

#### Step 2.1.2: Conduct User Testing (4-6h)
**Activities**:
- Recruit 5 test users
- Run testing sessions
- Observe and document
- Collect feedback

**Deliverable**: User testing report

---

#### Step 2.1.3: Iterate Based on Feedback (2-4h)
**Activities**:
- Analyze feedback
- Prioritize issues
- Fix critical UX issues
- Re-test if needed

**Git Commit**: `fix: address user testing feedback`

---

### 2.2: Onboarding Experience (4-6 hours)

#### Step 2.2.1: First-Run Tutorial (2-3h)
**Files to Create**:
- `crates/chat-cli/src/cli/skills/onboarding.rs`

**Implementation**:
```rust
pub async fn show_first_run_tutorial() -> Result<()> {
    println!("Welcome to Q Skills! 🎉\n");
    println!("Skills let you extend Q with custom capabilities.\n");
    
    println!("Quick Start:");
    println!("  1. Create a skill: q create skill my-skill");
    println!("  2. Use it in chat: q chat 'use my-skill'");
    println!("  3. List skills: q skills list\n");
    
    println!("Example skills are in: examples/skills/");
    println!("Learn more: docs/SKILLS_QUICKSTART.md\n");
    
    // Mark tutorial as shown
    mark_tutorial_shown()?;
    
    Ok(())
}
```

**Features**:
- Show on first run
- Quick start steps
- Example references
- Documentation links

**Git Commit**: `feat: add first-run tutorial`

---

#### Step 2.2.2: Interactive Example (2-3h)
**Files to Modify**:
- `crates/chat-cli/src/cli/skills_cli.rs`

**Implementation**:
```rust
pub async fn run_interactive_example() -> Result<()> {
    println!("Let's create your first skill!\n");
    
    // Guide user through creation
    let name = prompt("Skill name")?;
    let description = prompt("Description")?;
    
    // Create simple skill
    create_example_skill(name, description)?;
    
    println!("\n✓ Skill created!");
    println!("Try it: q chat 'use {} to test'", name);
    
    Ok(())
}
```

**Git Commit**: `feat: add interactive skill creation example`

---

### 2.3: Help System (2-4 hours)

#### Step 2.3.1: In-App Help (1-2h)
**Files to Modify**:
- `crates/chat-cli/src/cli/skills_cli.rs`

**Implementation**:
```rust
pub fn show_help() {
    println!("Q Skills Help\n");
    println!("Commands:");
    println!("  q skills list              List all skills");
    println!("  q skills info <name>       Show skill details");
    println!("  q create skill <name>      Create new skill");
    println!("  q chat 'use <skill>'       Use skill in chat\n");
    println!("Documentation: docs/SKILLS_QUICKSTART.md");
    println!("Examples: examples/skills/");
}
```

**Git Commit**: `feat: add in-app help for skills`

---

#### Step 2.3.2: Troubleshooting Guide (1-2h)
**Files to Create**:
- `docs/SKILLS_TROUBLESHOOTING.md`

**Content**:
- Common issues
- Solutions
- Debugging steps
- FAQ

**Git Commit**: `docs: add skills troubleshooting guide`

---

## Phase 3: Polish (Nice to Have)

**Timeline**: 5-15 hours  
**Priority**: 🟢 Post-launch improvements

### 3.1: Advanced Features (3-5 hours)

- Skill templates
- Skill validation tool
- Performance monitoring
- Usage analytics

### 3.2: Enhanced Documentation (2-4 hours)

- Video tutorials
- Interactive examples
- Best practices guide
- Community examples

### 3.3: Visual Improvements (2-4 hours)

- Better CLI formatting
- Color coding
- Progress bars
- Animations

### 3.4: User Education (2-4 hours)

- Webinar content
- Blog posts
- Tutorial series
- Community resources

---

## Implementation Schedule

### Week 1: Critical Gaps (Phase 1)
**Days 1-2**: Natural Language Invocation (6-8h)
- Mock agent
- NL to skill tests
- ChatSession integration

**Days 3-4**: Feedback Mechanisms (4-6h)
- Loading feedback
- Execution feedback

**Days 4-5**: Error UX (4-6h)
- Error redesign
- Recovery paths

**Day 5**: Discovery UX (3-5h)
- Enhanced list command
- Info command

### Week 2: Important Gaps (Phase 2)
**Days 1-2**: User Testing (8-12h)
- Protocol
- Testing
- Iteration

**Days 3-4**: Onboarding (4-6h)
- Tutorial
- Interactive example

**Day 5**: Help System (2-4h)
- In-app help
- Troubleshooting guide

### Week 3: Polish (Phase 3)
**Optional**: Advanced features and polish

---

## Success Criteria

### Phase 1 Complete When:
- ✅ Natural language invocation test passes
- ✅ Users see feedback at each step
- ✅ Error messages are user-friendly
- ✅ Users can discover skills easily

### Phase 2 Complete When:
- ✅ 5 users successfully complete test scenarios
- ✅ First-run tutorial implemented
- ✅ Help system available

### Phase 3 Complete When:
- ✅ Advanced features implemented
- ✅ Documentation enhanced
- ✅ Visual polish complete

---

## Validation Checklist

### After Phase 1:
- [ ] Natural language invocation test passes
- [ ] Skill loading shows feedback
- [ ] Skill execution shows progress
- [ ] Errors are user-friendly
- [ ] `q skills list` shows clear output
- [ ] `q skills info` shows details

### After Phase 2:
- [ ] 5 users complete test scenarios
- [ ] First-run tutorial works
- [ ] Help system accessible
- [ ] Troubleshooting guide complete

### After Phase 3:
- [ ] Advanced features working
- [ ] Documentation enhanced
- [ ] Visual polish complete

---

## Risk Mitigation

### Risk 1: User Testing Delays
**Mitigation**: Have backup test users ready

### Risk 2: Technical Blockers
**Mitigation**: Prioritize critical tests first

### Risk 3: Scope Creep
**Mitigation**: Stick to plan, defer nice-to-haves

---

## Resource Requirements

### Development Time
- Phase 1: 15-25 hours
- Phase 2: 10-20 hours
- Phase 3: 5-15 hours
- **Total**: 30-60 hours

### Testing Resources
- 5 test users
- Testing environment
- Feedback collection tools

### Documentation
- Technical writer (optional)
- Video production (optional)

---

## Deliverables

### Phase 1:
- Natural language invocation tests
- Feedback mechanisms
- User-friendly errors
- Discovery UX

### Phase 2:
- User testing report
- Onboarding tutorial
- Help system
- Troubleshooting guide

### Phase 3:
- Advanced features
- Enhanced documentation
- Visual polish

---

## Next Steps

1. **Review and approve plan**
2. **Allocate resources**
3. **Begin Phase 1, Step 1.1.1**
4. **Daily standups to track progress**
5. **Weekly reviews to adjust plan**

---

**Plan Created**: 2025-11-03  
**Estimated Completion**: 2-3 weeks  
**Priority**: High - Blocking production release
