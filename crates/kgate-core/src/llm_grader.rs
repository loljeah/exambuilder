//! LLM Grading for Open-Ended Answers (Phase 6)
//!
//! Provides AI-powered grading of free-form answers against answer keys.
//! Supports multiple grading strategies:
//! - Local keyword/semantic matching (default, no API needed)
//! - External LLM API integration (optional)
//!
//! Scoring rubric:
//! - 3 points: Complete answer with all key points
//! - 2 points: Partial answer with main concept
//! - 1 point: Surface-level awareness
//! - 0 points: Incorrect or blank

use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// Grading result for an open-ended answer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GradingResult {
    pub score: i32,           // 0-3 scale
    pub max_score: i32,       // Always 3
    pub percentage: f64,      // 0-100
    pub feedback: String,     // Explanation of score
    pub matched_concepts: Vec<String>,
    pub missing_concepts: Vec<String>,
    pub confidence: f64,      // 0-1, how confident the grader is
}

impl GradingResult {
    pub fn xp_multiplier(&self) -> f64 {
        match self.score {
            3 => 1.0,
            2 => 0.66,
            1 => 0.33,
            _ => 0.0,
        }
    }

    pub fn passed(&self) -> bool {
        self.score >= 2 // Partial credit counts as passing
    }
}

/// Answer key with expected concepts/keywords
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnswerKey {
    pub key_concepts: Vec<String>,      // Must-have concepts for full credit
    pub bonus_concepts: Vec<String>,    // Nice-to-have concepts
    pub incorrect_concepts: Vec<String>, // Concepts that indicate misunderstanding
    pub example_answer: Option<String>,  // Model answer for reference
}

impl AnswerKey {
    pub fn new(key_concepts: Vec<&str>) -> Self {
        Self {
            key_concepts: key_concepts.into_iter().map(String::from).collect(),
            bonus_concepts: Vec::new(),
            incorrect_concepts: Vec::new(),
            example_answer: None,
        }
    }

    pub fn with_example(mut self, example: &str) -> Self {
        self.example_answer = Some(example.to_string());
        self
    }

    pub fn with_incorrect(mut self, incorrect: Vec<&str>) -> Self {
        self.incorrect_concepts = incorrect.into_iter().map(String::from).collect();
        self
    }
}

/// Local grading engine (no external API required)
pub struct LocalGrader {
    /// Minimum similarity threshold for fuzzy matching
    similarity_threshold: f64,
    /// Whether to use stemming for word matching
    use_stemming: bool,
}

impl Default for LocalGrader {
    fn default() -> Self {
        Self {
            similarity_threshold: 0.7,
            use_stemming: true,
        }
    }
}

impl LocalGrader {
    pub fn new() -> Self {
        Self::default()
    }

    /// Grade an open-ended answer against the answer key
    pub fn grade(&self, user_answer: &str, answer_key: &AnswerKey) -> GradingResult {
        let user_words = self.extract_words(user_answer);
        let user_lower = user_answer.to_lowercase();

        // Check for incorrect concepts first
        for incorrect in &answer_key.incorrect_concepts {
            if self.contains_concept(&user_lower, &user_words, incorrect) {
                return GradingResult {
                    score: 0,
                    max_score: 3,
                    percentage: 0.0,
                    feedback: format!(
                        "Incorrect understanding detected. '{}' is not correct in this context.",
                        incorrect
                    ),
                    matched_concepts: Vec::new(),
                    missing_concepts: answer_key.key_concepts.clone(),
                    confidence: 0.9,
                };
            }
        }

        // Count matched key concepts
        let mut matched: Vec<String> = Vec::new();
        let mut missing: Vec<String> = Vec::new();

        for concept in &answer_key.key_concepts {
            if self.contains_concept(&user_lower, &user_words, concept) {
                matched.push(concept.clone());
            } else {
                missing.push(concept.clone());
            }
        }

        // Check bonus concepts
        let mut bonus_matched = 0;
        for concept in &answer_key.bonus_concepts {
            if self.contains_concept(&user_lower, &user_words, concept) {
                bonus_matched += 1;
            }
        }

        // Calculate score
        let key_count = answer_key.key_concepts.len();
        let matched_count = matched.len();

        let (score, feedback) = if key_count == 0 {
            // No key concepts defined, use length heuristic
            let word_count = user_words.len();
            if word_count >= 20 {
                (3, "Detailed response provided.".to_string())
            } else if word_count >= 10 {
                (2, "Adequate response.".to_string())
            } else if word_count >= 3 {
                (1, "Brief response, consider elaborating.".to_string())
            } else {
                (0, "Response too brief or empty.".to_string())
            }
        } else {
            let ratio = matched_count as f64 / key_count as f64;

            if ratio >= 0.9 || (matched_count == key_count) {
                let bonus_note = if bonus_matched > 0 {
                    format!(" Plus {} bonus concepts!", bonus_matched)
                } else {
                    String::new()
                };
                (3, format!("Excellent! All key concepts covered.{}", bonus_note))
            } else if ratio >= 0.6 {
                (
                    2,
                    format!(
                        "Good understanding. Covered {}/{} key concepts. Missing: {}",
                        matched_count,
                        key_count,
                        missing.join(", ")
                    ),
                )
            } else if ratio > 0.0 || !user_words.is_empty() {
                (
                    1,
                    format!(
                        "Partial understanding. Only {}/{} concepts. Review: {}",
                        matched_count,
                        key_count,
                        missing.join(", ")
                    ),
                )
            } else {
                (0, "No relevant concepts found in answer.".to_string())
            }
        };

        let percentage = (score as f64 / 3.0) * 100.0;
        let confidence = if key_count > 0 {
            0.7 + (matched_count as f64 / key_count as f64) * 0.3
        } else {
            0.5
        };

        GradingResult {
            score,
            max_score: 3,
            percentage,
            feedback,
            matched_concepts: matched,
            missing_concepts: missing,
            confidence,
        }
    }

