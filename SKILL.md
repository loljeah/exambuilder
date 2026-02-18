---
name: teachANDexam
description: >
  Anti-brainrot knowledge gate optimized for ADHD/neurodivergent learners.
  Auto-generates project-specific micro-exams using gamification, chunking,
  dopamine reward loops, and sprint-based pacing. Exams test real understanding
  of what you built — not memorization. No pass, no push.
---

# teachANDexam — Anti-Brainrot Knowledge Gate
## ADHD/Neurodivergent Optimized Edition

---

## Philosophy

You built it. Can you explain it?

AI-assisted dev creates brainrot: copy, paste, ship, pray.
This skill breaks the cycle with **project-specific exams** that prove understanding.
Not generic certs. YOUR configs. YOUR stack. YOUR architecture.

**Pass it or stop building.**

---

## ADHD Design Principles

Every aspect of this exam system is built around how ADHD brains actually work:

```
PROBLEM                         → SOLUTION IN THIS SYSTEM
────────────────────────────────────────────────────────────
Wall of text → shutdown         → Max 5 questions per sprint
Delayed reward → no motivation  → Instant score after each sprint
Boring = invisible              → Scenario-based, real project context
Time blindness                  → Timed sprints (5-8 min each)
Perfectionism paralysis         → 70% pass, partial credit, retakes welcome
Overwhelm from big exams        → Small progressive sprints, not one big test
Loss of momentum after failure  → "Streak" system, celebrate progress not perfection
Context switching cost           → Each sprint is ONE topic, ONE focus
```

### Core Mechanics

1. **SPRINTS not exams** — 2-3 questions per sprint, 3-5 minutes max
2. **Instant feedback** — score shown immediately after each sprint
3. **Progress bar** — visual XP/level tracker across sprints
4. **Variable difficulty** — mix easy wins with challenges (dopamine rhythm)
5. **Real scenarios** — questions reference actual project files (novelty + relevance)
6. **No penalty for retakes** — best score counts, learning is the goal
7. **Streak tracking** — consecutive sprints completed, motivates consistency
8. **Topic switching** — never more than 5 questions on the same topic in a row

---

## Trigger Conditions

Generate or update `exam_<ProjectName>.md` when ANY of these occur:

1. **New project** — git init, first file, fresh clone
2. **Milestone** — new module, major config change, infra shift
3. **Developer requests** — "exam me", "test me", "generate exam", "quiz me"
4. **Phase gate** — before merge to main, before deploy

Exams are **cumulative** — new sprints are appended, old ones remain.

---

## Exam Generation Process

### Step 1: Project Scan

Read these files (priority order):
- `configuration.nix`, `flake.nix`, `shell.nix`
- `docker-compose.yml`, `Dockerfile`
- `Makefile`, `justfile`, `Taskfile`
- `package.json`, `Cargo.toml`, `pyproject.toml`, `go.mod`
- `README.md`, docs/
- Network configs (.rsc exports, firewall rules, VLAN configs)
- CI/CD pipelines
- All config files in root

Determine:
```
WHAT exists    — languages, frameworks, tools, services, protocols
WHY chosen     — architecture decisions, trade-offs
HOW connected  — dependencies, data flow, network topology
WHERE it runs  — target environment, OS, hardware
WHAT BREAKS    — failure modes, security surface, pitfalls
```

### Step 2: Map to Knowledge Domains

| Domain | Focus |
|--------|-------|
| **Fundamentals** | What is this? Why does it exist? What problem? |
| **Config** | What do THESE settings do? Defaults? |
| **Networking** | Ports, protocols, DNS, routing in THIS project |
| **Security** | Auth, encryption, attack surface, hardening HERE |
| **Debugging** | Diagnose failures in THIS stack |
| **Architecture** | Why this approach? Trade-offs? |
| **Operations** | Deploy, rollback, backup, monitor THIS system |
| **Dependencies** | What depends on what? What breaks if X dies? |

### Step 3: Generate Sprints (NOT one big exam)

Each sprint = **2-3 questions**, **one focused topic**, **3-5 minute target**.

