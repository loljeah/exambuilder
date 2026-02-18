# KnowledgeGATEunlocker — Development Plan

## Project Structure

```
exambuilder/
├── src-tauri/                 # Rust backend
│   ├── src/
│   │   ├── main.rs           # Tauri entry point
│   │   ├── lib.rs            # Library exports
│   │   ├── db/
│   │   │   ├── mod.rs
│   │   │   ├── schema.rs     # SQLite schema
│   │   │   ├── models.rs     # Data models
│   │   │   └── queries.rs    # DB operations
│   │   ├── debt/
│   │   │   ├── mod.rs
│   │   │   ├── tracker.rs    # Debt accumulation logic
│   │   │   └── config.rs     # Debt weights/thresholds
│   │   ├── exam/
│   │   │   ├── mod.rs
│   │   │   ├── parser.rs     # Parse EXAM_*.md files
│   │   │   ├── grader.rs     # Grade answers
│   │   │   └── generator.rs  # Assemble exams from QA
│   │   ├── watcher/
│   │   │   ├── mod.rs
│   │   │   └── files.rs      # Watch KNOWLEDGE/QA/EXAM files
│   │   ├── gamification/
│   │   │   ├── mod.rs
│   │   │   ├── xp.rs         # XP calculations
│   │   │   ├── levels.rs     # Level thresholds
│   │   │   ├── streaks.rs    # Streak tracking
│   │   │   └── badges.rs     # Achievement definitions
│   │   ├── tray/
│   │   │   ├── mod.rs
│   │   │   └── menu.rs       # System tray menu
│   │   └── commands.rs       # Tauri IPC commands
│   ├── Cargo.toml
│   └── tauri.conf.json
├── src/                       # Frontend (Svelte/Vue/React)
│   ├── App.svelte
│   ├── components/
│   │   ├── Dashboard.svelte
│   │   ├── DebtMeter.svelte
│   │   ├── SprintCard.svelte
│   │   ├── XPBar.svelte
│   │   └── BadgeGrid.svelte
│   ├── stores/
│   │   └── state.ts
│   └── main.ts
├── cli/                       # CLI tool (optional)
│   ├── src/
│   │   └── main.rs
│   └── Cargo.toml
├── docs/
│   ├── CLAUDE.md             # Claude rules (deployed to ~/.claude/)
│   ├── SKILL.md              # Exam generation spec
│   └── ROADMAP.md            # Feature roadmap
├── deploy.sh
├── shell.nix                 # NixOS dev environment
├── flake.nix
└── README.md
```

---

## Phase 0: Project Bootstrap

### 0.1 — Nix Development Environment
```
[ ] Create shell.nix with:
    - rustc, cargo, rust-analyzer
    - nodejs, npm (for frontend)
    - tauri-cli
    - sqlite, sqlx-cli
    - watchexec (for dev)
[ ] Create flake.nix for reproducibility
[ ] Test: `nix develop` drops into working env
```

### 0.2 — Tauri Project Scaffold
```
[ ] cargo create-tauri-app exambuilder
[ ] Choose frontend: Svelte (lightest) or vanilla
[ ] Verify: `cargo tauri dev` opens window
[ ] Add system tray capability in tauri.conf.json
```

### 0.3 — SQLite Setup
```
[ ] Add sqlx dependency (async, compile-time checked)
[ ] Create initial schema (see below)
[ ] Set up migrations directory
[ ] Test: create DB, run migrations
```

---

## Phase 1: Core Data Layer

### 1.1 — SQLite Schema

