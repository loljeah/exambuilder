# Exam: exambuilder
# Generated: 2026-03-08
# Total Sprints: 5
# Pass: 60% per sprint | Retakes: unlimited
# Voice-Ready: yes

---

## Progress Dashboard

| Sprint | Topic | Qs | Status | Score | XP |
|--------|-------|----|--------|-------|----|
| 1 | Rust Workspace & CLI | 3 | TODO | -- | 0/30 |
| 2 | Parser State Machine | 3 | TODO | -- | 0/30 |
| 3 | Voice TTS System | 3 | TODO | -- | 0/35 |
| 4 | Grading & Gamification | 3 | TODO | -- | 0/30 |
| 5 | Security & Debugging | 2 | TODO | -- | 0/25 |

**Total XP: 0 / 150**
**Streak: 0 sprints**
**Level: -- (take first sprint to unlock)**
**Gate Status: BLOCKING**

---

## Sprint 1: Rust Workspace & CLI
Target: 3 min | Pass: 60% | 30 XP
Voice-compatible: yes

### Q1. [RECALL] Easy — 10 XP

In this project's `Cargo.toml`, there are two crates: `kgate` and `kgate-core`. What is `kgate-core` responsible for?

- A) The command-line interface and terminal UI
- B) The core library: parser, grader, database, and models
- C) Voice synthesis and audio playback
- D) The nix build system and packaging

### Q2. [COMPREHENSION] Medium — 10 XP

The CLI uses `clap` with the derive pattern. In `main.rs`, the `Cli` struct has `command: Option<Commands>`. Why is it `Option` instead of required?

- A) To allow optional logging flags
- B) So running `kgate` with no subcommand picks a random unpassed sprint
- C) Because clap requires all fields to be optional
- D) To support piped input from other programs

### Q3. [APPLICATION] Challenge — 10 XP

The `TakeCommands` enum uses nested subcommands so users type `kgate take exam 18 sprint 1 --voice`. What Rust pattern enables this nesting?

```rust
enum TakeCommands {
    Exam {
        number: usize,
        #[command(subcommand)]
        action: TakeExamAction,
    },
}
```

- A) Enum variants with named fields containing another `#[command(subcommand)]` enum
- B) Trait objects stored in a HashMap
- C) Recursive generic type parameters
- D) A custom derive macro that flattens the command tree

---

## Sprint 2: Parser State Machine
Target: 3 min | Pass: 60% | 30 XP
Voice-compatible: yes

### Q1. [RECALL] Easy — 10 XP

In `parser.rs`, the `in_answer_key` boolean flag prevents answer key content from leaking into parsed questions. What two conditions trigger this flag?

- A) The line starts with `---` or contains `Sprint`
- B) The line contains `Answer Key` or contains the key emoji
- C) The line starts with `**Q` or contains `Hint:`
- D) The file ends or an empty line appears after the last question

### Q2. [COMPREHENSION] Medium — 10 XP

The option parser strips letter prefixes using byte-level checks:

```rust
if raw.as_bytes()[0].is_ascii_uppercase()
    && raw.as_bytes()[1] == b')'
    && raw.as_bytes()[2] == b' '
{
    raw[3..].to_string()
```

Why use `as_bytes()` instead of `.chars()` here?

- A) Bytes are faster for single ASCII character checks at known positions
- B) The chars method doesn't support indexing
- C) Bytes handle Unicode better than chars
- D) The Rust compiler requires byte access for string slicing

### Q3. [APPLICATION] Challenge — 10 XP

Before the answer key bug fix, answer key lines like `- Q1: **B** - explanation` were parsed as extra options on the last question. Which **three** code blocks needed `!in_answer_key` guards?

- A) Sprint header parsing, question text parsing, code block parsing
- B) Code block handling, question text parsing, option parsing
- C) Question header parsing, answer line parsing, sprint saving
- D) Option parsing, XP extraction, hint parsing

---

## Sprint 3: Voice TTS System
Target: 3 min | Pass: 60% | 35 XP
Voice-compatible: yes

### Q1. [RECALL] Easy — 10 XP

The TTS system uses three engines with a fallback chain. What is the fallback order when the configured engine fails?

- A) espeak-ng, then piper, then kokoro
- B) kokoro, then piper, then espeak-ng
- C) piper, then espeak-ng, then kokoro
- D) The system picks randomly from available engines

### Q2. [COMPREHENSION] Medium — 10 XP

The `create_tts_with_fallback` function calls `test_tts_engine()` which runs `tts.speak_blocking(".")`. Why does it speak a period character instead of just checking if the binary exists?

