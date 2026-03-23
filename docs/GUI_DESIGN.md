# Knowledge Gate — GUI Application Design

## Overview

A native desktop GUI application for Knowledge Gate that provides a complete dashboard experience with avatar display, exam taking, analytics, shop, and settings.

---

## Technology Options

| Framework | Pros | Cons |
|-----------|------|------|
| **GTK4 + Go** | Native Linux, good theming | Limited cross-platform |
| **Fyne (Go)** | Pure Go, cross-platform | Less native look |
| **Tauri + Svelte** | Modern, lightweight, web tech | Requires JS knowledge |
| **Wails + Svelte** | Go backend + web frontend | Similar to Tauri |
| **egui (Rust)** | Immediate mode, fast | Different language |
| **Qt/QML** | Mature, polished | C++ or Python bindings |

**Recommended: Wails + Svelte**
- Go backend (reuse existing daemon code)
- Svelte frontend (modern, reactive, small bundle)
- Native window decorations
- ~10MB binary
- Hot reload during development

---

## Application Structure

```
┌─────────────────────────────────────────────────────────────────────┐
│  Knowledge Gate                                    ─  □  ×         │
├─────────────────────────────────────────────────────────────────────┤
│ ┌─────────┐                                                         │
│ │  🐱     │  Dashboard │ Exams │ Review │ Stats │ Shop │ Settings  │
│ │ Whiskers│                                                         │
│ │  💰1250 │─────────────────────────────────────────────────────────│
│ │  🔥14   │                                                         │
│ └─────────┘           [ MAIN CONTENT AREA ]                         │
│                                                                     │
│  SIDEBAR                                                            │
│  ───────                                                            │
│  📊 Dashboard                                                       │
│  📝 Exams                                                           │
│  🔄 Review                                                          │
│  📈 Statistics                                                      │
│  🛒 Shop                                                            │
│  🏆 Achievements                                                    │
│  ⚙️ Settings                                                        │
│                                                                     │
│ ─────────────────────────────────────────────────────────────────── │
│  Status: Connected │ Last sync: just now │ v1.0.0                   │
└─────────────────────────────────────────────────────────────────────┘
```

---

## Pages / Views

### 1. Dashboard (Home)

The main overview screen showing everything at a glance.

```
┌─────────────────────────────────────────────────────────────────────┐
│                         DASHBOARD                                   │
├─────────────────────────────────────────────────────────────────────┤
│                                                                     │
│  ┌──────────────────┐  ┌──────────────────┐  ┌──────────────────┐  │
│  │     AVATAR       │  │   QUICK STATS    │  │  DAILY PROGRESS  │  │
│  │                  │  │                  │  │                  │  │
│  │      /\_/\       │  │  Level 12        │  │  ████████░░ 80%  │  │
│  │     ( ^.^ )      │  │  1,250 XP        │  │                  │  │
│  │      > ^ <       │  │  🔥 14 streak    │  │  8/10 sprints    │  │
│  │                  │  │  💰 1,250 coins  │  │  today           │  │
│  │   😸 Happy       │  │                  │  │                  │  │
│  │   +15% XP bonus  │  │  Next: 250 XP    │  │  [View Details]  │  │
│  └──────────────────┘  └──────────────────┘  └──────────────────┘  │
│                                                                     │
│  ┌─────────────────────────────────────────┐  ┌──────────────────┐  │
│  │          DAILY CHALLENGES               │  │  WEEKLY GOALS    │  │
│  │                                         │  │                  │  │
│  │  ✓ Complete 2 sprints      +15c        │  │  Sprints: 7/10   │  │
│  │  ○ Get 80%+ score          +25c        │  │  ███████░░░ 70%  │  │
│  │  ○ Review 5 items          +20c        │  │                  │  │
│  │                                         │  │  Streak: ✓       │  │
│  │  [Claim 15 coins]                       │  │  Perfect: 1/3    │  │
│  └─────────────────────────────────────────┘  └──────────────────┘  │
│                                                                     │
│  ┌─────────────────────────────────────────────────────────────────┐│
│  │                    RECENT ACTIVITY                              ││
│  │                                                                 ││
│  │  10:05  ✓ Sprint 3 passed — 100%, +30 XP, +50 coins            ││
│  │  09:58  📚 Reviewed 3 knowledge items                          ││
│  │  09:45  🎯 Daily challenge completed                           ││
│  │  09:30  🔥 Streak extended to 14 days                          ││
│  └─────────────────────────────────────────────────────────────────┘│
│                                                                     │
│  ┌────────────────────────┐  ┌────────────────────────────────────┐ │
│  │   ITEMS DUE REVIEW     │  │      ACTIVE PROJECT                │ │
│  │                        │  │                                    │ │
│  │  3 items overdue       │  │  📁 exambuilder                    │ │
│  │  5 items due today     │  │  Sprints: 4/8 passed               │ │
│  │                        │  │  Next: Sprint 5 (Architecture)     │ │
│  │  [Start Review]        │  │  [Continue Learning]               │ │
│  └────────────────────────┘  └────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────────────┘
```

