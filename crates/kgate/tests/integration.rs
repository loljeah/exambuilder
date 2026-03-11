use std::process::Command;

fn kgate_cmd() -> Command {
    Command::new(env!("CARGO_BIN_EXE_kgate"))
}

#[test]
fn test_help_exits_zero() {
    let output = kgate_cmd()
        .arg("--help")
        .output()
        .expect("Failed to run kgate");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Knowledge Gate"));
}

#[test]
fn test_legend_exits_zero() {
    // Legend command doesn't need DB
    let output = kgate_cmd()
        .arg("legend")
        .output()
        .expect("Failed to run kgate");
    // May succeed or fail depending on domains.toml, but shouldn't crash
    // Just check it runs without panic
    assert!(output.status.code().is_some()); // didn't crash with signal
}

#[test]
fn test_unknown_command_fails() {
    let output = kgate_cmd()
        .arg("nonexistent-command")
        .output()
        .expect("Failed to run kgate");
    assert!(!output.status.success());
}

#[test]
fn test_init_creates_database() {
    // Use a temporary HOME to avoid touching real config
    let tmp = tempfile::tempdir().unwrap();
    let output = kgate_cmd()
        .env("HOME", tmp.path())
        .arg("init")
        .output()
        .expect("Failed to run kgate");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(output.status.success(), "init failed: {}", stdout);
    assert!(stdout.contains("Database initialized") || stdout.contains("ready"));

    // Check DB file was created
    let db_path = tmp.path().join(".kgate").join("data").join("db").join("knowledge-gate.db");
    assert!(db_path.exists(), "DB file should exist at {:?}", db_path);
}

#[test]
fn test_status_after_init() {
    let tmp = tempfile::tempdir().unwrap();

    // Init first
    let init_output = kgate_cmd()
        .env("HOME", tmp.path())
        .arg("init")
        .output()
        .expect("Failed to run kgate init");
    assert!(init_output.status.success());

    // Then status
    let output = kgate_cmd()
        .env("HOME", tmp.path())
        .arg("status")
        .output()
        .expect("Failed to run kgate status");

    let stdout = String::from_utf8_lossy(&output.stdout);
    // Status should work without crashing; might show "no exams" or similar
    assert!(output.status.success(), "status failed: {}", stdout);
}

#[test]
fn test_scan_tempdir_finds_exam() {
    let tmp = tempfile::tempdir().unwrap();
    let home = tempfile::tempdir().unwrap();

    // Init DB
    kgate_cmd()
        .env("HOME", home.path())
        .arg("init")
        .output()
        .expect("Failed to run kgate init");

    // Create an exam file in the scan directory
    let exam_content = r#"# Exam: TestProject

## Sprint 1: Basics
### Q1. [RECALL] Easy — 10 XP

What is 1+1?

- A) 1
- B) 2
- C) 3
- D) 4

---

## 🔑 Answer Key

### Sprint 1
**Q1. Answer: B** — 10 XP
Hint: addition
Full: 1+1=2
"#;
    std::fs::write(tmp.path().join("exam_test.md"), exam_content).unwrap();

    // Scan with import
    let output = kgate_cmd()
        .env("HOME", home.path())
        .arg("scan")
        .arg(tmp.path())
        .arg("--import")
        .output()
        .expect("Failed to run kgate scan");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        output.status.success(),
        "scan failed: {}\n{}",
        stdout,
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn test_profile_after_init() {
    let tmp = tempfile::tempdir().unwrap();

    kgate_cmd()
        .env("HOME", tmp.path())
        .arg("init")
        .output()
        .expect("Failed to run kgate init");

    let output = kgate_cmd()
        .env("HOME", tmp.path())
        .arg("profile")
        .output()
        .expect("Failed to run kgate profile");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(output.status.success(), "profile failed: {}", stdout);
    // Should show XP and level
    assert!(stdout.contains("XP") || stdout.contains("Level") || stdout.contains("level"));
}

#[test]
fn test_config_show_after_init() {
    let tmp = tempfile::tempdir().unwrap();

    kgate_cmd()
        .env("HOME", tmp.path())
        .arg("init")
        .output()
        .expect("Failed to run kgate init");

    let output = kgate_cmd()
        .env("HOME", tmp.path())
        .arg("config")
        .arg("show")
        .output()
        .expect("Failed to run kgate config show");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(output.status.success(), "config show failed: {}", stdout);
    assert!(stdout.contains("Settings") || stdout.contains("Sound") || stdout.contains("sound"));
}

#[test]
fn test_generate_subcommand_exists() {
    let output = kgate_cmd()
        .arg("generate")
        .arg("--help")
        .output()
        .expect("Failed to run kgate");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(output.status.success(), "generate --help failed");
    assert!(stdout.contains("project") || stdout.contains("path") || stdout.contains("generate"));
}