**Sprint structure optimized for ADHD:**
```
Sprint layout (2Q variant):
  Q1 — Easy win (Tier 1-2)   ← dopamine kickstart
  Q2 — Challenge (Tier 3-4)  ← peak engagement, earn it

Sprint layout (3Q variant):
  Q1 — Easy win (Tier 1)     ← dopamine kickstart
  Q2 — Medium (Tier 2-3)     ← momentum
  Q3 — Boss (Tier 3-4)       ← bonus XP

Ultra-micro = lowest possible activation energy to START.
You can always chain sprints. Starting is the hardest part.
```

**Difficulty tiers:**
```
Tier 1 — RECALL        "What port does X use in this config?"
Tier 2 — COMPREHENSION "Why is VLAN 10 isolated from VLAN 20 here?"
Tier 3 — APPLICATION   "Client can't reach NAS. Trace the path."
Tier 4 — ANALYSIS      "DNS server dies. What cascades?"
Tier 5 — SYNTHESIS     "Add IPv6 to these firewall rules." (bonus sprints only)
```

**Sprint count targets:**
- Initial project: 4-5 sprints (8-15 questions total)
- After milestone: +2-3 sprints appended
- Pre-deployment: 6-10 sprints total minimum
- More sprints, fewer Qs each = low barrier, high coverage

### Step 4: Answer Key

Every sprint includes answers with:
- **Hint** (shown on 1st failure): one-sentence nudge, doesn't give answer away
- **Full answer** (shown on 2nd failure): correct answer + brief WHY (2-3 sentences max)
- File reference: exact path and line in the project
- One documentation link (man page, RFC, official docs)

---

## Exam File Format

```markdown
# 🎯 Exam: <ProjectName>
# Generated: <ISO-8601>
# Total Sprints: <N>
# Pass: 70% per sprint | Retakes: unlimited
# Voice-Ready: yes

---

## ⚡ Progress Dashboard

| Sprint | Topic | Qs | Status | Score | XP |
|--------|-------|----|--------|-------|----|
| 1 | NixOS Basics | 3 | ⬜ TODO | — | 0/30 |
| 2 | Networking | 2 | ⬜ TODO | — | 0/25 |
| 3 | Security | 3 | ⬜ TODO | — | 0/30 |
| 4 | Debugging | 2 | ⬜ TODO | — | 0/25 |
| 5 | Architecture | 2 | ⬜ TODO | — | 0/25 |

**Total XP: 0 / 135**
**Streak: 0 sprints**
**Level: — (take first sprint to unlock)**
**Gate Status: ⚠️ BLOCKING**

---

## Sprint 1: <Topic Name>
⏱️ Target: 3 min | 🎯 Pass: 70% | ⚡ 30 XP
🎙️ Voice-compatible: yes

### Q1. [RECALL] ⭐ Easy — 10 XP
<short, clear question>

- A) <option>
- B) <option>
- C) <option>
- D) <option>

### Q2. [COMPREHENSION] ⭐⭐ Medium — 10 XP
<question>

### Q3. [APPLICATION] ⭐⭐⭐ Challenge — 10 XP
Given this from `configuration.nix`:
```nix
# actual code, max 5-8 lines
```
<scenario question>

---

## Sprint 2: <Next Topic>
⏱️ Target: 3 min | 🎯 Pass: 70% | ⚡ 25 XP
🎙️ Voice-compatible: yes

### Q1. [RECALL] ⭐ Easy — 10 XP
<question>

### Q2. [ANALYSIS] ⭐⭐⭐ Challenge — 15 XP
<scenario>

---

---

## 🔑 Answer Key

### Sprint 1

**Q1. Answer: C** — 10 XP
💡 Hint: "One of these commands skips the bootloader entirely."
📝 Full: <2-3 sentence explanation of why C is correct>
📁 `configuration.nix:42`
📖 <doc link>

**Q2. Expected key points:**
💡 Hint: "Think about what happens to your current session."
📝 Full:
- <key point 1>
- <key point 2>
📁 `configuration.nix:58`
📖 <doc link>

...

---

## 📊 Exam History

| Date | Sprint | Score | XP Earned | Streak |
|------|--------|-------|-----------|--------|
| — | — | — | — | — |

---

## 📚 Study Resources (unlocked after attempt)

Study resources are revealed AFTER first attempt, targeted to
weak areas. This prevents "studying to the test" and rewards
genuine engagement.

1. <Resource — shown only after attempt reveals gaps>
```

