# Knowledge Gate — Feature Plan

## Vision

Transform Knowledge Gate from a functional exam system into a comprehensive learning companion with spaced repetition, achievement tracking, analytics insights, and seamless backup/export.

---

## Feature Status Matrix

### Legend
- ✅ Complete — Fully working, CLI exposed
- 🔨 Backend Only — Logic exists, no CLI
- 📦 Schema Only — Database tables exist, no logic
- ❌ Not Started — Documented but not implemented

| Feature | Status | Priority |
|---------|--------|----------|
| **Core Exam System** |
| Sprint import/take/grade | ✅ | - |
| XP, Levels, Streaks | ✅ | - |
| Knowledge debt | ✅ | - |
| Voice mode (TTS/STT) | ✅ | - |
| **Analytics** |
| Event journal logging | 🔨 | P1 |
| Daily stats aggregation | 🔨 | P1 |
| Question-level analytics | 🔨 | P2 |
| Session tracking | 🔨 | P2 |
| **Knowledge Tracking** |
| Knowledge items | 🔨 | P1 |
| Spaced repetition (SM-2) | 🔨 | P1 |
| Mastery status | 🔨 | P2 |
| Review queue | 🔨 | P1 |
| **Gamification** |
| Badges | 📦 | P2 |
| Milestones | 📦 | P2 |
| Weekly goals | 📦 | P3 |
| **Organization** |
| Study notes | 📦 | P3 |
| Tags | 📦 | P3 |
| **Export** |
| Data export | 📦 | P2 |
| **Future** |
| Walk mode (Bluetooth) | ❌ | P4 |
| Kokoro TTS | ❌ | P4 |
| Web dashboard | ❌ | P4 |

---

## P1 Features — Wire Up Existing Backend

### 1. Knowledge Review Command

**Goal**: Expose spaced repetition to users

```bash
kgatectl review              # Get 5 concepts due for review
kgatectl review --limit 10   # Get 10 concepts
```

**Implementation**:
- Backend: `db.GetKnowledgeItemsForReview()` exists
- Add: `daemon.cmdReview()` command handler
- Add: `kgatectl review` subcommand
- Format: Show concept, category, mastery score, days since seen

**Output**:
```
Knowledge items due for review:
┌───────────────────────┬────────────┬─────────┬──────────┐
│ Concept               │ Category   │ Mastery │ Due      │
├───────────────────────┼────────────┼─────────┼──────────┤
│ Unix sockets          │ networking │ 45%     │ 2d ago   │
│ Go interfaces         │ language   │ 60%     │ today    │
│ SQLite WAL mode       │ database   │ 30%     │ 3d ago   │
└───────────────────────┴────────────┴─────────┴──────────┘
```

### 2. Stats Dashboard Command

**Goal**: Show learning analytics

```bash
kgatectl stats              # Today's stats
kgatectl stats --week       # Last 7 days
kgatectl stats 2026-03-20   # Specific date
```

**Implementation**:
- Backend: `db.GetDailyStats()` exists
- Add: `daemon.cmdStats()` command handler
- Add: `kgatectl stats` subcommand

**Output**:
```
Stats for 2026-03-23:
  Sessions: 3
  Sprints: 5 attempted, 4 passed (80%)
  Questions: 15 answered, 12 correct (80%)
  XP earned: 45
  Debt: +2 added, -6 cleared
  Streak: 4 days
```

### 3. Journal View Command

**Goal**: Activity timeline

```bash
kgatectl journal           # Last 20 events
kgatectl journal --limit 50
kgatectl journal --type sprint_passed
```

**Implementation**:
- Backend: `db.GetJournalEntries()` exists
- Add: `daemon.cmdJournal()` command handler
- Add: `kgatectl journal` subcommand

**Output**:
```
Recent activity:
  10:05 sprint_passed    Sprint 3 (Architecture) — 100%, +30 XP
  10:02 sprint_completed Sprint 3 — 3/3 correct
  09:58 project_activated exambuilder
  09:55 daemon_start     v1.0.0
```

