use anyhow::Result;
use console::style;

use kgate_core::{parse_exam_file, grade_sprint, Database};

/// Result of a single selftest check
struct CheckResult {
    name: String,
    status: CheckStatus,
    detail: String,
}

enum CheckStatus {
    Pass,
    Warn,
    Fail,
}

impl CheckResult {
    fn pass(name: &str, detail: &str) -> Self {
        Self {
            name: name.to_string(),
            status: CheckStatus::Pass,
            detail: detail.to_string(),
        }
    }

    fn warn(name: &str, detail: &str) -> Self {
        Self {
            name: name.to_string(),
            status: CheckStatus::Warn,
            detail: detail.to_string(),
        }
    }

    fn fail(name: &str, detail: &str) -> Self {
        Self {
            name: name.to_string(),
            status: CheckStatus::Fail,
            detail: detail.to_string(),
        }
    }

    fn print(&self) {
        let tag = match self.status {
            CheckStatus::Pass => style("[PASS]").green(),
            CheckStatus::Warn => style("[WARN]").yellow(),
            CheckStatus::Fail => style("[FAIL]").red(),
        };
        if self.detail.is_empty() {
            println!("{} {}", tag, self.name);
        } else {
            println!("{} {}: {}", tag, self.name, self.detail);
        }
    }
}

/// Run the built-in selftest suite
pub async fn cmd_selftest(verbose: bool, fix: bool) -> Result<()> {
    println!(
        "{} kgate selftest",
        style("🔧").cyan()
    );
    println!("{}", style("━".repeat(40)).dim());

    let mut results: Vec<CheckResult> = Vec::new();

    // 1. Data directory
    results.push(check_data_dir(fix));

    // 2. Database
    let db_result = check_database(fix).await;
    let db_ok = matches!(db_result.status, CheckStatus::Pass);
    results.push(db_result);

    // 3. Schema version (only if DB connected)
    if db_ok {
        results.push(check_schema().await);
    }

    // 4. Profile
    if db_ok {
        results.push(check_profile().await);
    }

    // 5. Parser
    results.push(check_parser());

    // 6. Grader
    results.push(check_grader());

    // 7. Voice tools (optional)
    results.push(check_voice_tts());
    results.push(check_voice_stt());

    // 8. Anthropic API (optional)
    results.push(check_anthropic_api());

    // 9. Exam files (scan current dir)
    results.push(check_exam_files());

    // 10. Config: domains.toml
    results.push(check_domains_config());

    // Print results
    println!();
    for r in &results {
        r.print();
        if verbose {
            if let CheckStatus::Fail | CheckStatus::Warn = r.status {
                if !r.detail.is_empty() {
                    println!("  {}", style(&r.detail).dim());
                }
            }
        }
    }

    // Summary
    let passed = results.iter().filter(|r| matches!(r.status, CheckStatus::Pass)).count();
    let warnings = results.iter().filter(|r| matches!(r.status, CheckStatus::Warn)).count();
    let failures = results.iter().filter(|r| matches!(r.status, CheckStatus::Fail)).count();

    println!("{}", style("━".repeat(40)).dim());
    println!(
        "{}/{} passed, {} warnings, {} failures",
        passed,
        results.len(),
        warnings,
        failures
    );

    if failures > 0 {
        println!(
            "\n{}",
            style("Some checks failed. Run with --fix to attempt auto-repair.").yellow()
        );
    }

    Ok(())
}

fn check_data_dir(fix: bool) -> CheckResult {
    let kgate_dir = crate::kgate_dir();
    let data_dir = crate::data_dir();
    let db_dir = data_dir.join("db");

    if kgate_dir.exists() && db_dir.exists() {
        CheckResult::pass("Data directory", &format!("{}", kgate_dir.display()))
    } else if fix {
        match std::fs::create_dir_all(&db_dir) {
            Ok(_) => {
                let _ = std::fs::create_dir_all(kgate_dir.join("export"));
                let _ = std::fs::create_dir_all(kgate_dir.join("sounds"));
                let _ = std::fs::create_dir_all(kgate_dir.join("bookmarks"));
                CheckResult::pass("Data directory", "created (--fix)")
            }
            Err(e) => CheckResult::fail("Data directory", &format!("could not create: {}", e)),
        }
    } else {
        CheckResult::fail(
            "Data directory",
            &format!("missing: {}. Run kgate init or --fix", kgate_dir.display()),
        )
    }
}