### 2. Exams View

Browse projects and take exams.

```
┌─────────────────────────────────────────────────────────────────────┐
│                           EXAMS                                     │
├─────────────────────────────────────────────────────────────────────┤
│                                                                     │
│  Project: [exambuilder        ▼]    [Import Exam File]             │
│                                                                     │
│  ┌─────────────────────────────────────────────────────────────────┐│
│  │  SPRINTS                                                        ││
│  ├─────────────────────────────────────────────────────────────────┤│
│  │                                                                 ││
│  │  ✓ Sprint 1: Go Basics                    100%  ⭐⭐⭐          ││
│  │    3 questions • Best: 100% • Attempts: 2                      ││
│  │    [Retake]                                                    ││
│  │                                                                 ││
│  │  ✓ Sprint 2: Database Layer               85%   ⭐⭐           ││
│  │    3 questions • Best: 85% • Attempts: 3                       ││
│  │    [Retake]                                                    ││
│  │                                                                 ││
│  │  ✓ Sprint 3: Socket Protocol              100%  ⭐⭐⭐          ││
│  │    3 questions • Best: 100% • Attempts: 1                      ││
│  │    [Retake]                                                    ││
│  │                                                                 ││
│  │  ○ Sprint 4: Voice Integration            --    ⭐⭐           ││
│  │    3 questions • Not attempted                                 ││
│  │    [Start Sprint]                                              ││
│  │                                                                 ││
│  │  🔒 Sprint 5: Architecture                --    ⭐⭐⭐          ││
│  │    Requires: Sprint 4                                          ││
│  │                                                                 ││
│  └─────────────────────────────────────────────────────────────────┘│
│                                                                     │
│  Total Progress: ████████░░ 75%    XP Available: 45               │
│                                                                     │
└─────────────────────────────────────────────────────────────────────┘
```

### 3. Taking an Exam (Modal/Full Screen)

```
┌─────────────────────────────────────────────────────────────────────┐
│  Sprint 4: Voice Integration                    Question 2 of 3    │
├─────────────────────────────────────────────────────────────────────┤
│                                                                     │
│  ┌─────────────────────────────────────────────────────────────────┐│
│  │                                                                 ││
│  │  [COMPREHENSION] ⭐⭐ — 15 XP                                   ││
│  │                                                                 ││
│  │  What protocol does the piper-daemon use for TTS requests?     ││
│  │                                                                 ││
│  │  ┌─────────────────────────────────────────────────────────┐   ││
│  │  │  // From voice/client.go                                │   ││
│  │  │  conn, err := net.Dial("unix", c.cfg.Voice.PiperSocket) │   ││
│  │  │  fmt.Fprintf(conn, "speak %s\n", text)                  │   ││
│  │  └─────────────────────────────────────────────────────────┘   ││
│  │                                                                 ││
│  └─────────────────────────────────────────────────────────────────┘│
│                                                                     │
│  ┌─────────────────────────────────────────────────────────────────┐│
│  │                                                                 ││
│  │  ○  A) HTTP REST API                                           ││
│  │                                                                 ││
│  │  ●  B) Unix socket with text protocol                          ││
│  │                                                                 ││
│  │  ○  C) gRPC with protobuf                                      ││
│  │                                                                 ││
│  │  ○  D) WebSocket                                               ││
│  │                                                                 ││
│  └─────────────────────────────────────────────────────────────────┘│
│                                                                     │
│  Progress: ██░░░░░░░░ 1/3                                          │
│                                                                     │
│  [← Previous]                              [Next →]  [Submit All]  │
│                                                                     │
│  ──────────────────────────────────────────────────────────────────│
│  🔊 Voice Mode: ON    ⏱️ Time: 2:34                                │
└─────────────────────────────────────────────────────────────────────┘
```

