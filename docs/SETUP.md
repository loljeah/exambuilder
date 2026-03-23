# Setup Guide

## Prerequisites

- Go 1.21+
- SQLite3
- For voice mode: piper-daemon, moonshine-daemon
- For system tray: GTK3 development libraries

### NixOS

```bash
nix develop
```

This provides all dependencies automatically.

### Other Linux

```bash
# Debian/Ubuntu
sudo apt install golang sqlite3 libgtk-3-dev libayatana-appindicator3-dev

# Fedora
sudo dnf install golang sqlite gtk3-devel libappindicator-gtk3-devel

# Arch
sudo pacman -S go sqlite gtk3 libappindicator-gtk3
```

## Installation

### From Source

```bash
# Clone
git clone https://github.com/loljeah/exambuilder.git
cd exambuilder

# Build
go build ./cmd/kgate-daemon
go build ./cmd/kgatectl

# Install to user bin
mkdir -p ~/.local/bin
cp kgate-daemon kgatectl ~/.local/bin/

# Ensure ~/.local/bin is in PATH
echo 'export PATH="$HOME/.local/bin:$PATH"' >> ~/.bashrc
```

### With Nix Flake

```bash
nix build .#kgate-daemon
nix build .#kgatectl

# Or run directly
nix run .#kgate-daemon
```

## Configuration

### Config File Location

```
~/.kgate/config.toml
```

The config is created automatically with defaults on first run.

### Default Configuration

```toml
[general]
# Root directory to watch for exam files
projects_root = "~/gitZ"

# Data directory (database, config)
data_dir = "~/.kgate"

# Unix socket path for daemon communication
socket_path = "/run/user/1000/kgate.sock"

[knowledge_debt]
# Debt threshold before read-only mode
threshold = 10

# Debt cleared per passed sprint
debt_per_sprint_cleared = 3

# Debt weights per action type
[knowledge_debt.weights]
concept_explained = 1
architecture_decision = 2
bug_fix = 1
new_file = 1
complex_code = 2
why_not_question = 1

[voice]
# Enable voice features
enabled = true

# piper-daemon socket for TTS
piper_daemon_socket = "/run/user/1000/piper-daemon.sock"

# moonshine-daemon socket for STT
moonshine_socket = "/tmp/moonshine/moonshine.sock"

[grading]
# Pass threshold percentage (60 = 2/3 correct)
pass_threshold = 60

# Streak count for bonus XP
streak_bonus_at = 3

# Attempt number to show hints
show_hints_on_fail = 1

# Attempt number to show full answers
show_answers_on_fail = 2
```

## Running the Daemon

### Manually

```bash
# With system tray
kgate-daemon

# Without system tray (headless/server)
kgate-daemon -no-tray

# Verbose logging
kgate-daemon -v
```

### As Systemd User Service

```bash
# Copy service file
mkdir -p ~/.config/systemd/user
cp systemd/kgate-daemon.service ~/.config/systemd/user/

# Enable and start
systemctl --user daemon-reload
systemctl --user enable kgate-daemon
systemctl --user start kgate-daemon

# Check status
systemctl --user status kgate-daemon
```

### NixOS Home Manager

Add to your Home Manager configuration:

```nix
{ pkgs, ... }:

{
  systemd.user.services.kgate-daemon = {
    Unit = {
      Description = "Knowledge Gate Daemon";
      After = [ "graphical-session.target" ];
      PartOf = [ "graphical-session.target" ];
    };
    Service = {
      Type = "simple";
      ExecStart = "${pkgs.kgate}/bin/kgate-daemon";
      Restart = "on-failure";
      RestartSec = 5;
    };
    Install.WantedBy = [ "graphical-session.target" ];
  };
}
```

## Voice Mode Setup

### piper-daemon (TTS)

```bash
# Clone and build piper-daemon
git clone https://github.com/loljeah/piper-daemon.git
cd piper-daemon
go build ./cmd/piper-daemon

# Start
./piper-daemon
```

Ensure the socket path matches your config:
```toml
[voice]
piper_daemon_socket = "/run/user/1000/piper-daemon.sock"
```

### moonshine-daemon (STT)

```bash
# Clone and build moonshine-daemon
git clone https://github.com/loljeah/moonshine.git
cd moonshine
go build ./cmd/moonshine-daemon

# Start
./moonshine-daemon
```

Ensure the socket path matches your config:
```toml
[voice]
moonshine_socket = "/tmp/moonshine/moonshine.sock"
```

## Creating Exam Files

### File Naming

Exam files must be named `exam_<anything>.md` and placed in a project directory under your `projects_root`.

Example: `~/gitZ/myproject/exam_myproject.md`

### File Format

```markdown
# Exam: Project Name
# Generated: 2024-01-01

---

## Sprint 1: Topic Name

### Q1. [RECALL] Easy — 10 XP

What is the question?

- A) First option
- B) Second option
- C) Third option
- D) Fourth option

### Q2. [COMPREHENSION] Medium — 10 XP

Another question with code:

```go
func example() {
    fmt.Println("code")
}
```

What does this code do?

- A) Option A
- B) Option B
- C) Option C
- D) Option D

---

## Answer Key

### Sprint 1

**Q1. Answer: B**
Hint: Think about X
Full: Detailed explanation

**Q2. Answer: C**
```

### Auto-Import

The daemon watches `projects_root` for `exam_*.md` files. When created or modified, they're automatically parsed and imported.

### Manual Import

```bash
kgatectl import /path/to/exam_file.md
```

## Troubleshooting

### Daemon won't start

```bash
# Check if socket exists
ls -la /run/user/$(id -u)/kgate.sock

# Remove stale socket
rm /run/user/$(id -u)/kgate.sock

# Check logs
journalctl --user -u kgate-daemon -f
```

### Can't connect to daemon

```bash
# Check daemon is running
pgrep kgate-daemon

# Test connection
echo "health" | nc -U /run/user/$(id -u)/kgate.sock
```

### Voice not working

```bash
# Check piper-daemon
echo "status" | nc -U /run/user/$(id -u)/piper-daemon.sock

# Check moonshine-daemon
echo "status" | nc -U /tmp/moonshine/moonshine.sock
```

### Database issues

```bash
# Database location
ls -la ~/.kgate/kgate.db

# Reset database (WARNING: loses all data)
rm ~/.kgate/kgate.db
```
