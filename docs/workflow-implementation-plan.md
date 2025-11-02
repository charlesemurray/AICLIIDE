# Workflow System Implementation Plan

## Overview

Phased implementation plan for the Q CLI workflow system, prioritizing core functionality and iterating toward full feature set.

## Phase 1: Foundation (Week 1-2)

### Goals
- Basic workflow execution engine
- Simple linear workflows
- Skill integration
- Manual triggers only

### Tasks

#### 1.1 Create Workflow Crate
- [ ] Create `crates/workflow-engine/` directory structure
- [ ] Set up `Cargo.toml` with dependencies (tokio, serde, serde_json)
- [ ] Define basic module structure

#### 1.2 Core Data Structures
- [ ] `Workflow` struct (name, version, steps)
- [ ] `Step` enum (Skill, Agent variants)
- [ ] `WorkflowState` struct (status, current_step, outputs)
- [ ] `StepOutput` struct (id, data, timestamp)

#### 1.3 Workflow Parser
- [ ] JSON deserialization for workflow definitions
- [ ] Basic validation (required fields, step IDs unique)
- [ ] Error types for parsing failures

#### 1.4 Simple Execution Engine
- [ ] `WorkflowEngine` struct
- [ ] `execute_workflow()` - linear execution only
- [ ] `execute_skill_step()` - invoke existing skills API
- [ ] Basic error handling (halt on failure)

#### 1.5 State Persistence
- [ ] Create workflow state directory (`~/.aws/amazonq/workflows/`)
- [ ] Save state to JSON files
- [ ] Load state from disk

#### 1.6 CLI Integration
- [ ] Add `workflow` subcommand to Q CLI
- [ ] `q workflow run <file>` - execute workflow from JSON file
- [ ] `q workflow status <id>` - show workflow status

### Deliverables
- Working linear workflow execution
- Skill step execution
- Basic state persistence
- CLI commands for run and status

### Example Workflow (Phase 1)
```json
{
  "name": "simple-test",
  "version": "1.0.0",
  "steps": [
    {
      "id": "step1",
      "type": "skill",
      "skill": "echo",
      "inputs": {"message": "Hello"}
    },
    {
      "id": "step2",
      "type": "skill",
      "skill": "echo",
      "inputs": {"message": "World"}
    }
  ]
}
```

## Phase 2: Data Flow & Context (Week 3)

### Goals
- Input/output wiring between steps
- Shared context (readonly)
- Output references

### Tasks

#### 2.1 Context System
- [ ] `WorkflowContext` struct (readonly, mutable)
- [ ] Context initialization from workflow definition
- [ ] Context access in step execution

#### 2.2 Output Wiring
- [ ] Parse output references (e.g., `"step1.data"`)
- [ ] Resolve references during execution
- [ ] Store step outputs in state
- [ ] Pass resolved inputs to skills

#### 2.3 Data Serialization
- [ ] Support JSON data in outputs
- [ ] Support text data in outputs
- [ ] Type conversion utilities

### Deliverables
- Steps can reference previous step outputs
- Readonly context available to all steps
- Data flows through workflow

### Example Workflow (Phase 2)
```json
{
  "name": "data-flow-test",
  "version": "1.0.0",
  "context": {
    "readonly": {
      "environment": "dev"
    }
  },
  "steps": [
    {
      "id": "fetch",
      "type": "skill",
      "skill": "http-get",
      "inputs": {"url": "https://api.example.com"},
      "outputs": ["response"]
    },
    {
      "id": "process",
      "type": "skill",
      "skill": "json-parse",
      "inputs": {"data": "fetch.response"},
      "outputs": ["parsed"]
    }
  ]
}
```

## Phase 3: Async & Parallel Execution (Week 4)

### Goals
- Asynchronous step execution
- Parallel step groups
- Event-driven architecture

### Tasks

#### 3.1 Event System
- [ ] `WorkflowEvent` enum (StepStarted, StepCompleted, StepFailed)
- [ ] Event channel (tokio mpsc)
- [ ] Event emitter in step execution
- [ ] Event listener in engine

#### 3.2 Async Execution
- [ ] Convert engine to async/await
- [ ] Spawn tasks for async steps
- [ ] Track background tasks
- [ ] Non-blocking step execution

#### 3.3 Parallel Groups
- [ ] `ParallelStep` variant
- [ ] Execute steps concurrently with `tokio::spawn`
- [ ] Wait strategies (all, any, none)
- [ ] Collect parallel outputs

### Deliverables
- Steps can execute asynchronously
- Parallel step groups work
- Event-driven execution model

