# KnowledgeGATEunlocker

A gamified, ADHD-optimized knowledge verification system. Celebrates mastery without prescribing direction.

**You built it. Can you explain it?**

## What Gets Blocked

**NOT git.** Push whenever.

**Claude.** Full lockdown at debt threshold.

## Read-Only Mode (Debt >= 10)

```
BLOCKED:
├── Write, Edit, NotebookEdit
├── Bash: echo >, cat <<, sed -i, tee, curl|sh
└── Any file creation/modification workaround

ALLOWED:
├── Read, Grep, Glob
├── Bash: ls, cat, git status (read-only)
└── Brief verbal answers (haiku-minimal)
```

## How It Works

```
Claude helps you build (+debt)
        ↓
Debt: concept +1, architecture +2, code +2
        ↓
At 10: LOCKDOWN
        ↓
Haiku responses only. No code.
        ↓
Pass exam sprint = -3 debt
        ↓
Debt cleared → full mode
```

## Why This Works for ADHD

- **No punishment for building** — push code anytime
- **Natural break point** — exam is a context switch, good for focus reset
- **Prevents scatter-brain abuse** — can't infinitely copy-paste without understanding
- **Read-only still helps** — Claude can explain what you built, just can't add more chaos

## Quick Start

```bash
./deploy.sh
```

Installs:
- `~/.claude/CLAUDE.md` — Claude Code behavior rules
- `~/.claude/skills/teachANDexam/SKILL.md` — Exam generation spec
- `~/gitZ/.knowledge-gate/` — Data directory + config

## File Pipeline

```
Claude Conversation
        ↓
KNOWLEDGE_<hash>.md  (continuous extraction, +debt)
        ↓
QA_<hash>.md         (transcript + quiz bank)
        ↓
EXAM_<hash>.md       (sprint-based micro-exams)
```

## Knowledge Debt

| Action | Debt |
|--------|------|
| Concept explained | +1 |
| Architecture decision | +2 |
| Bug fix with explanation | +1 |
| New file created | +1 |
| Complex code written | +2 |

**Threshold:** 10 = read-only mode
**Clear:** Pass sprint = -3 debt

## Stack

- **App:** Rust + Tauri
- **Database:** SQLite
- **Export:** JSON/MD (git-friendly)

## Roadmap

See [ROADMAP.md](ROADMAP.md)

**MVP:** Generate → Quiz → Grade → Track
**Phase 2:** Voice mode
**Phase 3:** Skill tree UI
**Phase 4:** Achievements
