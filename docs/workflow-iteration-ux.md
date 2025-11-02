# Workflow Iteration User Experience Design

## Core Principle

**Workflows evolve through conversation, not replacement.** The system understands context and intent, making surgical changes rather than wholesale rewrites.

## Customer Journey

### Journey 1: First-Time Workflow Creation

**User starts with a goal:**
```bash
$ q workflow create "Monitor API health and alert on failures"
```

**System responds:**
```
Analyzing your request...

I'll create a workflow that:
1. Checks API endpoint health
2. Evaluates response status
3. Sends alert if unhealthy

Available resources:
✓ http-fetch (existing skill)
✗ health-checker (will create)
✗ alert-sender (will create)

Creating workflow: api-health-monitor

[Generating...]

Proposed workflow has 3 steps:
  1. fetch-health → Check API endpoint
  2. evaluate → Determine if healthy
  3. alert → Send notification if needed

Would you like to:
  [v]iew full workflow
  [c]reate missing skills
  [s]ave and finish
  [e]dit before saving
  [q]uit

Choice:
```

**User chooses 'v' to view:**
```json
{
  "name": "api-health-monitor",
  "version": "1.0.0",
  "steps": [
    {
      "id": "fetch-health",
      "type": "skill",
      "skill": "http-fetch",
      "inputs": {
        "url": "context.api_url",
        "method": "GET"
      },
      "outputs": ["response"]
    },
    {
      "id": "evaluate",
      "type": "skill",
      "skill": "health-checker",
      "inputs": {
        "response": "fetch-health.response"
      },
      "outputs": ["is_healthy"]
    },
    {
      "id": "alert",
      "type": "conditional",
      "condition": "evaluate.is_healthy == false",
      "then": [
        {
          "id": "send-alert",
          "type": "skill",
          "skill": "alert-sender",
          "inputs": {
            "message": "API health check failed"
          }
        }
      ]
    }
  ]
}

Looks good? [y/n/edit]:
```

**User says 'y', system creates skills:**
```
Creating missing skills...

Skill: health-checker
  Type: code_inline
  Command: python3 check_health.py
  
  [Shows skill definition]
  
Accept? [y/n/edit]: y
✓ Created health-checker

Skill: alert-sender
  Type: code_inline
  Command: curl -X POST slack-webhook
  
Accept? [y/n/edit]: y
✓ Created alert-sender

✓ Workflow saved: ~/.aws/amazonq/workflows/api-health-monitor.json (v1)

You can now run: q workflow run api-health-monitor
```

### Journey 2: Quick Iteration (Same Session)

**User realizes they need retry logic:**
```bash
$ q workflow edit api-health-monitor "add retry logic to the API call"
```

**System responds:**
```
Loading workflow: api-health-monitor (v1)

Analyzing change request...

I'll modify the 'fetch-health' step to add retry logic.

Proposed changes:
  ~ Step 'fetch-health'
    + Add error_handling:
      - strategy: retry
      - max_attempts: 3
      - backoff: exponential

This will:
  • Keep all existing steps unchanged
  • Only add retry configuration to fetch-health
  • Maintain all data flow

Apply changes? [y/n/details]:
```

**User says 'y':**
```
✓ Workflow updated (v1 → v2)
✓ Previous version saved as backup

Changes applied:
  ~ Modified 1 step
  + Added 0 steps
  - Removed 0 steps
```

### Journey 3: Iterative Refinement (Multi-Turn)

**User wants to improve the workflow:**
```bash
$ q workflow refine api-health-monitor
```

**System enters interactive mode:**
```
Refining workflow: api-health-monitor (v2)
Current: 3 steps, monitors API health with retry logic

What would you like to improve?
  [a]dd a step
  [m]odify a step
  [r]emove a step
  [e]rror handling
  [p]arallel execution
  [d]one refining

Choice: a
```

**User chooses 'a' (add step):**
```
Where should I add the new step?
  1. Before 'fetch-health'
  2. After 'fetch-health'
  3. After 'evaluate'
  4. After 'alert'
  5. At the end

Choice: 2

What should this step do?
> log the response for debugging

Analyzing...

I'll add a logging step after 'fetch-health':
  
  Step: log-response
  Type: skill
  Skill: logger (existing)
  Inputs: 
    - message: fetch-health.response
    - level: debug

Add this step? [y/n/edit]: y

✓ Step added

Continue refining? [y/n]: n

✓ Workflow updated (v2 → v3)
```