- A) To verify the engine can actually produce audio output, not just that the binary is on PATH
- B) Because some engines need a warmup phrase before working
- C) The period character calibrates the audio device
- D) It tests that the speaker volume is not muted

### Q3. [ANALYSIS] Challenge — 15 XP

In `PiperTts::speak_blocking`, piper's stdout is piped to `aplay`:

```rust
let mut piper = Command::new("piper")
    .stdin(Stdio::piped())
    .stdout(Stdio::piped())
    .spawn()?;
let aplay_status = Command::new("aplay")
    .stdin(stdout)
    .status()?;
```

What would happen if `stdout` was set to `Stdio::null()` instead of `Stdio::piped()`?

- A) Piper would write audio to a temporary file instead
- B) No audio would play because aplay would receive no input data
- C) The system would fall back to espeak-ng automatically
- D) Piper would play audio through its own built-in player

---

## Sprint 4: Grading & Gamification
Target: 3 min | Pass: 60% | 30 XP
Voice-compatible: yes

### Q1. [RECALL] Easy — 10 XP

What is the pass threshold for a sprint, and what happens to XP if the sprint is failed?

- A) 60% to pass; XP is only awarded if the sprint is passed
- B) 50% to pass; partial XP awarded based on score
- C) 70% to pass; XP is always awarded regardless of pass/fail
- D) 80% to pass; XP is deducted on failure

### Q2. [COMPREHENSION] Medium — 10 XP

The feedback system uses progressive disclosure. On a **first** failed attempt, what does the user see?

- A) Full correct answers with explanations and study resources
- B) Only which questions were wrong plus one-sentence hints per wrong question
- C) Nothing — just the score and an option to retry
- D) The answer key for the entire exam

### Q3. [APPLICATION] Challenge — 10 XP

The spaced repetition module uses the SM-2 algorithm. Given an item with easiness factor 2.5 and a review quality of 3 (correct but difficult), what happens to the easiness factor?

```
EF' = EF + (0.1 - (5 - q) * (0.08 + (5 - q) * 0.02))
EF' = 2.5 + (0.1 - 2 * (0.08 + 2 * 0.02))
```

- A) It increases to 2.6 because the answer was correct
- B) It decreases to 2.36 because the response was slow
- C) It stays at 2.5 because quality 3 is the neutral point
- D) It resets to 1.3 because difficulty was noted

---

## Sprint 5: Security & Debugging
Target: 3 min | Pass: 60% | 25 XP
Voice-compatible: yes

### Q1. [COMPREHENSION] Easy — 10 XP

The knowledge debt system **locks out all code generation** when debt reaches 10. Which tools are still **allowed** in read-only mode?

- A) Read, Grep, Glob, and read-only Bash commands like `ls` and `git status`
- B) All tools work normally but with a warning message
- C) Only WebSearch and WebFetch
- D) No tools are allowed until the exam is passed

### Q2. [APPLICATION] Challenge — 15 XP

A user reports that voice mode crashes on a question with this error: `index out of bounds: the len is 0 but the index is 0`. The crash happens at `get_voice_answer()`. What is the most likely cause?

- A) The voice config file is missing
- B) A question has no MC options and the code tries to access an empty options list
- C) The microphone device is not connected
- D) The TTS engine failed to speak the question

---

## Answer Key

### Sprint 1

**Q1. Answer: B** — 10 XP
Hint: Look at the crate names — one is a library, one is a binary
Full: kgate-core contains parser, grader, database, models, spaced repetition, and analyzer — all the non-UI logic
Extra: Separating core logic from CLI allows the library to be reused by other frontends like a web UI or mobile app.

**Q2. Answer: B** — 10 XP
Hint: What happens when you type just `kgate` with no arguments?
Full: Making command optional allows the None match arm to trigger cmd_random(), which picks a random unpassed sprint
Extra: This pattern is common in CLI tools like cargo where no subcommand shows help, but here it starts learning immediately.

**Q3. Answer: A** — 10 XP
Hint: Notice the #[command(subcommand)] attribute on the inner field
Full: Clap derive macro allows enum variants with named fields where one field carries another subcommand enum, creating nested command trees
Extra: This is compile-time verified — invalid command combinations are rejected by the type system, not runtime checks.

### Sprint 2

**Q1. Answer: B** — 10 XP
Hint: Check the two string patterns the parser looks for on each line
Full: The parser sets in_answer_key = true when a line contains "Answer Key" or the key emoji character
Extra: This dual detection handles both plain and emoji header formats across different exam files.

