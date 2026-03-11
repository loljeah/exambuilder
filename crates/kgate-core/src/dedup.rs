use std::collections::HashSet;

use crate::analyzer::GeneratedQuestion;

/// Remove near-duplicate questions within a generation run.
/// Questions with Jaccard similarity > threshold are removed (keeping the first).
pub fn deduplicate_questions(questions: &mut Vec<GeneratedQuestion>, threshold: f32) {
    let mut keep = vec![true; questions.len()];

    for i in 0..questions.len() {
        if !keep[i] {
            continue;
        }
        for j in (i + 1)..questions.len() {
            if !keep[j] {
                continue;
            }
            let sim = text_similarity(&questions[i].question_text, &questions[j].question_text);
            if sim > threshold {
                keep[j] = false;
            }
        }
    }

    let mut idx = 0;
    questions.retain(|_| {
        let k = keep[idx];
        idx += 1;
        k
    });
}

/// Check new questions against existing collected questions.
/// Returns a Vec<bool> where true = too similar to an existing question.
pub fn check_against_existing(
    new: &[GeneratedQuestion],
    existing_texts: &[String],
    threshold: f32,
) -> Vec<bool> {
    new.iter()
        .map(|q| {
            existing_texts
                .iter()
                .any(|e| text_similarity(&q.question_text, e) > threshold)
        })
        .collect()
}

/// Tokenize and normalize text into a word set
pub fn word_set(text: &str) -> HashSet<String> {
    text.to_lowercase()
        .split_whitespace()
        .filter(|w| w.len() > 2)
        .map(|w| w.trim_matches(|c: char| !c.is_alphanumeric()).to_string())
        .filter(|w| !w.is_empty())
        .collect()
}

/// Jaccard similarity between two word sets (0.0 = different, 1.0 = identical)
pub fn jaccard_similarity(a: &HashSet<String>, b: &HashSet<String>) -> f32 {
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

/// Compute text similarity between two strings using Jaccard on word sets
pub fn text_similarity(a: &str, b: &str) -> f32 {
    jaccard_similarity(&word_set(a), &word_set(b))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_question(text: &str) -> GeneratedQuestion {
        GeneratedQuestion {
            question_text: text.to_string(),
            options: vec![
                "A) Option 1".to_string(),
                "B) Option 2".to_string(),
                "C) Option 3".to_string(),
                "D) Option 4".to_string(),
            ],
            correct_answer: 'A',
            tier: "RECALL".to_string(),
            difficulty: "Easy".to_string(),
            xp: 10,
            hint: "hint".to_string(),
            explanation: "explanation".to_string(),
            code_snippet: None,
            source_file: "test.rs".to_string(),
            source_line: 1,
            domains: vec![],
        }
    }

    #[test]
    fn test_identical_questions_deduplicated() {
        let mut qs = vec![
            make_question("What does the main function do in Rust?"),
            make_question("What does the main function do in Rust?"),
        ];
        deduplicate_questions(&mut qs, 0.7);
        assert_eq!(qs.len(), 1);
    }

    #[test]
    fn test_slightly_different_wording_deduplicated() {
        let mut qs = vec![
            make_question("What does the main function do in Rust programs?"),
            make_question("What does the main function do in Rust applications?"),
        ];
        deduplicate_questions(&mut qs, 0.7);
        assert_eq!(qs.len(), 1, "Near-duplicates should be removed");
    }

    #[test]
    fn test_completely_different_questions_kept() {
        let mut qs = vec![
            make_question("What does the main function do in Rust?"),
            make_question("How does error handling work with Result types in async code?"),
        ];
        deduplicate_questions(&mut qs, 0.7);
        assert_eq!(qs.len(), 2, "Different questions should both be kept");
    }

    #[test]
    fn test_empty_input_no_crash() {
        let mut qs: Vec<GeneratedQuestion> = Vec::new();
        deduplicate_questions(&mut qs, 0.7);
        assert!(qs.is_empty());
    }

    #[test]
    fn test_single_question_kept() {
        let mut qs = vec![make_question("What is a struct?")];
        deduplicate_questions(&mut qs, 0.7);
        assert_eq!(qs.len(), 1);
    }

    #[test]
    fn test_check_against_existing_flags_similar() {
        let new = vec![
            make_question("What does the main function do in Rust?"),
            make_question("How does error handling work?"),
        ];
        let existing = vec![
            "What does the main function do in Rust programs?".to_string(),
        ];
        let flags = check_against_existing(&new, &existing, 0.7);
        assert!(flags[0], "First question should be flagged as similar");
        assert!(!flags[1], "Second question should not be flagged");
    }

    #[test]
    fn test_check_against_empty_existing() {
        let new = vec![make_question("What is a struct?")];
        let existing: Vec<String> = Vec::new();
        let flags = check_against_existing(&new, &existing, 0.7);
        assert!(!flags[0]);
    }

    #[test]
    fn test_jaccard_identical_sets() {
        let a = word_set("hello world test");
        let b = word_set("hello world test");
        assert!((jaccard_similarity(&a, &b) - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_jaccard_disjoint_sets() {
        let a = word_set("alpha beta gamma");
        let b = word_set("delta epsilon zeta");
        assert!((jaccard_similarity(&a, &b)).abs() < f32::EPSILON);
    }

    #[test]
    fn test_jaccard_empty_sets() {
        let a: HashSet<String> = HashSet::new();
        let b: HashSet<String> = HashSet::new();
        assert!((jaccard_similarity(&a, &b)).abs() < f32::EPSILON);
    }

    #[test]
    fn test_word_set_filters_short_words() {
        let ws = word_set("a is an big function");
        assert!(!ws.contains("a"));
        assert!(!ws.contains("is"));
        assert!(!ws.contains("an"));
        assert!(ws.contains("big"));
        assert!(ws.contains("function"));
    }

    #[test]
    fn test_word_set_lowercases() {
        let ws = word_set("Hello WORLD Test");
        assert!(ws.contains("hello"));
        assert!(ws.contains("world"));
        assert!(ws.contains("test"));
    }

    #[test]
    fn test_text_similarity_symmetric() {
        let a = "What does the main function do?";
        let b = "How does error handling work?";
        assert!((text_similarity(a, b) - text_similarity(b, a)).abs() < f32::EPSILON);
    }
}
