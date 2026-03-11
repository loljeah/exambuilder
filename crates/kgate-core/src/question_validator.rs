use std::collections::HashSet;

use crate::analyzer::GeneratedQuestion;

#[derive(Debug, Clone)]
pub struct ValidationResult {
    pub valid: bool,
    pub score: f32,
    pub issues: Vec<ValidationIssue>,
}

#[derive(Debug, Clone)]
pub struct ValidationIssue {
    pub severity: Severity,
    pub field: String,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Severity {
    Error,
    Warning,
    Info,
}

/// Validate a single generated question
pub fn validate_question(q: &GeneratedQuestion) -> ValidationResult {
    let mut issues = Vec::new();
    let mut score: f32 = 1.0;

    // 1. Format: must have question text
    if q.question_text.is_empty() {
        issues.push(ValidationIssue {
            severity: Severity::Error,
            field: "question".to_string(),
            message: "Question text is empty".to_string(),
        });
        score -= 0.3;
    }

    // 2. Format: exactly 4 options
    if q.options.len() != 4 {
        issues.push(ValidationIssue {
            severity: Severity::Error,
            field: "options".to_string(),
            message: format!("Expected 4 options, got {}", q.options.len()),
        });
        score -= 0.3;
    }

    // 3. Format: answer must be A-D
    if !matches!(q.correct_answer, 'A' | 'B' | 'C' | 'D') {
        issues.push(ValidationIssue {
            severity: Severity::Error,
            field: "answer".to_string(),
            message: format!("Answer must be A-D, got '{}'", q.correct_answer),
        });
        score -= 0.3;
    }

    // 4. Answer option must exist in options
    if q.options.len() == 4 {
        let idx = (q.correct_answer as u8).wrapping_sub(b'A') as usize;
        if idx >= q.options.len() {
            issues.push(ValidationIssue {
                severity: Severity::Error,
                field: "answer".to_string(),
                message: "Answer index out of range".to_string(),
            });
            score -= 0.2;
        }
    }

    // 5. No empty options
    for (i, opt) in q.options.iter().enumerate() {
        let letter = (b'A' + i as u8) as char;
        // Strip "A) " prefix for content check
        let content = opt.trim_start_matches(|c: char| c.is_ascii_uppercase())
            .trim_start_matches(')')
            .trim();
        if content.is_empty() {
            issues.push(ValidationIssue {
                severity: Severity::Error,
                field: "options".to_string(),
                message: format!("Option {} is empty", letter),
            });
            score -= 0.2;
        }
    }

    // 6. No duplicate options
    let option_texts: Vec<String> = q.options.iter()
        .map(|o| o.to_lowercase().trim().to_string())
        .collect();
    let unique: HashSet<&String> = option_texts.iter().collect();
    if unique.len() < option_texts.len() {
        issues.push(ValidationIssue {
            severity: Severity::Error,
            field: "options".to_string(),
            message: "Duplicate options detected".to_string(),
        });
        score -= 0.2;
    }

    // 7. No "all of the above" / "none of the above"
    for opt in &q.options {
        let lower = opt.to_lowercase();
        if lower.contains("all of the above") || lower.contains("none of the above") {
            issues.push(ValidationIssue {
                severity: Severity::Error,
                field: "options".to_string(),
                message: format!("'All/None of the above' not allowed: {}", opt),
            });
            score -= 0.2;
        }
    }

    // 8. Question should end with ?
    if !q.question_text.trim().ends_with('?') {
        issues.push(ValidationIssue {
            severity: Severity::Warning,
            field: "question".to_string(),
            message: "Question does not end with '?'".to_string(),
        });
        score -= 0.05;
    }

    // 9. Length checks
    if q.question_text.len() > 200 {
        issues.push(ValidationIssue {
            severity: Severity::Warning,
            field: "question".to_string(),
            message: format!("Question too long: {} chars (max 200)", q.question_text.len()),
        });
        score -= 0.05;
    }

    for opt in &q.options {
        if opt.len() > 100 {
            issues.push(ValidationIssue {
                severity: Severity::Warning,
                field: "options".to_string(),
                message: format!("Option too long: {} chars (max 100)", opt.len()),
            });
            score -= 0.03;
        }
    }

    // 10. Code snippet length
    if let Some(ref code) = q.code_snippet {
        let line_count = code.lines().count();
        if line_count > 8 {
            issues.push(ValidationIssue {
                severity: Severity::Warning,
                field: "code_snippet".to_string(),
                message: format!("Code snippet too long: {} lines (max 8)", line_count),
            });
            score -= 0.05;
        }
    }

    // 11. Must have hint and explanation
    if q.hint.is_empty() {
        issues.push(ValidationIssue {
            severity: Severity::Warning,
            field: "hint".to_string(),
            message: "Missing hint".to_string(),
        });
        score -= 0.05;
    }

    if q.explanation.is_empty() {
        issues.push(ValidationIssue {
            severity: Severity::Warning,
            field: "explanation".to_string(),
            message: "Missing explanation".to_string(),
        });
        score -= 0.05;
    }

    // 12. Voice compatibility: no "see below", "see above", "diagram"
    let lower_q = q.question_text.to_lowercase();
    for phrase in &["see below", "see above", "see the diagram", "shown below", "shown above"] {
        if lower_q.contains(phrase) {
            issues.push(ValidationIssue {
                severity: Severity::Warning,
                field: "question".to_string(),
                message: format!("Not voice-compatible: contains '{}'", phrase),
            });
            score -= 0.1;
        }
    }

    // 13. Valid tier
    if !matches!(q.tier.as_str(), "RECALL" | "COMPREHENSION" | "APPLICATION" | "ANALYSIS") {
        issues.push(ValidationIssue {
            severity: Severity::Warning,
            field: "tier".to_string(),
            message: format!("Unknown tier: {}", q.tier),
        });
        score -= 0.05;
    }

    // 14. Valid XP
    if !matches!(q.xp, 10 | 15 | 20 | 25) {
        issues.push(ValidationIssue {
            severity: Severity::Info,
            field: "xp".to_string(),
            message: format!("Non-standard XP: {} (expected 10/15/20/25)", q.xp),
        });
    }

    score = score.clamp(0.0, 1.0);
    let has_errors = issues.iter().any(|i| i.severity == Severity::Error);

    ValidationResult {
        valid: !has_errors,
        score,
        issues,
    }
}

/// Validate all questions in a sprint, including cross-question diversity
pub fn validate_sprint(questions: &[GeneratedQuestion]) -> Vec<ValidationResult> {
    let mut results: Vec<ValidationResult> = questions.iter().map(validate_question).collect();

    // Check diversity between questions
    for i in 0..questions.len() {
        for j in (i + 1)..questions.len() {
            let sim = diversity_score(&questions[i], &questions[j]);
            if sim > 0.7 {
                results[j].issues.push(ValidationIssue {
                    severity: Severity::Warning,
                    field: "diversity".to_string(),
                    message: format!(
                        "Too similar to Q{} (similarity: {:.0}%)",
                        i + 1,
                        sim * 100.0
                    ),
                });
                results[j].score = (results[j].score - 0.1).clamp(0.0, 1.0);
            }
        }
    }

    results
}

/// Calculate Jaccard similarity between two questions (0.0 = different, 1.0 = identical)
pub fn diversity_score(a: &GeneratedQuestion, b: &GeneratedQuestion) -> f32 {
    let words_a = word_set(&a.question_text);
    let words_b = word_set(&b.question_text);
    jaccard_similarity(&words_a, &words_b)
}

fn word_set(text: &str) -> HashSet<String> {
    text.to_lowercase()
        .split_whitespace()
        .filter(|w| w.len() > 2) // Skip short words (a, is, the, etc.)
        .map(|w| w.trim_matches(|c: char| !c.is_alphanumeric()).to_string())
        .filter(|w| !w.is_empty())
        .collect()
}

fn jaccard_similarity(a: &HashSet<String>, b: &HashSet<String>) -> f32 {
    if a.is_empty() && b.is_empty() {
        return 0.0;
    }
    let intersection = a.intersection(b).count() as f32;
    let union = a.union(b).count() as f32;
    if union == 0.0 {
        0.0
    } else {
        intersection / union
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn valid_question() -> GeneratedQuestion {
        GeneratedQuestion {
            question_text: "What does the main function do?".to_string(),
            options: vec![
                "A) Starts the program".to_string(),
                "B) Compiles code".to_string(),
                "C) Imports libraries".to_string(),
                "D) Defines modules".to_string(),
            ],
            correct_answer: 'A',
            tier: "RECALL".to_string(),
            difficulty: "Easy".to_string(),
            xp: 10,
            hint: "Every program starts here".to_string(),
            explanation: "Main is the entry point.".to_string(),
            code_snippet: None,
            source_file: "src/main.rs".to_string(),
            source_line: 1,
            domains: vec![],
        }
    }

    #[test]
    fn test_valid_question_passes() {
        let result = validate_question(&valid_question());
        assert!(result.valid);
        assert!(result.score > 0.9);
        assert!(result.issues.iter().all(|i| i.severity != Severity::Error));
    }

    #[test]
    fn test_missing_answer_fails() {
        let mut q = valid_question();
        q.correct_answer = 'Z';
        let result = validate_question(&q);
        assert!(!result.valid);
        assert!(result.issues.iter().any(|i| i.severity == Severity::Error && i.field == "answer"));
    }

    #[test]
    fn test_all_of_above_rejected() {
        let mut q = valid_question();
        q.options[3] = "D) All of the above".to_string();
        let result = validate_question(&q);
        assert!(!result.valid);
    }

    #[test]
    fn test_duplicate_options_rejected() {
        let mut q = valid_question();
        q.options[2] = q.options[1].clone();
        let result = validate_question(&q);
        assert!(!result.valid);
    }

    #[test]
    fn test_long_question_warns() {
        let mut q = valid_question();
        q.question_text = "x".repeat(250) + "?";
        let result = validate_question(&q);
        assert!(result.valid); // warnings don't fail
        assert!(result.issues.iter().any(|i| i.severity == Severity::Warning && i.field == "question"));
    }

    #[test]
    fn test_no_question_mark_warns() {
        let mut q = valid_question();
        q.question_text = "What does main do".to_string();
        let result = validate_question(&q);
        assert!(result.valid);
        assert!(result.issues.iter().any(|i| i.message.contains("?")));
    }

    #[test]
    fn test_two_similar_questions_flagged() {
        let q1 = valid_question();
        let mut q2 = valid_question();
        q2.question_text = "What does the main function do in programs?".to_string();

        let results = validate_sprint(&[q1, q2]);
        assert!(results[1].issues.iter().any(|i| i.field == "diversity"));
    }

    #[test]
    fn test_two_different_questions_ok() {
        let q1 = valid_question();
        let mut q2 = valid_question();
        q2.question_text = "How does error handling work in Rust with Result types?".to_string();

        let sim = diversity_score(&q1, &q2);
        assert!(sim < 0.7, "Similarity {} should be < 0.7", sim);
    }

    #[test]
    fn test_voice_incompatible_flagged() {
        let mut q = valid_question();
        q.question_text = "Looking at the code shown below, what happens?".to_string();
        let result = validate_question(&q);
        assert!(result.issues.iter().any(|i| i.message.contains("voice")));
    }

    #[test]
    fn test_wrong_option_count_fails() {
        let mut q = valid_question();
        q.options = vec!["A) Yes".to_string(), "B) No".to_string()];
        let result = validate_question(&q);
        assert!(!result.valid);
    }

    #[test]
    fn test_code_snippet_too_long_warns() {
        let mut q = valid_question();
        q.code_snippet = Some("line\n".repeat(10));
        let result = validate_question(&q);
        assert!(result.valid);
        assert!(result.issues.iter().any(|i| i.field == "code_snippet"));
    }
}