    /// Extract words from text (lowercased, alphanumeric only)
    fn extract_words(&self, text: &str) -> HashSet<String> {
        text.to_lowercase()
            .split(|c: char| !c.is_alphanumeric())
            .filter(|w| w.len() > 2)
            .map(|w| {
                if self.use_stemming {
                    self.stem_word(w)
                } else {
                    w.to_string()
                }
            })
            .collect()
    }

    /// Simple stemming (remove common suffixes)
    fn stem_word(&self, word: &str) -> String {
        let mut w = word.to_string();

        // Common English suffixes
        let suffixes = ["ing", "ed", "ly", "tion", "ness", "ment", "able", "ible", "es", "s"];

        for suffix in &suffixes {
            if w.len() > suffix.len() + 2 && w.ends_with(suffix) {
                w = w[..w.len() - suffix.len()].to_string();
                break;
            }
        }

        w
    }

    /// Check if answer contains a concept (supports multi-word concepts)
    fn contains_concept(&self, text_lower: &str, words: &HashSet<String>, concept: &str) -> bool {
        let concept_lower = concept.to_lowercase();

        // Direct substring match for phrases
        if concept.contains(' ') {
            return text_lower.contains(&concept_lower);
        }

        // Word-level match with stemming
        let concept_stem = if self.use_stemming {
            self.stem_word(&concept_lower)
        } else {
            concept_lower.clone()
        };

        // Exact word match
        if words.contains(&concept_stem) {
            return true;
        }

        // Fuzzy match for typos
        for word in words {
            if self.similarity(word, &concept_stem) >= self.similarity_threshold {
                return true;
            }
        }

        false
    }

    /// Calculate similarity between two strings (Levenshtein-based)
    fn similarity(&self, a: &str, b: &str) -> f64 {
        if a == b {
            return 1.0;
        }

        let len_a = a.chars().count();
        let len_b = b.chars().count();

        if len_a == 0 || len_b == 0 {
            return 0.0;
        }

        let distance = self.levenshtein(a, b);
        let max_len = len_a.max(len_b);

        1.0 - (distance as f64 / max_len as f64)
    }

    /// Levenshtein distance
    fn levenshtein(&self, a: &str, b: &str) -> usize {
        let a_chars: Vec<char> = a.chars().collect();
        let b_chars: Vec<char> = b.chars().collect();

        let len_a = a_chars.len();
        let len_b = b_chars.len();

        let mut matrix = vec![vec![0usize; len_b + 1]; len_a + 1];

        for i in 0..=len_a {
            matrix[i][0] = i;
        }
        for j in 0..=len_b {
            matrix[0][j] = j;
        }

        for i in 1..=len_a {
            for j in 1..=len_b {
                let cost = if a_chars[i - 1] == b_chars[j - 1] { 0 } else { 1 };

                matrix[i][j] = (matrix[i - 1][j] + 1)
                    .min(matrix[i][j - 1] + 1)
                    .min(matrix[i - 1][j - 1] + cost);
            }
        }

        matrix[len_a][len_b]
    }
}

/// Question type for grading
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum QuestionType {
    MultipleChoice,
    OpenEnded,
    TrueFalse,
    FillInBlank,
}

/// Combined grader that handles all question types
pub struct UnifiedGrader {
    local_grader: LocalGrader,
}

impl Default for UnifiedGrader {
    fn default() -> Self {
        Self::new()
    }
}

