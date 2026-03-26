#!/usr/bin/env bash
# Quick rebuild + restart without nixos-rebuild.
# Usage:
#   ./scripts/dev-rebuild.sh          # rebuild all, restart daemon
#   ./scripts/dev-rebuild.sh gui      # rebuild GUI only, launch it
#   ./scripts/dev-rebuild.sh daemon   # rebuild daemon+CLI only, restart daemon
#   ./scripts/dev-rebuild.sh all      # rebuild all, restart daemon + launch GUI
set -euo pipefail

PROJ="$(cd "$(dirname "$0")/.." && pwd)"
BIN="$PROJ/build/bin"
TARGET="${1:-all}"

mkdir -p "$BIN"

stop_daemon() {
    if pgrep -f kgate-daemon >/dev/null 2>&1; then
        echo ":: Stopping old daemon..."
        pkill -f kgate-daemon || true
        sleep 1
    fi
}

build_pkg() {
    local pkg="$1" label="$2"
    echo ":: Building $label..."
    out=$(nix build "$PROJ#$pkg" --no-link --print-out-paths 2>&1 | tail -1)
    echo "   $out"
    stop_daemon
    chmod u+w "$BIN"/* 2>/dev/null || true
    cp -L "$out"/bin/* "$BIN"/
    echo "   -> copied to $BIN"
}

restart_daemon() {
    echo ":: Starting daemon..."
    "$BIN/kgate-daemon" &
    disown
    sleep 1
    if pgrep -f kgate-daemon >/dev/null 2>&1; then
        echo "   daemon running (PID $(pgrep -f kgate-daemon | head -1))"
    else
        echo "   ERROR: daemon failed to start"
        exit 1
    fi
}

launch_gui() {
    echo ":: Launching GUI..."
    "$BIN/kgate-gui" &
    disown
}

case "$TARGET" in
    gui)
        build_pkg gui "GUI"
        launch_gui
        ;;
    daemon)
        build_pkg daemon "daemon + CLI"
        restart_daemon
        ;;
    all)
        build_pkg default "all packages"
        restart_daemon
        launch_gui
        ;;
    *)
        build_pkg default "all packages"
        restart_daemon
        ;;
esac

echo ":: Done."
