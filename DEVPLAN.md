# KnowledgeGATEunlocker — Development Plan

## Build Order

**CLI FIRST.** Everything else wraps around it.

```
Phase 1: CLI (kgate)
    ↓
Phase 2: Core library (shared logic)
    ↓
Phase 3: File watcher daemon
    ↓
Phase 4: Tauri tray app (uses library)
    ↓
Phase 5: Dashboard UI
```

---

## Project Structure

```
exambuilder/
├── crates/
│   ├── kgate/                 # CLI binary — BUILD THIS FIRST
│   │   ├── src/
│   │   │   └── main.rs
│   │   └── Cargo.toml
│   ├── kgate-core/            # Shared library
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── db/
│   │   │   │   ├── mod.rs
│   │   │   │   ├── schema.rs
│   │   │   │   ├── models.rs
│   │   │   │   └── queries.rs
│   │   │   ├── debt/
│   │   │   │   ├── mod.rs
│   │   │   │   ├── tracker.rs
│   │   │   │   └── config.rs
│   │   │   ├── exam/
│   │   │   │   ├── mod.rs
│   │   │   │   ├── parser.rs
│   │   │   │   ├── grader.rs
│   │   │   │   └── generator.rs
│   │   │   └── gamification/
│   │   │       ├── mod.rs
│   │   │       ├── xp.rs
│   │   │       ├── levels.rs
│   │   │       ├── streaks.rs
│   │   │       └── badges.rs
│   │   └── Cargo.toml
│   └── kgate-daemon/          # File watcher daemon
│       ├── src/
│       │   └── main.rs
│       └── Cargo.toml
├── tauri-app/                 # Tray + UI (later)
│   ├── src-tauri/
│   └── src/
├── docs/
│   ├── CLAUDE.md
│   ├── SKILL.md
│   └── ROADMAP.md
├── migrations/                # SQLite migrations
│   └── 001_initial.sql
├── Cargo.toml                 # Workspace
├── shell.nix
├── flake.nix
├── deploy.sh
└── README.md
```

---

## Phase 1: CLI (`kgate`)

### 1.0 — Project Setup
```
[ ] Create Cargo workspace
[ ] Create kgate crate
[ ] Create kgate-core crate
[ ] shell.nix with rust toolchain
[ ] Verify: cargo build works
```

### 1.1 — Basic Commands
```
kgate init                     # Initialize DB + data dirs
kgate status                   # Show debt, profile, active project
kgate debt                     # Detailed debt breakdown
kgate debt add <type> [desc]   # Manually add debt (for testing)
kgate debt clear <amount>      # Manually clear debt (for testing)
```

### 1.2 — Project Commands
```
kgate project list             # List all tracked projects
kgate project add <path>       # Register a project
kgate project set <hash>       # Set active project
kgate project info [hash]      # Show project details
```

### 1.3 — Exam Commands
```
kgate exam list                # List sprints for active project
kgate exam show <sprint>       # Show questions for a sprint
kgate exam take <sprint>       # Interactive exam mode
kgate exam answer <sprint> <answers...>  # Submit answers directly
kgate exam grade <sprint>      # Show results for a sprint
```

### 1.4 — Profile Commands
```
kgate profile                  # Show XP, level, streak, badges
kgate profile history          # Recent activity
kgate profile export           # Export to JSON
```

### 1.5 — Parse Commands
```
kgate parse knowledge <file>   # Parse KNOWLEDGE_*.md, show debt items
kgate parse qa <file>          # Parse QA_*.md, show quiz bank
kgate parse exam <file>        # Parse EXAM_*.md, import sprints
```

---

## Phase 2: Core Library (`kgate-core`)

### 2.1 — Database Layer

```rust
// models.rs
pub struct Project { id, full_hash, path, name, created_at, last_active }
pub struct DebtEntry { id, project_id, action, weight, description, timestamp }
pub struct Sprint { id, project_id, number, topic, questions, status, ... }
pub struct Profile { total_xp, level, streak, best_streak, sprints_passed }
pub struct Badge { id, name, description, icon, rarity, unlocked_at }

// queries.rs
pub fn init_db(path: &Path) -> Result<SqlitePool>
pub fn get_or_create_project(pool: &Pool, path: &str) -> Result<Project>
pub fn add_debt(pool: &Pool, project_id: &str, action: &str, weight: i32) -> Result<()>
pub fn get_debt(pool: &Pool, project_id: &str) -> Result<i32>
pub fn clear_debt(pool: &Pool, project_id: &str, amount: i32) -> Result<()>
pub fn upsert_sprint(pool: &Pool, sprint: &Sprint) -> Result<()>
pub fn record_attempt(pool: &Pool, sprint_id: i32, score: i32) -> Result<AttemptResult>
pub fn update_profile(pool: &Pool, xp: i32, streak_delta: i32) -> Result<Profile>
pub fn unlock_badge(pool: &Pool, badge_id: &str) -> Result<()>
```

