#!/usr/bin/env bash
# setup-ollama.sh — Interactive Ollama setup with GPU detection
#
# Detects GPU hardware (NVIDIA, AMD/Radeon, Intel Arc, CPU-only),
# recommends an appropriate model, and guides through setup.
#
# Usage: ./scripts/setup-ollama.sh

set -euo pipefail

# ─── Colors (degrade gracefully if not a terminal) ───────────────────────────

if [ -t 1 ]; then
  BOLD='\033[1m'
  DIM='\033[2m'
  RED='\033[0;31m'
  GREEN='\033[0;32m'
  YELLOW='\033[0;33m'
  BLUE='\033[0;34m'
  CYAN='\033[0;36m'
  RESET='\033[0m'
else
  BOLD='' DIM='' RED='' GREEN='' YELLOW='' BLUE='' CYAN='' RESET=''
fi

# ─── Utility functions ───────────────────────────────────────────────────────

info()  { echo -e "  ${GREEN}✓${RESET} $*"; }
warn()  { echo -e "  ${YELLOW}⚠${RESET} $*"; }
err()   { echo -e "  ${RED}✗${RESET} $*"; }
step()  { echo -e "\n${BOLD}$*${RESET}"; }

prompt_yn() {
  local prompt=$1 default=${2:-y}
  local yn
  if [[ "$default" == "y" ]]; then
    read -rp "  $prompt [Y/n] " yn
    yn=${yn:-y}
  else
    read -rp "  $prompt [y/N] " yn
    yn=${yn:-n}
  fi
  [[ "${yn,,}" == "y" ]]
}

prompt_choice() {
  local prompt=$1
  local result
  read -rp "  $prompt " result
  echo "$result"
}

# ─── State variables ─────────────────────────────────────────────────────────

GPU_TYPE="cpu"      # nvidia | amd_rocm | amd_no_rocm | intel_arc | cpu
GPU_NAME=""
VRAM_MB=0
RAM_GB=0
REC_MODEL=""
REC_SIZE=""
REC_REASON=""
OLLAMA_STATUS=""    # missing | stopped | running
PULLED_MODEL=""
KGATE_CONFIG="${XDG_CONFIG_HOME:-$HOME/.config}/kgate/config.toml"
DEFAULT_MODEL="llama3.1:8b"

# ─── GPU Detection ───────────────────────────────────────────────────────────

detect_gpu() {
  # NVIDIA
  if command -v nvidia-smi &>/dev/null; then
    local nv_out
    nv_out=$(nvidia-smi --query-gpu=name,memory.total --format=csv,noheader,nounits 2>/dev/null || true)
    if [[ -n "$nv_out" ]]; then
      GPU_TYPE="nvidia"
      GPU_NAME=$(echo "$nv_out" | head -1 | cut -d',' -f1 | xargs)
      VRAM_MB=$(echo "$nv_out" | head -1 | cut -d',' -f2 | xargs)
      return
    fi
  fi

  # AMD / Radeon
  if command -v lspci &>/dev/null; then
    local amd_line
    amd_line=$(lspci 2>/dev/null | grep -iE 'vga|3d|display' | grep -iE 'amd|radeon|advanced micro' | head -1 || true)
    if [[ -n "$amd_line" ]]; then
      GPU_NAME=$(echo "$amd_line" | sed 's/.*: //')

      # Check for ROCm
      if command -v rocm-smi &>/dev/null; then
        GPU_TYPE="amd_rocm"
        local vram_line
        vram_line=$(rocm-smi --showmeminfo vram 2>/dev/null | grep -i 'total' | head -1 || true)
        if [[ -n "$vram_line" ]]; then
          # rocm-smi reports in bytes or MB depending on version
          local vram_val
          vram_val=$(echo "$vram_line" | grep -oP '\d+' | head -1)
          if (( vram_val > 100000 )); then
            VRAM_MB=$(( vram_val / 1048576 ))  # bytes → MB
          else
            VRAM_MB=$vram_val
          fi
        fi
      elif [[ -d /opt/rocm ]]; then
        GPU_TYPE="amd_rocm"
      else
        GPU_TYPE="amd_no_rocm"
        # Try to estimate VRAM from card name
        estimate_amd_vram "$GPU_NAME"
      fi
      return
    fi
  fi

  # Intel Arc
  if command -v lspci &>/dev/null; then
    local intel_line
    intel_line=$(lspci 2>/dev/null | grep -iE 'vga|3d|display' | grep -iE 'intel.*(arc|dg[12])' | head -1 || true)
    if [[ -n "$intel_line" ]]; then
      GPU_TYPE="intel_arc"
      GPU_NAME=$(echo "$intel_line" | sed 's/.*: //')
      return
    fi
  fi

  # CPU fallback
  GPU_TYPE="cpu"
  GPU_NAME="(no discrete GPU detected)"
}