### Example Workflow (Phase 3)
```json
{
  "name": "parallel-test",
  "version": "1.0.0",
  "steps": [
    {
      "id": "parallel-tasks",
      "type": "parallel",
      "steps": [
        {
          "id": "task1",
          "type": "skill",
          "skill": "process-a"
        },
        {
          "id": "task2",
          "type": "skill",
          "skill": "process-b"
        }
      ],
      "wait_for": "all"
    }
  ]
}
```

## Phase 4: Agent Integration (Week 5)

### Goals
- Agent step execution
- Agent guardrails
- Background agent tasks

### Tasks

#### 4.1 Agent Step Type
- [ ] `AgentStep` struct with guardrails
- [ ] Parse agent step definitions
- [ ] Validate agent exists

#### 4.2 Agent Execution
- [ ] Invoke agent with input context
- [ ] Apply guardrails (max_iterations, allowed_tools)
- [ ] Capture agent output
- [ ] Handle agent failures

#### 4.3 Background Agents
- [ ] Spawn agent as background task
- [ ] Monitor agent progress via events
- [ ] Timeout handling
- [ ] Graceful agent termination

### Deliverables
- Agent steps execute in workflows
- Guardrails enforced
- Agents run in background

### Example Workflow (Phase 4)
```json
{
  "name": "agent-test",
  "version": "1.0.0",
  "steps": [
    {
      "id": "analyze",
      "type": "agent",
      "agent": "code-analyzer",
      "inputs": {"path": "./src"},
      "async": true,
      "guardrails": {
        "max_iterations": 10,
        "allowed_tools": ["fs_read"]
      }
    }
  ]
}
```

## Phase 5: Error Handling (Week 6)

### Goals
- Retry logic
- Error strategies per step
- Rollback support

### Tasks

#### 5.1 Error Strategy System
- [ ] `ErrorStrategy` enum (Retry, Rollback, Skip, Halt)
- [ ] Parse error_handling from workflow definition
- [ ] Apply default and per-step strategies

#### 5.2 Retry Logic
- [ ] Implement retry with backoff (exponential, linear, fixed)
- [ ] Track retry attempts
- [ ] Max attempts enforcement

#### 5.3 Rollback
- [ ] Define rollback steps in workflow
- [ ] Execute rollback on failure
- [ ] Rollback state tracking

#### 5.4 Skip & Continue
- [ ] Skip failed step and continue
- [ ] Mark step as skipped in state
- [ ] Log skip reason

### Deliverables
- Configurable error handling per step
- Retry with backoff works
- Rollback executes on failure

### Example Workflow (Phase 5)
```json
{
  "name": "error-handling-test",
  "version": "1.0.0",
  "steps": [
    {
      "id": "fetch",
      "type": "skill",
      "skill": "http-get",
      "inputs": {"url": "https://api.example.com"}
    }
  ],
  "error_handling": {
    "per_step": {
      "fetch": {
        "strategy": "retry",
        "max_attempts": 3,
        "backoff": "exponential"
      }
    }
  }
}
```

## Phase 6: Control Flow (Week 7)

### Goals
- Conditional steps
- Loop steps
- Expression evaluation

### Tasks

#### 6.1 Expression Evaluator
- [ ] Simple expression parser (comparisons, boolean logic)
- [ ] Evaluate expressions against workflow state
- [ ] Support common operators (==, !=, <, >, &&, ||)

#### 6.2 Conditional Steps
- [ ] `ConditionalStep` struct
- [ ] Evaluate condition
- [ ] Execute then/else branches

#### 6.3 Loop Steps
- [ ] `LoopStep` struct
- [ ] Evaluate loop condition
- [ ] Execute loop body
- [ ] Max iterations enforcement

### Deliverables
- Conditional branching works
- Loops execute correctly
- Expression evaluation functional

### Example Workflow (Phase 6)
```json
{
  "name": "control-flow-test",
  "version": "1.0.0",
  "steps": [
    {
      "id": "check",
      "type": "skill",
      "skill": "file-exists",
      "inputs": {"path": "./data.json"},
      "outputs": ["exists"]
    },
    {
      "id": "conditional",
      "type": "conditional",
      "condition": "check.exists == true",
      "then": [
        {
          "id": "process",
          "type": "skill",
          "skill": "process-file"
        }
      ],
      "else": [
        {
          "id": "error",
          "type": "skill",
          "skill": "log-error"
        }
      ]
    }
  ]
}
```

## Phase 7: Resource Management (Week 8)

### Goals
- CPU and memory limits per workflow
- Concurrent workflow support
- Resource monitoring

### Tasks

#### 7.1 Resource Limits
- [ ] Parse resource_limits from workflow definition
- [ ] Apply memory limits (using system APIs)
- [ ] Apply CPU limits (cgroup or process priority)
- [ ] Timeout enforcement

