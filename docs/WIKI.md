# Knowledge Gate Wiki

## Table of Contents

1. [Concepts](#concepts)
2. [Exam Format](#exam-format)
3. [Grading System](#grading-system)
4. [Voice Mode](#voice-mode)
5. [Analytics & Journal](#analytics--journal)
6. [API Reference](#api-reference)
7. [Database Schema](#database-schema)
8. [Security](#security)

---

## Concepts

### Knowledge Debt

Knowledge debt accumulates as you receive help with coding tasks. Each action adds debt:

| Action | Debt |
|--------|------|
| Concept explained | +1 |
| Architecture decision | +2 |
| Bug fix with explanation | +1 |
| New file created | +1 |
| Complex code written | +2 |
| "Why X not Y" answered | +1 |

When debt reaches the threshold (default: 10), read-only mode activates.

### Read-Only Mode

In read-only mode:
- **Blocked**: Writing files, editing code, running write commands
- **Allowed**: Reading files, searching, read-only git commands

To exit read-only mode, pass exam sprints to clear debt.

### Sprints

Sprints are short exam sessions:
- 3 questions per sprint
- 3-5 minutes target time
- 60% pass threshold (2/3 correct)
- Unlimited retakes
- Best score counts

### XP and Levels

- XP earned only on passed sprints
- Level = 1 + (total_xp / 100)
- Streaks track consecutive passed sprints

---

## Exam Format

### File Structure

```markdown
# Exam: Project Name
# Generated: YYYY-MM-DD

---

## Sprint N: Topic Name

### QN. [TIER] Difficulty — XP

Question text?

- A) Option
- B) Option
- C) Option
- D) Option

---

## Answer Key

### Sprint N

**QN. Answer: X**
Hint: Brief hint
Full: Detailed explanation
```

### Question Tiers

| Tier | Description |
|------|-------------|
| RECALL | Basic facts, definitions |
| COMPREHENSION | Understanding concepts |
| APPLICATION | Using knowledge in context |
| ANALYSIS | Breaking down problems |

### Difficulty Levels

| Difficulty | Stars | XP |
|------------|-------|-----|
| Easy | 1 | 10 |
| Medium | 2 | 10-15 |
| Challenge/Hard | 3 | 15-20 |

### Code Blocks

Include code in questions:

```markdown
### Q1. [APPLICATION] Medium — 15 XP

What does this function return?

```go
func add(a, b int) int {
    return a + b
}
```

- A) The sum of a and b
- B) The product of a and b
- C) An error
- D) Nothing
```

### Voice Compatibility

For voice mode, questions should:
- Make sense when read aloud
- Avoid "see below" or "see diagram"
- Keep code blocks short (3-5 lines)
- Use clear, unambiguous language

---

## Grading System

### Pass Threshold

Default: 60% (configurable)

For a 3-question sprint:
- 3/3 correct = 100% = PASS
- 2/3 correct = 67% = PASS
- 1/3 correct = 33% = FAIL
- 0/3 correct = 0% = FAIL

### XP Calculation

XP is awarded only on pass:
- Sum of XP values for correct answers
- No partial credit for failed sprints

### Streaks

- Streak increments on each passed sprint
- Streak resets to 0 on failed sprint
- Best streak is tracked separately

### Progressive Disclosure

| Attempt | What's Shown |
|---------|--------------|
| 1st fail | Which questions wrong + hints |
| 2nd fail | Full answers + explanations |
| 3rd+ fail | Answers stay visible |

### Debt Clearing

- Pass sprint = -3 debt (configurable)
- All sprints passed = debt reset to 0

---

## Voice Mode

### TTS Mode (`--voice`)

- Questions read aloud via piper-daemon
- You type answers normally
- Results announced

### Full Voice Mode (`--voice-full`)

- Questions read aloud
- Answers spoken via moonshine-daemon
- Press Enter or wait for timeout to submit

### Answer Recognition

Spoken answers are normalized:

| Input | Normalized |
|-------|------------|
| "Alpha", "Alfa" | A |
| "Bravo" | B |
| "Charlie" | C |
| "Delta" | D |
| "Option A", "Answer A" | A |
| "First" | A |
| "Second" | B |
| "Third" | C |
| "Fourth", "Last" | D |

### Voice Daemons

**piper-daemon** (TTS):
- Neural TTS engine
- Low latency
- Unix socket API

**moonshine-daemon** (STT):
- Local speech recognition
- Privacy-preserving
- Unix socket API

---

## Analytics & Journal

Knowledge Gate tracks comprehensive analytics for learning insights and future features.

### Activity Journal

All significant events are logged with timestamps:

| Event Type | Description |
|------------|-------------|
| daemon_start | Daemon session started |
| daemon_stop | Daemon session ended |
| project_activated | Project switched |
| exam_imported | Exam file imported |
| sprint_started | Sprint attempt begun |
| sprint_completed | Sprint finished |
| sprint_passed | Sprint passed |
| sprint_failed | Sprint failed |
| question_answered | Individual answer recorded |
| debt_added | Knowledge debt increased |
| debt_cleared | Knowledge debt reduced |
| level_up | Level increased |
| streak_updated | Streak changed |
| streak_broken | Streak reset to 0 |

### Session Tracking

Each daemon run is a session with:
- Session ID (UUID)
- Start/end timestamps
- Duration
- Commands received count
- Sprints taken/passed
- XP earned

### Knowledge Items

Individual concepts are tracked for spaced repetition:
- Concept name and category
- Times seen/correct/incorrect
- Mastery score (0.0-1.0)
- SM-2 spaced repetition scheduling
- Next review date

### Question Analytics

Per-question statistics:
- Times shown/correct/incorrect
- Average response time
- Common wrong answers (for improving questions)
- First/last shown timestamps

### Daily Statistics

Aggregated daily metrics:
- Sessions count
- Active time
- Sprints attempted/passed
- Questions answered/correct
- XP earned
- Debt added/cleared
- Streak at end of day

### Milestones

Achievement milestones:
- XP milestones (100, 500, 1000, 5000)
- Streak milestones (3, 7, 30 days)
- Sprint milestones (10, 50, 100 passed)
- Mastery milestones (10, 50 concepts)
- Special achievements (perfect sprint, voice mode)

### Export

Analytics can be exported for backup or analysis:
- Full export: all data
- Incremental: changes since last export
- Journal only: activity log

---

## API Reference

### Socket Protocol

The daemon listens on a Unix socket. Commands are newline-terminated text.

### Commands

#### health
```
Request:  health
Response: OK
```

#### status
```
Request:  status
Response: OK project=name debt=5/10 level=2 xp=150 streak=3 pending=2
```

#### project [path]
```
Request:  project
Response: OK <id> <name>

Request:  project /path/to/project
Response: OK <id> <name>
```

#### projects
```
Request:  projects
Response: OK
<id> <name>
<id> <name>
```

#### sprints
```
Request:  sprints
Response: OK
<num>\t<topic>\t<status>\t<score>\t<attempts>
```

#### sprint <num>
```
Request:  sprint 1
Response: OK <questions_json>
```

#### grade <num> <answers_json>
```
Request:  grade 1 ["A","B","C"]
Response: OK {"passed":true,"score_percent":100,"xp_earned":30,...}
```

#### import <path>
```
Request:  import /path/to/exam.md
Response: OK imported 3 sprints for projectname
```

#### speak <text>
```
Request:  speak Hello world
Response: OK
```

#### quit
```
Request:  quit
Response: OK
```

### Error Responses

All errors start with `ERR`:
```
ERR no active project
ERR invalid sprint number
ERR path not allowed
```

---

## Database Schema

### Tables

#### projects
| Column | Type | Description |
|--------|------|-------------|
| id | TEXT | 8-char hash of path |
| full_hash | TEXT | Full SHA256 of path |
| path | TEXT | Absolute path |
| name | TEXT | Directory name |
| created_at | TEXT | ISO timestamp |
| last_active | TEXT | ISO timestamp |

#### sprints
| Column | Type | Description |
|--------|------|-------------|
| id | INTEGER | Auto-increment |
| project_id | TEXT | FK to projects |
| sprint_number | INTEGER | Sprint number |
| topic | TEXT | Sprint topic |
| questions_json | TEXT | JSON array |
| answer_key_json | TEXT | JSON object |
| status | TEXT | pending/passed |
| best_score | INTEGER | Best percentage |
| attempts | INTEGER | Attempt count |
| xp_available | INTEGER | Max XP |
| xp_earned | INTEGER | Earned XP |

#### attempts
| Column | Type | Description |
|--------|------|-------------|
| id | INTEGER | Auto-increment |
| sprint_id | INTEGER | FK to sprints |
| answers_json | TEXT | User answers |
| score | INTEGER | Percentage |
| passed | BOOLEAN | Pass/fail |
| xp_earned | INTEGER | XP this attempt |
| taken_at | TEXT | ISO timestamp |

#### profile
| Column | Type | Description |
|--------|------|-------------|
| id | INTEGER | Always 1 |
| total_xp | INTEGER | Cumulative XP |
| level | INTEGER | Current level |
| current_streak | INTEGER | Active streak |
| best_streak | INTEGER | All-time best |
| sprints_passed | INTEGER | Total passed |

#### debt_current
| Column | Type | Description |
|--------|------|-------------|
| project_id | TEXT | FK to projects |
| total | INTEGER | Current debt |
| last_updated | TEXT | ISO timestamp |

#### debt_log
| Column | Type | Description |
|--------|------|-------------|
| id | INTEGER | Auto-increment |
| project_id | TEXT | FK to projects |
| action | TEXT | Action type |
| weight | INTEGER | Debt added |
| description | TEXT | Details |
| created_at | TEXT | ISO timestamp |

#### journal
| Column | Type | Description |
|--------|------|-------------|
| id | INTEGER | Auto-increment |
| timestamp | TEXT | ISO timestamp |
| event_type | TEXT | Event type |
| project_id | TEXT | FK to projects |
| sprint_id | INTEGER | FK to sprints |
| data_json | TEXT | Event payload |
| session_id | TEXT | Session UUID |

#### sessions
| Column | Type | Description |
|--------|------|-------------|
| id | TEXT | UUID |
| started_at | TEXT | ISO timestamp |
| ended_at | TEXT | ISO timestamp |
| duration_seconds | INTEGER | Session length |
| commands_received | INTEGER | Command count |
| sprints_taken | INTEGER | Sprints attempted |
| sprints_passed | INTEGER | Sprints passed |
| xp_earned | INTEGER | XP this session |
| hostname | TEXT | Machine name |
| username | TEXT | User name |
| version | TEXT | Daemon version |

#### knowledge_items
| Column | Type | Description |
|--------|------|-------------|
| id | INTEGER | Auto-increment |
| project_id | TEXT | FK to projects |
| concept | TEXT | Concept name |
| category | TEXT | Category |
| tier | TEXT | Bloom's tier |
| status | TEXT | unseen/learning/mastered |
| times_seen | INTEGER | View count |
| times_correct | INTEGER | Correct count |
| mastery_score | REAL | 0.0-1.0 |
| next_review | TEXT | Next review date |
| ease_factor | REAL | SM-2 factor |

#### question_stats
| Column | Type | Description |
|--------|------|-------------|
| id | INTEGER | Auto-increment |
| sprint_id | INTEGER | FK to sprints |
| question_number | INTEGER | Question number |
| times_shown | INTEGER | Show count |
| times_correct | INTEGER | Correct count |
| avg_response_time_ms | INTEGER | Average time |
| wrong_answers_json | TEXT | Wrong answer freq |

#### daily_stats
| Column | Type | Description |
|--------|------|-------------|
| date | TEXT | YYYY-MM-DD |
| sessions_count | INTEGER | Sessions |
| sprints_attempted | INTEGER | Attempted |
| sprints_passed | INTEGER | Passed |
| xp_earned | INTEGER | XP |
| streak_at_end | INTEGER | Streak |

#### milestones
| Column | Type | Description |
|--------|------|-------------|
| id | TEXT | Milestone ID |
| name | TEXT | Display name |
| category | TEXT | xp/streak/sprints/mastery |
| threshold | INTEGER | Target value |
| current_value | INTEGER | Progress |
| unlocked | BOOLEAN | Achieved |

---

## Security

### Socket Security

- Unix socket with 0700 permissions (owner only)
- 5-second read timeout prevents DoS
- 8KB max command length

### File Import Security

- Path allowlist: home directories, /tmp
- No symlink following (uses Lstat)
- Only .md files allowed
- Path traversal blocked

### Config Security

- Config files created with 0640 permissions
- Data directory 0750 permissions
- No secrets stored in config

### Database Security

- SQLite with WAL mode
- Parameterized queries (no SQL injection)
- Located in user home (~/.kgate/)

### Systemd Hardening

The systemd service includes:
- NoNewPrivileges=true
- ProtectSystem=strict
- ProtectHome=read-only
- PrivateTmp=true
- ProtectKernelTunables=true