impl UnifiedGrader {
    pub fn new() -> Self {
        Self {
            local_grader: LocalGrader::new(),
        }
    }

    /// Grade any question type
    pub fn grade(
        &self,
        question_type: QuestionType,
        user_answer: &str,
        correct_answer: &str,
        answer_key: Option<&AnswerKey>,
    ) -> GradingResult {
        match question_type {
            QuestionType::MultipleChoice | QuestionType::TrueFalse => {
                self.grade_exact(user_answer, correct_answer)
            }
            QuestionType::OpenEnded => {
                if let Some(key) = answer_key {
                    self.local_grader.grade(user_answer, key)
                } else {
                    // Create basic answer key from correct answer
                    let basic_key = self.create_basic_key(correct_answer);
                    self.local_grader.grade(user_answer, &basic_key)
                }
            }
            QuestionType::FillInBlank => self.grade_fill_blank(user_answer, correct_answer),
        }
    }

    /// Exact match grading (MC, T/F)
    fn grade_exact(&self, user_answer: &str, correct_answer: &str) -> GradingResult {
        let user = user_answer.trim().to_uppercase();
        let correct = correct_answer.trim().to_uppercase();

        let is_correct = user == correct
            || (user.len() == 1 && correct.starts_with(&user))
            || (correct.len() == 1 && user.starts_with(&correct));

        if is_correct {
            GradingResult {
                score: 3,
                max_score: 3,
                percentage: 100.0,
                feedback: "Correct!".to_string(),
                matched_concepts: vec![correct_answer.to_string()],
                missing_concepts: Vec::new(),
                confidence: 1.0,
            }
        } else {
            GradingResult {
                score: 0,
                max_score: 3,
                percentage: 0.0,
                feedback: format!("Incorrect. The correct answer is {}.", correct_answer),
                matched_concepts: Vec::new(),
                missing_concepts: vec![correct_answer.to_string()],
                confidence: 1.0,
            }
        }
    }

    /// Fill-in-the-blank grading (flexible matching)
    fn grade_fill_blank(&self, user_answer: &str, correct_answer: &str) -> GradingResult {
        let user_clean = user_answer.trim().to_lowercase();
        let correct_clean = correct_answer.trim().to_lowercase();

        // Exact match
        if user_clean == correct_clean {
            return GradingResult {
                score: 3,
                max_score: 3,
                percentage: 100.0,
                feedback: "Correct!".to_string(),
                matched_concepts: vec![correct_answer.to_string()],
                missing_concepts: Vec::new(),
                confidence: 1.0,
            };
        }

        // Check if answer is contained or similar
        let similarity = self.local_grader.similarity(&user_clean, &correct_clean);

        if similarity >= 0.9 {
            GradingResult {
                score: 3,
                max_score: 3,
                percentage: 100.0,
                feedback: "Correct! (minor typo accepted)".to_string(),
                matched_concepts: vec![correct_answer.to_string()],
                missing_concepts: Vec::new(),
                confidence: 0.95,
            }
        } else if similarity >= 0.7 {
            GradingResult {
                score: 2,
                max_score: 3,
                percentage: 66.0,
                feedback: format!("Close! Expected: {}", correct_answer),
                matched_concepts: Vec::new(),
                missing_concepts: vec![correct_answer.to_string()],
                confidence: 0.8,
            }
        } else {
            GradingResult {
                score: 0,
                max_score: 3,
                percentage: 0.0,
                feedback: format!("Incorrect. Expected: {}", correct_answer),
                matched_concepts: Vec::new(),
                missing_concepts: vec![correct_answer.to_string()],
                confidence: 1.0,
            }
        }
    }

    /// Create a basic answer key from a model answer
    fn create_basic_key(&self, correct_answer: &str) -> AnswerKey {
        // Extract significant words as key concepts
        let words: Vec<&str> = correct_answer
            .split_whitespace()
            .filter(|w| w.len() > 3)
            .filter(|w| !STOP_WORDS.contains(&w.to_lowercase().as_str()))
            .take(5)
            .collect();

        AnswerKey {
            key_concepts: words.into_iter().map(String::from).collect(),
            bonus_concepts: Vec::new(),
            incorrect_concepts: Vec::new(),
            example_answer: Some(correct_answer.to_string()),
        }
    }
}

