# Workflow System Design

## Overview

A workflow system for Q CLI that orchestrates skills and agents to execute complex, multi-step tasks. Supports both interactive and background execution with state persistence and resource management.

## Core Concepts

### Workflow
A JSON-defined sequence of steps that can include skills, agents, and control flow logic. Workflows can be triggered by user input or schedules.

### Skills
Discrete executable units from the existing Q CLI skills system (code_inline, code_session, conversation, prompt_inline).

### Agents
Autonomous background workers that execute long-running complex tasks with guardrails to keep them on track.

### Steps
Individual units of work within a workflow. Each step can be:
- A skill execution
- An agent invocation
- A control flow operation (conditional, parallel group)

## Architecture

### Execution Model

**Event-Driven with Polling Fallback**
- Workflow engine listens on event channels (tokio)
- Steps emit completion/failure/progress events
- Agents run in background tasks
- Scheduler polls for time-based triggers
- State machine tracks workflow progress

### State Management

**Persistence**
- Primary: Disk-based (JSON/SQLite)
- Optional: Database support
- State includes: workflow status, step outputs, context, execution history

**Isolation**
- Each workflow instance has isolated state
- Concurrent workflows don't interfere
- Shared read-only context for workflow-wide config

### Data Flow

**Hybrid Model**
- Explicit inputs/outputs for step-to-step data flow
- Read-only shared context for workflow config/metadata
- Optional mutable shared state with access controls
- Primary formats: unstructured text, JSON, structured objects

### Resource Management

**Per-Workflow Limits**
- CPU quota (prevent blocking other workflows)
- Memory limits
- Configurable timeouts
- Disk space monitoring (system-managed)

## Workflow Definition Schema

### Basic Structure

```json
{
  "name": "workflow-name",
  "version": "1.0.0",
  "description": "Workflow description",
  "triggers": [...],
  "context": {...},
  "steps": [...],
  "error_handling": {...},
  "resource_limits": {...}
}
```

### Triggers

```json
{
  "triggers": [
    {
      "type": "manual",
      "description": "User-initiated"
    },
    {
      "type": "schedule",
      "cron": "0 9 * * *",
      "timezone": "UTC"
    }
  ]
}
```

### Context

```json
{
  "context": {
    "readonly": {
      "environment": "production",
      "user": "alice"
    },
    "mutable": {
      "counter": 0
    }
  }
}
```

### Steps

#### Skill Step
```json
{
  "id": "fetch-data",
  "type": "skill",
  "skill": "http-fetch",
  "inputs": {
    "url": "https://api.example.com/data"
  },
  "outputs": ["data"],
  "async": false,
  "timeout": 30
}
```

#### Agent Step
```json
{
  "id": "analyze",
  "type": "agent",
  "agent": "data-analyzer",
  "inputs": {
    "data": "fetch-data.data"
  },
  "outputs": ["analysis"],
  "async": true,
  "guardrails": {
    "max_iterations": 10,
    "allowed_tools": ["fs_read", "execute_bash"]
  }
}
```

#### Parallel Group
```json
{
  "id": "parallel-tasks",
  "type": "parallel",
  "steps": [
    {"id": "task1", "type": "skill", "skill": "process-a"},
    {"id": "task2", "type": "skill", "skill": "process-b"}
  ],
  "wait_for": "all"
}
```

#### Conditional Step
```json
{
  "id": "conditional",
  "type": "conditional",
  "condition": "fetch-data.status == 200",
  "then": [
    {"id": "success", "type": "skill", "skill": "process"}
  ],
  "else": [
    {"id": "failure", "type": "skill", "skill": "log-error"}
  ]
}
```

#### Loop Step
```json
{
  "id": "loop",
  "type": "loop",
  "condition": "counter < 5",
  "steps": [
    {"id": "process", "type": "skill", "skill": "process-item"}
  ],
  "max_iterations": 10
}
```

### Error Handling

```json
{
  "error_handling": {
    "default": "halt",
    "per_step": {
      "fetch-data": {
        "strategy": "retry",
        "max_attempts": 3,
        "backoff": "exponential"
      },
      "analyze": {
        "strategy": "skip",
        "continue_on_error": true
      }
    },
    "rollback": {
      "enabled": true,
      "steps": ["cleanup"]
    }
  }
}
```

**Strategies**:
- `retry`: Retry with backoff
- `rollback`: Execute rollback steps
- `skip`: Continue to next step
- `halt`: Stop workflow execution
- `custom`: User-defined error handler

### Resource Limits

```json
{
  "resource_limits": {
    "max_memory_mb": 512,
    "max_cpu_percent": 50,
    "max_execution_time": 3600,
    "max_concurrent_steps": 5
  }
}
```

## Workflow Lifecycle

1. **Definition**: User creates workflow JSON (manually or via wizard)
2. **Registration**: Workflow loaded and validated
3. **Trigger**: Manual or scheduled activation
4. **Initialization**: State created, context loaded
5. **Execution**: Steps executed according to definition
6. **Monitoring**: Events emitted, state persisted
7. **Completion**: Final state saved, cleanup performed
8. **Archival**: Historical data retained

## Integration with Q CLI

### CLI Commands

```bash
# List workflows
q workflow list

# Create workflow (wizard)
q workflow create

# Run workflow
q workflow run <name>

# Show workflow status
q workflow status <id>

# Stop workflow
q workflow stop <id>

# View workflow logs
q workflow logs <id>
```

### Skills Integration

Workflows invoke existing skills through the skills system API. No changes to skill definitions required.

### Agents Integration

Agents are invoked as background tasks with:
- Input context from workflow
- Output captured to workflow state
- Guardrails enforced by workflow engine
- Progress events emitted to workflow

### MCP Integration

External integrations handled through MCP servers configured in agent definitions or skill configurations.

## Wizard System

Interactive workflow creation assistant:

1. **Purpose**: Ask user what they want to accomplish
2. **Steps**: Guide through step definition
3. **Skills/Agents**: Suggest available skills/agents
4. **Data Flow**: Help wire inputs/outputs
5. **Error Handling**: Configure failure strategies
6. **Review**: Show generated JSON
7. **Save**: Write workflow definition to disk

## Security Considerations

- Workflow definitions validated before execution
- Resource limits enforced per workflow
- Agent guardrails prevent runaway execution
- File access controlled through existing Q CLI permissions
- State isolation between concurrent workflows
- Audit logging for workflow execution

## Future Enhancements

- Nested workflows (workflows calling workflows)
- Workflow templates and reusable components
- Visual workflow editor
- Workflow marketplace/sharing
- Advanced scheduling (dependencies, priorities)
- Distributed execution (multi-machine)
- Workflow versioning and rollback
- Real-time collaboration on workflows