---

## Question Design Rules

### Content Rules
1. **No trivia** — every question tests knowledge needed to operate this project
2. **Use real project content** — actual config snippets, file paths, service names
3. **Max 8 lines of code** in any snippet — trim to the relevant part
4. **Scenario > recall** — prefer "what happens when" over "what is"
5. **Plausible wrong answers** — not obviously wrong, test real understanding
6. **Test the WHY** — "why X instead of Y" beats "what is X"

### ADHD-Specific Rules
7. **One question = one concept** — never combine multiple concepts in one Q
8. **Questions must fit on one screen** — if you have to scroll, it's too long
9. **Start each sprint with a confidence builder** — Q1 should be achievable
10. **No "all of the above" or "none of the above"** — these cause decision paralysis
11. **Bold the key action word** in each question stem
12. **Code snippets max 5-8 lines** — highlight the relevant line if possible
13. **Vary question format** within a sprint — MC, fill-blank, scenario, true/false

### Mandatory Topics
14. **Every exam must include security** — at least 1 sprint or 2+ questions
15. **Every exam must include debugging** — at least 1 scenario question
16. **NixOS projects must cover**: nix language basics, module system, rebuild commands, generation management

---

## Scoring & Gamification

### XP System
```
Tier 1 RECALL        → 10 XP
Tier 2 COMPREHENSION → 10 XP
Tier 3 APPLICATION   → 10 XP
Tier 4 ANALYSIS      → 15 XP
Tier 5 SYNTHESIS     → 20 XP (bonus sprints only)
```
Flat 10 XP for Tiers 1-3 keeps scoring simple and avoids punishing
easier questions. Tier 4-5 reward extra depth.

### Open-Ended Grading (3-point scale)
| Score | Meaning | XP Multiplier |
|-------|---------|---------------|
| 3 — Complete | All key points, deep understanding | 100% |
| 2 — Partial | Main concept, missed details | 66% |
| 1 — Surface | Awareness without operational depth | 33% |
| 0 — Wrong | Incorrect or blank | 0% |

### Passing
- **70% per sprint** to pass that sprint
- **Security floor**: must score >0 on every security question
- **All sprints passed** = gate opens
- **Retakes unlimited** — best score counts
- **Partial progress saves** — pass sprint 1 today, sprint 2 tomorrow

### Failure Flow (Progressive Reveal)
```
1st attempt, failed (<70%):
  → Show WHICH questions were wrong
  → Show HINT per wrong question (nudge toward the answer, don't give it)
  → "1/3 on Sprint 2. Hint: Q2 — think about which interface is tagged. Retry?"

2nd attempt, failed:
  → Show full correct answers + explanations
  → Unlock study resources for that sprint's topic
  → "Still 1/3. Here's what Q2 was looking for: [full answer]. 
     Check out [resource]. Try again when ready."

3rd+ attempt:
  → Same as 2nd (answers visible, resources unlocked)
  → No penalty, no guilt. "Learning is the goal, not the score."
```
This prevents two failure modes:
- Memorizing answers without understanding (hints-first forces thinking)
- Shame spiral from repeated blind failure (answers revealed after 2nd try)

### Streak System
- Complete a sprint = +1 streak
- Streak of 3+ = unlock bonus study resources
- Streak broken? No penalty, just resets. Start fresh.

### Level Progression
```
  0-30  XP  →  Level 1: Packet Pusher
 31-80  XP  →  Level 2: Config Wrangler  
 81-150 XP  →  Level 3: System Operator
151-250 XP  →  Level 4: Infra Builder
251+    XP  →  Level 5: Stack Architect
```

---

## Voice Exam App — Design Spec

The voice app is a **first-class target** for this system, not an afterthought.

### Core: Voice-Only Mode (no screen required)
```
App reads question aloud via TTS
  ↓
User answers by voice (STT captures)
  ↓
LLM grades answer against answer key
  ↓
App speaks result: "Correct! 10 XP. Next question..."
  ↓
After sprint: "Sprint 1 done. 2 out of 3. 70% — you passed! 
              Sprint 2 when you're ready."
```