### Journey 4: Scoped Modification

**User wants to change just one thing:**
```bash
$ q workflow edit api-health-monitor --step fetch-health "increase timeout to 30 seconds"
```

**System responds:**
```
Modifying step: fetch-health

Current configuration:
  skill: http-fetch
  timeout: (default)

Proposed change:
  ~ timeout: default → 30

Apply? [y/n]: y

✓ Updated step 'fetch-health' (v3 → v4)
```

### Journey 5: Mistake Recovery

**User makes a bad change:**
```bash
$ q workflow edit api-health-monitor "remove the alert step"
```

**System warns:**
```
⚠️  Warning: This will remove the 'alert' step

Impact analysis:
  - Step 'alert' will be removed
  - Conditional logic will be lost
  - No alerts will be sent on failure

This seems like a significant change. Are you sure? [y/n/why]: why

The 'alert' step is the core notification mechanism. Removing it means:
  • API failures won't trigger notifications
  • The workflow will only check health, not act on it
  • You might want to modify the alert instead of removing it

Still remove? [y/n/modify instead]: n

Cancelled. Workflow unchanged.
```

**User realizes mistake, tries again:**
```bash
$ q workflow edit api-health-monitor "change alert to email instead of Slack"
```

**System responds:**
```
Modifying step: send-alert (inside 'alert' conditional)

Current: Uses alert-sender skill (Slack webhook)
Proposed: Change to email-sender skill

I'll need to create a new skill: email-sender

Proposed changes:
  ~ Step 'send-alert'
    - skill: alert-sender → email-sender
  + New skill: email-sender
    - type: code_inline
    - command: sendmail

Apply? [y/n]: y

Creating skill: email-sender...
✓ Created email-sender

✓ Workflow updated (v4 → v5)
```

### Journey 6: Complex Multi-Step Change

**User wants bigger changes:**
```bash
$ q workflow edit api-health-monitor "add parallel health checks for multiple endpoints"
```

**System responds:**
```
Analyzing change request...

This is a significant change that will:
  • Convert single endpoint check to multiple endpoints
  • Add parallel execution
  • Modify data flow

I'll show you the plan before making changes.

Proposed modifications:

1. Modify context to accept multiple URLs:
   ~ context.api_url → context.api_urls (array)

2. Replace 'fetch-health' with parallel group:
   - Remove: fetch-health (single check)
   + Add: parallel-health-checks
     - Contains: fetch-health-1, fetch-health-2, fetch-health-3
     - Wait strategy: all

3. Modify 'evaluate' to handle multiple responses:
   ~ evaluate.inputs: single response → array of responses

4. Update alert condition:
   ~ condition: any endpoint unhealthy

This is a major restructuring. Would you like to:
  [v]iew full diff
  [p]roceed with changes
  [i]terative mode (step by step)
  [c]ancel

Choice: i
```

**User chooses iterative mode:**
```
Let's do this step by step.

Step 1/4: Modify context for multiple URLs
  ~ context.api_url → context.api_urls

Apply this change? [y/n/skip]: y
✓ Applied

Step 2/4: Add parallel health checks
  + parallel-health-checks group
    - fetch-health-1
    - fetch-health-2  
    - fetch-health-3

Apply this change? [y/n/skip]: y
✓ Applied

Step 3/4: Modify evaluate step
  ~ Handle array of responses

Apply this change? [y/n/skip]: y
✓ Applied

Step 4/4: Update alert condition
  ~ Check if any endpoint failed

Apply this change? [y/n/skip]: y
✓ Applied

All changes applied!

✓ Workflow updated (v5 → v6)
✓ Backup saved: api-health-monitor.v5.json

Summary:
  ~ Modified 2 steps
  + Added 1 parallel group (3 steps)
  - Removed 1 step
```

### Journey 7: Reviewing History

**User wants to see what changed:**
```bash
$ q workflow history api-health-monitor
```

**System shows:**
```
Workflow: api-health-monitor

v6 (current) - 2 minutes ago
  Added parallel health checks for multiple endpoints
  
v5 - 5 minutes ago
  Changed alert from Slack to email
  
v4 - 10 minutes ago
  Increased fetch timeout to 30s
  
v3 - 15 minutes ago
  Added logging step after fetch
  
v2 - 20 minutes ago
  Added retry logic to API call
  
v1 - 25 minutes ago
  Initial creation

View diff between versions? [y/n]: y

Compare which versions? [from] [to]: v1 v6
```