### 4. Hard Questions Command

**Goal**: Identify weak areas

```bash
kgatectl hard              # 10 hardest questions
kgatectl hard --limit 5
```

**Implementation**:
- Backend: `db.GetHardestQuestions()` exists
- Add: `daemon.cmdHard()` command handler

**Output**:
```
Hardest questions (by accuracy):
  Sprint 2 Q3: 33% (1/3 correct)
  Sprint 5 Q1: 40% (2/5 correct)
  Sprint 1 Q2: 50% (3/6 correct)
```

### 5. Knowledge Stats Command

**Goal**: Mastery overview

```bash
kgatectl knowledge         # Knowledge overview
```

**Implementation**:
- Backend: `db.GetKnowledgeStats()` exists
- Add: `daemon.cmdKnowledge()` command handler

**Output**:
```
Knowledge mastery (exambuilder):
  Total concepts: 24
  ├── Unseen:   8 (33%)
  ├── Learning: 12 (50%)
  └── Mastered: 4 (17%)
```

---

## P2 Features — Complete Gamification

### 6. Achievements System

**Goal**: Badge unlocks for motivation

```bash
kgatectl achievements      # List all achievements
kgatectl achievements --unlocked
```

**Implementation**:
1. Add unlock logic in `cmdGrade()`:
   ```go
   if result.Passed {
       d.db.CheckAndUnlockBadges(projectID)
       d.db.CheckAndUnlockMilestones()
   }
   ```

2. Add unlock triggers:
   - Perfect sprint → "Perfectionist" badge
   - First voice mode → "Voice Activated" badge
   - 100 XP → "First Steps" milestone
   - 3 streak → "On a Roll" milestone

3. Add journal event: `EventBadgeUnlocked`, `EventMilestoneReached`

4. Add TTS celebration: "Achievement unlocked: Knowledge Seeker!"

**Output**:
```
Achievements:
  🏆 Unlocked:
    ✨ First Steps — Earn 100 XP (unlocked 2d ago)
    🔥 On a Roll — 3 sprint streak (unlocked today)

  🔒 Locked:
    📚 Getting Serious — Earn 500 XP (progress: 150/500)
    🧠 Knowledge Seeker — Earn 1000 XP (progress: 150/1000)
    ⚡ Weekly Warrior — 7 sprint streak (progress: 3/7)
```

### 7. Export Command

**Goal**: Backup and portability

```bash
kgatectl export                     # Export all to JSON
kgatectl export --format csv        # CSV format
kgatectl export --journal-only      # Just activity log
kgatectl export -o backup.json      # Custom filename
```

**Implementation**:
1. Add `db.ExportAll()` method
2. Support formats: JSON, CSV
3. Track in `export_history` table
4. Include: profile, sprints, attempts, journal, knowledge

**Output**:
```json
{
  "exported_at": "2026-03-23T10:00:00Z",
  "version": "1.0.0",
  "profile": { "total_xp": 150, "level": 2, ... },
  "projects": [...],
  "sprints": [...],
  "attempts": [...],
  "journal": [...],
  "knowledge_items": [...]
}
```

### 8. Import from Backup

```bash
kgatectl import-backup backup.json
```

**Implementation**:
- Validate JSON structure
- Merge with existing data (skip duplicates)
- Update profile if imported has higher stats

---

## P3 Features — Advanced Features

### 9. Weekly Goals

**Goal**: Structured learning targets

```bash
kgatectl goals                 # Show current week goals
kgatectl goals --set sprints=5 xp=100
```

**Implementation**:
1. Add goal progress update in `UpdateProfile()`
2. Add `db.GetCurrentWeekGoal()`, `db.UpdateGoalProgress()`
3. Check goal completion on sprint pass

**Output**:
```
Weekly Goals (Mar 18 - Mar 24):
  Sprints: ████████░░ 4/5 (80%)
  XP:      ███████░░░ 70/100 (70%)
  Streak:  ██████████ 7/7 (100%) ✓ Complete!
```

