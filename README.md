



# Ferris Wiggum

![Ferris Wiggum](ferris_wiggum.png)

A minimal effort Dioxus fullstack implementation of Geoffrey Huntley's Ralph autonomous AI agent technique. The server manages persistent Ralph sessions (git, cursor-agent CLI), while the UI provides real-time monitoring that can connect/disconnect without affecting running sessions.

Builds off Ralph + Cursor CLI implementations:
https://github.com/agrimsingh/ralph-wiggum-cursor
https://github.com/flourishprosper/ralph-main

## Architecture

```
┌─────────────────────────────────────────────┐
│  Client Layer (Connect/Disconnect)         │
│  ├── Web (Browser)                          │
│  ├── Desktop (Native)                       │
│  └── Mobile                                 │
└────────────────┬────────────────────────────┘
                 │
┌────────────────▼────────────────────────────┐
│  Shared Layer                               │
│  ├── UI Components (packages/ui/ralph)      │
│  └── Server Functions (packages/api/ralph)  │
└────────────────┬────────────────────────────┘
                 │
┌────────────────▼────────────────────────────┐
│  Server Layer (Persistent)                  │
│  ├── Ralph Core Engine (packages/ralph)     │
│  ├── Session Management                     │
│  ├── cursor-agent CLI Integration           │
│  └── Git Operations                         │
└─────────────────────────────────────────────┘
```

## Key Features

### From ralph-main
- ✅ PRD-driven workflow with structured user stories
- ✅ Story prioritization and dependency ordering
- ✅ AGENTS.md pattern documentation
- ✅ Skills for PRD generation and conversion
- ✅ Automatic archiving of previous runs

### From ralph-wiggum-cursor
- ✅ Token tracking with WARN/ROTATE thresholds
- ✅ Gutter detection (repeated failures, file thrashing)
- ✅ Guardrails/Signs learning system
- ✅ Context health visualization
- ✅ Commit frequently, state in git

### New in Dioxus Version
- ✅ Persistent server sessions (survives UI disconnect)
- ✅ Cross-platform UI (web, desktop, mobile)

## Project Structure

```
ferris_wiggum/
├── packages/
│   ├── ralph/              # Core Ralph engine (server-only)
│   │   ├── src/
│   │   │   ├── types.rs    # Core types (Session, Story, etc.)
│   │   │   ├── session.rs  # Session state machine
│   │   │   ├── cursor.rs   # cursor-agent CLI runner
│   │   │   ├── git.rs      # Git operations
│   │   │   ├── parser.rs   # Token tracking & parsing
│   │   │   ├── signals.rs  # WARN, ROTATE, GUTTER signals
│   │   │   └── guardrails.rs # Signs learning system
│   │   └── assets/
│   │       ├── prompts/    # iteration.md, wrapup.md, rotate.md
│   │       └── skills/     # prd.md, convert.md, build.md
│   │
│   ├── api/                # Server functions
│   │   └── src/ralph.rs    # Ralph API endpoints
│   │
│   ├── ui/                 # Shared UI components
│   │   ├── src/ralph/
│   │   │   ├── session_list.rs
│   │   │   ├── session_dashboard.rs
│   │   │   ├── activity_log.rs
│   │   │   ├── token_meter.rs
│   │   │   ├── story_progress.rs
│   │   │   └── guardrails_panel.rs
│   │   └── assets/styling/ralph.css
│   │
│   └── web/                # Web client
│       └── src/views/ralph/
│           ├── dashboard.rs    # /ralph
│           ├── session.rs      # /ralph/:id
│           └── new_session.rs  # /ralph/new
│
├── ralph-main/            # Original shell script implementation (reference)
└── ralph-wiggum-cursor/   # Original cursor implementation (reference)
```

## Getting Started

### Prerequisites

- Rust 1.75+
- Dioxus CLI: `cargo install dioxus-cli`
- cursor-agent CLI
- Git

### Development

```bash
# Serve the web app
cd packages/web
dx serve


```

The app will be available at http://localhost:8080

### Usage

1. **Create a Session** - Click "New Session" and provide:
   - Project path (must be a git repository)
   - Model selection
   - Token thresholds
   - Optional branch name
2. **Start the Session** - Click "Start" to begin the Ralph loop

## How Ralph Works

Ralph implements an autonomous agent loop:

1. **Session Manager** spawns persistent sessions on the server
2. **Cursor Runner** executes cursor-agent CLI with prompts
3. **Stream Parser** tracks token usage and emits signals:
   - **WARN** at 70k tokens → wrap up current work
   - **ROTATE** at 80k tokens → fresh context
   - **GUTTER** on repeated failures → stop and report
4. **Git Operations** commit progress frequently
5. **Guardrails** learn from failures to prevent recurrence

### State Persistence

State lives in:
- **Git history** - All committed changes
- **prd.json** - User stories with passes/fails
- **.ralph/progress.md** - Learnings and accomplishments
- **.ralph/guardrails.md** - Accumulated "signs"
- **AGENTS.md** - Codebase patterns

### Context Management

The core insight: LLM context is like memory - you can `malloc()` (read files) but cannot `free()` except by starting fresh.

Ralph rotates to fresh context at 80k tokens, continuing from git history.

## API Reference

### Server Functions

```rust
// Session Management
create_session(project_path, config) -> Session
list_sessions() -> Vec<Session>
get_session(id) -> Session
start_session(id) -> Session
pause_session(id) -> Session
stop_session(id) -> Session

// PRD Management
set_prd(id, prd) -> Session
convert_prd(id, markdown) -> Prd

// Guardrails
get_guardrails(id) -> Vec<Guardrail>
add_guardrail(id, guardrail) -> ()
```

### Types

```rust
pub struct Session {
    pub id: String,
    pub project_path: String,
    pub status: SessionStatus,
    pub config: SessionConfig,
    pub prd: Option<Prd>,
    pub current_iteration: u32,
    pub token_usage: TokenUsage,
}

pub enum SessionStatus {
    Idle,
    Running { story_id: String },
    Paused,
    WaitingForRotation,
    Gutter { reason: String },
    Complete,
    Failed { error: String },
}

pub struct SessionConfig {
    pub model: String,
    pub max_iterations: u32,
    pub warn_threshold: u32,      // Default 70,000
    pub rotate_threshold: u32,    // Default 80,000
    pub branch_name: Option<String>,
    pub open_pr: bool,
}
```


## References

- [Original Ralph Technique](https://ghuntley.com/ralph/) - Geoffrey Huntley
- [Context Engineering](https://ghuntley.com/allocations/) - The malloc/free metaphor
- [Dioxus](https://dioxuslabs.com/) - Fullstack Rust UI framework

## License

MIT

## Credits

- **Original technique**: Geoffrey Huntley
- https://github.com/flourishprosper/ralph-main
- https://github.com/agrimsingh/ralph-wiggum-cursor