### 4. Exam Results

```
┌─────────────────────────────────────────────────────────────────────┐
│                      SPRINT COMPLETED                               │
├─────────────────────────────────────────────────────────────────────┤
│                                                                     │
│                           ✓ PASSED!                                 │
│                                                                     │
│                    ┌───────────────────┐                           │
│                    │                   │                           │
│                    │       85%         │                           │
│                    │                   │                           │
│                    │    ████████░░     │                           │
│                    │                   │                           │
│                    └───────────────────┘                           │
│                                                                     │
│                      2/3 Correct                                    │
│                                                                     │
│  ┌─────────────────────────────────────────────────────────────────┐│
│  │  REWARDS EARNED                                                 ││
│  │                                                                 ││
│  │  ⭐ +25 XP (base)                                               ││
│  │  😸 +4 XP (happy bonus 15%)                                     ││
│  │  💰 +50 coins (first pass)                                      ││
│  │  🔥 Streak extended: 15 days                                    ││
│  │                                                                 ││
│  │  Total: +29 XP, +50 coins                                       ││
│  └─────────────────────────────────────────────────────────────────┘│
│                                                                     │
│  ┌─────────────────────────────────────────────────────────────────┐│
│  │  QUESTION BREAKDOWN                                             ││
│  │                                                                 ││
│  │  Q1 ✓  What is the main entry point...           Correct       ││
│  │  Q2 ✓  What protocol does piper-daemon...        Correct       ││
│  │  Q3 ✗  Which function handles...                 Wrong         ││
│  │        Your answer: C  •  Correct: A                           ││
│  │        [View Explanation]                                       ││
│  └─────────────────────────────────────────────────────────────────┘│
│                                                                     │
│  [Back to Sprints]    [Retake Sprint]    [Next Sprint →]           │
│                                                                     │
└─────────────────────────────────────────────────────────────────────┘
```

### 5. Review View (Spaced Repetition)

```
┌─────────────────────────────────────────────────────────────────────┐
│                          REVIEW                                     │
├─────────────────────────────────────────────────────────────────────┤
│                                                                     │
│  Knowledge items ready for review                                   │
│                                                                     │
│  ┌─────────────────────────────────────────────────────────────────┐│
│  │  DUE TODAY (5)                                  [Start Review]  ││
│  ├─────────────────────────────────────────────────────────────────┤│
│  │                                                                 ││
│  │  Unix Sockets          networking     45%   Due: today         ││
│  │  Go Interfaces         language       60%   Due: today         ││
│  │  SQLite WAL Mode       database       30%   Due: today         ││
│  │  Error Handling        patterns       55%   Due: today         ││
│  │  Channel Patterns      concurrency    40%   Due: today         ││
│  │                                                                 ││
│  └─────────────────────────────────────────────────────────────────┘│
│                                                                     │
│  ┌─────────────────────────────────────────────────────────────────┐│
│  │  OVERDUE (3)                                                    ││
│  ├─────────────────────────────────────────────────────────────────┤│
│  │                                                                 ││
│  │  Context Cancellation  patterns       25%   Overdue: 2 days    ││
│  │  Mutex vs RWMutex      concurrency    35%   Overdue: 1 day     ││
│  │  JSON Marshaling       encoding       50%   Overdue: 1 day     ││
│  │                                                                 ││
│  └─────────────────────────────────────────────────────────────────┘│
│                                                                     │
│  ┌─────────────────────────────────────────────────────────────────┐│
│  │  MASTERY OVERVIEW                                               ││
│  │                                                                 ││
│  │  Total: 24 concepts                                             ││
│  │  ├── Mastered:  4  ████░░░░░░░░░░░░  17%                       ││
│  │  ├── Learning:  12 ████████████░░░░  50%                       ││
│  │  └── Unseen:    8  ████████░░░░░░░░  33%                       ││
│  │                                                                 ││
│  └─────────────────────────────────────────────────────────────────┘│
│                                                                     │
└─────────────────────────────────────────────────────────────────────┘
```

