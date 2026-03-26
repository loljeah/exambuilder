# Debug & Visual Verification Workflow

## ProofShot — Visual Proof of UI Changes

ProofShot captures screenshots and video of the running frontend to verify UI changes without manual inspection.

### Prerequisites

```bash
# agent-browser must be installed (proofshot dependency)
npm install --prefix ~/.local/share/agent-browser agent-browser

# Add to PATH for the session
export PATH="$HOME/.local/share/agent-browser/node_modules/.bin:$PATH"
```

On NixOS, `npm install -g` won't work (nix store is read-only). Use the `--prefix` approach above.

### Starting a Proofshot Session

The Wails GUI is a native desktop app, not a web server. For proofshot, use the **vite dev server** directly (serves the Svelte frontend on port 8080):

```bash
proofshot start \
  --run "cd gui/frontend && npm run dev -- --port 8080" \
  --port 8080 \
  --description "What you are verifying" \
  --force
```

Notes:
- `--force` overrides any stale session from a previous run
- The vite dev server serves the frontend without the Go backend (API calls will fail/timeout)
- For full backend testing, use `wails dev` instead, but proofshot can't control the native window

### Driving the Browser

```bash
# See all interactive elements (buttons, inputs, links)
proofshot exec snapshot -i

# Navigate to a specific page (click sidebar buttons)
proofshot exec click @e11          # e.g., Projects tab

# Fill a form field
proofshot exec fill @e13 "some text"

# Take a screenshot (saved to proofshot-artifacts/)
proofshot exec screenshot step-name.png
```

### Stopping the Session

```bash
proofshot stop
```

Generates a `SUMMARY.md` with all screenshots, error counts, and duration. Artifacts saved to `proofshot-artifacts/<timestamp>_<description>/`.

### Limitations

- **No Go backend**: Running vite standalone means `window.go.*` API calls will timeout. Good for verifying layout, navigation, and component rendering — not for testing data flow.
- **No video on headless**: Video recording may not produce output in headless Chromium. Screenshots are the primary proof.
- **Port 8080**: Matches the `wails.json` `frontend:dev:watcher` config. Don't change without updating `wails.json`.

## Quick Rebuild & Test (Full Stack)

For testing with the actual Go backend (daemon + GUI binary):

```bash
./scripts/dev-rebuild.sh           # build all, restart daemon
./scripts/dev-rebuild.sh gui       # build GUI only, launch it
./scripts/dev-rebuild.sh daemon    # build daemon+CLI only, restart daemon
./scripts/dev-rebuild.sh all       # build all, restart daemon + launch GUI
```

This rebuilds via `nix build`, copies binaries to `build/bin/`, and restarts services. The GUI launches as a native desktop window.

## Hot-Reload Development

For iterative frontend work (no rebuild needed):

```bash
nix develop
cd gui && wails dev      # auto-reloads on Svelte/Go changes
```

This starts both the vite dev server (port 8080) and the Go backend with Wails bindings.
