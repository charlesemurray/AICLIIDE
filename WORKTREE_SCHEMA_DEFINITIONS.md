# Worktree System - Schema Definitions

## Overview

This document defines all data structures, configuration schemas, and file formats for the parallel sessions with worktrees feature.

---

## Core Data Structures

### WorkspaceContext

**Purpose**: Represents the current workspace and its git repositories

**Location**: Runtime only (not persisted)

```rust
pub struct WorkspaceContext {
    /// Workspace root directory (current directory when Q started)
    pub root: PathBuf,
    
    /// Git repositories found in workspace
    pub repos: Vec<RepoInfo>,
    
    /// Conversation scope for this session
    pub scope: ConversationScope,
    
    /// Session name (if in a worktree session)
    pub session_name: Option<String>,
}
```

**Example**:
```json
{
  "root": "/workspace/my-project",
  "repos": [
    {
      "path": "/workspace/my-project/frontend",
      "name": "frontend",
      "base_branch": "main",
      "worktree": null,
      "has_changes": false
    },
    {
      "path": "/workspace/my-project/backend",
      "name": "backend",
      "base_branch": "develop",
      "worktree": "/workspace/my-project-feature-auth/backend",
      "has_changes": true
    }
  ],
  "scope": "Workspace",
  "session_name": "feature-auth"
}
```

---

### RepoInfo

**Purpose**: Information about a single git repository in the workspace

```rust
pub struct RepoInfo {
    /// Absolute path to repository
    pub path: PathBuf,
    
    /// Repository name (directory name)
    pub name: String,
    
    /// Base branch (branch at session start, merge target)
    pub base_branch: String,
    
    /// Created worktree path (if any)
    pub worktree: Option<PathBuf>,
    
    /// Whether this repo has uncommitted changes
    pub has_changes: bool,
}
```

---

### ConversationScope

**Purpose**: Defines where conversations are stored and what they cover

```rust
pub enum ConversationScope {
    /// Conversation covers entire workspace (default)
    Workspace,
    
    /// Conversation limited to specific repository
    Repo(PathBuf),
}
```

**Serialization**:
```json
// Workspace scope
{"type": "Workspace"}

// Repo scope
{"type": "Repo", "path": "/workspace/my-project/frontend"}
```

---

### WorkspaceInfo

**Purpose**: Workspace metadata stored in SessionMetadata

**Location**: Persisted in `.amazonq/conversation.json`

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceInfo {
    /// Workspace root directory
    pub root: PathBuf,
    
    /// Repositories that have worktrees created
    pub repos_with_worktrees: Vec<RepoWorktreeInfo>,
    
    /// Conversation scope
    pub scope: ConversationScope,
}
```

**Example**:
```json
{
  "root": "/workspace/my-project",
  "repos_with_worktrees": [
    {
      "repo_path": "/workspace/my-project/frontend",
      "repo_name": "frontend",
      "base_branch": "main",
      "worktree_path": "/workspace/my-project-feature-auth/frontend",
      "created_at": "2025-11-02T22:00:00Z"
    },
    {
      "repo_path": "/workspace/my-project/backend",
      "repo_name": "backend",
      "base_branch": "develop",
      "worktree_path": "/workspace/my-project-feature-auth/backend",
      "created_at": "2025-11-02T22:05:00Z"
    }
  ],
  "scope": {"type": "Workspace"}
}
```

---

### RepoWorktreeInfo

**Purpose**: Information about a created worktree for a repository

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepoWorktreeInfo {
    /// Original repository path
    pub repo_path: PathBuf,
    
    /// Repository name
    pub repo_name: String,
    
    /// Base branch to merge back to
    pub base_branch: String,
    
    /// Worktree path
    pub worktree_path: PathBuf,
    
    /// When worktree was created
    pub created_at: DateTime<Utc>,
}
```

---

## Extended Schemas

### SessionMetadata Extension

**Purpose**: Extend existing SessionMetadata from Session Management V2