### 6. Statistics View

```
┌─────────────────────────────────────────────────────────────────────┐
│                        STATISTICS                                   │
├─────────────────────────────────────────────────────────────────────┤
│                                                                     │
│  Period: [Last 7 days ▼]    Project: [All ▼]                       │
│                                                                     │
│  ┌─────────────────────────────────────────────────────────────────┐│
│  │  XP OVER TIME                                                   ││
│  │                                                                 ││
│  │  120│                                    ╭─╮                    ││
│  │     │              ╭─╮         ╭─╮      │ │                    ││
│  │  80 │    ╭─╮      │ │   ╭─╮   │ │      │ │                    ││
│  │     │    │ │      │ │   │ │   │ │  ╭─╮ │ │                    ││
│  │  40 │╭─╮ │ │  ╭─╮ │ │   │ │   │ │  │ │ │ │                    ││
│  │     │ │ │ │  │ │ │ │   │ │   │ │  │ │ │ │                    ││
│  │   0 └─┴─┴─┴──┴─┴─┴─┴───┴─┴───┴─┴──┴─┴─┴─┘                    ││
│  │      Mon Tue Wed Thu Fri Sat Sun                                ││
│  └─────────────────────────────────────────────────────────────────┘│
│                                                                     │
│  ┌───────────────────┐ ┌───────────────────┐ ┌───────────────────┐ │
│  │  SPRINTS          │ │  ACCURACY         │ │  TIME SPENT       │ │
│  │                   │ │                   │ │                   │ │
│  │  Attempted: 15    │ │  Overall: 82%     │ │  Total: 2h 15m    │ │
│  │  Passed: 12       │ │  Best: 100%       │ │  Avg/day: 19m     │ │
│  │  Pass rate: 80%   │ │  Worst: 45%       │ │  Today: 35m       │ │
│  └───────────────────┘ └───────────────────┘ └───────────────────┘ │
│                                                                     │
│  ┌─────────────────────────────────────────────────────────────────┐│
│  │  HARDEST QUESTIONS                                              ││
│  │                                                                 ││
│  │  Sprint 2 Q3: "What triggers migration..."    33% (1/3)        ││
│  │  Sprint 5 Q1: "Which pattern handles..."      40% (2/5)        ││
│  │  Sprint 1 Q2: "How does the grader..."        50% (3/6)        ││
│  │                                                                 ││
│  └─────────────────────────────────────────────────────────────────┘│
│                                                                     │
│  ┌─────────────────────────────────────────────────────────────────┐│
│  │  STREAKS & CONSISTENCY                                          ││
│  │                                                                 ││
│  │  Current streak: 🔥 14 days                                     ││
│  │  Best streak: 🏆 23 days                                        ││
│  │  Active days this month: 18/23 (78%)                           ││
│  │                                                                 ││
│  │  Mar 2026                                                       ││
│  │  ┌──────────────────────────────────────────────────────┐      ││
│  │  │ Mo Tu We Th Fr Sa Su                                 │      ││
│  │  │ ██ ██ ██ ░░ ██ ██ ██                                 │      ││
│  │  │ ██ ██ ██ ██ ██ ░░ ██                                 │      ││
│  │  │ ██ ██ ██ ██ ██ ██ ░░                                 │      ││
│  │  │ ██ ██ ██ ░░                                          │      ││
│  │  └──────────────────────────────────────────────────────┘      ││
│  └─────────────────────────────────────────────────────────────────┘│
│                                                                     │
└─────────────────────────────────────────────────────────────────────┘
```

### 7. Shop View

