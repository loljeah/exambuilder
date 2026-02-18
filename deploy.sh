#!/usr/bin/env bash
# deploy.sh — Install KnowledgeGATEunlocker system
#
# Installs:
#   1. Claude Code integration files (CLAUDE.md, SKILL.md)
#   2. Knowledge gate data directory structure
#
# NOTE: No git hooks — we block Claude, not git pushes

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
KNOWLEDGE_GATE_DIR="${HOME}/gitZ/.knowledge-gate"

echo "🎯 Installing KnowledgeGATEunlocker..."
echo ""

# 1. CLAUDE.md → ~/.claude/CLAUDE.md
echo "→ Installing CLAUDE.md..."
mkdir -p ~/.claude
cp "${SCRIPT_DIR}/CLAUDE.md" ~/.claude/CLAUDE.md
echo "  ✓ ~/.claude/CLAUDE.md"

# 2. SKILL.md → ~/.claude/skills/teachANDexam/SKILL.md
echo "→ Installing SKILL.md..."
mkdir -p ~/.claude/skills/teachANDexam
cp "${SCRIPT_DIR}/SKILL.md" ~/.claude/skills/teachANDexam/SKILL.md
echo "  ✓ ~/.claude/skills/teachANDexam/SKILL.md"

# 3. Create knowledge gate data directory structure
echo "→ Creating data directory structure..."
mkdir -p "${KNOWLEDGE_GATE_DIR}/db"
mkdir -p "${KNOWLEDGE_GATE_DIR}/export/projects"
mkdir -p "${KNOWLEDGE_GATE_DIR}/projects"
echo "  ✓ ${KNOWLEDGE_GATE_DIR}/"
echo "    ├── db/"
echo "    ├── export/projects/"
echo "    └── projects/"

# 4. Create default config if not exists
if [ ! -f "${KNOWLEDGE_GATE_DIR}/config.toml" ]; then
    echo "→ Creating default config..."
    cat > "${KNOWLEDGE_GATE_DIR}/config.toml" << 'EOF'
[general]
projects_root = "~/gitZ"
data_dir = "~/gitZ/.knowledge-gate"

[knowledge_debt]
threshold = 10
debt_per_sprint_cleared = 3
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
debt_warning_at = 7

[appearance]
theme = "system"

[grading]
pass_threshold = 60
streak_bonus_at = 3
show_hints_on_fail = 1
show_answers_on_fail = 2

[voice]
enabled = false
tts_voice = "default"
stt_mode = "push-to-talk"
EOF
    echo "  ✓ ${KNOWLEDGE_GATE_DIR}/config.toml"
else
    echo "  ⊘ config.toml already exists, skipping"
fi

echo ""
echo "╔═══════════════════════════════════════════════════════╗"
echo "║  ✓ KnowledgeGATEunlocker installed successfully!      ║"
echo "║                                                       ║"
echo "║  Claude Code integration:                             ║"
echo "║    ~/.claude/CLAUDE.md                                ║"
echo "║    ~/.claude/skills/teachANDexam/SKILL.md             ║"
echo "║                                                       ║"
echo "║  Data directory:                                      ║"
echo "║    ~/gitZ/.knowledge-gate/                            ║"
echo "║                                                       ║"
echo "║  How it works:                                        ║"
echo "║    - Claude tracks knowledge debt as you build        ║"
echo "║    - At 10 concepts, Claude enters read-only mode     ║"
echo "║    - Pass exam sprints to clear debt and resume       ║"
echo "║    - Git pushes are NEVER blocked                     ║"
echo "╚═══════════════════════════════════════════════════════╝"
