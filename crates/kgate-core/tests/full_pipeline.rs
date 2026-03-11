//! Integration tests for kgate-core: full pipeline from parsing through grading to storage.

use chrono::Utc;
use kgate_core::{
    grade_sprint, get_feedback, parse_exam_file,
    scan_directory, find_exam_file,
    Database, Sprint,
    SpacedRepetitionEngine, ReviewItem, ReviewQuality,
};
use std::fs;

/// Shared test exam markdown used across multiple tests.
const TEST_EXAM_MD: &str = r#"# Exam: TestProject

## Sprint 1: Basics
⏱️ Target: 3 min | 🎯 Pass: 60% | ⚡ 30 XP
🎙️ Voice-compatible: yes

### Q1. [RECALL] Easy — 10 XP

What is 1+1?

- A) 1
- B) 2
- C) 3
- D) 4

### Q2. [COMPREHENSION] Medium — 10 XP

What is the square root of 4?

- A) 1
- B) 2
- C) 3
- D) 4

### Q3. [APPLICATION] Challenge — 10 XP

What is 2^3?

- A) 6
- B) 8
- C) 4
- D) 16

---

## Sprint 2: Advanced
⏱️ Target: 3 min | 🎯 Pass: 60% | ⚡ 30 XP
🎙️ Voice-compatible: yes

### Q1. [RECALL] Easy — 10 XP

What is 5+5?

- A) 5
- B) 10
- C) 15
- D) 20

### Q2. [COMPREHENSION] Medium — 10 XP

What is 3*4?

- A) 7
- B) 10
- C) 12
- D) 15

### Q3. [APPLICATION] Challenge — 10 XP

What is 10/2?

- A) 2
- B) 5
- C) 3
- D) 10

---

## 🔑 Answer Key

### Sprint 1
**Q1. Answer: B** — 10 XP
Hint: Simple addition
Full: 1+1 equals 2

**Q2. Answer: B** — 10 XP
Hint: Think of perfect squares
Full: sqrt(4) = 2

**Q3. Answer: B** — 10 XP
Hint: Two cubed
Full: 2^3 = 2*2*2 = 8

### Sprint 2
**Q1. Answer: B** — 10 XP
Hint: Count on fingers
Full: 5+5 = 10

**Q2. Answer: C** — 10 XP
Hint: Multiplication table
Full: 3*4 = 12

**Q3. Answer: B** — 10 XP
Hint: Division
Full: 10/2 = 5
"#;

// ---------------------------------------------------------------------------
// 1. Parse exam -> grade sprint 1 with correct answers -> verify pass + full XP
// ---------------------------------------------------------------------------
#[test]
fn test_parse_grade_pipeline() {
    let exam = parse_exam_file(TEST_EXAM_MD).expect("parse should succeed");

    let sprint1 = &exam.sprints[0];
    // Correct answers for sprint 1: B, B, B
    let answers = ['B', 'B', 'B'];
    let result = grade_sprint(sprint1, &answers, 1);

    assert!(result.passed, "Sprint should pass with all correct answers");
    assert_eq!(result.score_percent, 100);
    assert_eq!(result.correct_count, 3);
    assert_eq!(result.xp_earned, 30, "Full XP should be awarded on pass");
    assert_eq!(result.xp_possible, 30);

    // Verify feedback reports a pass
    let feedback = get_feedback(&result, &sprint1.questions);
    assert!(
        feedback.message.contains("PASSED"),
        "Feedback message should contain PASSED"
    );
    assert!(!feedback.show_hints);
    assert!(!feedback.show_answers);
}

// ---------------------------------------------------------------------------
// 2. Parse exam -> grade with wrong answers -> verify failure
// ---------------------------------------------------------------------------
#[test]
fn test_parse_grade_fail() {
    let exam = parse_exam_file(TEST_EXAM_MD).expect("parse should succeed");

    let sprint1 = &exam.sprints[0];
    // All wrong answers
    let answers = ['A', 'C', 'D'];
    let result = grade_sprint(sprint1, &answers, 1);

    assert!(!result.passed, "Sprint should fail with all wrong answers");
    assert_eq!(result.score_percent, 0);
    assert_eq!(result.correct_count, 0);
    assert_eq!(result.xp_earned, 0, "No XP when failing");

    // Feedback for attempt 1 should show hints, not full answers
    let feedback = get_feedback(&result, &sprint1.questions);
    assert!(feedback.show_hints, "First failed attempt should show hints");
    assert!(!feedback.show_answers);
}