estimate_amd_vram() {
  local name="${1,,}"
  # Common Radeon models and their VRAM
  if [[ "$name" == *"7900 xtx"* ]]; then VRAM_MB=24576
  elif [[ "$name" == *"7900 xt"* ]]; then VRAM_MB=20480
  elif [[ "$name" == *"7900 gre"* ]]; then VRAM_MB=16384
  elif [[ "$name" == *"7800 xt"* ]]; then VRAM_MB=16384
  elif [[ "$name" == *"7700 xt"* ]]; then VRAM_MB=12288
  elif [[ "$name" == *"7600"* ]]; then VRAM_MB=8192
  elif [[ "$name" == *"6950 xt"* ]]; then VRAM_MB=16384
  elif [[ "$name" == *"6900 xt"* ]]; then VRAM_MB=16384
  elif [[ "$name" == *"6800 xt"* ]]; then VRAM_MB=16384
  elif [[ "$name" == *"6800"* ]]; then VRAM_MB=16384
  elif [[ "$name" == *"6700 xt"* ]]; then VRAM_MB=12288
  elif [[ "$name" == *"6700"* ]]; then VRAM_MB=10240
  elif [[ "$name" == *"6600 xt"* ]]; then VRAM_MB=8192
  elif [[ "$name" == *"6600"* ]]; then VRAM_MB=8192
  elif [[ "$name" == *"6500 xt"* ]]; then VRAM_MB=4096
  elif [[ "$name" == *"580"* ]]; then VRAM_MB=8192
  elif [[ "$name" == *"570"* ]]; then VRAM_MB=8192
  elif [[ "$name" == *"vega"* ]]; then VRAM_MB=8192
  else VRAM_MB=0
  fi
}

detect_ram() {
  RAM_GB=$(free --giga 2>/dev/null | awk '/^Mem:/{print $2}' || echo 0)
}

# ─── Model Recommendation ───────────────────────────────────────────────────