**System shows diff:**
```
Changes from v1 to v6:

Context:
  ~ api_url → api_urls (array)

Steps:
  - fetch-health (single)
  + parallel-health-checks (group)
    + fetch-health-1
    + fetch-health-2
    + fetch-health-3
  + log-response
  ~ evaluate (handles multiple responses)
  ~ send-alert (email instead of Slack)

Error Handling:
  + fetch-health-*: retry (3 attempts, exponential backoff)

Skills Created:
  + health-checker
  + alert-sender → email-sender
  + logger
```

### Journey 8: Rollback

**User wants to undo changes:**
```bash
$ q workflow rollback api-health-monitor --to v4
```

**System responds:**
```
⚠️  Rolling back from v6 to v4

This will undo:
  v6: Parallel health checks
  v5: Email alerts

You'll lose:
  - Parallel execution
  - Multiple endpoint support
  - Email integration

You'll restore:
  - Single endpoint check
  - Slack alerts
  - 30s timeout

Proceed? [y/n/save-current]: save-current

✓ Saved current version as: api-health-monitor-parallel-backup.json
✓ Rolled back to v4

Current workflow is now at v4.
You can restore v6 from: api-health-monitor-parallel-backup.json
```

### Journey 9: Branching Workflow

**User wants to try something without losing current version:**
```bash
$ q workflow branch api-health-monitor experimental-alerts
```

**System responds:**
```
Creating branch: experimental-alerts (from v4)

✓ Created: api-health-monitor-experimental-alerts.json

You can now edit this branch without affecting the main workflow:
  q workflow edit api-health-monitor-experimental-alerts "try different alert methods"

To merge back:
  q workflow merge experimental-alerts into api-health-monitor
```

### Journey 10: Collaborative Workflow

**User wants to share and get feedback:**
```bash
$ q workflow export api-health-monitor
```

**System responds:**
```
Exported workflow package:
  api-health-monitor-package.zip
  
Contains:
  - api-health-monitor.json (workflow)
  - health-checker.json (skill)
  - email-sender.json (skill)
  - README.md (usage instructions)

Share this package with others:
  q workflow import api-health-monitor-package.zip
```

## Key UX Principles

### 1. **Context Awareness**
- System remembers what workflow you're working on
- Understands "the alert step" without needing IDs
- Tracks your intent across multiple edits

### 2. **Progressive Disclosure**
- Simple changes are simple: `q workflow edit name "change"`
- Complex changes offer guidance: iterative mode, step-by-step
- Always show impact before applying

### 3. **Safety First**
- Automatic versioning (no manual v1, v2 naming)
- Warnings for destructive changes
- Easy rollback
- Backup before major changes

### 4. **Conversational**
- Natural language for changes
- System explains what it will do
- User can ask "why" at any point
- Interactive refinement mode for exploration

### 5. **Transparent**
- Always show what changed
- Diff view available
- History tracking
- Impact analysis

### 6. **Forgiving**
- Easy undo/rollback
- Branch for experiments
- Save current before rollback
- No destructive operations without confirmation

## Command Summary

```bash
# Creation
q workflow create "description"

# Iteration
q workflow edit <name> "change description"
q workflow edit <name> --step <step-id> "change"
q workflow refine <name>  # Interactive mode

# History & Recovery
q workflow history <name>
q workflow diff <name> --from v1 --to v3
q workflow rollback <name> --to v2

# Branching
q workflow branch <name> <branch-name>
q workflow merge <branch> into <name>

# Sharing
q workflow export <name>
q workflow import <package>

# Inspection
q workflow show <name>
q workflow validate <name>
q workflow test <name>
```

## Implementation Priority

### Phase 1: Core Iteration (Week 1-2)
- `create` command with approval flow
- `edit` command with diff preview
- Automatic versioning
- Basic rollback

### Phase 2: Interactive Refinement (Week 3)
- `refine` interactive mode
- Scoped edits (`--step` flag)
- Impact analysis
- Warning system

### Phase 3: History & Recovery (Week 4)
- `history` command
- `diff` between versions
- Enhanced rollback with save-current
- Version comparison

### Phase 4: Advanced Features (Week 5+)
- Branching workflows
- Export/import packages
- Collaborative features
- Workflow testing

## Success Metrics

- User can iterate on workflow without fear of breaking it
- 90%+ of edits are surgical (not full rewrites)
- Users understand what will change before applying
- Rollback used < 10% of the time (changes are right first time)
- Average workflow reaches v5+ (shows active iteration)