#### 7.2 Workflow Isolation
- [ ] Separate state per workflow instance
- [ ] Prevent state interference
- [ ] Resource accounting per workflow

#### 7.3 Concurrent Workflows
- [ ] Track active workflows
- [ ] Limit total concurrent workflows
- [ ] Queue workflows if limit reached

### Deliverables
- Resource limits enforced
- Multiple workflows run concurrently
- No interference between workflows

## Phase 8: Scheduling (Week 9)

### Goals
- Scheduled workflow triggers
- Cron expression support
- Scheduler daemon

### Tasks

#### 8.1 Scheduler
- [ ] `WorkflowScheduler` struct
- [ ] Parse cron expressions (use `cron` crate)
- [ ] Poll for scheduled triggers
- [ ] Launch workflows on schedule

#### 8.2 Trigger Management
- [ ] Register scheduled workflows
- [ ] Enable/disable schedules
- [ ] List scheduled workflows

#### 8.3 Background Daemon
- [ ] Run scheduler in background
- [ ] Persist scheduler state
- [ ] Graceful shutdown

### Deliverables
- Workflows trigger on schedule
- Cron expressions work
- Scheduler runs in background

## Phase 9: Wizard System (Week 10)

### Goals
- Interactive workflow creation
- Guided step definition
- JSON generation

### Tasks

#### 9.1 Wizard Framework
- [ ] Interactive prompt system (use `dialoguer` crate)
- [ ] Multi-step wizard flow
- [ ] Input validation

#### 9.2 Workflow Creation Flow
- [ ] Ask for workflow name, description
- [ ] Guide through step creation
- [ ] Suggest available skills/agents
- [ ] Configure inputs/outputs
- [ ] Set error handling

#### 9.3 JSON Generation
- [ ] Build workflow definition from wizard inputs
- [ ] Validate generated JSON
- [ ] Save to file
- [ ] Offer to run immediately

### Deliverables
- `q workflow create` wizard works
- Generates valid workflow JSON
- User-friendly experience

## Phase 10: CLI Enhancements (Week 11)

### Goals
- Complete CLI command set
- Workflow management
- Logging and monitoring

### Tasks

#### 10.1 Additional Commands
- [ ] `q workflow list` - list all workflows
- [ ] `q workflow stop <id>` - stop running workflow
- [ ] `q workflow logs <id>` - view workflow logs
- [ ] `q workflow validate <file>` - validate workflow JSON
- [ ] `q workflow delete <name>` - delete workflow definition

#### 10.2 Logging
- [ ] Structured logging for workflow events
- [ ] Log to file per workflow instance
- [ ] Log rotation

#### 10.3 Status Display
- [ ] Rich status output (progress, current step)
- [ ] Real-time updates for running workflows
- [ ] Historical execution data

### Deliverables
- Complete CLI command set
- Good logging and monitoring
- User-friendly status display

## Testing Strategy

### Unit Tests
- Test each module independently
- Mock external dependencies (skills, agents)
- Cover error cases

### Integration Tests
- End-to-end workflow execution
- Test with real skills
- Test concurrent workflows

### Example Workflows
- Create example workflows for common use cases
- Use as integration tests
- Include in documentation

## Documentation

### User Documentation
- [ ] Workflow definition guide
- [ ] JSON schema reference
- [ ] CLI command reference
- [ ] Example workflows
- [ ] Troubleshooting guide

### Developer Documentation
- [ ] Architecture overview
- [ ] API documentation
- [ ] Extension guide
- [ ] Contributing guide

## Success Criteria

### Phase 1-3 (MVP)
- Linear workflows execute successfully
- Data flows between steps
- Parallel execution works
- Basic CLI commands functional

### Phase 4-6 (Core Features)
- Agents integrate with workflows
- Error handling robust
- Control flow (conditionals, loops) works

### Phase 7-10 (Production Ready)
- Resource management prevents blocking
- Scheduling works reliably
- Wizard creates valid workflows
- Complete CLI tooling
- Comprehensive documentation

## Timeline Summary

| Phase | Duration | Focus |
|-------|----------|-------|
| 1 | Week 1-2 | Foundation & basic execution |
| 2 | Week 3 | Data flow & context |
| 3 | Week 4 | Async & parallel |
| 4 | Week 5 | Agent integration |
| 5 | Week 6 | Error handling |
| 6 | Week 7 | Control flow |
| 7 | Week 8 | Resource management |
| 8 | Week 9 | Scheduling |
| 9 | Week 10 | Wizard system |
| 10 | Week 11 | CLI enhancements |

**Total: 11 weeks to full feature set**

**MVP (Phases 1-3): 4 weeks**