// ---------------------------------------------------------------------------
// 3. Parse roundtrip: verify metadata extracted correctly
// ---------------------------------------------------------------------------
#[test]
fn test_parse_roundtrip() {
    let exam = parse_exam_file(TEST_EXAM_MD).expect("parse should succeed");

    // Project name
    assert_eq!(exam.project_name, "TestProject");

    // Sprint count
    assert_eq!(exam.sprints.len(), 2, "Should parse 2 sprints");

    // Sprint 1 metadata
    let s1 = &exam.sprints[0];
    assert_eq!(s1.number, 1);
    assert_eq!(s1.topic, "Basics");
    assert_eq!(s1.questions.len(), 3, "Sprint 1 should have 3 questions");
    assert_eq!(s1.total_xp, 30);

    // Sprint 2 metadata
    let s2 = &exam.sprints[1];
    assert_eq!(s2.number, 2);
    assert_eq!(s2.topic, "Advanced");
    assert_eq!(s2.questions.len(), 3, "Sprint 2 should have 3 questions");
    assert_eq!(s2.total_xp, 30);

    // Each question has exactly 4 options
    for sprint in &exam.sprints {
        for q in &sprint.questions {
            assert_eq!(
                q.options.len(),
                4,
                "Q{} in Sprint {} should have 4 options, got {}",
                q.number,
                sprint.number,
                q.options.len()
            );
        }
    }

    // Answer key was applied correctly
    assert_eq!(s1.questions[0].answer, 'B');
    assert_eq!(s1.questions[1].answer, 'B');
    assert_eq!(s1.questions[2].answer, 'B');
    assert_eq!(s2.questions[0].answer, 'B');
    assert_eq!(s2.questions[1].answer, 'C');
    assert_eq!(s2.questions[2].answer, 'B');

    // Hints and explanations were applied
    assert_eq!(
        s1.questions[0].hint.as_deref(),
        Some("Simple addition")
    );
    assert_eq!(
        s1.questions[0].explanation.as_deref(),
        Some("1+1 equals 2")
    );
}

// ---------------------------------------------------------------------------
// 4. DB sprint roundtrip: create project -> upsert sprints -> retrieve
// ---------------------------------------------------------------------------
#[tokio::test]
async fn test_db_sprint_roundtrip() {
    let dir = tempfile::tempdir().unwrap();
    let db_path = dir.path().join("test_pipeline.db");
    let db = Database::new(&db_path).await.unwrap();
    db.init().await.unwrap();

    // Create a project
    let project = db
        .get_or_create_project("/tmp/test-pipeline", "test-pipeline")
        .await
        .unwrap();

    // Parse exam and upsert sprints
    let exam = parse_exam_file(TEST_EXAM_MD).unwrap();

    for parsed_sprint in &exam.sprints {
        let questions_json = serde_json::to_string(&parsed_sprint.questions).unwrap();
        let sprint = Sprint {
            id: 0,
            project_id: project.id.clone(),
            sprint_number: parsed_sprint.number,
            topic: parsed_sprint.topic.clone(),
            questions_json,
            answer_key_json: "{}".to_string(),
            status: "pending".to_string(),
            best_score: None,
            attempts: 0,
            xp_available: parsed_sprint.total_xp,
            xp_earned: 0,
            created_at: Utc::now(),
            passed_at: None,
            sprint_id: None,
            source_project_name: Some(exam.project_name.clone()),
        };
        db.upsert_sprint(&sprint).await.unwrap();
    }

    // Retrieve and verify
    let stored_sprints = db.get_sprints(&project.id).await.unwrap();
    assert_eq!(stored_sprints.len(), 2, "Should have 2 stored sprints");

    assert_eq!(stored_sprints[0].sprint_number, 1);
    assert_eq!(stored_sprints[0].topic, "Basics");
    assert_eq!(stored_sprints[0].xp_available, 30);
    assert_eq!(stored_sprints[0].status, "pending");
    assert_eq!(
        stored_sprints[0].source_project_name.as_deref(),
        Some("TestProject")
    );

    assert_eq!(stored_sprints[1].sprint_number, 2);
    assert_eq!(stored_sprints[1].topic, "Advanced");
    assert_eq!(stored_sprints[1].xp_available, 30);
}

