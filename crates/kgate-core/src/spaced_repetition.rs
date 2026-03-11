//! Spaced Repetition System (Phase 3)
//!
//! Implements SM-2 algorithm variant for optimal review scheduling.
//! Tracks wrong answers and resurfaces questions at optimal intervals.

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};

/// Review item representing a question that needs spaced repetition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewItem {
    pub id: i64,
    pub project_id: String,
    pub sprint_number: i32,
    pub question_number: i32,
    pub question_text: String,
    pub correct_answer: String,
    pub domain: String,

    // SM-2 algorithm fields
    pub easiness_factor: f64,      // EF, starts at 2.5
    pub repetition_count: i32,      // n, number of successful reviews
    pub interval_days: i32,         // current interval in days
    pub next_review: DateTime<Utc>, // when to review next
    pub last_reviewed: Option<DateTime<Utc>>,

    // Performance tracking
    pub times_correct: i32,
    pub times_wrong: i32,
    pub streak: i32,                // consecutive correct answers
}

/// Quality rating for a review (0-5 scale per SM-2)
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ReviewQuality {
    CompleteBlackout = 0,    // Complete failure, no memory
    IncorrectRemembered = 1, // Wrong but remembered after seeing answer
    IncorrectEasy = 2,       // Wrong but answer was easy to recall
    CorrectDifficult = 3,    // Correct with serious difficulty
    CorrectHesitation = 4,   // Correct after some hesitation
    Perfect = 5,             // Perfect response with no hesitation
}

impl ReviewQuality {
    pub fn from_score(correct: bool, response_time_ms: u64) -> Self {
        if !correct {
            ReviewQuality::IncorrectRemembered
        } else if response_time_ms > 10000 {
            ReviewQuality::CorrectDifficult
        } else if response_time_ms > 5000 {
            ReviewQuality::CorrectHesitation
        } else {
            ReviewQuality::Perfect
        }
    }

    pub fn from_bool(correct: bool) -> Self {
        if correct {
            ReviewQuality::CorrectHesitation
        } else {
            ReviewQuality::IncorrectRemembered
        }
    }
}

/// Spaced Repetition Engine
pub struct SpacedRepetitionEngine;

impl SpacedRepetitionEngine {
    /// Calculate next review interval using SM-2 algorithm
    pub fn calculate_next_review(item: &ReviewItem, quality: ReviewQuality) -> ReviewUpdate {
        let q = quality as i32;

        // Calculate new easiness factor
        // EF' = EF + (0.1 - (5 - q) * (0.08 + (5 - q) * 0.02))
        let ef_delta = 0.1 - (5.0 - q as f64) * (0.08 + (5.0 - q as f64) * 0.02);
        let new_ef = (item.easiness_factor + ef_delta).max(1.3); // EF never goes below 1.3

        let (new_interval, new_rep_count) = if q < 3 {
            // Failed review - reset to beginning
            (1, 0)
        } else {
            // Successful review
            let new_count = item.repetition_count + 1;
            let interval = match new_count {
                1 => 1,
                2 => 6,
                _ => (item.interval_days as f64 * new_ef).round() as i32,
            };
            (interval.max(1), new_count)
        };

        let next_review = Utc::now() + Duration::days(new_interval as i64);

        ReviewUpdate {
            easiness_factor: new_ef,
            repetition_count: new_rep_count,
            interval_days: new_interval,
            next_review,
            was_correct: q >= 3,
        }
    }

    /// Get items due for review
    pub fn get_due_items(items: &[ReviewItem], limit: usize) -> Vec<&ReviewItem> {
        let now = Utc::now();
        let mut due: Vec<_> = items
            .iter()
            .filter(|item| item.next_review <= now)
            .collect();

        // Sort by urgency (most overdue first)
        due.sort_by(|a, b| a.next_review.cmp(&b.next_review));
        due.truncate(limit);
        due
    }

    /// Calculate mastery percentage for a domain
    pub fn domain_mastery(items: &[ReviewItem], domain: &str) -> f64 {
        let domain_items: Vec<_> = items.iter().filter(|i| i.domain == domain).collect();

        if domain_items.is_empty() {
            return 0.0;
        }

        let total_ef: f64 = domain_items.iter().map(|i| i.easiness_factor).sum();
        let avg_ef = total_ef / domain_items.len() as f64;

        // Convert EF (1.3 - 2.5+) to percentage (0-100)
        ((avg_ef - 1.3) / 1.7 * 100.0).clamp(0.0, 100.0)
    }

