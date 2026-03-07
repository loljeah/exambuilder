//! Adaptive Difficulty Engine (Phase 4)
//!
//! Adjusts question difficulty based on performance history per domain.
//! Uses a sliding window of recent performance to recommend optimal difficulty.

use serde::{Deserialize, Serialize};

/// Difficulty levels (1-5)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum DifficultyLevel {
    Beginner = 1,
    Easy = 2,
    Medium = 3,
    Hard = 4,
    Expert = 5,
}

impl DifficultyLevel {
    pub fn from_i32(val: i32) -> Self {
        match val {
            1 => DifficultyLevel::Beginner,
            2 => DifficultyLevel::Easy,
            3 => DifficultyLevel::Medium,
            4 => DifficultyLevel::Hard,
            5 => DifficultyLevel::Expert,
            _ => DifficultyLevel::Medium,
        }
    }

    pub fn to_tier(&self) -> &'static str {
        match self {
            DifficultyLevel::Beginner => "RECALL",
            DifficultyLevel::Easy => "RECALL",
            DifficultyLevel::Medium => "COMPREHENSION",
            DifficultyLevel::Hard => "APPLICATION",
            DifficultyLevel::Expert => "ANALYSIS",
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            DifficultyLevel::Beginner => "Beginner",
            DifficultyLevel::Easy => "Easy",
            DifficultyLevel::Medium => "Medium",
            DifficultyLevel::Hard => "Hard",
            DifficultyLevel::Expert => "Expert",
        }
    }
}

/// Performance window for tracking recent answers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceWindow {
    pub domain: String,
    pub recent_answers: Vec<bool>, // Last N answers (true = correct)
    pub window_size: usize,
}

impl PerformanceWindow {
    pub fn new(domain: &str, window_size: usize) -> Self {
        Self {
            domain: domain.to_string(),
            recent_answers: Vec::with_capacity(window_size),
            window_size,
        }
    }

    /// Add a new answer to the window
    pub fn add_answer(&mut self, correct: bool) {
        if self.recent_answers.len() >= self.window_size {
            self.recent_answers.remove(0);
        }
        self.recent_answers.push(correct);
    }

    /// Get recent accuracy (0.0 - 1.0)
    pub fn accuracy(&self) -> f64 {
        if self.recent_answers.is_empty() {
            return 0.5; // Default to 50% for no data
        }
        let correct = self.recent_answers.iter().filter(|&&x| x).count();
        correct as f64 / self.recent_answers.len() as f64
    }

    /// Get streak of consecutive correct/incorrect
    pub fn current_streak(&self) -> (bool, usize) {
        if self.recent_answers.is_empty() {
            return (true, 0);
        }

        let last = *self.recent_answers.last().unwrap();
        let streak = self
            .recent_answers
            .iter()
            .rev()
            .take_while(|&&x| x == last)
            .count();

        (last, streak)
    }
}

/// Adaptive difficulty engine
pub struct AdaptiveDifficultyEngine {
    /// Minimum accuracy to consider increasing difficulty
    pub increase_threshold: f64,
    /// Maximum accuracy to consider decreasing difficulty
    pub decrease_threshold: f64,
    /// Consecutive correct answers needed to increase
    pub streak_to_increase: usize,
    /// Consecutive wrong answers to decrease
    pub streak_to_decrease: usize,
}

impl Default for AdaptiveDifficultyEngine {
    fn default() -> Self {
        Self {
            increase_threshold: 0.80, // 80%+ accuracy to consider increase
            decrease_threshold: 0.50, // Below 50% to consider decrease
            streak_to_increase: 5,    // 5 correct in a row
            streak_to_decrease: 3,    // 3 wrong in a row
        }
    }
}

impl AdaptiveDifficultyEngine {
    pub fn new() -> Self {
        Self::default()
    }

