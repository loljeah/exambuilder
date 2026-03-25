# Knowledge Gate (kgate)

A gamified, ADHD-optimized knowledge verification system. **You built it. Can you explain it?**

## Overview

Knowledge Gate tracks "knowledge debt" as you work on projects. When debt reaches the threshold, you take short exam sprints to clear it.

### Features

- **Sprint-based exams**: 3 questions per sprint, 3-5 minutes each
- **Gamification**: XP, levels, coins, streaks, achievements, daily/weekly rewards
- **Hint tokens**: Buy hints with coins, reveal them on-the-fly during exams
- **Knowledge Forge**: Generate new sprints and exams via local LLM (Ollama)
- **Voice mode**: TTS reads questions, STT accepts spoken answers
- **Auto-import**: File watcher detects `exam_*.md` files automatically
- **Desktop GUI**: Wails + Svelte app with dashboard, store, stats, and exam runner
- **System tray**: Visual debt indicator (green/yellow/red)
- **ADHD-friendly**: Short sprints, instant feedback, no punishment for building
- **Analytics**: Activity journal, per-question stats, knowledge mastery tracking
- **Spaced repetition**: SM-2 algorithm for optimal review scheduling

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

### Prerequisites

- [Nix](https://nixos.org/download/) with flakes enabled
- (Optional) [Ollama](https://ollama.ai) for LLM-powered exam generation

### CLI Mode

```bash
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

### GUI Mode (Wails)

```bash
nix develop
cd gui && wails dev      # Development with hot-reload
cd gui && wails build    # Production build
```

### Ollama Setup (for Knowledge Forge)

The Knowledge Forge lets you generate new exam content using a local LLM. Run the guided setup script:

```bash
nix develop
./scripts/setup-ollama.sh
```

The script will:
1. **Detect your GPU** (NVIDIA, AMD/Radeon, Intel Arc, or CPU-only)
2. **Recommend a model** based on your VRAM/RAM
3. **Start Ollama** if it's not running
4. **Pull the model** interactively
5. **Update your config** if needed

#### GPU Support

| GPU | Detection | Acceleration |
|-----|-----------|--------------|
| NVIDIA | `nvidia-smi` | CUDA (automatic) |
| AMD/Radeon | `lspci` + `rocm-smi` | ROCm (requires `hardware.amdgpu.opencl.enable = true` in NixOS) |
| Intel Arc | `lspci` | Experimental |
| CPU-only | fallback | Works, slower inference |

#### Model Recommendations

| Hardware | Model | Size |
|----------|-------|------|
| 24GB+ VRAM | `llama3.1:70b-instruct-q4_0` | ~40 GB |
| 12-24GB VRAM | `llama3.1:8b` | 4.7 GB |
| 8-12GB VRAM | `llama3.1:8b-instruct-q4_0` | ~4 GB |
| 6-8GB VRAM | `llama3.2:3b` | 2 GB |
| CPU 32GB+ RAM | `llama3.1:8b` | 4.7 GB |
| CPU 16GB RAM | `llama3.2:3b` | 2 GB |
| CPU <16GB RAM | `llama3.2:1b` | 1 GB |

#### Manual Setup

If you prefer to set up Ollama manually:

```bash
ollama serve &                  # Start the service
ollama pull llama3.1:8b         # Pull the default model
```

To use a different model, update `~/.config/kgate/config.toml`:

```toml
[ollama]
base_url = "http://localhost:11434"
model = "llama3.2:3b"           # Change this
timeout_seconds = 120
max_retries = 2
```

#### AMD/Radeon on NixOS

If you have an AMD GPU but Ollama falls back to CPU mode, enable ROCm in your `configuration.nix`:

```nix
hardware.amdgpu.opencl.enable = true;
```

Then rebuild: `sudo nixos-rebuild switch`

## Hint Tokens

Buy hint tokens with coins, then spend them during exams to reveal a hint before answering.

| Pack | Tokens | Cost | Per Token |
|------|--------|------|-----------|
| Starter | 3 | 30 coins | 10 coins |
| Value | 10 | 80 coins | 8 coins |
| Bulk | 25 | 150 coins | 6 coins |

- Hints are **proactive** (use before answering) vs. post-failure hints which are reactive
- Each hint can only be used once per question (no double-spend)
- Hint usage does **not** penalize your score — you paid for the right to use them
- In voice mode, hints are read aloud via TTS

## Knowledge Forge (LLM Generation)

Generate new sprints and exams on any domain using a local LLM. Generation unlocks as you level up:

| Domain Level | Unlocked | Cost |
|---|---|---|
| 3+ | Sprint (3 questions) | 50 coins (first FREE per domain) |
| 5+ | Custom difficulty sprint | 75 coins |
| 8+ | Full exam (3 sprints) | 200 coins |
| 10+ | Cross-domain challenge | 100 coins |

The dopamine loop: **Study > Earn coins + XP > Level up > Unlock generation > Generate new content > Study more**

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

## Data Locations

All data follows XDG Base Directory spec for easy backup/restore:

| Purpose | Location |
|---------|----------|
| Config | `~/.config/kgate/config.toml` |
| Database | `~/.local/share/kgate/kgate.db` |
| Socket | `/run/user/$UID/kgate.sock` |

### Backup & Restore

```bash
# Backup everything
tar -czvf kgate-backup.tar.gz ~/.config/kgate ~/.local/share/kgate

# Restore on new system
tar -xzvf kgate-backup.tar.gz -C ~/
```

## Project Structure

```
exambuilder/
├── cmd/
│   ├── kgate-daemon/     # Daemon entry point
│   │   └── migrations/   # Embedded SQL migrations (001-005)
│   └── kgatectl/         # CLI entry point
├── gui/                  # Wails desktop app
│   ├── app.go            # Backend (Wails bindings)
│   └── frontend/         # Svelte SPA
│       └── src/pages/    # Dashboard, Store, TakeExam, Stats, etc.
├── internal/
│   ├── config/           # Configuration (TOML)
│   ├── daemon/           # Socket server
│   ├── db/               # SQLite layer + analytics
│   ├── exam/             # Parser + grader
│   ├── gamification/     # Wallet, achievements, hints, XP
│   ├── llm/              # Ollama client, prompts, parser, generator
│   ├── tray/             # System tray
│   ├── voice/            # TTS/STT clients
│   └── watcher/          # File watcher
├── scripts/
│   └── setup-ollama.sh   # Interactive LLM setup with GPU detection
├── assets/               # Embedded icons
├── systemd/              # Service files
├── docs/                 # Documentation
└── flake.nix             # Nix build + devShell
```

## Stack

- **Language:** Go
- **GUI:** Wails v2 + Svelte
- **Database:** SQLite (WAL mode)
- **LLM:** Ollama (local inference)
- **IPC:** Unix sockets
- **TTS:** piper-daemon
- **STT:** moonshine-daemon
- **Tray:** fyne.io/systray

## License

MIT