```sql
-- Projects table
CREATE TABLE projects (
    id TEXT PRIMARY KEY,           -- SHA hash (8 chars)
    full_hash TEXT NOT NULL,       -- Full SHA for collision check
    path TEXT NOT NULL UNIQUE,     -- Absolute path
    name TEXT NOT NULL,            -- Human-readable name
    created_at TEXT NOT NULL,
    last_active TEXT NOT NULL
);

-- Knowledge debt tracking
CREATE TABLE debt_log (
    id INTEGER PRIMARY KEY,
    project_id TEXT NOT NULL REFERENCES projects(id),
    action TEXT NOT NULL,          -- 'concept', 'architecture', etc.
    weight INTEGER NOT NULL,
    description TEXT,
    timestamp TEXT NOT NULL
);

-- Current debt per project (materialized for speed)
CREATE TABLE debt_current (
    project_id TEXT PRIMARY KEY REFERENCES projects(id),
    total INTEGER NOT NULL DEFAULT 0,
    last_updated TEXT NOT NULL
);

-- Exam sprints
CREATE TABLE sprints (
    id INTEGER PRIMARY KEY,
    project_id TEXT NOT NULL REFERENCES projects(id),
    sprint_number INTEGER NOT NULL,
    topic TEXT NOT NULL,
    questions_json TEXT NOT NULL,  -- JSON array of questions
    status TEXT NOT NULL DEFAULT 'pending',  -- pending, passed, failed
    best_score INTEGER,
    attempts INTEGER DEFAULT 0,
    xp_available INTEGER NOT NULL,
    xp_earned INTEGER DEFAULT 0,
    created_at TEXT NOT NULL,
    passed_at TEXT,
    UNIQUE(project_id, sprint_number)
);

-- Global profile
CREATE TABLE profile (
    id INTEGER PRIMARY KEY CHECK (id = 1),  -- Singleton
    total_xp INTEGER NOT NULL DEFAULT 0,
    level INTEGER NOT NULL DEFAULT 1,
    current_streak INTEGER NOT NULL DEFAULT 0,
    best_streak INTEGER NOT NULL DEFAULT 0,
    sprints_passed INTEGER NOT NULL DEFAULT 0,
    last_activity TEXT
);

-- Badges/achievements
CREATE TABLE badges (
    id TEXT PRIMARY KEY,           -- 'git_basics', 'nixos_init', etc.
    name TEXT NOT NULL,
    description TEXT NOT NULL,
    icon TEXT,                     -- emoji or icon path
    rarity TEXT DEFAULT 'common',  -- common, rare, legendary
    unlocked_at TEXT,
    project_id TEXT REFERENCES projects(id)  -- NULL = global badge
);

-- Skill tree nodes
CREATE TABLE skills (
    id TEXT PRIMARY KEY,
    category TEXT NOT NULL,        -- 'git', 'docker', 'nixos', etc.
    name TEXT NOT NULL,
    parent_id TEXT REFERENCES skills(id),
    xp_required INTEGER NOT NULL,
    xp_current INTEGER DEFAULT 0,
    unlocked INTEGER DEFAULT 0,
    unlocked_at TEXT
);
```

### 1.2 — Data Models (Rust)

```
[ ] Project struct
[ ] DebtEntry struct
[ ] Sprint struct with Question enum (MC, Open, CodeTrace)
[ ] Profile struct
[ ] Badge struct
[ ] Skill struct
```

### 1.3 — Database Operations

```
[ ] init_db() — create tables if not exist
[ ] Project CRUD
[ ] add_debt(project_id, action, weight, description)
[ ] get_current_debt(project_id) -> i32
[ ] clear_debt(project_id, amount)
[ ] Sprint CRUD
[ ] record_attempt(sprint_id, score)
[ ] update_profile(xp_delta, streak_delta)
[ ] unlock_badge(badge_id)
```

---

## Phase 2: File Watcher & Parser

### 2.1 — File Watcher

```
[ ] Watch ~/gitZ/.knowledge-gate/projects/ for changes
[ ] Detect KNOWLEDGE_*.md, QA_*.md, EXAM_*.md
[ ] Parse hash from filename, lookup project
[ ] Trigger appropriate handler
```

### 2.2 — KNOWLEDGE Parser

```
[ ] Parse markdown structure:
    ## [Topic] — [Timestamp]
    **Context:** ...
    **Key Points:** ...
    **File References:** ...
[ ] Extract debt-worthy items
[ ] Update debt_log table
[ ] Update debt_current table
```

### 2.3 — QA Parser

```
[ ] Section 1: Parse Q&A transcript
[ ] Section 2: Parse generated quiz bank
[ ] Store in intermediate format for exam assembly
```

### 2.4 — EXAM Parser

```
[ ] Parse sprint headers: ## Sprint N: <Topic>
[ ] Parse questions: ### QN. [TIER] ⭐... — XP
[ ] Parse MC options: - A) through - D)
[ ] Parse answer key: ## 🔑 Answer Key
[ ] Create Sprint records in DB
```

---

## Phase 3: Grading Engine

### 3.1 — Answer Input

```
[ ] Accept answers via:
    - CLI: `kgate answer 1 A B "my open answer"`
    - Tauri IPC from UI
    - File-based (answers.json in project dir)
```

### 3.2 — Grading Logic

```
[ ] MC: exact match = full XP, wrong = 0
[ ] Open-ended: 3-point scale
    - 3 = all key points (100% XP)
    - 2 = main concept (66% XP)
    - 1 = surface awareness (33% XP)
    - 0 = wrong (0 XP)
[ ] Code-trace: compare against expected output
[ ] Calculate sprint percentage
```