```
┌─────────────────────────────────────────────────────────────────────┐
│                            SHOP                                     │
├─────────────────────────────────────────────────────────────────────┤
│                                                                     │
│  💰 1,250 coins                    [All] [Hats] [Held] [Auras] [BG]│
│                                                                     │
│  ┌─────────────────────────────────────────────────────────────────┐│
│  │  FEATURED                                          🔥 20% OFF   ││
│  │  ┌─────────┐                                                    ││
│  │  │  👑     │  Crown — Legendary                                 ││
│  │  │         │  "Rule your knowledge kingdom"                     ││
│  │  │  800c   │  Was: 1000c                                        ││
│  │  │  [Buy]  │                                                    ││
│  │  └─────────┘                                                    ││
│  └─────────────────────────────────────────────────────────────────┘│
│                                                                     │
│  ┌─────────────────────────────────────────────────────────────────┐│
│  │  HATS                                                           ││
│  │                                                                 ││
│  │  ┌───────┐ ┌───────┐ ┌───────┐ ┌───────┐ ┌───────┐ ┌───────┐  ││
│  │  │ 🎩    │ │ 🎓    │ │ 👒    │ │ 🤠    │ │ 👑    │ │ 🌟    │  ││
│  │  │ Top   │ │ Grad  │ │ Sun   │ │Cowboy │ │Crown  │ │ Halo  │  ││
│  │  │ 50c   │ │ 50c   │ │ 150c  │ │ 150c  │ │ 800c  │ │1000c  │  ││
│  │  │ ✓Own  │ │ [Buy] │ │ [Buy] │ │ 🔒L10 │ │ [Buy] │ │ 🔒L25 │  ││
│  │  └───────┘ └───────┘ └───────┘ └───────┘ └───────┘ └───────┘  ││
│  └─────────────────────────────────────────────────────────────────┘│
│                                                                     │
│  ┌─────────────────────────────────────────────────────────────────┐│
│  │  HELD ITEMS                                                     ││
│  │                                                                 ││
│  │  ┌───────┐ ┌───────┐ ┌───────┐ ┌───────┐ ┌───────┐ ┌───────┐  ││
│  │  │ 📚    │ │ ☕    │ │ 🎮    │ │ 🗡️    │ │ 🪄    │ │ 💎    │  ││
│  │  │ Book  │ │Coffee │ │Ctrl   │ │Sword  │ │ Wand  │ │Crystal│  ││
│  │  │ 50c   │ │ 50c   │ │ 150c  │ │ 150c  │ │ 400c  │ │1000c  │  ││
│  │  │ [Buy] │ │ ✓Own  │ │ [Buy] │ │ [Buy] │ │ [Buy] │ │ 🔒L50 │  ││
│  │  └───────┘ └───────┘ └───────┘ └───────┘ └───────┘ └───────┘  ││
│  └─────────────────────────────────────────────────────────────────┘│
│                                                                     │
└─────────────────────────────────────────────────────────────────────┘
```

### 8. Achievements View

```
┌─────────────────────────────────────────────────────────────────────┐
│                        ACHIEVEMENTS                                 │
├─────────────────────────────────────────────────────────────────────┤
│                                                                     │
│  Unlocked: 12/35                             [All] [Unlocked] [🔒] │
│                                                                     │
│  ┌─────────────────────────────────────────────────────────────────┐│
│  │  RECENTLY UNLOCKED                                              ││
│  │                                                                 ││
│  │  ✨ On Fire — 7-day streak                         +50 coins   ││
│  │     Unlocked: 2 hours ago                                       ││
│  │                                                                 ││
│  │  ✨ Sharpshooter — First 100% sprint               +50 coins   ││
│  │     Unlocked: yesterday                                         ││
│  │                                                                 ││
│  └─────────────────────────────────────────────────────────────────┘│
│                                                                     │
│  ┌─────────────────────────────────────────────────────────────────┐│
│  │  LEARNING                                          8/12         ││
│  │                                                                 ││
│  │  ✓ First Steps      Complete first sprint          +25c        ││
│  │  ✓ Sprint Master    Complete 10 sprints            +50c        ││
│  │  ✓ Sharpshooter     First 100% score               +50c        ││
│  │  ○ Perfectionist    10 perfect sprints in a row    +200c       ││
│  │    Progress: 3/10   ███░░░░░░░                                 ││
│  │  🔒 Century         Complete 100 sprints           +300c       ││
│  │    Progress: 47/100 █████░░░░░                                 ││
│  │                                                                 ││
│  └─────────────────────────────────────────────────────────────────┘│
│                                                                     │
│  ┌─────────────────────────────────────────────────────────────────┐│
│  │  STREAKS                                           2/5          ││
│  │                                                                 ││
│  │  ✓ On Fire          7-day streak                   +50c        ││
│  │  ✓ Week Warrior     Complete goal: 7-day streak    +75c        ││
│  │  ○ Month Master     30-day streak                  +200c       ││
│  │    Progress: 14/30  █████░░░░░                                 ││
│  │                                                                 ││
│  └─────────────────────────────────────────────────────────────────┘│
│                                                                     │
│  ┌─────────────────────────────────────────────────────────────────┐│
│  │  SECRET                                            ?/5          ││
│  │                                                                 ││
│  │  ✓ Night Owl        ???                            +100c       ││
│  │  🔒 ???             Hidden until unlocked                      ││
│  │  🔒 ???             Hidden until unlocked                      ││
│  │                                                                 ││
│  └─────────────────────────────────────────────────────────────────┘│
│                                                                     │
└─────────────────────────────────────────────────────────────────────┘
```