    /// Calculate recommended difficulty based on performance
    pub fn recommend_difficulty(
        &self,
        current: DifficultyLevel,
        window: &PerformanceWindow,
    ) -> DifficultyRecommendation {
        let accuracy = window.accuracy();
        let (streak_positive, streak_count) = window.current_streak();

        // Check for streak-based adjustment
        if streak_positive && streak_count >= self.streak_to_increase {
            if current < DifficultyLevel::Expert {
                return DifficultyRecommendation {
                    level: DifficultyLevel::from_i32(current as i32 + 1),
                    reason: "Hot streak! Time for a challenge.".to_string(),
                    confidence: 0.9,
                };
            }
        }

        if !streak_positive && streak_count >= self.streak_to_decrease {
            if current > DifficultyLevel::Beginner {
                return DifficultyRecommendation {
                    level: DifficultyLevel::from_i32(current as i32 - 1),
                    reason: "Let's reinforce the basics.".to_string(),
                    confidence: 0.9,
                };
            }
        }

        // Check accuracy-based adjustment
        if accuracy >= self.increase_threshold && window.recent_answers.len() >= 5 {
            if current < DifficultyLevel::Expert {
                return DifficultyRecommendation {
                    level: DifficultyLevel::from_i32(current as i32 + 1),
                    reason: format!("{}% accuracy - ready for more!", (accuracy * 100.0) as i32),
                    confidence: 0.7,
                };
            }
        }

        if accuracy < self.decrease_threshold && window.recent_answers.len() >= 5 {
            if current > DifficultyLevel::Beginner {
                return DifficultyRecommendation {
                    level: DifficultyLevel::from_i32(current as i32 - 1),
                    reason: "Building stronger foundations.".to_string(),
                    confidence: 0.7,
                };
            }
        }

        // Stay at current level
        DifficultyRecommendation {
            level: current,
            reason: "Good progress at this level.".to_string(),
            confidence: 0.5,
        }
    }

    /// Select questions based on adaptive difficulty
    pub fn select_questions<'a, Q>(
        &self,
        available: &'a [Q],
        target_level: DifficultyLevel,
        count: usize,
        get_difficulty: impl Fn(&Q) -> DifficultyLevel,
    ) -> Vec<&'a Q> {
        let target = target_level as i32;

        // Sort by distance from target difficulty
        let mut scored: Vec<_> = available
            .iter()
            .map(|q| {
                let diff = get_difficulty(q) as i32;
                let distance = (diff - target).abs();
                (q, distance)
            })
            .collect();

        scored.sort_by_key(|(_, dist)| *dist);

        // Take the closest ones, with some variety
        scored.into_iter().take(count).map(|(q, _)| q).collect()
    }

    /// Generate a difficulty curve for a sprint (easy → medium → hard)
    pub fn generate_difficulty_curve(&self, base_level: DifficultyLevel, question_count: usize) -> Vec<DifficultyLevel> {
        let base = base_level as i32;

        match question_count {
            1 => vec![base_level],
            2 => {
                // Easy then hard
                vec![
                    DifficultyLevel::from_i32((base - 1).max(1)),
                    DifficultyLevel::from_i32((base + 1).min(5)),
                ]
            }
            3 => {
                // Easy, medium, hard
                vec![
                    DifficultyLevel::from_i32((base - 1).max(1)),
                    base_level,
                    DifficultyLevel::from_i32((base + 1).min(5)),
                ]
            }
            _ => {
                // Gradual progression
                (0..question_count)
                    .map(|i| {
                        let progress = i as f64 / (question_count - 1) as f64;
                        let level = base - 1 + (progress * 2.0).round() as i32;
                        DifficultyLevel::from_i32(level.clamp(1, 5))
                    })
                    .collect()
            }
        }
    }
}

/// Recommendation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DifficultyRecommendation {
    pub level: DifficultyLevel,
    pub reason: String,
    pub confidence: f64,
}

