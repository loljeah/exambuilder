# Knowledge Gate — Development Plan

## Executive Summary

Based on a comprehensive codebase analysis using security, testing, observability, and resilience review skills, this document outlines the prioritized development roadmap.

---

## Current State Assessment

### What's Working
- Core exam flow: import → take → grade → XP/level/streak
- Knowledge debt tracking and clearing
- Voice mode (TTS + STT via external daemons)
- Journal event logging (14 event types)
- Daily stats aggregation
- Session tracking
- Spaced repetition calculation (SM-2 algorithm)

### What's Incomplete
- 15 database tables created but not fully utilized
- Badges/milestones/goals have schema but no CLI or triggers
- Knowledge review queue calculated but not exposed
- Analytics collected but not viewable
- Export/backup features not implemented

### Critical Issues Found

| Category | High Priority Issues | Count |
|----------|---------------------|-------|
| Security | Error message disclosure, protocol injection | 18 |
| Testing | Zero test coverage | 0 tests |
| Observability | No structured logging, no metrics export | 5 gaps |
| Resilience | Missing circuit breakers, no DB pool config | 8 gaps |

---

## Phase 1: Foundation Hardening (Week 1-2)

### 1.1 Security Fixes

**P0 — Critical**

1. **Wrap error messages** (16 locations in `server.go`)
   - Replace `"ERR " + err.Error()` with generic messages
   - Log detailed errors server-side only
   ```go
   // Before
   return "ERR " + err.Error()
   // After
   log.Error("command failed", "error", err, "cmd", cmd)
   return "ERR internal error"
   ```

2. **Fix protocol injection** (`kgatectl/main.go`)
   - Validate no newlines in command arguments
   - Or switch to length-prefixed protocol
   ```go
   if strings.Contains(path, "\n") {
       return fmt.Errorf("invalid path")
   }
   ```

3. **Tighten import path validation** (`server.go:cmdImport`)
   - Remove `/tmp` and `/home` from allowlist
   - Only allow project directory or configured ProjectsRoot
   - Use `filepath.EvalSymlinks()` to canonicalize

**P1 — High**

4. **Fix SQL string concatenation** (`knowledge.go:206-220`)
   - Replace dynamic SQL with CASE statement
   ```sql
   mastered_at = CASE WHEN ? AND status != 'mastered'
                      THEN datetime('now')
                      ELSE mastered_at END
   ```

5. **Socket race condition** (`server.go:48-55`)
   - Set permissions before creating socket
   - Or use atomic socket creation

### 1.2 Database Resilience

**P0 — Critical**

1. **Add connection pool configuration** (`db.go`)
   ```go
   db.SetMaxOpenConns(25)
   db.SetMaxIdleConns(5)
   db.SetConnMaxLifetime(5 * time.Minute)
   ```

2. **Add query timeouts** (all DB calls)
   ```go
   ctx, cancel := context.WithTimeout(ctx, 5*time.Second)
   defer cancel()
   row := d.QueryRowContext(ctx, sql, args...)
   ```

3. **Transaction retry for SQLITE_BUSY** (`queries.go`)
   ```go
   func withRetry(fn func() error) error {
       for i := 0; i < 3; i++ {
           err := fn()
           if err == nil || !strings.Contains(err.Error(), "BUSY") {
               return err
           }
           time.Sleep(time.Duration(10<<i) * time.Millisecond)
       }
       return fn()
   }
   ```

### 1.3 Observability Foundation

1. **Migrate to structured logging**
   - Add `log/slog` (Go 1.21+)
   - Inject session ID as field
   - Add configurable log levels via `KGATE_LOG_LEVEL`

2. **Enhance health check**
   ```go
   case "health":
       return s.healthCheck() // Returns JSON with DB, watcher status
   ```

3. **Fix silent error paths**
   - `watcher.go:44`: Log before returning nil
   - `server.go:85`: Log connection errors

---

## Phase 2: Test Foundation (Week 2-3)

### 2.1 Unit Tests (Priority Order)

1. **`internal/exam/parser_test.go`** — 10 tests
   - Sprint header parsing
   - Question extraction with code blocks
   - Answer key formats
   - Edge cases: empty, malformed, unicode

2. **`internal/exam/grader_test.go`** — 8 tests
   - Answer normalization (A, alpha, 1, first)
   - Score calculation
   - Pass threshold boundary (59% vs 60%)
   - XP calculation