recommend_model() {
  local vram_gb=$(( VRAM_MB / 1024 ))

  case "$GPU_TYPE" in
    nvidia)
      if (( vram_gb >= 24 )); then
        REC_MODEL="llama3.1:70b-instruct-q4_0"
        REC_SIZE="~40 GB"
        REC_REASON="Best quality — fits your ${vram_gb}GB VRAM with 4-bit quantization"
      elif (( vram_gb >= 12 )); then
        REC_MODEL="llama3.1:8b"
        REC_SIZE="4.7 GB"
        REC_REASON="Sweet spot of speed and quality for ${vram_gb}GB VRAM"
      elif (( vram_gb >= 8 )); then
        REC_MODEL="llama3.1:8b-instruct-q4_0"
        REC_SIZE="~4 GB"
        REC_REASON="Quantized to fit ${vram_gb}GB VRAM"
      elif (( vram_gb >= 6 )); then
        REC_MODEL="llama3.2:3b"
        REC_SIZE="2 GB"
        REC_REASON="Compact model for ${vram_gb}GB VRAM"
      else
        REC_MODEL="llama3.2:1b"
        REC_SIZE="1 GB"
        REC_REASON="Minimal model for limited VRAM"
      fi
      ;;

    amd_rocm)
      if (( vram_gb >= 16 )); then
        REC_MODEL="llama3.1:8b"
        REC_SIZE="4.7 GB"
        REC_REASON="ROCm-accelerated, great quality for ${vram_gb}GB VRAM"
      elif (( vram_gb >= 8 )); then
        REC_MODEL="llama3.2:3b"
        REC_SIZE="2 GB"
        REC_REASON="Conservative fit for ${vram_gb}GB Radeon VRAM with ROCm"
      else
        REC_MODEL="llama3.2:3b"
        REC_SIZE="2 GB"
        REC_REASON="Safe default for ROCm"
      fi
      ;;

    amd_no_rocm)
      # Without ROCm, Ollama uses CPU — size by RAM not VRAM
      if (( RAM_GB >= 32 )); then
        REC_MODEL="llama3.1:8b"
        REC_SIZE="4.7 GB"
        REC_REASON="CPU mode (no ROCm) — ${RAM_GB}GB RAM can handle 8b"
      elif (( RAM_GB >= 16 )); then
        REC_MODEL="llama3.2:3b"
        REC_SIZE="2 GB"
        REC_REASON="CPU mode (no ROCm) — comfortable fit for ${RAM_GB}GB RAM"
      else
        REC_MODEL="llama3.2:1b"
        REC_SIZE="1 GB"
        REC_REASON="CPU mode (no ROCm) — minimal model for ${RAM_GB}GB RAM"
      fi
      ;;

    intel_arc)
      REC_MODEL="llama3.2:3b"
      REC_SIZE="2 GB"
      REC_REASON="Intel Arc support is experimental — using conservative model"
      ;;

    cpu)
      if (( RAM_GB >= 32 )); then
        REC_MODEL="llama3.1:8b"
        REC_SIZE="4.7 GB"
        REC_REASON="CPU-only — ${RAM_GB}GB RAM can run 8b (slower than GPU)"
      elif (( RAM_GB >= 16 )); then
        REC_MODEL="llama3.2:3b"
        REC_SIZE="2 GB"
        REC_REASON="CPU-only — fits well in ${RAM_GB}GB RAM"
      else
        REC_MODEL="llama3.2:1b"
        REC_SIZE="1 GB"
        REC_REASON="CPU-only — minimal model for ${RAM_GB}GB RAM"
      fi
      ;;
  esac
}

# ─── Ollama Service ──────────────────────────────────────────────────────────

check_ollama() {
  if ! command -v ollama &>/dev/null; then
    OLLAMA_STATUS="missing"
    return
  fi

  if curl -sf http://localhost:11434/api/tags &>/dev/null 2>&1; then
    OLLAMA_STATUS="running"
  else
    OLLAMA_STATUS="stopped"
  fi
}

start_ollama() {
  echo -e "  Starting Ollama..."
  ollama serve &>/dev/null &
  local pid=$!

  # Wait up to 8 seconds for service to be ready
  for i in {1..8}; do
    if curl -sf http://localhost:11434/api/tags &>/dev/null 2>&1; then
      info "Ollama started (PID $pid)"
      OLLAMA_STATUS="running"
      return 0
    fi
    sleep 1
  done

  err "Ollama did not start within 8 seconds"
  echo -e "  Try manually: ${CYAN}ollama serve${RESET}"
  return 1
}

# ─── Model Management ────────────────────────────────────────────────────────