**Location**: Persisted in `.amazonq/conversation.json`

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionMetadata {
    // Existing fields from Session Management V2
    pub version: u32,
    pub id: String,
    pub status: SessionStatus,
    pub created: OffsetDateTime,
    pub last_active: OffsetDateTime,
    pub first_message: String,
    pub name: Option<String>,
    pub file_count: usize,
    pub message_count: usize,
    pub background_task: Option<BackgroundTaskInfo>,
    pub custom_fields: HashMap<String, serde_json::Value>,
    
    // NEW: Workspace-specific information
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workspace_info: Option<WorkspaceInfo>,
}
```

**Example**:
```json
{
  "version": 1,
  "id": "feature-auth",
  "status": "Active",
  "created": "2025-11-02T22:00:00Z",
  "last_active": "2025-11-02T22:30:00Z",
  "first_message": "Add authentication to frontend and backend",
  "name": "feature/auth",
  "file_count": 5,
  "message_count": 12,
  "workspace_info": {
    "root": "/workspace/my-project",
    "repos_with_worktrees": [
      {
        "repo_path": "/workspace/my-project/frontend",
        "repo_name": "frontend",
        "base_branch": "main",
        "worktree_path": "/workspace/my-project-feature-auth/frontend",
        "created_at": "2025-11-02T22:00:00Z"
      }
    ],
    "scope": {"type": "Workspace"}
  }
}
```

---

## Configuration Schemas

### Agent Configuration Extension

**Purpose**: Allow agents to specify conversation scope

**Location**: `~/.aws/amazonq/cli-agents/<agent-name>.json`

```json
{
  "name": "frontend-agent",
  "description": "Agent for frontend development",
  "tools": ["fs_read", "fs_write"],
  
  // NEW: Conversation scope
  "conversationScope": "repo",
  "repoPath": "frontend/",
  
  // NEW: Worktree behavior
  "requiresWorktree": true
}
```

**Schema**:
```typescript
interface AgentConfig {
  // ... existing fields ...
  
  // Optional: Conversation scope
  conversationScope?: "workspace" | "repo";
  
  // Optional: Repository path (required if scope is "repo")
  repoPath?: string;
  
  // Optional: Whether this agent requires worktrees
  requiresWorktree?: boolean;
}
```

---

### Skill Metadata Extension

**Purpose**: Allow skills to specify conversation scope

**Location**: `~/.q-skills/<skill-name>.json`

```json
{
  "name": "deploy-backend",
  "description": "Deploy backend service",
  "parameters": [],
  
  // NEW: Conversation scope
  "conversationScope": "repo",
  "repoPath": "backend/",
  
  // NEW: Worktree behavior
  "requiresWorktree": true
}
```

**Schema**:
```typescript
interface SkillMetadata {
  // ... existing fields ...
  
  // Optional: Conversation scope
  conversationScope?: "workspace" | "repo";
  
  // Optional: Repository path (required if scope is "repo")
  repoPath?: string;
  
  // Optional: Whether this skill requires worktrees
  requiresWorktree?: boolean;
}
```

---

## File System Structure

### Workspace Layout

```
/workspace/my-project/              # Workspace root
├── .amazonq/                       # Workspace-level Q data
│   ├── conversation.json           # Workspace conversation
│   ├── sessions/                   # Session metadata
│   │   ├── feature-auth.json
│   │   └── bugfix-parser.json
│   ├── rules/                      # Workspace rules
│   └── prompts/                    # Workspace prompts
│
├── frontend/                       # Git repo 1
│   ├── .git/
│   ├── .cargo/                     # Build config (if worktree)
│   │   └── config.toml
│   ├── .gitignore
│   ├── target/                     # Build artifacts
│   └── src/
│
└── backend/                        # Git repo 2
    ├── .git/
    ├── .cargo/
    ├── target/
    └── src/
```

### Worktree Layout

```
/workspace/my-project-feature-auth/ # Worktree workspace
├── .amazonq/
│   └── conversation.json           # Feature conversation
│
├── frontend/                       # Worktree of frontend repo
│   ├── .cargo/
│   │   └── config.toml             # Separate build config
│   ├── target/                     # Separate build artifacts
│   └── src/
│
└── backend/                        # Worktree of backend repo
    ├── .cargo/
    ├── target/
    └── src/
```

---

## Build Configuration

### Cargo Config

**Purpose**: Configure separate build directory for each worktree

**Location**: `<worktree>/.cargo/config.toml`

**Content**:
```toml
[build]
target-dir = "target"
```

**Auto-generated**: Yes, when worktree is created

**Gitignored**: Yes, added to `.gitignore` automatically

---

## CLI Arguments

### ChatArgs Extension

**Purpose**: Allow users to specify worktree and scope options

```rust
#[derive(Parser)]
pub struct ChatArgs {
    // ... existing args ...
    
    /// Create or use a worktree with specified name
    #[arg(long)]
    pub worktree: Option<String>,
    
