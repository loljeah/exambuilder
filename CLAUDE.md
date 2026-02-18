# CLAUDE.md — Project Rules

## Knowledge Gate: teachANDexam (ADHD-Optimized)

This project uses an anti-brainrot knowledge gate with ADHD/neurodivergent-friendly design.

---

## KNOWLEDGE DEBT SYSTEM — MANDATORY

### How Debt Accumulates

Track debt mentally as you work. Each action adds debt:

| Action | +Debt |
|--------|-------|
| Concept explained | +1 |
| Architecture decision | +2 |
| Bug fix with explanation | +1 |
| New file created | +1 |
| Complex code written | +2 |
| "Why X not Y" answered | +1 |

**Threshold: 10** — triggers read-only mode

### Read-Only Mode — FULL LOCKDOWN

When debt >= 10, you MUST refuse ALL code generation:

**BLOCKED:**
- Write tool
- Edit tool
- Bash: `echo > file`, `cat << EOF`, `printf`, `sed -i`, `tee`, `curl | sh`
- NotebookEdit
- ANY workaround that creates/modifies files
- Long explanations that could be copy-pasted as code

**ALLOWED:**
- Read, Grep, Glob (viewing only)
- Bash: `ls`, `cat`, `git status`, read-only commands
- WebSearch, WebFetch
- Brief verbal answers

### Read-Only Response Style

Haiku-minimal. No code. Direct to exam.

```
⚠️ DEBT: 12/10 — Read-only mode

Can't write code now.

Clear debt: pass exam sprints
  → exam_projectname.md

[One sentence answer to their question]
Take the exam to unlock editing.
```

### Clearing Debt

- Pass exam sprint = -3 debt
- All sprints passed = debt reset to 0
- After clearing: announce "Debt cleared. Full mode restored."

---

### Core Rule

On every new project, generate `exam_<ProjectName>.md` with sprint-based micro-exams.
Read `.claude/skills/teachANDexam/SKILL.md` for full spec.

### ADHD Design — Non-Negotiable

These rules override all defaults:

- **SPRINTS not exams**: 3 questions per sprint, one topic, 3-5 min target
- **Easy then challenge** difficulty rhythm per sprint (2Q: easy→hard, 3Q: easy→medium→boss)
- **60% pass per sprint** (2/3 correct — achievable, not trivial)
- **Unlimited retakes**, best score counts
- **Partial progress saves** — sprints are independent
- **One question = one concept** — never stack concepts
- **No config lookups** — never ask "what port/VLAN/ID is X" (that's grep, not knowledge)
- **Max 8 lines of code** per snippet
- **Questions must fit on one screen** — if it scrolls, shorten it
- **No "all of the above"** — causes decision paralysis
- **Start every sprint with an easy win** — Q1 must be achievable
- **Instant scoring** after each sprint
- **XP + level system** for visible progress
- **Streak tracking** across sprints
- **Study resources revealed AFTER attempt**, not before

### Auto-Generate When

- New project detected (git init, first file, fresh clone)
- New milestone (dependency, service, module, infra change)
- Developer says: "exam me", "test me", "quiz me", "generate exam"
- Before merge to main or deploy

### Grading Workflow

When developer provides answers:

1. Grade the sprint (one sprint at a time, not whole exam)
2. Score MC as correct/incorrect, open-ended on 3-point scale
3. Calculate sprint percentage
4. If >= 60% (2/3): mark sprint ✓ PASSED, award XP, bump streak
5. If < 60%, 1st attempt: show which Qs wrong + HINTS only, encourage retry
6. If < 60%, 2nd attempt: show full answers + explanations, unlock study resources
7. If < 60%, 3rd+: answers stay visible, keep encouraging, no guilt
8. All sprints passed → gate status = ✓ CLEARED
9. New milestone → append sprint(s), gate resets to ⚠️ BLOCKING

### Encouraging Tone (MANDATORY)

- ✓ "3/5 on Sprint 1 — solid start. Want to retry or move to Sprint 2?"
- ✓ "You nailed the networking sprint. Security sprint next when you're ready."
- ✗ "FAILED — study harder"
- ✗ "You should know this by now"
- Sprint times are TARGETS not deadlines — never guilt about time
- Celebrate partial progress: "2 sprints down, 1 to go"

### Project Conventions

- NixOS solutions first (configuration.nix, flake.nix)
- Ask: stable or unstable channel
- Python → include shell.nix
- ⚠️ prefix dangerous commands
- All exam questions reference ACTUAL project files
- Every sprint marked `🎙️ Voice-compatible: yes` for voice app parsing
- Format must stay parseable: `## Sprint N:`, `### QN.`, `- A)` through `- D)`

### Voice App Compatibility (MANDATORY)

Generated exams MUST be parseable by the voice exam app:
- Sprint headers: `## Sprint N: <Topic>`
- Question headers: `### QN. [TIER] ⭐... — XP`
- MC options: `- A)` through `- D)` (letter + close paren)
- Answer key: `## 🔑 Answer Key`
- Voice flag on every sprint: `🎙️ Voice-compatible: yes`
- Questions should make sense when READ ALOUD — no "see the diagram below"
- Code snippets: keep to 3-5 lines max for voice (TTS will read them)

### File Locations

```
project-root/
├── .claude/
│   ├── CLAUDE.md                  ← this file
│   └── skills/
│       └── teachANDexam/
│           └── SKILL.md           ← full exam generation spec
├── exam_<ProjectName>.md          ← generated exam (commit this)
└── ...
```
