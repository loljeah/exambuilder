# KnowledgeGATEunlocker — User Guide

**kgate** is a gamified, ADHD-optimized knowledge verification CLI. It generates exams from your codebases, tracks your understanding through sprint-based micro-exams, and gates Claude Code's ability to write code until you prove you understand what was built.

**You built it. Can you explain it?**

---

## Table of Contents

1. [Installation](#installation)
2. [Quick Start](#quick-start)
3. [Core Concepts](#core-concepts)
4. [Commands Reference](#commands-reference)
5. [Taking Exams](#taking-exams)
6. [Generating Exams](#generating-exams)
7. [Voice Mode](#voice-mode)
8. [Gamification System](#gamification-system)
9. [Knowledge Debt System](#knowledge-debt-system)
10. [Spaced Repetition](#spaced-repetition)
11. [Knowledge Domains](#knowledge-domains)
12. [Question Harvesting](#question-harvesting)
13. [Configuration](#configuration)
14. [Self-Diagnostics](#self-diagnostics)
15. [Exam File Format](#exam-file-format)
16. [Development Setup](#development-setup)

---

## Installation

### Prerequisites

- **NixOS/nix-shell** (recommended) or a Rust toolchain (rustc, cargo)
- **SQLite** (for the local database)
- **Optional for voice mode:** espeak-ng, piper-tts, whisper-cpp

### Build from Source

```bash
# Enter the nix dev environment (handles all dependencies)
cd exambuilder
nix-shell

# Build
just build

# Install to ~/.local/bin
just install
```

After building, `kgate` is available as a CLI command.

### Deploy Claude Code Integration

The deploy script installs the Claude Code rules that enforce the knowledge debt system:

```bash
./deploy.sh
```

This installs:
- `~/.claude/CLAUDE.md` — Claude Code behavior rules (read-only mode enforcement)
- `~/.claude/skills/teachANDexam/SKILL.md` — Exam generation spec
- `~/gitZ/.knowledge-gate/` — Data directory with default `config.toml`

---

## Quick Start

```bash
# 1. Initialize the database and profile
kgate init

# 2. Register a project
kgate project add ~/gitZ/myproject

# 3. Scan for existing exam files and import them
kgate scan ~/gitZ/myproject --import

# 4. Or generate a new exam from the codebase
kgate generate ~/gitZ/myproject

# 5. See your exams and progress
kgate status

# 6. Take a sprint
kgate take exam 1 sprint 1

# 7. Check your profile
kgate profile
```

Or just run `kgate` with no arguments — it picks a random unpassed sprint and launches it.

---

## Core Concepts

### Sprints, Not Exams

Exams are broken into **sprints** — small batches of 2-3 questions on a single topic. Each sprint is designed to be completed in 3-5 minutes. Sprints are independent: you can pass them in any order, skip around, and come back later.

### Pass Threshold

You need **60%** (2 out of 3 questions) to pass a sprint. This is intentionally achievable — the goal is to verify understanding, not to gatekeep.

### Unlimited Retakes

Failed a sprint? Try again. There is no penalty for retaking. Your best score counts. The system uses **progressive disclosure**:

- **1st failed attempt:** Shows which questions you got wrong + hints
- **2nd failed attempt:** Shows full answers + explanations
- **3rd+ attempts:** Answers stay visible, keep trying without guilt

### XP and Leveling

Every correct answer earns XP. XP accumulates across all projects into a global profile with levels, streaks, badges, and achievements.

---

## Commands Reference

### No Arguments — Random Sprint

```bash
kgate
```

Picks a random unpassed sprint from any tracked project and launches it. This is the default entry point — just type `kgate` and start learning.

---

### `kgate init`

Initializes the system. Creates the database, data directories, and your Knowledge Identity.

```bash
kgate init
```

Creates:
- `~/.kgate/data/db/knowledge-gate.db` — SQLite database
- `~/.kgate/domains.toml` — Domain icon definitions
- `~/.kgate/export/`, `~/.kgate/sounds/`, `~/.kgate/bookmarks/` — Data directories
- A unique Knowledge Identity (SHA hash used as your learner ID)

Run this once before using any other command.

---

### `kgate status` / `kgate list`

Shows the full progress dashboard — all exams, sprints, profile stats.

```bash
kgate status
```

Output includes:
- All tracked projects with exam counts
- Sprint progress bars per exam (filled blocks = passed, empty = pending)
- Domain icons for each exam's topics
- XP earned vs available
- Profile card: level, XP progress bar, accuracy, combo, streaks, perfect sprints

Example:
```
  01. ○ myproject       🦀🌐  ░░░░░░  0/3  30 XP
  02. ◐ exambuilder     🦀🔒  ██░░░░  2/5  60/150 XP
  03. ✓ homelab         🐧❄️  ██████  3/3  90 XP

  Level 2: Config Wrangler
  XP: 150/200 ████████████░░░░
  Accuracy: 78% (28/36)
  Streak: 3 sprints
```

Status icons:
- `✓` — all sprints passed
- `◐` — some sprints passed
- `○` — no sprints attempted

---

### `kgate project`

Manage tracked projects.

#### `kgate project add <path>`

Register a project directory for tracking. Automatically imports any `exam_*.md` file found in the directory.

```bash
kgate project add ~/gitZ/myproject
```

The project gets a short hash ID (first 8 chars of SHA hash of the path) used for internal reference.

#### `kgate project list`

Show all tracked projects with their IDs and debt status.

```bash
kgate project list
```

Output:
```
Projects:
  → myproject    (a1b2c3d4) — debt: 0/10
    homelab      (e5f6g7h8) — debt: 3/10
  🔒 oldproject  (i9j0k1l2) — debt: 10/10
```
- `→` marks the active project
- `🔒` marks projects in debt lockdown (debt >= 10)

#### `kgate project set <id>`

Switch the active project by its 8-character hash ID.

```bash
kgate project set a1b2c3d4
```

#### `kgate project info`

Show details about the current active project.

---

### `kgate take exam <N> sprint <S> [--voice]`

Take a specific sprint from a specific exam. This is the core learning flow.

```bash
# Text mode
kgate take exam 1 sprint 1

# Voice mode (TTS reads questions, optionally STT for answers)
kgate take exam 1 sprint 1 --voice
```

**Parameters:**
- `<N>` — Exam number from `kgate status` (1-indexed)
- `<S>` — Sprint number within that exam (1-indexed)
- `--voice` / `-v` — Enable voice mode

**How it works:**

1. The sprint loads and shows one question at a time
2. Each question displays:
   - Question number, difficulty tier, XP value
   - Question text (and code snippet if present)
   - Four options numbered 1-4
3. **Fast answer mode** (default): Press 1, 2, 3, or 4 for instant selection — no Enter key needed
4. Immediate feedback per question:
   - Correct: chime sound, combo increments
   - Wrong: buzz sound, combo resets
5. After all questions, sprint results show:
   - Score percentage
   - Pass/fail verdict
   - XP earned (only if passed)
   - Updated streak and combo

**Difficulty tiers shown on questions:**
- `[RECALL]` — Direct knowledge retrieval
- `[COMPREHENSION]` — Understanding concepts
- `[APPLICATION]` — Applying knowledge to scenarios
- `[ANALYSIS]` — Deep reasoning about code behavior

---

### `kgate show exam <N>`

Display full details of an exam — all sprints, topics, XP values, and progress.

```bash
kgate show exam 1
```

Shows:
- Exam name and source project
- Domain icons
- Each sprint with status, topic, XP, and best score
- Overall progress summary
- Command hint for taking the next sprint

---

### `kgate scan [path] [--import]`

Scan a directory tree for exam files and optionally import them.

```bash
# Just scan and report
kgate scan ~/gitZ

# Scan and import found exams into the database
kgate scan ~/gitZ --import
```

Scans up to 3 levels deep. Looks for:
- `exam_*.md` — Exam files (sprint-based micro-exams)
- `QA_*.md` — Q&A transcript files
- `KNOWLEDGE_*.md` — Knowledge extraction files
- Git repositories

Each found exam is parsed and its sprints are registered in the database.

---

### `kgate debt`

Manage knowledge debt for the active project.

#### `kgate debt` (no arguments)

Show current debt level.

```bash
kgate debt
# Output: Debt for myproject: 5/10
```

#### `kgate debt add <action> [description]`

Manually add debt (mostly for testing).

```bash
kgate debt add concept "Learned about async runtime"
kgate debt add architecture "Added database layer"
kgate debt add code "Wrote parser module"
```

**Debt weights by action type:**

| Action | Weight |
|--------|--------|
| `concept` | +1 |
| `architecture` | +2 |
| `bugfix` | +1 |
| `newfile` | +1 |
| `code` | +2 |
| anything else | +1 |

#### `kgate debt clear <amount>`

Manually reduce debt.

```bash
kgate debt clear 3
```

In normal use, debt is cleared by passing exam sprints (each passed sprint = -3 debt).

---

### `kgate profile`

Show your learner profile with XP, level, and streak stats.

```bash
kgate profile
```

Output:
```
  Level 3: System Operator
  XP: 187/200 ████████████████░░░
  Sprints passed: 12
  Current streak: 5
  Best streak: 8
```

---

### `kgate badges`

Show all unlocked badges.

```bash
kgate badges
```

See [Gamification System](#gamification-system) for the full badge list.

---

### `kgate history [--limit N]`

Show recent exam attempt history.

```bash
kgate history
kgate history --limit 20
```

Each entry shows: project, sprint, score percentage, XP earned, pass/fail, timestamp.

---

### `kgate domains`

Show knowledge domain mastery across all projects.

```bash
kgate domains
```

Displays each domain with:
- Icon and name
- Mastery level (0-6: Novice through Legendary)
- Visual mastery bar
- Total XP in that domain
- Accuracy percentage and question counts
- Strongest inter-domain connections

See [Knowledge Domains](#knowledge-domains) for details.

---

### `kgate collection [--limit N]`

Show all collected questions across all sprints.

```bash
kgate collection
kgate collection --limit 50
```

Lists questions with their tier, XP value, text, and domain tags.

---

### `kgate achievements`

Show unlocked achievements (major milestones beyond badges).

```bash
kgate achievements
```

---

### `kgate whoami`

Show your Knowledge Identity card — unique ID, display name, level, and stats summary.

```bash
kgate whoami
```

---

### `kgate legend`

Show the domain icon legend — maps icons to domain names.

```bash
kgate legend
```

Output:
```
🦀 Rust        🐍 Python      🐚 Bash
❄️ Nix         🟨 JavaScript  🔷 TypeScript
🐳 Docker      🔀 Git         🐧 Linux
🌐 Networking  🔒 Security    🗄️ Databases
🔌 APIs        🔧 Hardware    📟 Embedded/IoT
🎮 GPU         🤖 AI/ML       🔄 DevOps
🧪 Testing     🏗️ Architecture 🕹️ Gaming
🎵 Audio       📺 Video       🧊 3D Printing
💬 Chat        📊 Operations
```

---

### `kgate config`

Manage user settings.

#### `kgate config show`

Show all current settings.

#### `kgate config sound [on|off]`

Enable or disable audio feedback (chimes on correct, buzz on wrong, fanfare on level-up).

```bash
kgate config sound on
kgate config sound off
```

#### `kgate config fast-answer [on|off]`

Enable or disable fast-answer mode. When on, pressing 1-4 immediately selects an answer without needing to press Enter.

```bash
kgate config fast-answer on
```

#### `kgate config name <name>`

Set your display name for the profile.

```bash
kgate config name "Alex"
```

---

### `kgate export-bookmarks [output]`

Extract study resource URLs from exam files and export to JSON.

```bash
kgate export-bookmarks
kgate export-bookmarks ~/study-links.json
```

Default output: `~/.kgate/bookmarks/bookmarks.json`

Scans all `exam_*.md` files for markdown links in Study Resources sections.

---

### `kgate review [--limit N]`

Start a spaced repetition review session for questions due for review.

```bash
kgate review
kgate review --limit 5
```

See [Spaced Repetition](#spaced-repetition) for how the review algorithm works.

---

### `kgate grade <answer> --concepts <list>`

Test the LLM grader on an open-ended answer.

```bash
kgate grade "Ownership prevents data races at compile time" \
  --concepts "ownership, borrow checker, memory safety"
```

Returns: score (0-3), XP multiplier, matched/missing concepts, pass/fail, and feedback.

---

### `kgate catalog`

Manage the domain-organized question catalog.

#### `kgate catalog list`

List all domains with question counts and accuracy stats.

#### `kgate catalog show <domain>`

Show questions for a specific domain.

```bash
kgate catalog show rust
```

#### `kgate catalog stats`

Show catalog-wide statistics.

#### `kgate catalog export [output]`

Export the catalog to JSON.

---

### `kgate harvest`

Build a growing question catalog from project codebases.

#### `kgate harvest add <path>`

Harvest questions from a project directory by analyzing its code.

```bash
kgate harvest add ~/gitZ/myproject
```

#### `kgate harvest all`

Harvest from all tracked projects.

#### `kgate harvest tree`

Show the question tree organized by domain and category.

#### `kgate harvest stats`

Show harvest statistics — total questions, domains covered, last harvest time.

#### `kgate harvest export [output]`

Export the full catalog to JSON.

---

### `kgate selftest [--verbose] [--fix]`

Run system diagnostics to verify everything is working.

```bash
kgate selftest
kgate selftest --verbose
kgate selftest --fix
```

See [Self-Diagnostics](#self-diagnostics) for what gets checked.

---

### `kgate voice`

Voice mode setup and testing.

#### `kgate voice setup`

Interactive wizard to configure TTS and STT engines.

#### `kgate voice test-speak <text>`

Test text-to-speech with sample text.

```bash
kgate voice test-speak "Hello, this is a test"
```

#### `kgate voice test-listen`

Test speech-to-text — records your voice and transcribes it.

#### `kgate voice config`

Show current voice configuration.

See [Voice Mode](#voice-mode) for full details.

---

### `kgate generate <path> [options]`

Auto-generate an exam from codebase analysis.

See [Generating Exams](#generating-exams) for full details.

---

## Taking Exams

### The Sprint Flow

A sprint contains 2-3 questions on a single topic. Questions follow a difficulty rhythm:

- **Q1:** Easy (warm-up win to build momentum)
- **Q2:** Medium (builds on the concept)
- **Q3:** Hard/Boss (challenge question, highest XP)

### Question Types

All questions are multiple-choice with four options (A through D). Each tests a single concept — questions never stack multiple concepts.

**Difficulty tiers:**

| Tier | What It Tests | Typical XP |
|------|--------------|------------|
| RECALL | Direct facts you should know | 10 |
| COMPREHENSION | Understanding why/how | 10 |
| APPLICATION | Using knowledge in context | 10-15 |
| ANALYSIS | Reasoning about behavior | 15 |

### Answering

By default, **fast-answer mode** is enabled. Press 1, 2, 3, or 4 to instantly select your answer. No Enter key needed. This keeps the flow snappy and ADHD-friendly.

If you prefer the arrow-key selection style, disable fast-answer:

```bash
kgate config fast-answer off
```

### Scoring

- Each question is all-or-nothing: correct = full XP, wrong = 0 XP
- Sprint score = (correct / total) * 100%
- **Pass:** 60% or higher (2 out of 3)
- **Fail:** Below 60%
- XP is only awarded when the sprint is passed
- Your best score counts — retakes can only improve your record

### Progressive Disclosure on Failure

The system reveals information gradually to encourage genuine learning:

| Attempt | What You See |
|---------|-------------|
| 1st fail | Which questions were wrong + brief hints |
| 2nd fail | Full correct answers + explanations |
| 3rd+ fail | Answers stay visible, study resources unlocked |

This prevents you from just memorizing answers on the first try while still helping you learn.

### Sound Effects

When sound is enabled (`kgate config sound on`):

- **Correct answer:** A5 (880 Hz) chime
- **Wrong answer:** A3 (220 Hz) buzz
- **Sprint passed:** Victory jingle (ascending C-E-G)
- **Level up:** Ascending fanfare (C5-E5-G5-C6)
- **Badge unlocked:** E5+G5 fanfare

---

## Generating Exams

### Two Modes

#### Template Mode (Offline)

Analyzes your codebase and generates generic questions based on detected patterns (functions, structs, configs, dependencies). No API key needed.

```bash
kgate generate ~/gitZ/myproject --templates
```

#### LLM Mode (Anthropic API)

Uses Claude to generate project-specific, nuanced questions that reference your actual code.

```bash
export ANTHROPIC_API_KEY="sk-ant-..."
kgate generate ~/gitZ/myproject --llm
```

#### Auto-Detection

If neither `--llm` nor `--templates` is specified, kgate checks for `ANTHROPIC_API_KEY`:
- Set: uses LLM mode
- Not set: falls back to templates

### Generation Pipeline

1. **Codebase Analysis** — Scans your project for:
   - Functions, methods, and their signatures
   - Structs, enums, traits
   - Configuration files (Cargo.toml, flake.nix, docker-compose.yml, etc.)
   - Dependencies and imports
   - Error handling patterns
   - API endpoints
   - Security-relevant patterns
   - File counts and complexity assessment

2. **Question Generation** — Creates questions referencing your actual code with:
   - Plausible wrong answers (not obviously wrong)
   - Code snippets from your project (max 5 lines for voice compatibility)
   - File path references so you can check the source

3. **Validation** — Each question is checked for:
   - Minimum quality (not too vague, not trivial)
   - No duplicate concepts across the exam
   - Cosine similarity deduplication (0.7 threshold)
   - Proper format (4 options, one correct answer)

4. **Assembly** — Questions are organized into 2-5 sprints of 3 questions each, with difficulty progression per sprint.

### Options

```bash
kgate generate <path> [OPTIONS]

Options:
  --llm              Force LLM generation (requires ANTHROPIC_API_KEY)
  --templates        Force template generation (no API needed)
  --dry-run          Preview the exam without writing to disk
  --model <model>    Override LLM model (default: claude-opus-4-20250514)
  -o, --output <path>  Custom output file path
```

### After Generation

Import the generated exam:

```bash
kgate scan ~/gitZ/myproject --import
```

Or add the project (auto-imports exams):

```bash
kgate project add ~/gitZ/myproject
```

---

## Voice Mode

Voice mode lets you take exams hands-free. Questions are read aloud by a TTS engine, and optionally your answers are captured by speech-to-text.

### Setup

```bash
kgate voice setup
```

The interactive wizard configures:
- **TTS engine:** Which text-to-speech to use
- **TTS voice:** Language and voice selection
- **STT engine:** Speech recognition settings
- **Microphone:** Input device selection
- **Calibration:** Silence threshold and wait times

### TTS Engines (Ordered by Quality)

| Engine | Quality | Notes |
|--------|---------|-------|
| **Kokoro** | Best (neural) | Warm, natural voices. Requires Python + kokoro package |
| **Piper** | Good (neural) | Many voices, ONNX models. Available via nix |
| **espeak-ng** | Basic (robotic) | Always available as fallback |

The system uses a fallback chain: if your configured engine fails, it tries the next one down.

**Kokoro voices** (if installed):
- `af_heart` — Warm, natural female voice (recommended)
- `af_bella` — Expressive female voice
- `bf_emma` — British female voice

### STT Engine

- **Whisper** (whisper-cpp): OpenAI's speech recognition, runs locally

### Voice Exam Flow

```bash
# Full voice mode (TTS reads + STT listens)
kgate take exam 1 sprint 1 --voice

# Or use the voice subcommand
kgate voice
```

What happens:
1. TTS announces the sprint topic
2. TTS reads each question and all four options
3. TTS asks "Your answer?"
4. You speak your answer (A, B, C, or D)
5. STT transcribes and normalizes your response ("bee" -> "B")
6. TTS announces if you're correct and the XP earned
7. Next question auto-reads
8. After the sprint, TTS reads the summary

### Voice Testing

```bash
# Test that TTS works
kgate voice test-speak "Testing one two three"

# Test that STT can hear you
kgate voice test-listen

# Check current voice config
kgate voice config
```

### Voice Configuration File

Saved at `~/.kgate/voice.toml`:

```toml
[general]
enabled = true

[tts]
engine = "piper"          # "kokoro", "piper", or "espeak-ng"
voice = "en-gb"
speed = 150
piper_model = "~/.kgate/voices/piper/en_GB-cori-high.onnx"

[stt]
engine = "whisper"
model = "base"            # whisper model size
language = "en"

[calibration]
silence_threshold = 0.3   # mic sensitivity
max_wait_time_ms = 30000  # max listen time
confirm_answers = true    # ask to confirm spoken answers
sample_rate = 16000
```

---

## Gamification System

### XP (Experience Points)

Every correct answer on a passed sprint earns XP. XP accumulates globally across all projects. The amount depends on the question's difficulty tier.

### Levels

| Level | Title | XP Required |
|-------|-------|-------------|
| 1 | Novice | 0 |
| 2 | Config Wrangler | 50 |
| 3 | System Operator | 130 |
| 4 | Stack Builder | 250 |
| 5 | Infra Architect | 430 |
| 6 | Master | 680 |

Higher levels continue with increasing XP thresholds.

### Streaks

**Sprint streak:** Consecutive sprints passed without a failure. Broken when you fail a sprint. Tracked as current streak and best streak.

**Combo chain:** Consecutive questions answered correctly within a session. Resets on a wrong answer. Visual indicators:
- 3+ correct: fire indicator
- 5+ correct: double fire
- 10+ correct: triple fire

### Badges

13 badges available:

| Badge | Trigger | Rarity |
|-------|---------|--------|
| First Sprint | Pass your first sprint | Common |
| Streak 3 | 3 consecutive sprints passed | Uncommon |
| Streak 5 | 5 consecutive sprints passed | Rare |
| Streak 10 | 10 consecutive sprints passed | Legendary |
| Level 2 | Reach level 2 | Common |
| Level 3 | Reach level 3 | Uncommon |
| Level 5 | Reach level 5 | Rare |
| Perfect | 100% score on a sprint | Uncommon |
| Project Clear | All sprints passed in one project | Uncommon |
| Comeback | Pass after 2+ failed attempts | Common |
| Speed Demon | 3 sprints in one session | Uncommon |
| XP 100 | Earn 100 total XP | Common |
| XP 500 | Earn 500 total XP | Uncommon |
| XP 1000 | Earn 1000 total XP | Rare |

Badges are awarded automatically and persist across sessions.

### Profile Stats Tracked

- Total XP earned
- Current level and title
- Questions passed / attempted
- Accuracy percentage
- Current combo and best combo
- Current sprint streak and best streak
- Perfect sprint count
- Total sprints passed

---

## Knowledge Debt System

This is the core enforcement mechanism. Knowledge debt tracks how much Claude has explained or built for you without verification that you understood it.

### How Debt Accumulates

When Claude Code helps you in a session, debt accrues:

| Action | Debt |
|--------|------|
| Concept explained | +1 |
| Architecture decision | +2 |
| Bug fix with explanation | +1 |
| New file created | +1 |
| Complex code written | +2 |
| "Why X not Y" answered | +1 |

### The Threshold

**Debt >= 10 = Read-Only Mode**

When debt hits 10, Claude Code enters **full lockdown**:

**Blocked (Claude cannot use):**
- Write tool (no new files)
- Edit tool (no modifications)
- Bash file creation (`echo >`, `cat <<`, `sed -i`, `tee`, `printf`, `curl | sh`)
- NotebookEdit
- Any workaround that creates or modifies files
- Long code explanations that could be copy-pasted

**Still allowed:**
- Read, Grep, Glob (viewing code)
- Read-only Bash (`ls`, `cat`, `git status`, `git log`)
- WebSearch, WebFetch (research)
- Brief verbal answers (haiku-minimal style)

### Clearing Debt

- **Pass an exam sprint:** -3 debt per sprint passed
- **Pass all sprints in an exam:** Debt resets to 0

### Managing Debt via CLI

```bash
# Check current debt
kgate debt

# Manually add (for testing)
kgate debt add concept "Learned about trait objects"

# Manually clear (for testing)
kgate debt clear 5
```

In practice, debt is tracked by Claude Code mentally during a session per the CLAUDE.md rules.

---

## Spaced Repetition

kgate includes an SM-2 spaced repetition system for long-term retention of concepts you've been tested on.

### How It Works

Questions you encounter are tracked with an **easiness factor** (EF). After each review:

1. Rate your recall quality (0-5):
   - 0: Complete blackout
   - 1: Wrong, but recognized the answer
   - 2: Wrong, but answer felt familiar
   - 3: Correct, but required effort
   - 4: Correct, with some hesitation
   - 5: Perfect, instant recall

2. The algorithm adjusts:
   - **EF (easiness factor):** Decreases for difficult items, increases for easy ones (range: 1.3 to 2.5+)
   - **Interval:** Days until next review (grows exponentially for well-known items)
   - **Repetition count:** Tracks how many successful reviews

3. Items appear in review sessions when their next review date is due

### Starting a Review Session

```bash
kgate review
kgate review --limit 5
```

Reviews are ordered by urgency — most overdue items appear first.

---

## Knowledge Domains

kgate tracks your mastery across 25+ knowledge domains, automatically detected from your projects.

### Built-in Domains

| Icon | Domain | Category |
|------|--------|----------|
| 🦀 | Rust | Language |
| 🐍 | Python | Language |
| 🐚 | Bash | Language |
| ❄️ | Nix | Language |
| 🟨 | JavaScript | Language |
| 🔷 | TypeScript | Language |
| 🐳 | Docker | Tool |
| 🔀 | Git | Tool |
| 🐧 | Linux | Tech |
| 🌐 | Networking | Concept |
| 🔒 | Security | Concept |
| 🗄️ | Databases | Concept |
| 🔌 | APIs | Concept |
| 🔧 | Hardware | Tech |
| 📟 | Embedded/IoT | Tech |
| 🎮 | GPU/Graphics | Tech |
| 🤖 | AI/ML | Concept |
| 🔄 | DevOps | Concept |
| 🧪 | Testing | Concept |
| 🏗️ | Architecture | Concept |
| 🕹️ | Gaming | Tech |
| 🎵 | Audio | Tech |
| 📺 | Video | Tech |
| 🧊 | 3D Printing | Tech |
| 💬 | Chat/Messaging | Concept |
| 📊 | Operations | Concept |

### Domain Detection

Domains are auto-detected from your codebase using keyword matching. For example, a project with `Cargo.toml` and `tokio` usage tags questions as 🦀 Rust. A project with `flake.nix` tags as ❄️ Nix.

### Domain Mastery Levels

| Level | Title | Meaning |
|-------|-------|---------|
| 0 | Novice | No questions attempted |
| 1 | Apprentice | First questions answered |
| 2 | Journeyman | Building understanding |
| 3 | Expert | Strong knowledge |
| 4 | Master | Deep mastery |
| 5 | Grandmaster | Exceptional |
| 6 | Legendary | Complete mastery |

View your domain progress:

```bash
kgate domains
```

### Domain Connections

The system tracks inter-domain relationships. If you pass a sprint covering both Rust and Databases, those domains get a stronger connection, showing your cross-domain expertise.

### Customizing Domains

Edit `~/.kgate/domains.toml` or the project-level `domains.toml` to add custom domains:

```toml
[domains.mycustomdomain]
name = "My Domain"
icon = "🎯"
category = "concept"
keywords = ["keyword1", "keyword2", "keyword3"]
```

---

## Question Harvesting

The harvest system builds a growing question catalog from your codebases, separate from specific exams.

### Workflow

```bash
# Harvest from a specific project
kgate harvest add ~/gitZ/myproject

# Harvest from all tracked projects
kgate harvest all

# See what you've collected
kgate harvest tree
kgate harvest stats

# Export for backup
kgate harvest export ~/catalog-backup.json
```

### How Harvesting Works

1. Scans the codebase for code elements (functions, structs, configs, etc.)
2. Generates questions based on detected patterns
3. Tags questions with domains
4. Deduplicates against existing catalog
5. Stores in the question catalog database

The catalog grows over time and can be used to generate fresh exams with different question selections.

---

## Configuration

### Config File

The deploy script creates `~/gitZ/.knowledge-gate/config.toml` with defaults:

```toml
[general]
projects_root = "~/gitZ"
data_dir = "~/gitZ/.knowledge-gate"

[knowledge_debt]
threshold = 10                         # debt limit before lockdown
debt_per_sprint_cleared = 3            # debt reduction per passed sprint
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
debt_warning_at = 7                    # warn at this debt level

[appearance]
theme = "system"                       # system | dark | light

[grading]
pass_threshold = 60                    # percent needed to pass
streak_bonus_at = 3                    # streak length for bonus
show_hints_on_fail = 1                 # attempt number to show hints
show_answers_on_fail = 2               # attempt number to show answers

[voice]
enabled = false
tts_voice = "default"
stt_mode = "push-to-talk"
```

### CLI Config Commands

```bash
kgate config show           # show all settings
kgate config sound on       # enable sound effects
kgate config sound off      # disable sound effects
kgate config fast-answer on # press 1-4 for instant answer
kgate config name "Alex"    # set display name
```

### Environment Variables

| Variable | Purpose |
|----------|---------|
| `ANTHROPIC_API_KEY` | Required for `kgate generate --llm` |
| `RUST_LOG` | Log level (debug, info, warn, error) |

### File Locations

```
~/.kgate/
├── data/db/knowledge-gate.db      # SQLite database (all state)
├── domains.toml                   # Domain definitions
├── voice.toml                     # Voice mode config
├── voices/piper/                  # Piper ONNX voice models
├── sounds/                        # Custom sound files
├── export/                        # Data exports
├── bookmarks/bookmarks.json       # Exported study URLs
└── catalog.json                   # Harvested question catalog
```

---

## Self-Diagnostics

```bash
kgate selftest
kgate selftest --verbose
kgate selftest --fix
```

### What Gets Checked

| Check | What It Verifies |
|-------|-----------------|
| Data directory | `~/.kgate/` exists and is writable |
| Database | SQLite connection works |
| Schema | All 6 migration tables exist (11 tables total) |
| Profile | Singleton profile record exists |
| Parser | Can parse a test exam file |
| Grader | Scoring logic: 100% = pass, 33% = fail |
| Voice TTS | espeak-ng, piper, or kokoro is available |
| Voice STT | whisper-cpp is installed |
| Anthropic API | `ANTHROPIC_API_KEY` is set and valid |
| Exam files | exam_*.md files in current directory parse correctly |
| Config | domains.toml loads and is valid |

### Output Legend

- `[PASS]` — Check passed
- `[WARN]` — Non-critical issue (e.g., voice engine not installed)
- `[FAIL]` — Critical issue that needs fixing

### Auto-Fix

With `--fix`, selftest attempts to repair issues:
- Creates missing directories
- Initializes missing database tables
- Creates default config files

---

## Exam File Format

Exams are stored as Markdown files (`exam_<name>.md`). They follow a strict format for parsing and voice compatibility.

### Structure

```markdown
# Exam: ProjectName
# Generated: 2026-03-15
# Total Sprints: 5
# Pass: 60% per sprint | Retakes: unlimited
# Voice-Ready: yes

---

## Progress Dashboard

| Sprint | Topic | Qs | Status | Score | XP |
|--------|-------|----|--------|-------|----|
| 1 | Topic Name | 3 | TODO | -- | 0/30 |

---

## Sprint 1: Topic Name
Target: 3 min | Pass: 60% | 30 XP
Voice-compatible: yes

### Q1. [RECALL] Easy — 10 XP

Question text here.

- A) Option one
- B) Option two
- C) Option three
- D) Option four

### Q2. [COMPREHENSION] Medium — 10 XP

Another question with optional code:

```rust
fn example() {
    println!("short snippet");
}
```

- A) Option one
- B) Option two
- C) Option three
- D) Option four

---

## Answer Key

### Sprint 1

**Q1. Answer: B** — 10 XP
Hint: Brief nudge toward the answer
Full: Detailed explanation of why B is correct

**Q2. Answer: C** — 10 XP
Hint: Think about X
Full: Explanation with context

---

## Study Resources (unlocked after attempt)

- [Resource Title](https://url)
- [Another Resource](https://url)
```

### Format Rules (Voice Compatibility)

These rules ensure exams work with the voice app and TTS:

- Sprint headers: `## Sprint N: Topic`
- Question headers: `### QN. [TIER] Difficulty — XP`
- MC options: `- A)` through `- D)` (letter + close paren)
- Answer key: `## Answer Key` or `## 🔑 Answer Key`
- Voice flag: `Voice-compatible: yes` on every sprint
- Code snippets: 3-5 lines max (TTS reads them aloud)
- Questions must make sense when read aloud — no "see the diagram below"
- One question = one concept (never combine)
- No "all of the above" options

### Manual Exam Creation

You can write exam files by hand following this format. Place them in your project directory as `exam_projectname.md` and import with:

```bash
kgate scan /path/to/project --import
```

---

## Development Setup

### Using Nix (Recommended)

```bash
cd exambuilder
nix-shell
```

This provides: rustc, cargo, clippy, rustfmt, rust-analyzer, cargo-watch, cargo-edit, sqlite, openssl, alsa-lib, libpulseaudio, espeak-ng, piper-tts, whisper-cpp, just.

### Task Runner (just)

```bash
just              # list all recipes
just build        # compile all crates
just release      # release build
just run <args>   # run kgate with arguments
just test         # run all tests (unit + integration)
just test-unit    # unit tests only
just test-integ   # integration tests only
just lint         # clippy linting
just lint-all     # clippy including test code
just fmt          # format code
just fmt-check    # check formatting
just check        # full pipeline: fmt + lint + test
just watch        # auto-rebuild on file changes
just dev          # auto-rebuild + run
just install      # build release + copy to ~/.local/bin
just selftest     # run diagnostics
just clean        # remove build artifacts
```

### Running Tests

```bash
# All 186 tests
just test

# Or directly
cargo test --workspace
```

### Project Architecture

```
exambuilder/
├── crates/
│   ├── kgate/              # CLI binary
│   │   ├── src/
│   │   │   ├── main.rs     # Entry point, clap definitions
│   │   │   ├── cli/        # Command handlers
│   │   │   ├── exam/       # Exam-taking logic
│   │   │   ├── voice/      # TTS/STT subsystem
│   │   │   ├── domains.rs  # Domain display
│   │   │   ├── scan.rs     # File scanner
│   │   │   └── sound.rs    # Audio effects
│   │   └── tests/
│   └── kgate-core/         # Shared library
│       ├── src/
│       │   ├── models.rs   # Domain objects
│       │   ├── db.rs       # Database layer
│       │   ├── parser.rs   # Exam markdown parser
│       │   ├── grader.rs   # Sprint scoring
│       │   ├── analyzer.rs # Codebase analysis
│       │   ├── anthropic.rs # Anthropic API client
│       │   ├── llm_generator.rs # LLM question generation
│       │   ├── prompts.rs  # Prompt templates
│       │   ├── question_validator.rs
│       │   ├── harvest.rs  # Knowledge extraction
│       │   ├── scanner.rs  # File scanner
│       │   ├── dedup.rs    # Question deduplication
│       │   ├── adaptive.rs # Adaptive difficulty
│       │   └── spaced_repetition.rs # SM-2 algorithm
│       └── tests/
├── migrations/             # SQLite schema (6 versions)
└── domains.toml            # Domain definitions
```

---

## Common Workflows

### Starting with a New Project

```bash
kgate project add ~/gitZ/newproject
kgate generate ~/gitZ/newproject
kgate scan ~/gitZ/newproject --import
kgate status
kgate take exam 1 sprint 1
```

### Daily Learning Routine

```bash
# Pick up where you left off
kgate

# Or check what's pending
kgate status

# Review previously learned material
kgate review
```

### After Claude Builds Something

```bash
# Check your debt
kgate debt

# If locked out, take sprints to clear
kgate take exam 1 sprint 1
kgate take exam 1 sprint 2
kgate take exam 1 sprint 3
# Each passed sprint = -3 debt
```

### Bulk Import Multiple Projects

```bash
kgate scan ~/gitZ --import
kgate status
```

### Voice Learning While Walking (Future)

```bash
# Start walk mode
kgate take exam 1 sprint 1 --voice
# Questions read aloud, speak your answers
```

---

## Tips

- **Just type `kgate`** — no arguments needed for daily use. It picks a random unfinished sprint.
- **Fast-answer mode** is on by default — press 1-4, no Enter needed. Keeps the flow fast.
- **Sound effects** add dopamine hits on correct answers. Enable with `kgate config sound on`.
- **Don't worry about failing** — retakes are unlimited, progressive hints help you learn, and your best score always counts.
- **Sprints are independent** — skip hard ones, come back later. Partial progress is always saved.
- **Generate exams for every project** — the more you test yourself, the more XP and domain mastery you build.
- **Run `kgate selftest`** if something seems broken — it checks everything and can auto-fix common issues.