### 9. Settings View

```
┌─────────────────────────────────────────────────────────────────────┐
│                          SETTINGS                                   │
├─────────────────────────────────────────────────────────────────────┤
│                                                                     │
│  ┌─────────────────────────────────────────────────────────────────┐│
│  │  GENERAL                                                        ││
│  │                                                                 ││
│  │  Projects Root      [~/code                    ] [Browse]      ││
│  │  Theme              [Dark ▼]                                   ││
│  │  Start on Login     [✓]                                        ││
│  │  Minimize to Tray   [✓]                                        ││
│  │                                                                 ││
│  └─────────────────────────────────────────────────────────────────┘│
│                                                                     │
│  ┌─────────────────────────────────────────────────────────────────┐│
│  │  VOICE                                                          ││
│  │                                                                 ││
│  │  Enable Voice Mode  [✓]                                        ││
│  │  TTS Voice          [Default ▼]                                ││
│  │  Speech Rate        [████████░░] 1.0x                          ││
│  │  Read Questions     [✓]                                        ││
│  │  Announce Results   [✓]                                        ││
│  │                                                                 ││
│  └─────────────────────────────────────────────────────────────────┘│
│                                                                     │
│  ┌─────────────────────────────────────────────────────────────────┐│
│  │  EXAMS                                                          ││
│  │                                                                 ││
│  │  Pass Threshold     [████████░░] 60%                           ││
│  │  Show Timer         [✓]                                        ││
│  │  Reveal After       [2 ▼] attempts                             ││
│  │                                                                 ││
│  └─────────────────────────────────────────────────────────────────┘│
│                                                                     │
│  ┌─────────────────────────────────────────────────────────────────┐│
│  │  NOTIFICATIONS                                                  ││
│  │                                                                 ││
│  │  Desktop Alerts     [✓]                                        ││
│  │  Streak Reminder    [✓]  at [20:00 ▼]                          ││
│  │  Achievement Popup  [✓]                                        ││
│  │  Sound Effects      [✓]                                        ││
│  │                                                                 ││
│  └─────────────────────────────────────────────────────────────────┘│
│                                                                     │
│  ┌─────────────────────────────────────────────────────────────────┐│
│  │  DATA                                                           ││
│  │                                                                 ││
│  │  [Export All Data]    [Import Backup]    [Reset Progress]      ││
│  │                                                                 ││
│  │  Database: ~/.local/share/kgate/kgate.db (2.3 MB)              ││
│  │  Config: ~/.config/kgate/config.toml                           ││
│  │                                                                 ││
│  └─────────────────────────────────────────────────────────────────┘│
│                                                                     │
└─────────────────────────────────────────────────────────────────────┘
```

---

## Component Library

### Cards
```svelte
<Card>
  <CardHeader>Title</CardHeader>
  <CardContent>...</CardContent>
  <CardFooter>Actions</CardFooter>
</Card>
```

### Progress Bars
```svelte
<ProgressBar value={75} max={100} />
<ProgressBar value={3} max={10} showLabel />
```

### Buttons
```svelte
<Button variant="primary">Start Sprint</Button>
<Button variant="secondary">Cancel</Button>
<Button variant="ghost">Back</Button>
<Button disabled>Locked</Button>
```

### Stats Display
```svelte
<StatCard icon="🔥" label="Streak" value="14 days" />
<StatCard icon="💰" label="Coins" value="1,250" />
```

### Avatar Component
```svelte
<Avatar
  creature="cat"
  mood="happy"
  hat="crown"
  held="wand"
  aura="sparkles"
  size="large"
/>
```

---

## File Structure