### 10. Study Notes

**Goal**: Personal annotations

```bash
kgatectl note add "Remember: WAL mode requires shared memory"
kgatectl note add --sprint 3 --question 2 "Tricky: option C looks right but..."
kgatectl note list
kgatectl note search "WAL"
```

**Implementation**:
1. Add `db.AddNote()`, `db.GetNotes()`, `db.SearchNotes()`
2. Link to project, sprint, question, or knowledge item
3. Show notes after wrong answers

### 11. Tags

**Goal**: Organize sprints and projects

```bash
kgatectl tag create "networking" --color blue
kgatectl tag add networking Sprint:3
kgatectl sprints --tag networking
```

**Implementation**:
1. Add `db.CreateTag()`, `db.AddTagToItem()`, `db.GetTaggedItems()`
2. Filter sprints/projects by tag

---

## P4 Features — Future Vision

### 12. Walk Mode

**Goal**: Exercise while learning

- Bluetooth button integration
- Audio-only exam flow
- Press button = answer A, double-press = B, etc.
- GPS tracking for "study walks"

### 13. Kokoro TTS

**Goal**: Better voice quality

- Integrate Kokoro neural TTS
- Emotional voice variations
- Faster synthesis

### 14. Web Dashboard

**Goal**: Visual analytics

- Chart.js visualizations
- Progress over time
- Heatmap of activity
- Shareable stats

### 15. Mobile Companion

**Goal**: On-the-go learning

- Flutter/React Native app
- Sync with daemon via API
- Offline mode with sync

---

## Implementation Priority Queue

### Sprint 1 (Week 1)
- [ ] Add `kgatectl review` command
- [ ] Add `kgatectl stats` command
- [ ] Add `kgatectl journal` command

### Sprint 2 (Week 2)
- [ ] Add `kgatectl hard` command
- [ ] Add `kgatectl knowledge` command
- [ ] Wire up achievement unlock triggers

### Sprint 3 (Week 3)
- [ ] Add `kgatectl achievements` command
- [ ] Add badge/milestone unlock events
- [ ] Add TTS celebration for unlocks

### Sprint 4 (Week 4)
- [ ] Add `kgatectl export` command
- [ ] Add `kgatectl import-backup` command
- [ ] Track export history

### Sprint 5 (Week 5-6)
- [ ] Add `kgatectl goals` command
- [ ] Wire up weekly goal tracking
- [ ] Add goal completion notifications

### Sprint 6 (Week 6-8)
- [ ] Add `kgatectl note` commands
- [ ] Add `kgatectl tag` commands
- [ ] Show notes after wrong answers

---

## Feature Dependencies

```
Core Exam (✅)
├── Knowledge Tracking (🔨)
│   ├── Spaced Repetition (🔨)
│   │   └── Review Command (P1)
│   └── Mastery Status (🔨)
│       └── Knowledge Command (P1)
├── Analytics (🔨)
│   ├── Daily Stats (🔨)
│   │   └── Stats Command (P1)
│   └── Question Analytics (🔨)
│       └── Hard Command (P1)
├── Gamification (📦)
│   ├── Badges (📦)
│   │   └── Achievements Command (P2)
│   ├── Milestones (📦)
│   │   └── Achievements Command (P2)
│   └── Weekly Goals (📦)
│       └── Goals Command (P3)
└── Export (📦)
    └── Export Command (P2)
```

---

## Success Criteria

| Feature | Success Metric |
|---------|---------------|
| Review command | Users can see concepts due for review |
| Stats command | Users can track daily progress |
| Achievements | At least 5 badges earnable |
| Export | Full data round-trip (export → import) |
| Weekly goals | Goal completion rate visible |

---

## Non-Goals (Explicitly Out of Scope)

- Multi-user support
- Cloud sync
- Social features
- Exam generation (AI-assisted)
- Real-time multiplayer quizzes
- Payment/subscription features
