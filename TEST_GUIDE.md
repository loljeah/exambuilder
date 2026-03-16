# Manual Test Guide — Phase A + Phase D

Covers all new features from the stabilization and LLM generation phases.
Run all commands from the project root inside `nix-shell`.

---

## Prerequisites

```bash
nix-shell                    # enter dev environment
just build                   # compile everything
kgate init                   # ensure DB exists (skip if already done)
```

---

## 1. Automated Tests

Run the full suite first to confirm everything compiles and passes.

```bash
just check                   # fmt-check + lint-all + test (unit + integration)
```

Expected: 186 tests pass, 0 failures, 0 warnings from clippy.

Break it down if needed:

```bash
just test-unit               # 160 unit tests (lib only)
just test-integ              # 24 integration tests (CLI + pipeline + LLM)
just lint-all                # clippy on workspace including tests
just fmt-check               # formatting check
```

---

## 2. Selftest Command

### 2a. Basic selftest

```bash
kgate selftest
```

Expected output: table of PASS/WARN/FAIL checks with a summary line.
Most checks should PASS. Common WARNs:
- "whisper not found" (STT binary missing — OK for non-voice setups)
- "ANTHROPIC_API_KEY not set" (expected if no API key configured)

### 2b. Verbose selftest

```bash
kgate selftest --verbose
```

Expected: same checks but with detail lines under each WARN/FAIL.

### 2c. Fix mode

```bash
kgate selftest --fix
```

Expected: attempts to fix issues (re-run migrations, create missing dirs).

---

## 3. Generate Command — Template Mode

### 3a. Dry run on this project

```bash
kgate generate . --templates --dry-run
```

Expected:
- Shows "Analyzing codebase" with element count
- Shows "Mode: Templates"
- Prints first 50 lines of generated exam markdown
- Does NOT write any file

### 3b. Generate to a specific output file

```bash
kgate generate . --templates -o /tmp/test_exam.md
```

Expected:
- Creates `/tmp/test_exam.md`
- File contains valid exam markdown with sprint headers, questions, answer key

### 3c. Verify the generated exam is parseable

```bash
kgate scan /tmp --import
```

Expected: finds and imports the exam file without errors.

### 3d. Generate from a different project

```bash
kgate generate /path/to/any/project --templates --dry-run
```

Try with various project types (Rust, Python, Nix configs) to verify
the analyzer detects elements from different languages.

---

## 4. Generate Command — LLM Mode

Requires `ANTHROPIC_API_KEY` set in environment.

### 4a. Auto-detect mode

```bash
export ANTHROPIC_API_KEY="sk-ant-..."
kgate generate . --dry-run
```

Expected: auto-detects LLM mode, shows "Mode: LLM (claude-opus-4-20250514)",
generates questions via the API, shows progress bar, displays preview.

### 4b. Force LLM mode

```bash
ANTHROPIC_API_KEY="sk-ant-..." kgate generate . --llm --dry-run
```

Expected: same as above but explicitly forced.

### 4c. LLM mode without API key should fail

```bash
unset ANTHROPIC_API_KEY
kgate generate . --llm --dry-run
```

Expected: error message about missing ANTHROPIC_API_KEY.

### 4d. Custom model override

```bash
ANTHROPIC_API_KEY="sk-ant-..." kgate generate . --llm --model claude-sonnet-4-20250514 --dry-run
```

Expected: shows the overridden model name in output.

### 4e. Full LLM generation (writes file)

```bash
ANTHROPIC_API_KEY="sk-ant-..." kgate generate . --llm -o /tmp/llm_exam.md
```

Expected:
- Progress bar showing sprint generation
- Token usage summary
- File written to `/tmp/llm_exam.md`
- Questions reference actual files/functions from the project
- Questions are more specific and varied than template mode

### 4f. Verify LLM exam parses correctly

```bash
kgate scan /tmp --import
```

Expected: imports the LLM-generated exam without parse errors.

### 4g. Compare template vs LLM output

```bash
kgate generate . --templates -o /tmp/template_exam.md
ANTHROPIC_API_KEY="sk-ant-..." kgate generate . --llm -o /tmp/llm_exam.md
diff /tmp/template_exam.md /tmp/llm_exam.md
```

Expected: LLM questions are project-specific and diverse. Template questions
are generic patterns ("What does pub mean?", "What is a crate dependency?").

---

## 5. CLI Refactoring Verification

These commands should work exactly as before the refactor. Verify no regressions.

```bash
kgate status
kgate profile
kgate badges
kgate history
kgate domains
kgate config show
kgate legend
kgate whoami
kgate list
```

Expected: all commands produce output, no crashes, no missing data.

---

## 6. Exam Workflow End-to-End

### 6a. Generate, import, take

```bash
kgate generate . --templates -o exam_exambuilder.md
kgate scan . --import
kgate list
kgate take exam 1 sprint 1
```

Expected: shows exam in list, can take sprint, grading works.

### 6b. Profile updates after taking exam

```bash
kgate profile
kgate history
```

Expected: XP and attempt history reflect the sprint you just took.

---

## 7. Justfile Recipes

Verify all recipes work:

```bash
just                         # lists all recipes
just build                   # compiles
just test                    # runs test-unit + test-integ
just selftest                # runs kgate selftest
just lint                    # clippy (production code)
just lint-all                # clippy (including tests)
just fmt-check               # check formatting
just check                   # full pipeline: fmt + lint + test
```

---

## 8. Edge Cases

### 8a. Empty directory

```bash
mkdir /tmp/empty_project
kgate generate /tmp/empty_project --templates --dry-run
```

Expected: "Not enough code elements" message, no crash.

### 8b. Non-existent path

```bash
kgate generate /nonexistent/path --templates
```

Expected: filesystem error, not a panic.

### 8c. --llm and --templates together

```bash
kgate generate . --llm --templates --dry-run
```

Expected: `--templates` takes priority (forces template mode).

---

## Summary Checklist

| # | Test | Status |
|---|------|--------|
| 1 | `just check` passes (186 tests) | [ ] |
| 2a | `kgate selftest` runs | [ ] |
| 2b | `kgate selftest --verbose` shows details | [ ] |
| 3a | Template dry-run shows preview | [ ] |
| 3b | Template generates valid file | [ ] |
| 3c | Generated exam imports via scan | [ ] |
| 4a | LLM auto-detect works with API key | [ ] |
| 4b | `--llm` forces LLM mode | [ ] |
| 4c | `--llm` without key fails cleanly | [ ] |
| 4e | Full LLM generation writes file | [ ] |
| 4f | LLM exam imports via scan | [ ] |
| 5 | Existing commands work (no regressions) | [ ] |
| 6a | End-to-end: generate, import, take | [ ] |
| 7 | All justfile recipes work | [ ] |
| 8a | Empty directory handled | [ ] |
| 8b | Bad path handled | [ ] |