### Voice-Specific Rules
- MC questions: read options as "A... B... C... D..." with pause
- User can say letter OR the answer content
- Open-ended: LLM compares spoken answer to key points in answer key
- Partial credit spoken: "Partial credit — you got the main idea. 
  The missing piece was..."
- Always speak XP earned and running total after each question
- Push-to-talk by default (tap to answer), user can switch to wake word

### Parsing Spec for Voice App
The exam markdown format is designed for clean parsing:
```
Sprint delimiter:  "## Sprint N:"
Question delimiter: "### QN."
Tier tag:          "[RECALL]" | "[COMPREHENSION]" | "[APPLICATION]" | "[ANALYSIS]"
MC options:        "- A)" through "- D)"
Answer delimiter:  "## 🔑 Answer Key"
Voice flag:        "🎙️ Voice-compatible: yes"
```

### Git Pre-Push Hook
```bash
#!/usr/bin/env bash
# .git/hooks/pre-push — Knowledge gate
EXAM="exam_$(basename $(git rev-parse --show-toplevel)).md"
if [ -f "$EXAM" ]; then
    STATUS=$(grep -oP 'Gate Status: \K.*' "$EXAM" | head -1)
    if [[ "$STATUS" == *"BLOCKING"* ]]; then
        echo "⚠️  KNOWLEDGE GATE BLOCKED"
        echo "Pass your exam sprints first: $EXAM"
        echo "Override: git push --no-verify"
        exit 1
    fi
fi
```

### Claude Code Integration

1. Detect new project → generate exam with initial sprints
2. Store as `exam_<ProjectName>.md` in project root
3. When developer provides answers → grade sprint, update dashboard
4. Score >= 70% per sprint → mark sprint ✓ PASSED
5. All sprints passed → change gate to ✓ CLEARED
6. Score < 70% → reveal study resources for weak areas, encourage retake
7. New milestone → append new sprint(s), gate resets to ⚠️ BLOCKING

### Future: Voice Exam App

See **Voice Exam App — Design Spec** section above for full specification
including voice-only mode, multiplayer/accountability, and parsing spec.

---

## Example: NixOS + MikroTik Project

### Sprint 1: NixOS Fundamentals
⏱️ 3 min | 🎯 70% | ⚡ 30 XP | 🎙️ Voice-compatible

**Q1. [RECALL] ⭐ — 10 XP**
Which command **rebuilds** NixOS and switches to the new config
WITHOUT adding a boot entry?

- A) `nixos-rebuild switch`
- B) `nixos-rebuild test`
- C) `nixos-rebuild boot`
- D) `nixos-rebuild dry-activate`

**Q2. [APPLICATION] ⭐⭐⭐ — 10 XP**
You change `services.openssh.enable = true` to `false` in
`configuration.nix` and run `nixos-rebuild switch`.
You're connected via SSH. What **happens next**?

**Q3. [ANALYSIS] 🏆 — 10 XP**
System won't boot after bad rebuild. You're at the console.
**Name the command** to roll back to the previous generation.

---

### Sprint 2: Network — VLANs
⏱️ 3 min | 🎯 70% | ⚡ 20 XP | 🎙️ Voice-compatible

**Q1. [RECALL] ⭐ — 10 XP**
In this project, what VLAN ID is assigned to the **IoT network**?

**Q2. [COMPREHENSION] ⭐⭐ — 10 XP**
Why are IoT devices on a **separate VLAN** from workstations?
Name two reasons.

---

## Coordinator Notes

- **Tone**: Direct, encouraging, no condescension. "Let's see what you know" not "prove yourself"
- **Filename**: `exam_<ProjectName>.md` in project root
- **Sprint naming**: Short, descriptive. "NixOS Basics" not "Section 1: Fundamental Concepts of NixOS"
- **Never combine** answer key with questions in visual proximity — answers at bottom
- **Reveal study resources AFTER attempt** — prevents "studying to the test"
- **Regeneration**: "regenerate exam" = fresh exam from current state, archive old
- **⚠️ Never skip security questions** — even for hobby projects
- **Encourage, don't punish** — "3/5 on Sprint 1, nice start! Sprint 2 when ready." not "FAILED"
- **ADHD-friendly language**: short sentences, active voice, concrete examples
- **No time pressure guilt**: sprints have TARGET times, not hard limits