async fn check_database(fix: bool) -> CheckResult {
    let db_path = crate::db_path();

    if !db_path.exists() {
        if fix {
            // Try to create and init
            let parent = db_path.parent().unwrap();
            if let Err(e) = std::fs::create_dir_all(parent) {
                return CheckResult::fail("Database", &format!("cannot create dir: {}", e));
            }
            match Database::new(&db_path).await {
                Ok(db) => match db.init().await {
                    Ok(_) => CheckResult::pass("Database", "created and initialized (--fix)"),
                    Err(e) => CheckResult::fail("Database", &format!("init failed: {}", e)),
                },
                Err(e) => CheckResult::fail("Database", &format!("connect failed: {}", e)),
            }
        } else {
            CheckResult::fail(
                "Database",
                &format!("not found at {}. Run kgate init", db_path.display()),
            )
        }
    } else {
        match Database::new(&db_path).await {
            Ok(_) => CheckResult::pass("Database connection", ""),
            Err(e) => CheckResult::fail("Database connection", &format!("{}", e)),
        }
    }
}

async fn check_schema() -> CheckResult {
    let db_path = crate::db_path();
    match Database::new(&db_path).await {
        Ok(db) => {
            // Verify schema by exercising key queries that touch different tables
            // If any of these fail, the schema is incomplete
            let mut checks_passed = 0;

            // profile table
            if db.get_profile().await.is_ok() {
                checks_passed += 1;
            }
            // projects table
            if db.list_projects().await.is_ok() {
                checks_passed += 1;
            }
            // settings table
            if db.get_setting("sound_enabled").await.is_ok() {
                checks_passed += 1;
            }
            // knowledge_identity table
            if db.get_knowledge_id().await.is_ok() {
                checks_passed += 1;
            }
            // badges table (get_badges)
            if db.get_badges().await.is_ok() {
                checks_passed += 1;
            }
            // review_items table (spaced repetition)
            if db.get_review_stats().await.is_ok() {
                checks_passed += 1;
            }

            if checks_passed == 6 {
                CheckResult::pass("Schema", "6/6 table checks passed (6 migrations)")
            } else {
                CheckResult::fail(
                    "Schema",
                    &format!("{}/6 table checks passed — re-run kgate init", checks_passed),
                )
            }
        }
        Err(e) => CheckResult::fail("Schema", &format!("cannot connect: {}", e)),
    }
}

async fn check_profile() -> CheckResult {
    let db_path = crate::db_path();
    match Database::new(&db_path).await {
        Ok(db) => match db.get_profile().await {
            Ok(profile) => CheckResult::pass(
                "Profile",
                &format!("level {} | {} XP", profile.level, profile.total_xp),
            ),
            Err(_) => CheckResult::warn("Profile", "no profile row (run kgate init)"),
        },
        Err(e) => CheckResult::fail("Profile", &format!("{}", e)),
    }
}

fn check_parser() -> CheckResult {
    let test_exam = r#"# Exam: SelfTest

## Sprint 1: Basics
### Q1. [RECALL] Easy — 10 XP
What is 1+1?
- A) 1
- B) 2
- C) 3
- D) 4

### Q2. [RECALL] Easy — 10 XP
What is 2+2?
- A) 2
- B) 3
- C) 4
- D) 5

### Q3. [COMPREHENSION] Medium — 10 XP
What is 3+3?
- A) 5
- B) 6
- C) 7
- D) 8

---

## Sprint 2: Intermediate
### Q1. [RECALL] Easy — 10 XP
What is 4+4?
- A) 6
- B) 7
- C) 8
- D) 9