### 3.3 — Scoring Flow

```
[ ] Grade sprint
[ ] If >= 70%:
    - Mark sprint PASSED
    - Award XP to profile
    - Reduce debt by 3
    - Increment streak
    - Check for badge unlocks
[ ] If < 70%:
    - Increment attempts
    - Return appropriate feedback (hints vs answers)
    - Keep streak (no penalty)
```

### 3.4 — Feedback Generation

```
[ ] Attempt 1 fail: hints only
[ ] Attempt 2 fail: full answers + explanations
[ ] Attempt 3+: answers visible, encouraging tone
[ ] Template responses (ADHD-friendly, no guilt)
```

---

## Phase 4: System Tray App

### 4.1 — Tray Icon

```
[ ] Register system tray with Tauri
[ ] Icon states:
    - Green: debt < 7, all clear
    - Yellow: debt 7-9, warning
    - Red: debt >= 10, locked
[ ] Tooltip shows current debt/threshold
```

### 4.2 — Tray Menu

```
[ ] "Dashboard" — open main window
[ ] "Current Debt: X/10"
[ ] "Take Exam" — open exam UI
[ ] Separator
[ ] "Projects" submenu
[ ] "Settings"
[ ] "Quit"
```

### 4.3 — Notifications

```
[ ] Debt warning at threshold - 3
[ ] Lockdown notification at threshold
[ ] Achievement unlocked (with sound)
[ ] Sprint passed celebration
[ ] Streak milestone (3, 5, 10...)
```

---

## Phase 5: Dashboard UI

### 5.1 — Main Dashboard

```
[ ] XP bar with level indicator
[ ] Current debt meter (visual)
[ ] Streak counter with flame icon
[ ] Recent activity feed
[ ] Project list with status
```

### 5.2 — Exam View

```
[ ] Sprint selector
[ ] Question display (one at a time for focus)
[ ] Answer input (MC buttons, text area)
[ ] Progress indicator (Q 2/3)
[ ] Submit sprint button
[ ] Results screen with XP animation
```

### 5.3 — Profile View

```
[ ] Total stats (XP, level, sprints passed)
[ ] Badge collection grid
[ ] Skill tree preview (Phase 3+)
[ ] History of achievements
```

---

## Phase 6: CLI Tool (Optional)

```
[ ] kgate status — show debt, projects, profile
[ ] kgate debt — detailed debt breakdown
[ ] kgate exam [project] — list pending sprints
[ ] kgate answer <sprint> <answers...> — submit answers
[ ] kgate export — export profile to JSON
```

---

## Technical Decisions

### Async Runtime
- tokio for async operations
- sqlx with runtime-tokio feature

### State Management
- Tauri state for shared app state
- SQLite as single source of truth
- Frontend stores sync via IPC

### File Watching
- notify crate for cross-platform file watching
- Debounce rapid changes (100ms)

### Sound
- rodio crate for achievement sounds
- Ship small WAV files in bundle

### Theming
- CSS variables for light/dark
- System preference detection via Tauri

---

## Testing Strategy

### Unit Tests
```
[ ] Debt calculation
[ ] Grading logic
[ ] XP/level formulas
[ ] Parser correctness
```

### Integration Tests
```
[ ] DB operations
[ ] File watch → parse → store flow
[ ] Grade → update profile flow
```

### Manual Testing
```
[ ] Full flow: build with Claude → debt accumulates → lockdown → exam → unlock
[ ] Tray icon state changes
[ ] Notifications fire correctly
[ ] Sound plays on achievement
```

---

## Milestones

| Milestone | Deliverable | Definition of Done |
|-----------|-------------|-------------------|
| M0 | Dev environment | `nix develop` works, Tauri window opens |
| M1 | Data layer | Schema created, CRUD operations work |
| M2 | File parsing | KNOWLEDGE/QA/EXAM files parsed correctly |
| M3 | Grading | CLI can grade a sprint, updates DB |
| M4 | Tray app | Icon shows, menu works, notifications fire |
| M5 | Dashboard | Basic stats display in window |
| M6 | MVP | Full loop: generate → quiz → grade → track |

---

## Open Decisions

1. **Frontend framework**: Svelte (lighter) vs Vue (more familiar)?
2. **Open-ended grading**: LLM call or keyword matching?
3. **Export format**: JSON only or also YAML?
4. **Portable mode**: Support running from USB/different machines?
5. **First badge set**: What achievements to ship initially?