/// Domain performance profile
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainProfile {
    pub domain: String,
    pub current_level: DifficultyLevel,
    pub tier_accuracies: TierAccuracies,
    pub total_questions: i32,
    pub performance_window: PerformanceWindow,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TierAccuracies {
    pub recall: f64,
    pub comprehension: f64,
    pub application: f64,
    pub analysis: f64,
}

impl DomainProfile {
    pub fn new(domain: &str) -> Self {
        Self {
            domain: domain.to_string(),
            current_level: DifficultyLevel::Medium,
            tier_accuracies: TierAccuracies::default(),
            total_questions: 0,
            performance_window: PerformanceWindow::new(domain, 10),
        }
    }

    /// Update profile with a new answer
    pub fn record_answer(&mut self, tier: &str, correct: bool) {
        self.total_questions += 1;
        self.performance_window.add_answer(correct);

        // Update tier-specific accuracy (simple moving average)
        let weight = 0.1; // 10% weight for new answer
        let correct_val = if correct { 1.0 } else { 0.0 };

        match tier.to_uppercase().as_str() {
            "RECALL" => {
                self.tier_accuracies.recall =
                    self.tier_accuracies.recall * (1.0 - weight) + correct_val * weight;
            }
            "COMPREHENSION" => {
                self.tier_accuracies.comprehension =
                    self.tier_accuracies.comprehension * (1.0 - weight) + correct_val * weight;
            }
            "APPLICATION" => {
                self.tier_accuracies.application =
                    self.tier_accuracies.application * (1.0 - weight) + correct_val * weight;
            }
            "ANALYSIS" => {
                self.tier_accuracies.analysis =
                    self.tier_accuracies.analysis * (1.0 - weight) + correct_val * weight;
            }
            _ => {}
        }
    }

    /// Get weakest tier (lowest accuracy)
    pub fn weakest_tier(&self) -> &'static str {
        let tiers = [
            ("RECALL", self.tier_accuracies.recall),
            ("COMPREHENSION", self.tier_accuracies.comprehension),
            ("APPLICATION", self.tier_accuracies.application),
            ("ANALYSIS", self.tier_accuracies.analysis),
        ];

        tiers
            .iter()
            .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
            .map(|(name, _)| *name)
            .unwrap_or("RECALL")
    }

    /// Get strongest tier
    pub fn strongest_tier(&self) -> &'static str {
        let tiers = [
            ("RECALL", self.tier_accuracies.recall),
            ("COMPREHENSION", self.tier_accuracies.comprehension),
            ("APPLICATION", self.tier_accuracies.application),
            ("ANALYSIS", self.tier_accuracies.analysis),
        ];

        tiers
            .iter()
            .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
            .map(|(name, _)| *name)
            .unwrap_or("RECALL")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_performance_window() {
        let mut window = PerformanceWindow::new("rust", 5);

        // Add some answers
        window.add_answer(true);
        window.add_answer(true);
        window.add_answer(false);
        window.add_answer(true);

        assert_eq!(window.accuracy(), 0.75); // 3/4

        // Test streak
        let (positive, count) = window.current_streak();
        assert!(positive);
        assert_eq!(count, 1);

        // Fill to capacity and overflow
        window.add_answer(true);
        window.add_answer(true);
        assert_eq!(window.recent_answers.len(), 5);
    }

    #[test]
    fn test_streak_detection() {
        let mut window = PerformanceWindow::new("rust", 10);

        window.add_answer(true);
        window.add_answer(true);
        window.add_answer(true);

        let (positive, count) = window.current_streak();
        assert!(positive);
        assert_eq!(count, 3);

        window.add_answer(false);
        window.add_answer(false);

        let (positive, count) = window.current_streak();
        assert!(!positive);
        assert_eq!(count, 2);
    }

    #[test]
    fn test_difficulty_increase_on_streak() {
        let engine = AdaptiveDifficultyEngine::new();
        let mut window = PerformanceWindow::new("rust", 10);

        // Build a hot streak
        for _ in 0..5 {
            window.add_answer(true);
        }

        let rec = engine.recommend_difficulty(DifficultyLevel::Medium, &window);
        assert_eq!(rec.level, DifficultyLevel::Hard);
        assert!(rec.reason.contains("streak"));
    }

    #[test]
    fn test_difficulty_decrease_on_failures() {
        let engine = AdaptiveDifficultyEngine::new();
        let mut window = PerformanceWindow::new("rust", 10);

        // Build a failure streak
        for _ in 0..3 {
            window.add_answer(false);
        }

        let rec = engine.recommend_difficulty(DifficultyLevel::Medium, &window);
        assert_eq!(rec.level, DifficultyLevel::Easy);
    }

    #[test]
    fn test_difficulty_curve() {
        let engine = AdaptiveDifficultyEngine::new();

        let curve_3 = engine.generate_difficulty_curve(DifficultyLevel::Medium, 3);
        assert_eq!(curve_3.len(), 3);
        assert_eq!(curve_3[0], DifficultyLevel::Easy);
        assert_eq!(curve_3[1], DifficultyLevel::Medium);
        assert_eq!(curve_3[2], DifficultyLevel::Hard);

        let curve_2 = engine.generate_difficulty_curve(DifficultyLevel::Medium, 2);
        assert_eq!(curve_2.len(), 2);
        assert!(curve_2[0] < curve_2[1]);
    }

    #[test]
    fn test_domain_profile() {
        let mut profile = DomainProfile::new("rust");

        profile.record_answer("RECALL", true);
        profile.record_answer("RECALL", true);
        profile.record_answer("APPLICATION", false);

        assert!(profile.tier_accuracies.recall > 0.0);
        assert_eq!(profile.total_questions, 3);
    }

    #[test]
    fn test_difficulty_level_bounds() {
        let engine = AdaptiveDifficultyEngine::new();
        let mut window = PerformanceWindow::new("rust", 10);

        // Try to go below minimum
        for _ in 0..5 {
            window.add_answer(false);
        }
        let rec = engine.recommend_difficulty(DifficultyLevel::Beginner, &window);
        assert_eq!(rec.level, DifficultyLevel::Beginner); // Can't go lower

        // Try to go above maximum
        let mut window2 = PerformanceWindow::new("rust", 10);
        for _ in 0..5 {
            window2.add_answer(true);
        }
        let rec2 = engine.recommend_difficulty(DifficultyLevel::Expert, &window2);
        assert_eq!(rec2.level, DifficultyLevel::Expert); // Can't go higher
    }
}
