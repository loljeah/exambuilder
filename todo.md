# kgate — TODO

## Uncommitted Changes (needs commit)

- [x] `flake.nix` — Added `production` build tag to fix GUI binary launch
- [x] `internal/tray/tray.go` — Tray fixes from code review
- [x] `scripts/dev-rebuild.sh` — New: quick rebuild + restart script
- [x] `debug-workflow.md` — New: proofshot visual verification workflow
- [x] `README.md` — Added "Quick Rebuild & Test" section
- [x] `gui/frontend/src/App.svelte` — Removed Review + Stats tabs
- [x] `gui/frontend/src/lib/components/Sidebar.svelte` — Renamed Store, removed Review + Stats, fixed version
- [x] `gui/frontend/src/pages/Projects.svelte` — Stripped exam-taking, import-only
- [x] `gui/frontend/src/pages/Dashboard.svelte` — Hidden Review Items card
- [x] `gui/frontend/src/lib/api.ts` — Removed cosmetic shop API bindings
- [x] `gui/app.go` — Removed cosmetic shop Wails bindings
- [x] `internal/daemon/server.go` — Fixed slog BADKEY, removed cosmetic shop commands
- [x] `internal/gamification/achievements.go` — Zeroed cosmetic stats
- [x] `internal/gamification/shop.go` — DELETED (cosmetic dead code)
- [x] `internal/gamification/equipment.go` — DELETED (cosmetic dead code)
- [x] `internal/gamification/shop_seed.go` — DELETED (cosmetic seed data)
- [x] `internal/llm/parser_test.go` — New: 7 tests for TOML extraction
- [x] `.gitignore` — Added build artifacts, proofshot, GUI build dirs
- [x] `todo.md` — This file

## Bugs / Warnings

- [x] Daemon log: `!BADKEY=/run/user/1000/kgate.sock` — FIXED: slog key-value pairs
- [ ] WebKit signal warning: "Overriding existing handler for signal 10" — cosmetic, works fine

## Future Features (disabled for now)

- [ ] Review tab — spaced repetition review sessions (SM-2 backend exists, UI is stub)
- [ ] Statistics tab — XP charts, accuracy history, streak graphs
- [ ] Cosmetic shop — equipment slots, hats, auras (backend removed, seed data remains in DB)

## Infrastructure

- [x] Add `.gitignore` entries for build artifacts
- [x] Automated tests — 26 tests: exam parser (19) + LLM parser (7)
- [ ] CI/CD pipeline (GitHub Actions or similar)