**Q2. Answer: A** — 10 XP
Hint: Think about what we know about these characters — are they always ASCII?
Full: Since option prefixes are always single ASCII bytes (A-D, paren, space), byte indexing is safe and avoids iterator overhead from chars()
Extra: In Rust, string indexing with brackets on str panics on non-UTF8 boundaries, but as_bytes() gives raw access safe for known-ASCII positions.

**Q3. Answer: B** — 10 XP
Hint: Which blocks parse content that could appear in the answer key section?
Full: Code block handling, question text parsing, and option parsing all lacked in_answer_key guards, allowing answer key lines to be processed as question content
Extra: The bug was subtle because current_question was never cleared when entering the answer key, so it stayed as a target for appending.

### Sprint 3

**Q1. Answer: B** — 10 XP
Hint: The order goes from highest quality to most widely available
Full: Fallback order is kokoro (neural, best quality) then piper (neural, lighter) then espeak-ng (robotic but always available)
Extra: Kokoro requires a Python environment with the kokoro package, making it the least portable but best-sounding option.

**Q2. Answer: A** — 10 XP
Hint: A binary on PATH does not guarantee it can actually produce sound
Full: The smoke test verifies end-to-end audio: binary exists, model loads, audio device works — not just that which finds the command
Extra: This caught real bugs where piper was installed but had no ONNX voice model downloaded, causing silent failure.

**Q3. Answer: B** — 15 XP
Hint: Think about what Stdio::piped() does versus Stdio::null()
Full: Stdio::piped() connects piper stdout to a pipe that aplay reads from. Stdio::null() sends output to /dev/null so aplay stdin would get EOF immediately with no audio data.
Extra: This is a Unix pipe pattern — piper generates raw PCM audio on stdout, aplay consumes it on stdin, avoiding temporary files entirely.

### Sprint 4

**Q1. Answer: A** — 10 XP
Hint: Check the grader — what does it do with XP when passed is false?
Full: Pass threshold is 60% (score_percent >= 60). XP is only awarded when the sprint is passed — failed attempts earn 0 XP
Extra: This prevents XP farming by repeatedly failing sprints, keeping the reward tied to demonstrated understanding.

**Q2. Answer: B** — 10 XP
Hint: The system reveals information progressively — least info first
Full: First failure shows which questions were wrong plus hint-only nudges. Full answers are not revealed until the second failed attempt.
Extra: This progressive disclosure is an ADHD-friendly pattern — it forces a genuine retry before revealing answers, building stronger memory traces.

**Q3. Answer: B** — 10 XP
Hint: Plug the numbers in: q=3 means the difficulty factor (5-q)=2
Full: EF' = 2.5 + (0.1 - 2*(0.08 + 2*0.02)) = 2.5 + (0.1 - 0.24) = 2.5 - 0.14 = 2.36. The EF decreases because quality 3 indicates difficulty.
Extra: The SM-2 neutral point is quality 4, not 3 — only quality 4 and 5 increase the easiness factor.

### Sprint 5

**Q1. Answer: A** — 10 XP
Hint: Read-only means you can look but not touch
Full: In debt lockdown, only read-only tools work: Read, Grep, Glob, and harmless bash like ls, cat, git status. All write/edit/create tools are blocked.
Extra: This enforcement extends to bash workarounds too — echo to file, sed -i, tee, and curl pipe sh are all explicitly blocked.

**Q2. Answer: B** — 15 XP
Hint: What happens if a question has no A/B/C/D options?
Full: If a question is open-ended (no MC options), options.len() is 0, and any code trying to index into the options array panics. The MC filter fix prevents this.
Extra: This is why the codebase now filters with .filter(|q| !q.options.is_empty()) before entering the question loop — defense in depth against malformed exam data.

---

## Study Resources (unlocked after attempt)

- [The Rust Book: Enums and Pattern Matching](https://doc.rust-lang.org/book/ch06-00-enums.html)
- [Clap Derive Tutorial](https://docs.rs/clap/latest/clap/_derive/_tutorial/index.html)
- [SM-2 Algorithm Explained](https://www.supermemo.com/en/blog/application-of-a-computer-to-improve-the-results-of-student)
- [Rust std::process::Command](https://doc.rust-lang.org/std/process/struct.Command.html)
- [Unix Pipes in Rust](https://doc.rust-lang.org/std/process/struct.Stdio.html)

---

Voice-compatible: yes
Last updated: 2026-03-08