list_installed_models() {
  if [[ "$OLLAMA_STATUS" != "running" ]]; then
    return
  fi

  local tags_json
  tags_json=$(curl -s http://localhost:11434/api/tags 2>/dev/null || echo '{}')

  local models
  models=$(echo "$tags_json" | grep -oP '"name"\s*:\s*"\K[^"]+' || true)

  if [[ -z "$models" ]]; then
    echo -e "  ${DIM}No models installed${RESET}"
    return
  fi

  echo "$models" | while read -r m; do
    if [[ "$m" == "$REC_MODEL" ]]; then
      echo -e "  - ${GREEN}$m${RESET} (recommended)"
    else
      echo -e "  - $m"
    fi
  done
}

model_installed() {
  local model=$1
  if [[ "$OLLAMA_STATUS" != "running" ]]; then
    return 1
  fi
  curl -s http://localhost:11434/api/tags 2>/dev/null | grep -qP "\"name\"\s*:\s*\"${model}\"" 2>/dev/null
}

pull_model() {
  local model=$1
  echo ""
  echo -e "  Pulling ${BOLD}$model${RESET}..."
  echo -e "  ${DIM}(This may take a few minutes depending on your connection)${RESET}"
  echo ""

  if ollama pull "$model"; then
    echo ""
    info "Successfully pulled $model"
    PULLED_MODEL="$model"
    return 0
  else
    echo ""
    err "Failed to pull $model"
    echo -e "  Try manually: ${CYAN}ollama pull $model${RESET}"
    return 1
  fi
}

# ─── Config Update ───────────────────────────────────────────────────────────

update_config() {
  local new_model=$1

  if [[ "$new_model" == "$DEFAULT_MODEL" ]]; then
    info "Model matches default config — no config change needed"
    return
  fi

  if [[ ! -f "$KGATE_CONFIG" ]]; then
    # Create minimal config with ollama section
    mkdir -p "$(dirname "$KGATE_CONFIG")"
    cat > "$KGATE_CONFIG" << EOF
[ollama]
base_url = "http://localhost:11434"
model = "$new_model"
timeout_seconds = 120
max_retries = 2
EOF
    info "Created $KGATE_CONFIG with model = $new_model"
    return
  fi

  # Config exists — update model line
  if ! prompt_yn "Update config to use ${BOLD}$new_model${RESET} instead of default $DEFAULT_MODEL?"; then
    echo -e "  ${DIM}Skipped. You can set it manually in $KGATE_CONFIG${RESET}"
    return
  fi

  # Backup
  cp "$KGATE_CONFIG" "${KGATE_CONFIG}.bak"

  if grep -q '^\[ollama\]' "$KGATE_CONFIG"; then
    # Update existing model line within [ollama] section
    sed -i "/^\[ollama\]/,/^\[/{s|^model = .*|model = \"$new_model\"|}" "$KGATE_CONFIG"
    # If no model line existed, add one after [ollama]
    if ! grep -A5 '^\[ollama\]' "$KGATE_CONFIG" | grep -q '^model ='; then
      sed -i "/^\[ollama\]/a model = \"${new_model//\//\\/}\"" "$KGATE_CONFIG"
    fi
  else
    # Append section
    printf '\n[ollama]\nmodel = "%s"\n' "$new_model" >> "$KGATE_CONFIG"
  fi

  info "Updated $KGATE_CONFIG (backup: ${KGATE_CONFIG}.bak)"
}

# ─── Main ────────────────────────────────────────────────────────────────────

main() {
  echo ""
  echo -e "${BOLD}╔═══════════════════════════════════════════════════════╗${RESET}"
  echo -e "${BOLD}║  Ollama Setup — Knowledge Forge LLM Configuration    ║${RESET}"
  echo -e "${BOLD}╚═══════════════════════════════════════════════════════╝${RESET}"

  # ── Step 1: Detect hardware ──
  step "1. Detecting hardware..."
  detect_gpu
  detect_ram

  case "$GPU_TYPE" in
    nvidia)
      info "NVIDIA GPU: ${BOLD}$GPU_NAME${RESET} ($(( VRAM_MB / 1024 ))GB VRAM)"
      ;;
    amd_rocm)
      info "AMD/Radeon GPU: ${BOLD}$GPU_NAME${RESET}"
      if (( VRAM_MB > 0 )); then
        info "VRAM: $(( VRAM_MB / 1024 ))GB (ROCm detected)"
      else
        info "ROCm detected (VRAM unknown)"
      fi
      ;;
    amd_no_rocm)
      warn "AMD/Radeon GPU: ${BOLD}$GPU_NAME${RESET}"
      if (( VRAM_MB > 0 )); then
        warn "VRAM: ~$(( VRAM_MB / 1024 ))GB (estimated) — ${RED}ROCm not detected${RESET}"
      else
        warn "ROCm not detected — Ollama will use CPU mode"
      fi
      echo ""
      echo -e "  ${YELLOW}To enable GPU acceleration on NixOS:${RESET}"
      echo -e "    Add to ${CYAN}configuration.nix${RESET}:"
      echo -e "      ${DIM}hardware.amdgpu.opencl.enable = true;${RESET}"
      echo -e "    Then: ${CYAN}sudo nixos-rebuild switch${RESET}"
      echo ""
      echo -e "  ${DIM}For now, using CPU mode with a smaller model.${RESET}"
      ;;
    intel_arc)
      warn "Intel Arc GPU: ${BOLD}$GPU_NAME${RESET}"
      warn "Ollama's Intel Arc support is experimental"
      ;;
    cpu)
      info "No discrete GPU detected — using CPU mode"
      ;;
  esac

  info "System RAM: ${RAM_GB}GB"

  # ── Step 2: Recommend model ──
  step "2. Model recommendation"
  recommend_model

  echo -e "  Model:  ${BOLD}${CYAN}$REC_MODEL${RESET} ($REC_SIZE)"
  echo -e "  Reason: $REC_REASON"

  # ── Step 3: Check Ollama ──
  step "3. Checking Ollama status..."
  check_ollama

  case "$OLLAMA_STATUS" in
    missing)
      err "Ollama not found in PATH"
      echo -e "  Enter the dev shell first: ${CYAN}nix develop${RESET}"
      echo -e "  Or install Ollama: ${CYAN}nix-env -iA nixpkgs.ollama${RESET}"
      exit 1
      ;;
    stopped)
      warn "Ollama installed but not running"
      if prompt_yn "Start Ollama now?"; then
        start_ollama || exit 1
      else
        err "Ollama must be running to pull models"
        echo -e "  Start manually: ${CYAN}ollama serve${RESET}"
        exit 0
      fi
      ;;
    running)
      info "Ollama is running"
      ;;
  esac

  # ── Step 4: Show installed models ──
  step "4. Installed models"
  list_installed_models

  # ── Step 5: Interactive pull ──
  step "5. Model setup"

  if model_installed "$REC_MODEL"; then
    info "${BOLD}$REC_MODEL${RESET} is already installed"
    PULLED_MODEL="$REC_MODEL"
  else
    echo -e "  ${BOLD}$REC_MODEL${RESET} ($REC_SIZE) is not installed."
    echo ""

    local choice
    choice=$(prompt_choice "Pull ${BOLD}$REC_MODEL${RESET}? [Y/n/model name] ")
    choice=${choice:-y}

    if [[ "${choice,,}" == "y" ]]; then
      pull_model "$REC_MODEL" || true
    elif [[ "${choice,,}" == "n" ]]; then
      echo -e "  ${DIM}Skipped. Pull manually: ollama pull $REC_MODEL${RESET}"
    else
      # User entered a custom model name
      pull_model "$choice" || true
    fi
  fi

  # ── Step 6: Config update ──
  if [[ -n "$PULLED_MODEL" && "$PULLED_MODEL" != "$DEFAULT_MODEL" ]]; then
    step "6. Configuration"
    update_config "$PULLED_MODEL"
  fi

  # ── Done ──
  echo ""
  echo -e "${BOLD}╔═══════════════════════════════════════════════════════╗${RESET}"
  echo -e "${BOLD}║  Setup complete!                                      ║${RESET}"
  echo -e "${BOLD}║                                                       ║${RESET}"
  if [[ -n "$PULLED_MODEL" ]]; then
  echo -e "${BOLD}║${RESET}  Model: ${CYAN}$PULLED_MODEL${RESET}"
  fi
  echo -e "${BOLD}║${RESET}  Config: ${DIM}$KGATE_CONFIG${RESET}"
  echo -e "${BOLD}║${RESET}"
  echo -e "${BOLD}║${RESET}  Next steps:"
  echo -e "${BOLD}║${RESET}    ${CYAN}cd gui && wails dev${RESET}"
  echo -e "${BOLD}║${RESET}    Open Store > Knowledge Forge to generate exams"
  echo -e "${BOLD}║                                                       ║${RESET}"
  echo -e "${BOLD}╚═══════════════════════════════════════════════════════╝${RESET}"
}

main "$@"
