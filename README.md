# Knowledge Gate (kgate)

A gamified, ADHD-optimized knowledge verification system. **You built it. Can you explain it?**

## Overview

Knowledge Gate tracks "knowledge debt" as you work on projects. When debt reaches the threshold, you take short exam sprints to clear it.

### Features

- **Sprint-based exams**: 3 questions per sprint, 3-5 minutes each
- **Gamification**: XP, levels, streaks, progress tracking
- **Voice mode**: TTS reads questions, STT accepts spoken answers
- **Auto-import**: File watcher detects `exam_*.md` files automatically
- **System tray**: Visual debt indicator (green/yellow/red)
- **ADHD-friendly**: Short sprints, instant feedback, no punishment for building

## Architecture

```
┌─────────────┐     Unix Socket     ┌──────────────┐
│  kgatectl   │ ◄─────────────────► │ kgate-daemon │
│    (CLI)    │                     │   (Server)   │
└─────────────┘                     └──────┬───────┘
                                           │
                    ┌──────────────────────┼──────────────────────┐
                    │                      │                      │
              ┌─────▼─────┐         ┌──────▼──────┐        ┌──────▼──────┐
              │  SQLite   │         │ File Watcher │        │ System Tray │
              │    DB     │         │  (fsnotify)  │        │  (systray)  │
              └───────────┘         └──────────────┘        └─────────────┘
                                           │
                                    ┌──────▼──────┐
                                    │ Voice Daemons│
                                    │piper/moonshine│
                                    └─────────────┘
```

## Quick Start

```bash
# Build
nix develop
go build ./cmd/kgate-daemon
go build ./cmd/kgatectl

# Start daemon (with system tray)
./kgate-daemon

# In another terminal
./kgatectl status
./kgatectl import exam_myproject.md
./kgatectl take 1
```

## Knowledge Debt System

| Action | Debt |
|--------|------|
| Concept explained | +1 |
| Architecture decision | +2 |
| Bug fix with explanation | +1 |
| New file created | +1 |
| Complex code written | +2 |

**Threshold:** 10 = read-only mode
**Clear:** Pass sprint = -3 debt

### Read-Only Mode (Debt >= 10)

```
BLOCKED:
├── Write, Edit, NotebookEdit
├── Bash: echo >, cat <<, sed -i, tee, curl|sh
└── Any file creation/modification

ALLOWED:
├── Read, Grep, Glob
├── Bash: ls, cat, git status (read-only)
└── Brief verbal answers
```

## Commands

| Command | Description |
|---------|-------------|
| `kgatectl status` | Show debt, XP, level, streak |
| `kgatectl project [path]` | Get/set active project |
| `kgatectl projects` | List all projects |
| `kgatectl sprints` | List sprints for active project |
| `kgatectl take <n>` | Take sprint N interactively |
| `kgatectl take <n> --voice` | Voice mode (TTS) |
| `kgatectl take <n> --voice-full` | Full voice (TTS + STT) |
| `kgatectl import <file.md>` | Import exam file |
| `kgatectl profile` | Show XP, level, streak |
| `kgatectl quit` | Stop the daemon |

## Documentation

- [Setup Guide](docs/SETUP.md) - Installation and configuration
- [Wiki](docs/WIKI.md) - Detailed documentation
- [Systemd Service](systemd/README.md) - Service installation

## Why This Works for ADHD

- **No punishment for building** - push code anytime
- **Natural break point** - exam is a context switch, good for focus reset
- **Prevents scatter-brain abuse** - can't infinitely copy-paste without understanding
- **Read-only still helps** - can explain what you built, just can't add more chaos
- **Gamification** - XP, streaks, levels provide dopamine hits

## Project Structure

```
exambuilder/
├── cmd/
│   ├── kgate-daemon/     # Daemon entry point
│   └── kgatectl/         # CLI entry point
├── internal/
│   ├── config/           # Configuration
│   ├── daemon/           # Socket server
│   ├── db/               # SQLite layer
│   ├── exam/             # Parser + grader
│   ├── tray/             # System tray
│   ├── voice/            # TTS/STT clients
│   └── watcher/          # File watcher
├── assets/               # Embedded icons
├── migrations/           # SQL migrations
├── systemd/              # Service files
├── docs/                 # Documentation
└── flake.nix             # Nix build
```

## Stack

- **Language:** Go
- **Database:** SQLite (WAL mode)
- **IPC:** Unix sockets
- **TTS:** piper-daemon
- **STT:** moonshine-daemon
- **Tray:** fyne.io/systray

## License

MIT