/// Common stop words to ignore when extracting concepts
const STOP_WORDS: &[&str] = &[
    "the", "a", "an", "and", "or", "but", "in", "on", "at", "to", "for", "of", "with", "by",
    "from", "as", "is", "was", "are", "were", "been", "be", "have", "has", "had", "do", "does",
    "did", "will", "would", "could", "should", "may", "might", "must", "shall", "can", "this",
    "that", "these", "those", "it", "its", "they", "them", "their", "what", "which", "who",
    "whom", "when", "where", "why", "how", "all", "each", "every", "both", "few", "more", "most",
    "other", "some", "such", "no", "nor", "not", "only", "own", "same", "so", "than", "too",
    "very", "just", "also", "now", "here", "there", "then", "once", "again", "always", "never",
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exact_grading_correct() {
        let grader = UnifiedGrader::new();
        let result = grader.grade(QuestionType::MultipleChoice, "B", "B", None);

        assert_eq!(result.score, 3);
        assert!(result.passed());
    }

    #[test]
    fn test_exact_grading_incorrect() {
        let grader = UnifiedGrader::new();
        let result = grader.grade(QuestionType::MultipleChoice, "A", "B", None);

        assert_eq!(result.score, 0);
        assert!(!result.passed());
    }

    #[test]
    fn test_open_ended_full_credit() {
        let grader = LocalGrader::new();
        let key = AnswerKey::new(vec!["async", "Future", "await"]);

        let result = grader.grade(
            "Async functions return a Future that must be awaited to execute",
            &key,
        );

        assert_eq!(result.score, 3);
        assert!(result.matched_concepts.len() >= 2);
    }

    #[test]
    fn test_open_ended_partial_credit() {
        let grader = LocalGrader::new();
        let key = AnswerKey::new(vec!["async", "Future", "await", "concurrent"]);

        let result = grader.grade("Async functions are concurrent", &key);

        assert!(result.score >= 1);
        assert!(!result.missing_concepts.is_empty());
    }

    #[test]
    fn test_open_ended_incorrect_concept() {
        let grader = LocalGrader::new();
        let key = AnswerKey::new(vec!["async", "Future"])
            .with_incorrect(vec!["thread", "blocking"]);

        let result = grader.grade("Async functions create new threads", &key);

        assert_eq!(result.score, 0);
        assert!(result.feedback.contains("Incorrect"));
    }

    #[test]
    fn test_fill_blank_typo_tolerance() {
        let grader = UnifiedGrader::new();

        // Exact
        let result1 = grader.grade(QuestionType::FillInBlank, "ownership", "ownership", None);
        assert_eq!(result1.score, 3);

        // Minor typo
        let result2 = grader.grade(QuestionType::FillInBlank, "ownrship", "ownership", None);
        assert!(result2.score >= 2);
    }

    #[test]
    fn test_stemming() {
        let grader = LocalGrader::new();
        let key = AnswerKey::new(vec!["execute", "function"]);

        // "executing" should match "execute" due to stemming
        let result = grader.grade("When executing the function", &key);

        assert!(result.matched_concepts.len() >= 1);
    }

    #[test]
    fn test_similarity_calculation() {
        let grader = LocalGrader::new();

        assert!(grader.similarity("rust", "rust") == 1.0);
        assert!(grader.similarity("rust", "rusty") > 0.7);
        assert!(grader.similarity("rust", "python") < 0.5);
    }

    #[test]
    fn test_xp_multiplier() {
        let result3 = GradingResult {
            score: 3,
            max_score: 3,
            percentage: 100.0,
            feedback: String::new(),
            matched_concepts: Vec::new(),
            missing_concepts: Vec::new(),
            confidence: 1.0,
        };
        assert_eq!(result3.xp_multiplier(), 1.0);

        let result2 = GradingResult {
            score: 2,
            ..result3.clone()
        };
        assert!((result2.xp_multiplier() - 0.66).abs() < 0.01);

        let result1 = GradingResult {
            score: 1,
            ..result3.clone()
        };
        assert!((result1.xp_multiplier() - 0.33).abs() < 0.01);

        let result0 = GradingResult {
            score: 0,
            ..result3.clone()
        };
        assert_eq!(result0.xp_multiplier(), 0.0);
    }

    #[test]
    fn test_phrase_matching() {
        let grader = LocalGrader::new();
        let key = AnswerKey::new(vec!["pub fn", "return type"]);

        let result = grader.grade(
            "A pub fn declaration with a return type of Result",
            &key,
        );

        assert!(result.matched_concepts.len() >= 1);
    }

    #[test]
    fn test_empty_answer() {
        let grader = LocalGrader::new();
        let key = AnswerKey::new(vec!["concept"]);

        let result = grader.grade("", &key);

        assert_eq!(result.score, 0);
    }

    #[test]
    fn test_create_basic_key() {
        let grader = UnifiedGrader::new();
        let key = grader.create_basic_key(
            "Async functions return a Future that must be awaited",
        );

        assert!(!key.key_concepts.is_empty());
        // Should not include stop words like "that", "must", "be"
        assert!(!key.key_concepts.iter().any(|c| c == "that"));
    }
}