### Q2. [APPLICATION] Challenge — 15 XP
What is 5*5?
- A) 20
- B) 25
- C) 30
- D) 35

### Q3. [ANALYSIS] Challenge — 15 XP
What is 10/2?
- A) 2
- B) 3
- C) 5
- D) 10

---

## Sprint 3: Advanced
### Q1. [RECALL] Easy — 10 XP
What is 2^4?
- A) 8
- B) 12
- C) 16
- D) 32

### Q2. [COMPREHENSION] Medium — 10 XP
What is sqrt(9)?
- A) 2
- B) 3
- C) 4
- D) 9

### Q3. [APPLICATION] Challenge — 15 XP
What is 7*8?
- A) 48
- B) 54
- C) 56
- D) 63

---

## 🔑 Answer Key

### Sprint 1
**Q1. Answer: B** — 10 XP
Hint: Basic addition
Full: 1+1 = 2

**Q2. Answer: C** — 10 XP
Hint: Basic addition
Full: 2+2 = 4

**Q3. Answer: B** — 10 XP
Hint: Three plus three
Full: 3+3 = 6

### Sprint 2
**Q1. Answer: C** — 10 XP
Hint: Count by fours
Full: 4+4 = 8

**Q2. Answer: B** — 15 XP
Hint: Five squared
Full: 5*5 = 25

**Q3. Answer: C** — 15 XP
Hint: Division
Full: 10/2 = 5

### Sprint 3
**Q1. Answer: C** — 10 XP
Hint: Powers of two
Full: 2^4 = 16

**Q2. Answer: B** — 10 XP
Hint: Perfect squares
Full: sqrt(9) = 3

**Q3. Answer: C** — 15 XP
Hint: Multiplication table
Full: 7*8 = 56
"#;

    match parse_exam_file(test_exam) {
        Ok(exam) => {
            let sprint_count = exam.sprints.len();
            let q_count: usize = exam.sprints.iter().map(|s| s.questions.len()).sum();
            if sprint_count == 3 && q_count == 9 {
                CheckResult::pass(
                    "Parser",
                    &format!("{} sprints, {} questions", sprint_count, q_count),
                )
            } else {
                CheckResult::fail(
                    "Parser",
                    &format!(
                        "expected 3 sprints/9 questions, got {}/{}",
                        sprint_count, q_count
                    ),
                )
            }
        }
        Err(e) => CheckResult::fail("Parser", &format!("{}", e)),
    }
}

fn check_grader() -> CheckResult {
    // Build a simple sprint to test grading
    let test_sprint = kgate_core::ParsedSprint {
        number: 1,
        topic: "SelfTest".to_string(),
        target_minutes: 3,
        pass_percent: 60,
        total_xp: 30,
        questions: vec![
            kgate_core::ParsedQuestion {
                number: 1,
                tier: "RECALL".to_string(),
                difficulty: "Easy".to_string(),
                xp: 10,
                text: "Test Q1".to_string(),
                code_snippet: None,
                options: vec!["A".into(), "B".into(), "C".into(), "D".into()],
                answer: 'B',
                hint: None,
                explanation: None,
                extra: None,
            },
            kgate_core::ParsedQuestion {
                number: 2,
                tier: "RECALL".to_string(),
                difficulty: "Easy".to_string(),
                xp: 10,
                text: "Test Q2".to_string(),
                code_snippet: None,
                options: vec!["A".into(), "B".into(), "C".into(), "D".into()],
                answer: 'A',
                hint: None,
                explanation: None,
                extra: None,
            },
            kgate_core::ParsedQuestion {
                number: 3,
                tier: "RECALL".to_string(),
                difficulty: "Easy".to_string(),
                xp: 10,
                text: "Test Q3".to_string(),
                code_snippet: None,
                options: vec!["A".into(), "B".into(), "C".into(), "D".into()],
                answer: 'C',
                hint: None,
                explanation: None,
                extra: None,
            },
        ],
    };

    // Test 100% → pass
    let result_pass = grade_sprint(&test_sprint, &['B', 'A', 'C'], 1);
    // Test 33% → fail
    let result_fail = grade_sprint(&test_sprint, &['A', 'B', 'A'], 1);

    if result_pass.passed && result_pass.score_percent == 100 && !result_fail.passed && result_fail.score_percent == 33 {
        CheckResult::pass("Grader", "100% = pass, 33% = fail")
    } else {
        CheckResult::fail(
            "Grader",
            &format!(
                "unexpected: pass={}/{}, fail={}/{}",
                result_pass.passed, result_pass.score_percent, result_fail.passed, result_fail.score_percent
            ),
        )
    }
}