    /// Get review statistics
    pub fn get_stats(items: &[ReviewItem]) -> ReviewStats {
        let now = Utc::now();
        let due_count = items.iter().filter(|i| i.next_review <= now).count();
        let total_correct: i32 = items.iter().map(|i| i.times_correct).sum();
        let total_wrong: i32 = items.iter().map(|i| i.times_wrong).sum();
        let total_reviews = total_correct + total_wrong;

        let accuracy = if total_reviews > 0 {
            (total_correct as f64 / total_reviews as f64) * 100.0
        } else {
            0.0
        };

        let avg_ef: f64 = if items.is_empty() {
            2.5
        } else {
            items.iter().map(|i| i.easiness_factor).sum::<f64>() / items.len() as f64
        };

        ReviewStats {
            total_items: items.len(),
            due_now: due_count,
            total_reviews,
            accuracy,
            average_easiness: avg_ef,
            longest_streak: items.iter().map(|i| i.streak).max().unwrap_or(0),
        }
    }
}

/// Result of review calculation
#[derive(Debug, Clone)]
pub struct ReviewUpdate {
    pub easiness_factor: f64,
    pub repetition_count: i32,
    pub interval_days: i32,
    pub next_review: DateTime<Utc>,
    pub was_correct: bool,
}

/// Review statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewStats {
    pub total_items: usize,
    pub due_now: usize,
    pub total_reviews: i32,
    pub accuracy: f64,
    pub average_easiness: f64,
    pub longest_streak: i32,
}

/// Domain catalog entry for collecting questions by domain
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainCatalogEntry {
    pub domain: String,
    pub question_id: String,
    pub question_text: String,
    pub correct_answer: String,
    pub source_project: String,
    pub source_sprint: i32,
    pub tier: String,
    pub difficulty: String,
    pub times_seen: i32,
    pub times_correct: i32,
    pub last_seen: Option<DateTime<Utc>>,
    pub tags: Vec<String>,
}

/// Domain catalog for organizing questions across projects
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DomainCatalog {
    pub domains: std::collections::HashMap<String, Vec<DomainCatalogEntry>>,
    pub total_questions: usize,
    pub last_updated: Option<DateTime<Utc>>,
}

impl DomainCatalog {
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a question to the catalog
    pub fn add_question(&mut self, entry: DomainCatalogEntry) {
        let domain = entry.domain.clone();
        self.domains.entry(domain).or_default().push(entry);
        self.total_questions += 1;
        self.last_updated = Some(Utc::now());
    }

    /// Get questions for a specific domain
    pub fn get_domain_questions(&self, domain: &str) -> Option<&Vec<DomainCatalogEntry>> {
        self.domains.get(domain)
    }

    /// Get all domains with question counts
    pub fn domain_stats(&self) -> Vec<(String, usize, f64)> {
        self.domains
            .iter()
            .map(|(domain, questions)| {
                let total: i32 = questions.iter().map(|q| q.times_seen).sum();
                let correct: i32 = questions.iter().map(|q| q.times_correct).sum();
                let accuracy = if total > 0 {
                    (correct as f64 / total as f64) * 100.0
                } else {
                    0.0
                };
                (domain.clone(), questions.len(), accuracy)
            })
            .collect()
    }

    /// Get weak domains (accuracy < 70%)
    pub fn weak_domains(&self) -> Vec<String> {
        self.domain_stats()
            .into_iter()
            .filter(|(_, count, accuracy)| *count >= 3 && *accuracy < 70.0)
            .map(|(domain, _, _)| domain)
            .collect()
    }

    /// Export catalog to JSON for .kgate profile
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    /// Load catalog from JSON
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_item() -> ReviewItem {
        ReviewItem {
            id: 1,
            project_id: "test".to_string(),
            sprint_number: 1,
            question_number: 1,
            question_text: "Test question?".to_string(),
            correct_answer: "A".to_string(),
            domain: "rust".to_string(),
            easiness_factor: 2.5,
            repetition_count: 0,
            interval_days: 0,
            next_review: Utc::now(),
            last_reviewed: None,
            times_correct: 0,
            times_wrong: 0,
            streak: 0,
        }
    }

    #[test]
    fn test_sm2_perfect_response() {
        let item = create_test_item();
        let update = SpacedRepetitionEngine::calculate_next_review(&item, ReviewQuality::Perfect);

        assert!(update.was_correct);
        assert_eq!(update.repetition_count, 1);
        assert_eq!(update.interval_days, 1);
        assert!(update.easiness_factor >= 2.5);
    }

    #[test]
    fn test_sm2_failed_response() {
        let mut item = create_test_item();
        item.repetition_count = 3;
        item.interval_days = 10;

        let update = SpacedRepetitionEngine::calculate_next_review(
            &item,
            ReviewQuality::IncorrectRemembered,
        );

        assert!(!update.was_correct);
        assert_eq!(update.repetition_count, 0); // Reset
        assert_eq!(update.interval_days, 1);    // Back to 1 day
    }

