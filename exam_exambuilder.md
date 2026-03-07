# Exam: exambuilder
# Generated: 2026-02-20
# Languages: rust, nix
# Pass: 60% per sprint | Retakes: unlimited
# Voice-Ready: yes

---

## Sprint 1: Rust Fundamentals
⏱️ Target: 3 min | 🎯 Pass: 60% | ⚡ 30 XP
🎙️ Voice-compatible: yes

### Q1. [RECALL] Easy — 10 XP
What is `name` in Cargo.toml?

```
Dependency: name
```

- A) A build script
- B) A crate dependency
- C) A binary target
- D) A feature flag

### Q2. [RECALL] Easy — 10 XP
What is `tempfile` in Cargo.toml?

```
Dependency: tempfile
```

- A) A build script
- B) A crate dependency
- C) A binary target
- D) A feature flag

### Q3. [COMPREHENSION] Medium — 10 XP
Why is `scan_directory` marked as `pub`?

```
pub fn scan_directory(root: &Path, max_depth: usize) -> Result<ScanResult> {
    let mut projects: Vec<ScannedProject> = Vec::new();

    scan_recursive(root, root, &mut projects, 0, max_depth)?;
```

- A) It can be called from other modules
- B) It runs in a separate thread
- C) It's automatically tested
- D) It cannot panic

---

## 🔑 Answer Key

### Sprint 1

**Q1. Answer: B** — 10 XP
Hint: Dependencies are external crates
Full: name is an external crate that this project depends on
📁 `/home/ljsm/gitZ/exambuilder/crates/kgate-core/Cargo.toml:2`

**Q2. Answer: B** — 10 XP
Hint: Dependencies are external crates
Full: tempfile is an external crate that this project depends on
📁 `/home/ljsm/gitZ/exambuilder/crates/kgate-core/Cargo.toml:18`

**Q3. Answer: A** — 10 XP
Hint: Think about visibility in Rust
Full: pub makes the function accessible from other modules
📁 `/home/ljsm/gitZ/exambuilder/crates/kgate-core/src/scanner.rs:24`

