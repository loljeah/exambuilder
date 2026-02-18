# KnowledgeGATEunlocker — Feature Roadmap

## Vision

A gamified, ADHD-optimized knowledge verification system that **celebrates mastery** without **prescribing direction**. The system observes what you build, extracts knowledge, generates exams, and rewards understanding — but never tells you what to learn next.

**Core Philosophy:**
- Retrospective badges, not prescriptive gates
- Questions catch "fly-over readers" with hands in the cookie jar
- Instant micro-wins for dopamine rhythm
- No time pressure guilt, no failure shaming

---

## The Gate: What Gets Blocked

**NOT git pushes.** Git is free. Push whenever.

**What gets blocked: Claude's ability to write more code.**

When knowledge debt exceeds threshold, Claude enters **read-only mode**:
- Can explain, search, answer questions
- Can NOT write/edit code until debt is cleared
- Clears debt by passing exam sprints

### Read-Only Mode: Full Lockdown

In read-only mode, Claude is BLOCKED from:

```
BLOCKED — No code generation of any kind:
├── Write tool           # no new files
├── Edit tool            # no modifications
├── Bash with:
│   ├── echo "..." > file
│   ├── cat << EOF
│   ├── printf
│   ├── sed -i
│   ├── awk (writing)
│   ├── tee
│   ├── curl | sh
│   └── any file creation/modification
├── NotebookEdit         # no jupyter changes
└── Any creative workaround

ALLOWED — Explanation only:
├── Read tool            # view files
├── Grep/Glob            # search
├── Bash (read-only)     # ls, cat, git status, etc.
├── WebSearch/WebFetch   # research
└── Answering questions  # verbal explanation
```

**Response style in read-only mode:** Haiku-minimal. No lengthy explanations that could be copy-pasted as code. Direct user to exam.

Example read-only response:
```
⚠️ KNOWLEDGE DEBT: 12/10 — Read-only mode active

I can't write code right now.

To clear debt: answer your exam sprints
  → EXAM_a3f8b2c1.md (3 sprints pending)

Your question about nginx config:
The upstream block defines backend servers.
Take the exam to unlock editing.
```

### Knowledge Debt System

Debt accumulates as Claude helps you build:

| Action | Debt Weight |
|--------|-------------|
| Concept explained | +1 |
| Architecture decision | +2 |
| Bug fix with explanation | +1 |
| New file created | +1 |
| Complex code written | +2 |
| "Why X not Y" answered | +1 |

**Threshold:** 10 concepts = read-only mode triggered

**Clearing debt:**
- Pass exam sprint = -3 debt per sprint
- All sprints passed = debt reset to 0

### Why This Works for ADHD

- **No punishment for building** — push code anytime
- **Natural break point** — exam is a context switch, good for focus reset
- **Prevents scatter-brain abuse** — can't infinitely copy-paste without understanding
- **Read-only still helps** — Claude can explain what you built, just can't add more chaos

---

## Architecture Decisions

### Tech Stack
| Component | Choice | Rationale |
|-----------|--------|-----------|
| **App Framework** | Rust + Tauri | Native performance, small binary, cross-platform |
| **Database** | SQLite | Mature tooling, single file, decades stable |
| **Data Export** | JSON/MD | Git-friendly diffable exports alongside DB |
| **Claude Integration** | File watcher + CLI | Watch KNOWLEDGE/EXAM files, CLI for grading |
| **Tray Mode** | Always running | Persistent icon, instant access to stats |

### File System
```
~/gitZ/.knowledge-gate/
├── db/
│   └── knowledge-gate.db        # SQLite — all structured data
├── export/                      # Git-friendly JSON exports
│   ├── profile.json             # Global XP, level, achievements
│   ├── skills.json              # Skill tree state
│   └── projects/
│       └── <hash>.json          # Per-project progress
├── projects/
│   ├── KNOWLEDGE_<hash>.md      # Extracted knowledge base
│   ├── QA_<hash>.md             # Q&A transcript + generated quiz bank
│   └── EXAM_<hash>.md           # Active exam sprints
└── config.toml                  # User preferences
```