### 2.2 — Debt Tracker

```rust
pub struct DebtConfig {
    pub threshold: i32,           // default 10
    pub weights: HashMap<String, i32>,
    pub clear_per_sprint: i32,    // default 3
}

pub fn calculate_debt_weight(action: &str, config: &DebtConfig) -> i32
pub fn is_locked(debt: i32, config: &DebtConfig) -> bool
pub fn debt_after_sprint(current: i32, config: &DebtConfig) -> i32
```

### 2.3 — Parsers

```rust
// KNOWLEDGE parser
pub struct KnowledgeEntry { topic, timestamp, context, key_points, file_refs }
pub fn parse_knowledge(content: &str) -> Result<Vec<KnowledgeEntry>>

// QA parser
pub struct QATranscript { question, answer, deep_dive }
pub struct QuizQuestion { question, qtype, options, correct, catches }
pub fn parse_qa(content: &str) -> Result<(Vec<QATranscript>, Vec<QuizQuestion>)>

// EXAM parser
pub struct ParsedSprint { number, topic, questions, answer_key }
pub fn parse_exam(content: &str) -> Result<Vec<ParsedSprint>>
```

### 2.4 — Grader

```rust
pub enum Answer {
    MC(char),                    // 'A', 'B', 'C', 'D'
    Open(String),
    CodeTrace(String),
}

pub struct GradeResult {
    pub correct: bool,
    pub xp_earned: i32,
    pub feedback: String,
}

pub struct SprintResult {
    pub passed: bool,
    pub score_percent: i32,
    pub total_xp: i32,
    pub question_results: Vec<GradeResult>,
    pub attempt_number: i32,
    pub hint_or_answer: HintLevel,
}

pub fn grade_sprint(sprint: &Sprint, answers: &[Answer]) -> SprintResult
```

### 2.5 — Gamification

```rust
pub fn calculate_level(xp: i32) -> i32
pub fn xp_for_level(level: i32) -> i32
pub fn check_badge_unlocks(profile: &Profile, event: &str) -> Vec<Badge>
pub fn update_streak(current: i32, passed: bool) -> i32
```

---

## Phase 3: File Watcher Daemon (`kgate-daemon`)

```
[ ] Watch ~/gitZ/.knowledge-gate/projects/
[ ] On KNOWLEDGE_*.md change → parse, update debt
[ ] On QA_*.md change → parse, store quiz bank
[ ] On EXAM_*.md change → parse, import sprints
[ ] Debounce rapid changes (100ms)
[ ] Log to file for debugging
[ ] Systemd user service file
```

---

## Phase 4: Tauri Tray App

```
[ ] System tray icon (green/yellow/red)
[ ] Tray menu (status, take exam, settings)
[ ] Uses kgate-core library
[ ] Notifications via native system
[ ] Minimal — just tray, no dashboard yet
```

---

## Phase 5: Dashboard UI

```
[ ] Svelte frontend
[ ] Dashboard view (XP, debt, streak)
[ ] Exam view (take exam in UI)
[ ] Profile view (badges, history)
[ ] Uses Tauri IPC to kgate-core
```

---

## SQLite Schema