// ---------------------------------------------------------------------------
// 5. Full pipeline: parse -> grade -> record attempt in DB -> verify
// ---------------------------------------------------------------------------
#[tokio::test]
async fn test_full_pipeline_parse_grade_store() {
    let dir = tempfile::tempdir().unwrap();
    let db_path = dir.path().join("test_full.db");
    let db = Database::new(&db_path).await.unwrap();
    db.init().await.unwrap();

    let project = db
        .get_or_create_project("/tmp/full-pipeline", "full-pipeline")
        .await
        .unwrap();

    // Parse
    let exam = parse_exam_file(TEST_EXAM_MD).unwrap();

    // Upsert sprint 1 into DB so record_sprint_attempt can update it
    let parsed_s1 = &exam.sprints[0];
    let questions_json = serde_json::to_string(&parsed_s1.questions).unwrap();
    let db_sprint = Sprint {
        id: 0,
        project_id: project.id.clone(),
        sprint_number: parsed_s1.number,
        topic: parsed_s1.topic.clone(),
        questions_json,
        answer_key_json: "{}".to_string(),
        status: "pending".to_string(),
        best_score: None,
        attempts: 0,
        xp_available: parsed_s1.total_xp,
        xp_earned: 0,
        created_at: Utc::now(),
        passed_at: None,
        sprint_id: None,
        source_project_name: None,
    };
    db.upsert_sprint(&db_sprint).await.unwrap();

    // Grade sprint 1 with perfect answers
    let result = grade_sprint(parsed_s1, &['B', 'B', 'B'], 1);
    assert!(result.passed);

    // Record attempt
    db.record_sprint_attempt(
        &project.id,
        result.sprint_number,
        result.score_percent,
        result.passed,
        result.xp_earned,
    )
    .await
    .unwrap();

    // Verify attempt in history
    let history = db.get_project_history(&project.id, 10).await.unwrap();
    assert_eq!(history.len(), 1, "Should have 1 attempt recorded");
    assert_eq!(history[0].score_percent, 100);
    assert!(history[0].passed);
    assert_eq!(history[0].xp_earned, 30);
    assert_eq!(history[0].sprint_number, 1);

    // Verify sprint status updated
    let sprint_row = db.get_sprint(&project.id, 1).await.unwrap().unwrap();
    assert_eq!(sprint_row.status, "passed");
    assert_eq!(sprint_row.best_score, Some(100));
    assert_eq!(sprint_row.attempts, 1);
}

// ---------------------------------------------------------------------------
// 6. Scanner finds exam file written to a temp directory
// ---------------------------------------------------------------------------
#[test]
fn test_scanner_finds_exam_in_tempdir() {
    let tmp = tempfile::tempdir().unwrap();
    let exam_path = tmp.path().join("exam_test.md");
    fs::write(&exam_path, TEST_EXAM_MD).unwrap();

    // scan_directory should discover it
    let scan_result = scan_directory(tmp.path(), 0).unwrap();
    assert_eq!(
        scan_result.total_exams, 1,
        "Scanner should find 1 exam file"
    );
    assert!(
        scan_result.projects[0].exam_file.is_some(),
        "Project should have an exam_file"
    );

    // find_exam_file should locate it directly
    let found = find_exam_file(tmp.path());
    assert!(found.is_some(), "find_exam_file should return Some");
    let found_path = found.unwrap();
    assert!(
        found_path
            .file_name()
            .unwrap()
            .to_string_lossy()
            .starts_with("exam_"),
        "Found file should start with exam_"
    );
}

// ---------------------------------------------------------------------------
// 7. Spaced repetition: calculate next review for correct vs incorrect
// ---------------------------------------------------------------------------
#[test]
fn test_spaced_repetition_calculation() {
    let item = ReviewItem {
        id: 1,
        project_id: "test".to_string(),
        sprint_number: 1,
        question_number: 1,
        question_text: "What is 1+1?".to_string(),
        correct_answer: "B".to_string(),
        domain: "math".to_string(),
        easiness_factor: 2.5,
        repetition_count: 0,
        interval_days: 1,
        next_review: Utc::now(),
        last_reviewed: None,
        times_correct: 0,
        times_wrong: 0,
        streak: 0,
    };

    // Perfect answer: should be marked correct with interval >= 1
    let correct_update =
        SpacedRepetitionEngine::calculate_next_review(&item, ReviewQuality::Perfect);
    assert!(
        correct_update.was_correct,
        "Perfect quality should count as correct"
    );
    assert!(
        correct_update.interval_days >= 1,
        "Interval after correct should be >= 1 day"
    );
    assert_eq!(
        correct_update.repetition_count, 1,
        "Repetition count should increase to 1"
    );
    assert!(
        correct_update.easiness_factor >= 2.5,
        "EF should not decrease for perfect answer"
    );
    assert!(
        correct_update.next_review > Utc::now(),
        "Next review should be in the future"
    );

    // Failed answer: should reset repetition count and set interval to 1
    let wrong_update = SpacedRepetitionEngine::calculate_next_review(
        &item,
        ReviewQuality::CompleteBlackout,
    );
    assert!(
        !wrong_update.was_correct,
        "Blackout should count as incorrect"
    );
    assert_eq!(
        wrong_update.interval_days, 1,
        "Failed review should reset interval to 1"
    );
    assert_eq!(
        wrong_update.repetition_count, 0,
        "Failed review should reset repetition count to 0"
    );
    assert!(
        wrong_update.easiness_factor >= 1.3,
        "EF should never drop below 1.3"
    );

    // Correct interval should be >= wrong interval
    assert!(
        correct_update.interval_days >= wrong_update.interval_days,
        "Correct answer interval ({}) should be >= wrong answer interval ({})",
        correct_update.interval_days,
        wrong_update.interval_days
    );
}