3. **`internal/db/queries_test.go`** — 6 tests
   - Debt arithmetic (can't go negative)
   - Level calculation
   - Streak logic

### 2.2 Integration Tests

4. **`internal/db/db_integration_test.go`** — 12 tests
   - Migration execution (fresh DB)
   - Migration idempotency
   - Transaction rollback
   - Concurrent profile updates

5. **`internal/daemon/server_integration_test.go`** — 10 tests
   - Socket creation and permissions
   - Command parsing
   - Concurrent connections
   - Timeout handling
   - State isolation

### 2.3 Test Infrastructure

```go
// testing/testdb/testdb.go
func NewTestDB(t *testing.T) *db.DB {
    // In-memory SQLite with migrations
}

// testing/testserver/testserver.go
func NewTestServer(t *testing.T) (*daemon.Server, string) {
    // Temp socket, cleanup on test end
}
```

---

## Phase 3: Resilience Patterns (Week 3-4)

### 3.1 Circuit Breaker for Voice

```go
type CircuitBreaker struct {
    failures    int
    lastFailure time.Time
    state       string // closed, open, half-open
}

func (c *Client) Speak(text string) error {
    if c.breaker.IsOpen() {
        return ErrCircuitOpen
    }
    err := c.doSpeak(text)
    c.breaker.Record(err)
    return err
}
```

### 3.2 Graceful Degradation

- Voice unavailable → Log to stdout, continue
- Database busy → Retry with backoff
- Watcher error → Restart with exponential backoff

### 3.3 File Watcher Recovery

```go
func (w *Watcher) watchLoop() {
    for {
        select {
        case err := <-w.watcher.Errors:
            log.Error("watcher error", "error", err)
            w.restart() // Exponential backoff restart
        }
    }
}
```

---

## Phase 4: Feature Completion (Week 4-6)

### 4.1 Expose Existing Features

| Feature | CLI Command | Backend Method |
|---------|-------------|----------------|
| Knowledge review | `kgatectl review` | `db.GetKnowledgeItemsForReview()` |
| Daily stats | `kgatectl stats [date]` | `db.GetDailyStats()` |
| Hardest questions | `kgatectl hard` | `db.GetHardestQuestions()` |
| Knowledge stats | `kgatectl knowledge` | `db.GetKnowledgeStats()` |
| Journal view | `kgatectl journal [n]` | `db.GetJournalEntries()` |

### 4.2 Wire Up Gamification

**Badges & Milestones**
1. Add unlock triggers in `cmdGrade()`
2. Add CLI: `kgatectl achievements`
3. Log unlock events to journal

**Weekly Goals**
1. Add goal progress update in `UpdateProfile()`
2. Add CLI: `kgatectl goals`
3. Track actual vs target

### 4.3 Export/Backup

1. Implement `db.ExportToJSON()`
2. Add CLI: `kgatectl export [--format json|csv]`
3. Track in `export_history` table

---

## Phase 5: Polish (Week 6-8)

### 5.1 Study Notes

1. Add CRUD methods in `db/notes.go`
2. Add CLI: `kgatectl note add/list/edit/delete`
3. Link notes to knowledge items

### 5.2 Tags System

1. Add tag CRUD in `db/tags.go`
2. Add CLI: `kgatectl tag add/list/untag`
3. Filter sprints/projects by tag

### 5.3 Metrics Export

1. Add `/metrics` handler
2. Export Prometheus format
3. Add Grafana dashboard template

---

## Task Breakdown

### Immediate (This Week)

- [ ] Security: Wrap 16 error messages
- [ ] Security: Validate newlines in protocol
- [ ] DB: Add connection pool config
- [ ] Test: Create testdb helper
- [ ] Test: Write 5 parser tests

### Short Term (2 Weeks)

- [ ] Security: Tighten import path validation
- [ ] Security: Fix SQL concatenation
- [ ] DB: Add query timeouts
- [ ] Test: Complete parser + grader tests
- [ ] Observability: Add structured logging

### Medium Term (4 Weeks)

- [ ] Resilience: Voice circuit breaker
- [ ] Resilience: Watcher recovery
- [ ] Feature: Wire up knowledge review CLI
- [ ] Feature: Wire up stats CLI
- [ ] Test: Integration test suite

### Long Term (8 Weeks)

- [ ] Feature: Badges/milestones unlock
- [ ] Feature: Weekly goals tracking
- [ ] Feature: Export/backup
- [ ] Feature: Study notes
- [ ] Observability: Metrics export

---

## Success Metrics

| Metric | Current | Target |
|--------|---------|--------|
| Test coverage | 0% | 60% |
| Security issues | 18 | 0 |
| CLI feature coverage | 40% | 90% |
| P0 bugs | Unknown | 0 |
| Mean time to recover | N/A | < 5 min |

---

## Dependencies

- Go 1.21+ (for `log/slog`)
- No new external dependencies required
- Test framework: standard library `testing`
- SQLite in-memory for tests

---

## Risk Assessment

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| Breaking changes to exam format | Low | High | Parser versioning |
| Database migration failures | Medium | High | Backup before upgrade |
| Voice daemon unavailability | High | Low | Graceful degradation |
| Concurrent access bugs | Medium | Medium | Integration tests |

---

## Appendix A: Daemon Modes & Configuration Options

### Runtime Modes

| Mode | Flag/Config | Description |
|------|-------------|-------------|
| **Normal Mode** | (default) | Full daemon with tray, watcher, socket server |
| **Headless Mode** | `--no-tray` | No system tray, pure socket server |
| **Strict Mode** | `--strict` | Require exam pass before any git push/merge |
| **Lenient Mode** | `--lenient` | Log debt but don't block operations |
| **Voice Only Mode** | `--voice-only` | TTS/STT active, minimal visual output |
| **Silent Mode** | `--silent` | No TTS, visual-only feedback |
| **Focus Mode** | `--focus` | Disable notifications, streak reminders |
| **Debug Mode** | `-v` or `--verbose` | Detailed logging, trace output |
| **Dry Run Mode** | `--dry-run` | Parse exams but don't write to DB |
| **Exam-of-the-Day** | `--daily` | Only surface one sprint per project per day |

### Configurable Options (config.toml)

#### General Settings
```toml
[general]
projects_root = "~/code"            # Where to watch for exam files
db_path = "~/.local/share/kgate"    # Database location
log_level = "info"                  # debug, info, warn, error
log_format = "text"                 # text, json (for log aggregation)
socket_path = "/run/user/1000/kgate.sock"
```

#### Exam Behavior
```toml
[exam]
pass_threshold = 60                 # Percentage required to pass sprint
debt_per_concept = 1                # Debt added per concept explained
debt_threshold = 10                 # When to trigger read-only mode
auto_import = true                  # Auto-import exam files on change
import_on_startup = true            # Scan and import on daemon start
max_retakes = -1                    # -1 = unlimited, or set a cap
show_answers_after_attempt = 2      # Reveal answers after N fails
```

#### Voice Settings
```toml
[voice]
enabled = true
tts_socket = "/run/user/1000/piper.sock"
stt_socket = "/run/user/1000/whisper.sock"
speak_questions = true              # Read questions aloud
speak_results = true                # Announce pass/fail
speak_celebrations = true           # "Achievement unlocked!"
voice_speed = 1.0                   # TTS speed multiplier
voice_pitch = 1.0                   # TTS pitch adjustment
timeout_ms = 5000                   # Give up on TTS after this
fallback_silent = true              # If TTS fails, continue silently
```

#### Gamification Settings
```toml
[gamification]
xp_multiplier = 1.0                 # Global XP modifier
streak_bonus = true                 # Award bonus XP for streaks
streak_bonus_percent = 10           # Extra XP per streak day (cap at 5x)
perfect_sprint_bonus = 50           # Bonus XP for 100% sprint
level_curve = "sqrt"                # sqrt, linear, log (XP to level)
badges_enabled = true
milestones_enabled = true
weekly_goals_enabled = false        # Opt-in for weekly targets
```

#### Notification Settings
```toml
[notifications]
desktop_notify = true               # Use libnotify for alerts
streak_reminder = true              # Remind if streak about to break
reminder_time = "09:00"             # Time for streak reminder
debt_warning = true                 # Warn when approaching threshold
achievement_notify = true           # Popup on badge/milestone
summary_daily = false               # Send daily summary notification
summary_time = "20:00"              # Time for daily summary
```

#### Watcher Settings
```toml
[watcher]
enabled = true
debounce_ms = 100                   # Debounce file events
ignore_patterns = [".git", "node_modules", "__pycache__"]
exam_pattern = "exam_*.md"          # File pattern to watch
recursive = true                    # Watch subdirectories
max_depth = 5                       # Recursion limit
```

#### Knowledge Tracking Settings
```toml
[knowledge]
spaced_repetition = true            # Enable SM-2 algorithm
initial_interval = 1                # Days until first review
ease_factor_default = 2.5           # SM-2 starting ease
ease_factor_min = 1.3               # Minimum ease factor
ease_factor_max = 3.0               # Maximum ease factor
mastery_threshold = 0.8             # Score to mark as mastered
items_per_session = 10              # Review queue limit
```

#### Export Settings
```toml
[export]
auto_backup = false                 # Periodic automatic backup
backup_interval = "weekly"          # daily, weekly, monthly
backup_path = "~/.local/share/kgate/backups"
max_backups = 10                    # Rotate old backups
export_format = "json"              # json, csv
```

#### Security Settings
```toml
[security]
socket_permissions = "0600"         # Socket file mode
allowed_paths = ["~/code", "~/projects"]  # Import whitelist
max_file_size = 1048576             # 1MB max exam file
validate_symlinks = true            # Block symlink escape
```

### Environment Variables

| Variable | Description |
|----------|-------------|
| `KGATE_CONFIG` | Override config file location |
| `KGATE_LOG_LEVEL` | Override log level (debug/info/warn/error) |
| `KGATE_LOG_JSON` | Set to "1" for JSON log format |
| `KGATE_DB_PATH` | Override database location |
| `KGATE_SOCKET` | Override socket path |
| `KGATE_NO_VOICE` | Disable voice even if configured |
| `KGATE_DEBUG` | Enable debug mode |
| `KGATE_STRICT` | Enable strict mode |

### Systemd Service Options

```ini
# ~/.config/systemd/user/kgate.service
[Service]
ExecStart=/usr/bin/kgate-daemon --no-tray
Environment="KGATE_LOG_LEVEL=info"
Environment="KGATE_LOG_JSON=1"
Restart=on-failure
RestartSec=5s

[Install]
WantedBy=default.target
```

### Feature Flags (Runtime Toggleable)

Commands to toggle at runtime without restart:

```bash
kgatectl config set voice.enabled false      # Disable voice
kgatectl config set exam.debt_threshold 15   # Raise debt limit
kgatectl config set notifications.desktop_notify false
kgatectl config get                          # Show current config
kgatectl config reset                        # Reset to defaults
```

### Planned Mode Extensions

1. **Pomodoro Mode** (`--pomodoro`)
   - 25-min focus blocks
   - Force exam break after each block
   - Track pomodoros in journal

2. **Spaced Review Mode** (`--review-only`)
   - Daemon only surfaces review items
   - No new sprints, just reinforce

3. **Competition Mode** (`--compete`)
   - Leaderboard with other users
   - Timed sprints
   - Public streak display

4. **Teaching Mode** (`--teach`)
   - Generate explanations for wrong answers
   - Link to resources
   - AI-assisted concept breakdown

5. **Offline Mode** (`--offline`)
   - Queue operations for later sync
   - Work without network
   - Sync on reconnect

6. **Mobile Companion Mode**
   - Expose REST API (optional)
   - Bluetooth beacon for proximity
   - Push notifications to phone

---

## Appendix B: CLI Command Reference (Planned)

### Core Commands (Implemented)
```bash
kgatectl status         # Profile, XP, level, streak, debt
kgatectl import <path>  # Import exam file
kgatectl take <n>       # Take sprint N
kgatectl answer <ans>   # Submit answer(s)
kgatectl quit           # Quit current sprint
kgatectl health         # Daemon health check
kgatectl voice <text>   # TTS speak
```

### Analytics Commands (P1)
```bash
kgatectl review [--limit N]           # Knowledge items due
kgatectl stats [--week|--month|DATE]  # Learning stats
kgatectl journal [--limit N] [--type] # Activity log
kgatectl hard [--limit N]             # Hardest questions
kgatectl knowledge                    # Mastery overview
```

### Gamification Commands (P2)
```bash
kgatectl achievements [--unlocked|--locked]
kgatectl badges
kgatectl milestones
```

### Export Commands (P2)
```bash
kgatectl export [--format json|csv] [-o file]
kgatectl import-backup <file>
kgatectl backup                       # Manual backup
```

### Goals & Notes Commands (P3)
```bash
kgatectl goals [--set KEY=VAL]
kgatectl note add "text" [--sprint N] [--question Q]
kgatectl note list [--search term]
kgatectl note delete <id>
```

### Organization Commands (P3)
```bash
kgatectl tag create <name> [--color]
kgatectl tag add <tag> <item>
kgatectl tag remove <tag> <item>
kgatectl tag list
kgatectl sprints [--tag] [--project]
```

### Config Commands (New)
```bash
kgatectl config get [key]
kgatectl config set <key> <value>
kgatectl config reset [key]
kgatectl config path                  # Show config file location
```

### Debug Commands
```bash
kgatectl debug dump-db               # Export raw DB tables
kgatectl debug trace <command>       # Trace command execution
kgatectl debug replay <journal-id>   # Replay a journal event
kgatectl debug reset-streak          # Testing: reset streak
kgatectl debug add-xp <amount>       # Testing: add XP
```