```sql
-- migrations/001_initial.sql

CREATE TABLE projects (
    id TEXT PRIMARY KEY,
    full_hash TEXT NOT NULL,
    path TEXT NOT NULL UNIQUE,
    name TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    last_active TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE debt_log (
    id INTEGER PRIMARY KEY,
    project_id TEXT NOT NULL REFERENCES projects(id),
    action TEXT NOT NULL,
    weight INTEGER NOT NULL,
    description TEXT,
    timestamp TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE debt_current (
    project_id TEXT PRIMARY KEY REFERENCES projects(id),
    total INTEGER NOT NULL DEFAULT 0,
    last_updated TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE sprints (
    id INTEGER PRIMARY KEY,
    project_id TEXT NOT NULL REFERENCES projects(id),
    sprint_number INTEGER NOT NULL,
    topic TEXT NOT NULL,
    questions_json TEXT NOT NULL,
    answer_key_json TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'pending',
    best_score INTEGER,
    attempts INTEGER DEFAULT 0,
    xp_available INTEGER NOT NULL,
    xp_earned INTEGER DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    passed_at TEXT,
    UNIQUE(project_id, sprint_number)
);

CREATE TABLE profile (
    id INTEGER PRIMARY KEY CHECK (id = 1),
    total_xp INTEGER NOT NULL DEFAULT 0,
    level INTEGER NOT NULL DEFAULT 1,
    current_streak INTEGER NOT NULL DEFAULT 0,
    best_streak INTEGER NOT NULL DEFAULT 0,
    sprints_passed INTEGER NOT NULL DEFAULT 0,
    last_activity TEXT
);

CREATE TABLE badges (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    description TEXT NOT NULL,
    icon TEXT,
    rarity TEXT DEFAULT 'common',
    unlocked_at TEXT,
    project_id TEXT REFERENCES projects(id)
);

CREATE TABLE skills (
    id TEXT PRIMARY KEY,
    category TEXT NOT NULL,
    name TEXT NOT NULL,
    parent_id TEXT REFERENCES skills(id),
    xp_required INTEGER NOT NULL,
    xp_current INTEGER DEFAULT 0,
    unlocked INTEGER DEFAULT 0,
    unlocked_at TEXT
);

-- Initialize singleton profile
INSERT INTO profile (id) VALUES (1);
```

---

## CLI Output Examples

### `kgate status`
```
╭─────────────────────────────────────╮
│  KnowledgeGATEunlocker              │
├─────────────────────────────────────┤
│  Project: exambuilder (a3f8b2c1)    │
│  Debt: 7/10 ██████████░░░░ WARNING  │
│                                     │
│  Profile:                           │
│  Level 2: Config Wrangler           │
│  XP: 45/80 ████████░░░░░░           │
│  Streak: 3 🔥                       │
│                                     │
│  Pending: 2 sprints                 │
╰─────────────────────────────────────╯
```

### `kgate exam take 1`
```
Sprint 1: NixOS Basics
━━━━━━━━━━━━━━━━━━━━━━

Q1. [RECALL] ⭐ — 10 XP
Which command rebuilds NixOS and switches WITHOUT adding a boot entry?

  A) nixos-rebuild switch
  B) nixos-rebuild test
  C) nixos-rebuild boot
  D) nixos-rebuild dry-activate

Your answer: _
```

### `kgate profile`
```
╭─────────────────────────────────────╮
│  Level 2: Config Wrangler           │
│  XP: 45/80 ████████░░░░░░           │
├─────────────────────────────────────┤
│  Stats:                             │
│  Sprints passed: 4                  │
│  Current streak: 3 🔥               │
│  Best streak: 5                     │
├─────────────────────────────────────┤
│  Badges:                            │
│  🏅 First Sprint                    │
│  🏅 Git Basics                      │
│  🏅 NixOS Initiate                  │
╰─────────────────────────────────────╯
```

---

## Milestones

| # | Deliverable | Definition of Done |
|---|-------------|-------------------|
| M1 | Project scaffold | Workspace builds, shell.nix works |
| M2 | DB + models | Schema runs, CRUD operations work |
| M3 | CLI skeleton | `kgate status` shows mock data |
| M4 | Parsers | `kgate parse *` works on test files |
| M5 | Grader | `kgate exam take` grades correctly |
| M6 | Full CLI | All commands work end-to-end |
| M7 | Daemon | File watcher updates DB on changes |
| M8 | Tray app | Icon shows debt state |
| M9 | MVP | Full loop works |

---

## Dependencies

```toml
# kgate-core/Cargo.toml
[dependencies]
sqlx = { version = "0.7", features = ["runtime-tokio", "sqlite"] }
tokio = { version = "1", features = ["full"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
chrono = { version = "0.4", features = ["serde"] }
sha2 = "0.10"
thiserror = "1"

# kgate/Cargo.toml
[dependencies]
kgate-core = { path = "../kgate-core" }
clap = { version = "4", features = ["derive"] }
tokio = { version = "1", features = ["full"] }
dialoguer = "0.11"          # Interactive prompts
console = "0.15"            # Colors + styling
indicatif = "0.17"          # Progress bars
```

---

## Next Steps

1. Create Cargo workspace
2. Set up shell.nix
3. Create kgate-core with schema
4. Build `kgate init` and `kgate status`
5. Iterate from there