### ID System
- **Format:** SHA hash of project path (first 8 chars)
- **Example:** `KNOWLEDGE_a3f8b2c1.md`
- **UI displays:** Human-readable project name (from DB lookup)
- **Collision handling:** Full hash stored in DB, short hash for filenames

---

## File Pipeline

```
Claude Conversation
        ↓
   [Continuous extraction]
        ↓
KNOWLEDGE_<hash>.md
   - Key concepts explained
   - Config decisions documented
   - Architecture notes
   - "Why X not Y" captured
        ↓
   [Quiz generation]
        ↓
QA_<hash>.md
   - Section 1: Literal Q&A transcript
   - Section 2: Generated quiz bank (plausible distractors)
        ↓
   [Exam assembly]
        ↓
EXAM_<hash>.md
   - Sprint-based micro-exams
   - Context-dependent questions (YOUR config)
   - Code tracing questions
   - Explain-to-prove follow-ups
```

---

## Gamification System

### Global Profile (Transfers Across Projects)
- **XP:** Earned from correct answers
- **Level:** Progression tiers (Packet Pusher → Stack Architect)
- **Streak:** Consecutive sprints completed
- **Achievements:** Collected badges

### Skill Trees
- **Structure:** Tech-based + Concept-based + Auto-detected + User-created
- **Unlock Logic:** ADHD-friendly instant micro-wins
  - Every correct answer = immediate visual feedback + XP
  - No prerequisites blocking progress
  - Badges celebrate proof of mastery (retrospective, not prescriptive)

### Rewards (Retrospective Only)
- Visual badges + titles ("Git Wizard", "Docker Captain")
- Achievement journal — pure celebration, zero gatekeeping
- Surprise "hidden achievement" discoveries for variety

### Anti-Gaming Question Design
1. **Plausible distractors** — Wrong answers look right to skimmers
2. **Context-dependent** — Answer based on YOUR config, not generic
3. **Code tracing** — "What happens when X runs?" — must mentally execute
4. **Explain-to-prove** — MC + "why did you pick that?" follow-up
5. **ADHD-balanced** — Challenging but not overwhelming

---

## UI/UX Design

### System Tray App
- **Main View:** Dashboard stats (XP, level, streak, recent activity)
- **Notifications:** Sound + visual (satisfying chime + animation)
- **Theme:** System follows (match OS light/dark preference)

### Dashboard Elements
```
┌─────────────────────────────────────┐
│  KnowledgeGATEunlocker        [—][×]│
├─────────────────────────────────────┤
│  Level 3: System Operator           │
│  ████████████░░░░ 142/200 XP        │
│                                     │
│  🔥 Streak: 5 sprints               │
│  📊 Today: +45 XP                   │
│                                     │
│  Recent Unlocks:                    │
│  🏅 Git Basics                      │
│  🏅 NixOS Fundamentals              │
│                                     │
│  Projects:                          │
│  ✓ exambuilder — CLEARED            │
│  ⚠️ homelab — 2/4 sprints           │
│  ⬜ newproject — no exam yet        │
└─────────────────────────────────────┘
```

---

## Development Phases

### Phase 0: Foundation (Current)
- [x] CLAUDE.md with teachANDexam rules
- [x] SKILL.md full specification
- [x] Global pre-push hook
- [x] deploy.sh installer
- [ ] Project structure setup (Tauri + Rust)
- [ ] SQLite schema design
- [ ] File watcher skeleton

### Phase 1: MVP — Full Loop Minimal
**Goal:** Generate → Quiz → Grade → Track with bare UI

- [ ] **File Generation**
  - [ ] KNOWLEDGE extraction rules for Claude
  - [ ] QA generation (transcript + quiz bank)
  - [ ] EXAM assembly from QA bank
  - [ ] File watcher detecting changes

- [ ] **Grading Engine**
  - [ ] CLI grader (`kgate grade <sprint>`)
  - [ ] SQLite tracking (answers, scores, timestamps)
  - [ ] Git-friendly JSON export after each grade

- [ ] **Tray App**
  - [ ] System tray icon (Tauri)
  - [ ] Dashboard view (stats display)
  - [ ] Basic XP/level calculation
  - [ ] Native notifications (achievement sounds)