    #[test]
    fn test_sm2_interval_growth() {
        let mut item = create_test_item();

        // First review
        let update1 = SpacedRepetitionEngine::calculate_next_review(&item, ReviewQuality::Perfect);
        assert_eq!(update1.interval_days, 1);

        // Second review
        item.repetition_count = 1;
        item.interval_days = 1;
        let update2 = SpacedRepetitionEngine::calculate_next_review(&item, ReviewQuality::Perfect);
        assert_eq!(update2.interval_days, 6);

        // Third review - interval should grow
        item.repetition_count = 2;
        item.interval_days = 6;
        let update3 = SpacedRepetitionEngine::calculate_next_review(&item, ReviewQuality::Perfect);
        assert!(update3.interval_days > 6);
    }

    #[test]
    fn test_easiness_factor_bounds() {
        let mut item = create_test_item();
        item.easiness_factor = 1.5;

        // Multiple failures should not drop EF below 1.3
        for _ in 0..5 {
            let update = SpacedRepetitionEngine::calculate_next_review(
                &item,
                ReviewQuality::CompleteBlackout,
            );
            item.easiness_factor = update.easiness_factor;
        }

        assert!(item.easiness_factor >= 1.3);
    }

    #[test]
    fn test_get_due_items() {
        let past = Utc::now() - Duration::days(1);
        let future = Utc::now() + Duration::days(1);

        let items = vec![
            ReviewItem {
                next_review: past,
                ..create_test_item()
            },
            ReviewItem {
                id: 2,
                next_review: future,
                ..create_test_item()
            },
        ];

        let due = SpacedRepetitionEngine::get_due_items(&items, 10);
        assert_eq!(due.len(), 1);
        assert_eq!(due[0].id, 1);
    }

    #[test]
    fn test_domain_catalog() {
        let mut catalog = DomainCatalog::new();

        catalog.add_question(DomainCatalogEntry {
            domain: "rust".to_string(),
            question_id: "q1".to_string(),
            question_text: "What is ownership?".to_string(),
            correct_answer: "A".to_string(),
            source_project: "test".to_string(),
            source_sprint: 1,
            tier: "RECALL".to_string(),
            difficulty: "Easy".to_string(),
            times_seen: 5,
            times_correct: 4,
            last_seen: Some(Utc::now()),
            tags: vec!["ownership".to_string()],
        });

        catalog.add_question(DomainCatalogEntry {
            domain: "rust".to_string(),
            question_id: "q2".to_string(),
            question_text: "What is borrowing?".to_string(),
            correct_answer: "B".to_string(),
            source_project: "test".to_string(),
            source_sprint: 1,
            tier: "COMPREHENSION".to_string(),
            difficulty: "Medium".to_string(),
            times_seen: 3,
            times_correct: 2,
            last_seen: Some(Utc::now()),
            tags: vec!["borrowing".to_string()],
        });

        assert_eq!(catalog.total_questions, 2);
        assert_eq!(catalog.get_domain_questions("rust").unwrap().len(), 2);

        let stats = catalog.domain_stats();
        assert_eq!(stats.len(), 1);
        assert_eq!(stats[0].1, 2); // 2 questions
    }

    #[test]
    fn test_review_quality_from_score() {
        assert_eq!(
            ReviewQuality::from_score(false, 1000),
            ReviewQuality::IncorrectRemembered
        );
        assert_eq!(
            ReviewQuality::from_score(true, 15000),
            ReviewQuality::CorrectDifficult
        );
        assert_eq!(
            ReviewQuality::from_score(true, 7000),
            ReviewQuality::CorrectHesitation
        );
        assert_eq!(
            ReviewQuality::from_score(true, 2000),
            ReviewQuality::Perfect
        );
    }

    #[test]
    fn test_catalog_json_roundtrip() {
        let mut catalog = DomainCatalog::new();
        catalog.add_question(DomainCatalogEntry {
            domain: "test".to_string(),
            question_id: "q1".to_string(),
            question_text: "Test?".to_string(),
            correct_answer: "A".to_string(),
            source_project: "proj".to_string(),
            source_sprint: 1,
            tier: "RECALL".to_string(),
            difficulty: "Easy".to_string(),
            times_seen: 1,
            times_correct: 1,
            last_seen: None,
            tags: vec![],
        });

        let json = catalog.to_json().unwrap();
        let loaded = DomainCatalog::from_json(&json).unwrap();

        assert_eq!(loaded.total_questions, 1);
    }
}