    /// Disable worktree creation
    #[arg(long)]
    pub no_worktree: bool,
    
    /// Session type
    #[arg(long, value_enum)]
    pub session_type: Option<SessionType>,
    
    /// Conversation scope (workspace or repo)
    #[arg(long, value_enum)]
    pub scope: Option<ConversationScope>,
    
    /// Repository path (for repo scope)
    #[arg(long)]
    pub repo: Option<PathBuf>,
}
```

**Examples**:
```bash
# Create worktree with specific name
q chat --worktree feature-auth

# Disable worktrees
q chat --no-worktree

# Repo-scoped conversation
q chat --scope repo --repo frontend/

# Feature session (auto-creates worktree)
q chat --type feature
```

---

## Session Commands

### /sessions Output Format

```
📋 Active Sessions:

Workspace: /workspace/my-project/

  • main (workspace-level)
    Repos: frontend (main), backend (develop)
    Status: Idle
    Last activity: 2 hours ago
    
  • feature/auth (workspace-level)  ← current
    Repos: frontend (worktree), backend (worktree)
    Base branches: frontend→main, backend→develop
    Status: Working
    Last activity: 10 seconds ago
    Files modified: 5
    Disk usage: 4.1 GB
    
  • bugfix/parser (repo-level: backend)
    Repo: backend (worktree)
    Base branch: develop
    Status: Waiting for approval
    Last activity: 2 minutes ago
    Disk usage: 1.8 GB
```

---

## Merge Results

### WorkspaceMergeResult

**Purpose**: Result of merging all repos in a workspace session

```rust
pub struct WorkspaceMergeResult {
    pub repos: Vec<RepoMergeResult>,
}

pub struct RepoMergeResult {
    pub repo_name: String,
    pub base_branch: String,
    pub success: bool,
    pub conflicts: Vec<PathBuf>,
    pub files_merged: usize,
}
```

**Example**:
```json
{
  "repos": [
    {
      "repo_name": "frontend",
      "base_branch": "main",
      "success": true,
      "conflicts": [],
      "files_merged": 3
    },
    {
      "repo_name": "backend",
      "base_branch": "develop",
      "success": false,
      "conflicts": ["src/auth.rs"],
      "files_merged": 2
    }
  ]
}
```

---

## Migration

### From V1 (Path-Based) to V2 (Workspace-Aware)

**Existing conversations** (stored by path):
```
Key: "/workspace/my-project"
```

**New conversations** (workspace-aware):
```
Key: "my-project/main"  (workspace-level)
Key: "my-project/frontend/main"  (repo-level)
```

**Migration strategy**:
1. Detect old path-based keys
2. Convert to workspace-aware keys
3. Maintain backward compatibility
4. Optional: Migrate on first use

---

## Validation Rules

### Workspace Detection
- Must be a directory
- May contain zero or more git repos
- Non-git directories are allowed

### Repository Detection
- Must have `.git` directory
- Must have valid git configuration
- Must be on a valid branch

### Worktree Creation
- Name must be valid git branch name
- Path must not exist
- Base branch must exist
- Sufficient disk space (warn if <5GB available)

### Conversation Scope
- Workspace scope: conversation at workspace root
- Repo scope: conversation in specified repo
- Repo path must exist and be a git repo

---

## Error Codes

```rust
pub enum WorktreeError {
    /// Workspace root is not a directory
    InvalidWorkspaceRoot(PathBuf),
    
    /// No git repos found in workspace
    NoReposFound,
    
    /// Repository not found for file
    RepoNotFound(PathBuf),
    
    /// Worktree already exists
    WorktreeExists(String),
    
    /// Insufficient disk space
    InsufficientDiskSpace { required: u64, available: u64 },
    
    /// Invalid conversation scope
    InvalidScope(String),
    
    /// Repo path not found
    RepoPathNotFound(PathBuf),
}
```

---

## Telemetry Events

### Workspace Detection
```json
{
  "event": "workspace_detected",
  "repos_count": 2,
  "has_worktrees": false
}
```

### Worktree Creation
```json
{
  "event": "worktree_created",
  "repo_name": "frontend",
  "session_name": "feature-auth",
  "lazy_created": true
}
```

### Merge Completion
```json
{
  "event": "workspace_merged",
  "repos_count": 2,
  "conflicts_count": 1,
  "success": false
}
```

---

## Version History

- **V1**: Initial design with single-repo worktrees
- **V2**: Workspace-aware with multi-repo support, lazy creation