fn check_voice_tts() -> CheckResult {
    let has_espeak = which::which("espeak-ng").is_ok();
    let has_piper = which::which("piper").is_ok();
    let has_kokoro = which::which("kokoro").is_ok();

    let mut engines: Vec<&str> = Vec::new();
    if has_kokoro {
        engines.push("kokoro");
    }
    if has_piper {
        engines.push("piper");
    }
    if has_espeak {
        engines.push("espeak-ng");
    }

    if engines.is_empty() {
        CheckResult::warn("Voice TTS", "no TTS engine found (espeak-ng, piper, kokoro)")
    } else {
        CheckResult::pass("Voice TTS", &engines.join(", "))
    }
}

fn check_voice_stt() -> CheckResult {
    let has_whisper = which::which("whisper-cpp").is_ok() || which::which("whisper").is_ok();

    if has_whisper {
        CheckResult::pass("Voice STT", "whisper found")
    } else {
        CheckResult::warn("Voice STT", "whisper not found")
    }
}

fn check_anthropic_api() -> CheckResult {
    match std::env::var("ANTHROPIC_API_KEY") {
        Ok(key) if !key.is_empty() => {
            CheckResult::pass("Anthropic API", "ANTHROPIC_API_KEY set")
        }
        _ => CheckResult::warn("Anthropic API", "ANTHROPIC_API_KEY not set"),
    }
}

fn check_exam_files() -> CheckResult {
    let cwd = std::env::current_dir().unwrap_or_default();

    match kgate_core::scan_directory(&cwd, 2) {
        Ok(result) => {
            if result.total_exams > 0 {
                // Try parsing each exam file
                let mut parse_errors = 0;
                for proj in &result.projects {
                    if let Some(ref exam_path) = proj.exam_file {
                        if let Ok(content) = std::fs::read_to_string(exam_path) {
                            if parse_exam_file(&content).is_err() {
                                parse_errors += 1;
                            }
                        }
                    }
                }
                if parse_errors > 0 {
                    CheckResult::warn(
                        "Exam files",
                        &format!("{} found, {} parse errors", result.total_exams, parse_errors),
                    )
                } else {
                    CheckResult::pass(
                        "Exam files",
                        &format!("{} found, 0 errors", result.total_exams),
                    )
                }
            } else {
                CheckResult::warn("Exam files", "none found in current directory")
            }
        }
        Err(e) => CheckResult::warn("Exam files", &format!("scan error: {}", e)),
    }
}

fn check_domains_config() -> CheckResult {
    let kgate_dir = crate::kgate_dir();
    let user_domains = kgate_dir.join("domains.toml");

    if user_domains.exists() {
        match std::fs::read_to_string(&user_domains) {
            Ok(content) => match toml::from_str::<toml::Value>(&content) {
                Ok(_) => CheckResult::pass("Config", "domains.toml valid"),
                Err(e) => CheckResult::fail("Config", &format!("domains.toml parse error: {}", e)),
            },
            Err(e) => CheckResult::fail("Config", &format!("cannot read domains.toml: {}", e)),
        }
    } else {
        // Check built-in defaults exist
        let project_domains = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(|p| p.parent())
            .map(|p| p.join("domains.toml"));

        match project_domains {
            Some(p) if p.exists() => CheckResult::pass("Config", "using project domains.toml"),
            _ => CheckResult::warn("Config", "no domains.toml (using built-in defaults)"),
        }
    }
}