- [ ] **Knowledge Debt Tracking**
  - [ ] Debt counter in KNOWLEDGE file header
  - [ ] Read-only mode enforcement in CLAUDE.md rules
  - [ ] Debt warning notifications at threshold - 3

### Phase 2: Voice Mode
**Goal:** Hands-free exam taking

- [ ] TTS question reading
- [ ] STT answer capture
- [ ] LLM grading of spoken answers
- [ ] Voice navigation ("next question", "repeat")
- [ ] Push-to-talk + wake word options

### Phase 3: Skill Tree UI
- [ ] Interactive node graph visualization
- [ ] Tech-based trees (Git, Docker, NixOS, etc.)
- [ ] Auto-detected skills from project stack
- [ ] User-created custom nodes
- [ ] Unlock animations

### Phase 4: Achievement System
- [ ] Badge collection gallery
- [ ] Titles and profile customization
- [ ] Hidden achievement discoveries
- [ ] Streak celebrations
- [ ] Progress milestones

### Phase 5: Polish & Expand
- [ ] Multi-project dashboard
- [ ] Custom themes/skins
- [ ] Cloud sync (optional)
- [ ] Mobile companion app
- [ ] Community badge sharing

---

## Claude Code Integration Spec

### Continuous Knowledge Extraction
Claude should append to `KNOWLEDGE_<hash>.md` when:
- Explaining a concept
- Making an architecture decision
- Debugging (what was wrong + why)
- Answering "why X instead of Y"

**Format:**
```markdown
## [Topic] — [Timestamp]

**Context:** <what triggered this>

**Key Points:**
- Point 1
- Point 2

**File References:**
- `path/to/file.nix:42` — <what's there>
```

### QA Generation
Claude should append to `QA_<hash>.md`:

**Section 1 — Transcript:**
```markdown
### Q: <User's literal question>
**A:** <Claude's answer summary>
**Deep dive:** <optional extended explanation>
```

**Section 2 — Quiz Bank:**
```markdown
### Generated Q: <question derived from conversation>
**Type:** MC | Open | Code-trace
**Plausible distractors:** A, B, C, D
**Correct:** <answer>
**Catches:** <what misconception this catches>
```

### Exam Assembly Trigger
Generate/update `EXAM_<hash>.md` when:
- User says: "exam me", "quiz me", "test me", "generate exam"
- Milestone detected (new service, major config change)
- Pre-merge/pre-deploy phase

---

## Config File (`config.toml`)

```toml
[general]
projects_root = "~/gitZ"
data_dir = "~/gitZ/.knowledge-gate"

[knowledge_debt]
threshold = 10              # concepts before read-only mode
debt_per_sprint_cleared = 3 # debt reduction per passed sprint
weights.concept_explained = 1
weights.architecture_decision = 2
weights.bug_fix = 1
weights.new_file = 1
weights.complex_code = 2
weights.why_not_question = 1

[notifications]
sound = true
volume = 0.7
achievement_sound = "level-up.wav"
debt_warning_at = 7         # warn when approaching threshold

[appearance]
theme = "system"  # system | dark | light

[grading]
pass_threshold = 60
streak_bonus_at = 3
show_hints_on_fail = 1  # which attempt to show hints
show_answers_on_fail = 2  # which attempt to show answers

[voice]
enabled = false  # Phase 2
tts_voice = "default"
stt_mode = "push-to-talk"  # push-to-talk | wake-word
```

---

## Open Questions

1. **Backup strategy** — Auto-backup DB? Frequency?
2. **Multi-machine sync** — Git repo for exports? Cloud service?
3. **Achievement rarity** — Should some badges be rare/legendary?
4. **Social features** — Share achievements? Leaderboards? (privacy concerns)
5. **Offline mode** — How to handle LLM grading without network?

---

## Success Metrics

- **Primary:** User can push code with confidence they understand it
- **Secondary:**
  - Reduced "what does this do again?" moments
  - Knowledge retention after breaks
  - Reduced imposter syndrome through proof of competence
- **Anti-metrics:**
  - Time spent on exams (should be minimal, not a chore)
  - Frustration/abandonment rate
  - Gaming/cheating attempts (questions should be ungameable)