```
gui/
├── wails.json
├── main.go              # Wails app entry
├── app.go               # Go backend bindings
├── frontend/
│   ├── src/
│   │   ├── App.svelte
│   │   ├── main.js
│   │   ├── lib/
│   │   │   ├── stores/      # Svelte stores
│   │   │   │   ├── avatar.js
│   │   │   │   ├── wallet.js
│   │   │   │   └── exams.js
│   │   │   └── components/
│   │   │       ├── Avatar.svelte
│   │   │       ├── Card.svelte
│   │   │       ├── Button.svelte
│   │   │       ├── ProgressBar.svelte
│   │   │       └── ...
│   │   ├── pages/
│   │   │   ├── Dashboard.svelte
│   │   │   ├── Exams.svelte
│   │   │   ├── TakeExam.svelte
│   │   │   ├── Review.svelte
│   │   │   ├── Stats.svelte
│   │   │   ├── Shop.svelte
│   │   │   ├── Achievements.svelte
│   │   │   └── Settings.svelte
│   │   └── styles/
│   │       ├── global.css
│   │       └── themes/
│   ├── index.html
│   └── vite.config.js
└── build/
    └── appicon.png
```

---

## Development Phases

### Phase 1: Scaffold & Dashboard (Week 1-2)
- Set up Wails + Svelte project
- Create component library (Card, Button, ProgressBar)
- Implement Dashboard view
- Connect to daemon via socket

### Phase 2: Exam Flow (Week 2-3)
- Exams list view
- Take exam modal/page
- Question display with code highlighting
- Results view
- Voice mode toggle

### Phase 3: Review & Stats (Week 3-4)
- Review page with due items
- Spaced repetition UI
- Statistics page with charts
- Activity calendar

### Phase 4: Gamification UI (Week 4-5)
- Avatar display component
- Shop page
- Inventory/equip UI
- Achievements page
- Daily challenges/goals

### Phase 5: Settings & Polish (Week 5-6)
- Settings page
- Theme support (dark/light)
- Tray integration
- Notifications
- Keyboard shortcuts

---

## Backend Bindings (app.go)

```go
package main

import (
    "github.com/loljeah/exambuilder/internal/db"
    "github.com/loljeah/exambuilder/internal/gamification"
)

type App struct {
    db  *db.DB
    gam *gamification.Service
}

// Dashboard
func (a *App) GetDashboardData() DashboardData
func (a *App) GetRecentActivity(limit int) []ActivityEntry

// Avatar
func (a *App) GetAvatar() Avatar
func (a *App) SetCreatureType(t string) error
func (a *App) GetWallet() Wallet

// Exams
func (a *App) GetProjects() []Project
func (a *App) GetSprints(projectID string) []Sprint
func (a *App) GetSprintQuestions(sprintID string) []Question
func (a *App) SubmitAnswers(sprintID string, answers []string) SprintResult

// Shop
func (a *App) GetShopItems(category string) []ShopItem
func (a *App) PurchaseItem(itemID string) error
func (a *App) GetInventory() []string
func (a *App) EquipItem(itemID string) error

// Achievements
func (a *App) GetAchievements() []Achievement
func (a *App) GetUnlockedAchievements() []string

// Daily
func (a *App) ClaimDailyReward() (int, error)
func (a *App) GetDailyChallenges() []Challenge
func (a *App) GetWeeklyGoals() []Goal

// Stats
func (a *App) GetStats(period string) StatsData
func (a *App) GetActivityCalendar(month string) []DayActivity

// Settings
func (a *App) GetSettings() Settings
func (a *App) UpdateSettings(s Settings) error
func (a *App) ExportData() (string, error)
func (a *App) ImportData(path string) error
```

---

## Keyboard Shortcuts

| Key | Action |
|-----|--------|
| `Ctrl+1` | Dashboard |
| `Ctrl+2` | Exams |
| `Ctrl+3` | Review |
| `Ctrl+4` | Stats |
| `Ctrl+5` | Shop |
| `Ctrl+6` | Achievements |
| `Ctrl+,` | Settings |
| `Ctrl+Q` | Quit |
| `Space` | Select answer (in exam) |
| `Enter` | Submit / Confirm |
| `Escape` | Cancel / Back |
| `1-4` | Quick select answer A-D |
| `V` | Toggle voice mode |
