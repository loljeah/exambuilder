use anyhow::Result;
use chrono::{DateTime, Utc};
use sqlx::{sqlite::SqlitePoolOptions, SqlitePool};
use std::path::Path;

use crate::models::{
    Achievement, Badge, CollectedQuestion, DebtCurrent, Domain, DomainConnection, ExamAttempt,
    KnowledgeIdentity, Profile, Project, Setting, Sprint,
};

pub struct Database {
    pool: SqlitePool,
}

impl Database {
    pub async fn new(path: &Path) -> Result<Self> {
        let url = format!("sqlite:{}?mode=rwc", path.display());
        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect(&url)
            .await?;
        Ok(Self { pool })
    }

    pub async fn init(&self) -> Result<()> {
        sqlx::query(include_str!("../../../migrations/001_initial.sql"))
            .execute(&self.pool)
            .await?;
        sqlx::query(include_str!("../../../migrations/002_history_badges.sql"))
            .execute(&self.pool)
            .await?;
        sqlx::query(include_str!("../../../migrations/003_knowledge_profile.sql"))
            .execute(&self.pool)
            .await?;
        sqlx::query(include_str!("../../../migrations/004_spaced_repetition.sql"))
            .execute(&self.pool)
            .await?;
        // Migration 005 uses ALTER TABLE, run each statement separately
        let migration_005 = include_str!("../../../migrations/005_enhanced_stats.sql");
        for statement in migration_005.split(';') {
            // Strip comment-only lines before checking if the segment has real SQL
            let stripped: String = statement
                .lines()
                .filter(|l| !l.trim_start().starts_with("--"))
                .collect::<Vec<_>>()
                .join("\n");
            let stmt = stripped.trim();
            if !stmt.is_empty() {
                // Ignore errors for already-existing columns
                let _ = sqlx::query(stmt).execute(&self.pool).await;
            }
        }
        // Migration 006: sprint IDs and source project names
        let migration_006 = include_str!("../../../migrations/006_sprint_ids.sql");
        for statement in migration_006.split(';') {
            let stripped: String = statement
                .lines()
                .filter(|l| !l.trim_start().starts_with("--"))
                .collect::<Vec<_>>()
                .join("\n");
            let stmt = stripped.trim();
            if !stmt.is_empty() {
                let _ = sqlx::query(stmt).execute(&self.pool).await;
            }
        }
        Ok(())
    }

    // Profile
    pub async fn get_profile(&self) -> Result<Profile> {
        let profile = sqlx::query_as::<_, Profile>("SELECT * FROM profile WHERE id = 1")
            .fetch_one(&self.pool)
            .await?;
        Ok(profile)
    }

    pub async fn update_profile_xp(&self, xp_delta: i32) -> Result<Profile> {
        sqlx::query(
            "UPDATE profile SET total_xp = total_xp + ?, last_activity = datetime('now') WHERE id = 1",
        )
        .bind(xp_delta)
        .execute(&self.pool)
        .await?;

        // Check for level up
        let profile = self.get_profile().await?;
        let xp_needed = profile.xp_for_next_level();
        if profile.total_xp >= xp_needed {
            sqlx::query("UPDATE profile SET level = level + 1 WHERE id = 1")
                .execute(&self.pool)
                .await?;
        }

        self.get_profile().await
    }

    pub async fn update_streak(&self, passed: bool) -> Result<i32> {
        if passed {
            sqlx::query(
                "UPDATE profile SET
                    current_streak = current_streak + 1,
                    best_streak = MAX(best_streak, current_streak + 1),
                    sprints_passed = sprints_passed + 1
                WHERE id = 1",
            )
            .execute(&self.pool)
            .await?;
        } else {
            sqlx::query("UPDATE profile SET current_streak = 0 WHERE id = 1")
                .execute(&self.pool)
                .await?;
        }
        let profile = self.get_profile().await?;
        Ok(profile.current_streak)
    }

    /// Update question stats after answering (combo chain tracking)
    pub async fn update_question_stats(&self, correct: bool) -> Result<()> {
        if correct {
            sqlx::query(
                "UPDATE profile SET
                    questions_passed = questions_passed + 1,
                    questions_attempted = questions_attempted + 1,
                    current_combo = current_combo + 1,
                    best_combo = MAX(best_combo, current_combo + 1)
                WHERE id = 1",
            )
            .execute(&self.pool)
            .await?;
        } else {
            sqlx::query(
                "UPDATE profile SET
                    questions_attempted = questions_attempted + 1,
                    current_combo = 0
                WHERE id = 1",
            )
            .execute(&self.pool)
            .await?;
        }
        Ok(())
    }

    /// Record a perfect sprint (100% score)
    pub async fn record_perfect_sprint(&self) -> Result<()> {
        sqlx::query("UPDATE profile SET perfect_sprints = perfect_sprints + 1 WHERE id = 1")
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    /// Add study time in seconds
    pub async fn add_study_time(&self, seconds: i32) -> Result<()> {
        sqlx::query("UPDATE profile SET total_study_seconds = total_study_seconds + ? WHERE id = 1")
            .bind(seconds)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    // Projects
    pub async fn get_or_create_project(&self, path: &str, name: &str) -> Result<Project> {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(path.as_bytes());
        let full_hash = format!("{:x}", hasher.finalize());
        let short_id = &full_hash[..8];

        sqlx::query(
            "INSERT OR IGNORE INTO projects (id, full_hash, path, name) VALUES (?, ?, ?, ?)",
        )
        .bind(short_id)
        .bind(&full_hash)
        .bind(path)
        .bind(name)
        .execute(&self.pool)
        .await?;

        let project = sqlx::query_as::<_, Project>("SELECT * FROM projects WHERE id = ?")
            .bind(short_id)
            .fetch_one(&self.pool)
            .await?;
        Ok(project)
    }

    pub async fn list_projects(&self) -> Result<Vec<Project>> {
        let projects =
            sqlx::query_as::<_, Project>("SELECT * FROM projects ORDER BY last_active DESC")
                .fetch_all(&self.pool)
                .await?;
        Ok(projects)
    }

    // Debt
    pub async fn get_debt(&self, project_id: &str) -> Result<i32> {
        let result = sqlx::query_as::<_, DebtCurrent>(
            "SELECT * FROM debt_current WHERE project_id = ?",
        )
        .bind(project_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(result.map(|d| d.total).unwrap_or(0))
    }

    pub async fn add_debt(&self, project_id: &str, action: &str, weight: i32, description: Option<&str>) -> Result<i32> {
        // Log the debt entry
        sqlx::query(
            "INSERT INTO debt_log (project_id, action, weight, description) VALUES (?, ?, ?, ?)",
        )
        .bind(project_id)
        .bind(action)
        .bind(weight)
        .bind(description)
        .execute(&self.pool)
        .await?;

        // Update current total
        sqlx::query(
            "INSERT INTO debt_current (project_id, total) VALUES (?, ?)
            ON CONFLICT(project_id) DO UPDATE SET total = total + ?, last_updated = datetime('now')",
        )
        .bind(project_id)
        .bind(weight)
        .bind(weight)
        .execute(&self.pool)
        .await?;

        self.get_debt(project_id).await
    }

    pub async fn clear_debt(&self, project_id: &str, amount: i32) -> Result<i32> {
        sqlx::query(
            "UPDATE debt_current SET total = MAX(0, total - ?), last_updated = datetime('now') WHERE project_id = ?",
        )
        .bind(amount)
        .bind(project_id)
        .execute(&self.pool)
        .await?;

        self.get_debt(project_id).await
    }

    // Sprints
    pub async fn get_sprints(&self, project_id: &str) -> Result<Vec<Sprint>> {
        let sprints = sqlx::query_as::<_, Sprint>(
            "SELECT * FROM sprints WHERE project_id = ? ORDER BY sprint_number",
        )
        .bind(project_id)
        .fetch_all(&self.pool)
        .await?;
        Ok(sprints)
    }

    pub async fn get_sprint(&self, project_id: &str, sprint_number: i32) -> Result<Option<Sprint>> {
        let sprint = sqlx::query_as::<_, Sprint>(
            "SELECT * FROM sprints WHERE project_id = ? AND sprint_number = ?",
        )
        .bind(project_id)
        .bind(sprint_number)
        .fetch_optional(&self.pool)
        .await?;
        Ok(sprint)
    }

    pub async fn upsert_sprint(&self, sprint: &Sprint) -> Result<()> {
        sqlx::query(
            "INSERT INTO sprints (project_id, sprint_number, topic, questions_json, answer_key_json, xp_available, sprint_id, source_project_name)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?)
            ON CONFLICT(project_id, sprint_number) DO UPDATE SET
                topic = excluded.topic,
                questions_json = excluded.questions_json,
                answer_key_json = excluded.answer_key_json,
                xp_available = excluded.xp_available,
                sprint_id = excluded.sprint_id,
                source_project_name = excluded.source_project_name",
        )
        .bind(&sprint.project_id)
        .bind(sprint.sprint_number)
        .bind(&sprint.topic)
        .bind(&sprint.questions_json)
        .bind(&sprint.answer_key_json)
        .bind(sprint.xp_available)
        .bind(&sprint.sprint_id)
        .bind(&sprint.source_project_name)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn record_sprint_attempt(
        &self,
        project_id: &str,
        sprint_number: i32,
        score: i32,
        passed: bool,
        xp_earned: i32,
    ) -> Result<()> {
        // Log to history
        sqlx::query(
            "INSERT INTO exam_attempts (project_id, sprint_number, score_percent, passed, xp_earned)
            VALUES (?, ?, ?, ?, ?)",
        )
        .bind(project_id)
        .bind(sprint_number)
        .bind(score)
        .bind(passed)
        .bind(xp_earned)
        .execute(&self.pool)
        .await?;

        // Update sprint record
        if passed {
            sqlx::query(
                "UPDATE sprints SET
                    attempts = attempts + 1,
                    best_score = MAX(COALESCE(best_score, 0), ?),
                    status = 'passed',
                    passed_at = datetime('now'),
                    xp_earned = xp_available
                WHERE project_id = ? AND sprint_number = ?",
            )
            .bind(score)
            .bind(project_id)
            .bind(sprint_number)
            .execute(&self.pool)
            .await?;
        } else {
            sqlx::query(
                "UPDATE sprints SET
                    attempts = attempts + 1,
                    best_score = MAX(COALESCE(best_score, 0), ?)
                WHERE project_id = ? AND sprint_number = ?",
            )
            .bind(score)
            .bind(project_id)
            .bind(sprint_number)
            .execute(&self.pool)
            .await?;
        }
        Ok(())
    }

    // History
    pub async fn get_history(&self, limit: i32) -> Result<Vec<ExamAttempt>> {
        let attempts = sqlx::query_as::<_, ExamAttempt>(
            "SELECT * FROM exam_attempts ORDER BY timestamp DESC LIMIT ?",
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;
        Ok(attempts)
    }

    pub async fn get_project_history(&self, project_id: &str, limit: i32) -> Result<Vec<ExamAttempt>> {
        let attempts = sqlx::query_as::<_, ExamAttempt>(
            "SELECT * FROM exam_attempts WHERE project_id = ? ORDER BY timestamp DESC LIMIT ?",
        )
        .bind(project_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;
        Ok(attempts)
    }

    // Badges
    pub async fn get_badges(&self) -> Result<Vec<Badge>> {
        let badges = sqlx::query_as::<_, Badge>(
            "SELECT * FROM badges WHERE unlocked_at IS NOT NULL ORDER BY unlocked_at DESC",
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(badges)
    }

    pub async fn unlock_badge(&self, badge_id: &str, project_id: Option<&str>) -> Result<bool> {
        // Check if already unlocked
        let existing = sqlx::query_as::<_, Badge>(
            "SELECT * FROM badges WHERE id = ? AND unlocked_at IS NOT NULL",
        )
        .bind(badge_id)
        .fetch_optional(&self.pool)
        .await?;

        if existing.is_some() {
            return Ok(false);
        }

        sqlx::query(
            "UPDATE badges SET unlocked_at = datetime('now'), project_id = ? WHERE id = ?",
        )
        .bind(project_id)
        .bind(badge_id)
        .execute(&self.pool)
        .await?;

        Ok(true)
    }

    pub async fn check_and_award_badges(&self, project_id: &str) -> Result<Vec<String>> {
        let mut awarded: Vec<String> = Vec::new();
        let profile = self.get_profile().await?;

        // First sprint
        if profile.sprints_passed == 1
            && self.unlock_badge("first_sprint", Some(project_id)).await? {
                awarded.push("first_sprint".to_string());
            }

        // Streak badges
        if profile.current_streak >= 3
            && self.unlock_badge("streak_3", None).await? {
                awarded.push("streak_3".to_string());
            }
        if profile.current_streak >= 5
            && self.unlock_badge("streak_5", None).await? {
                awarded.push("streak_5".to_string());
            }
        if profile.current_streak >= 10
            && self.unlock_badge("streak_10", None).await? {
                awarded.push("streak_10".to_string());
            }

        // Level badges
        if profile.level >= 2
            && self.unlock_badge("level_2", None).await? {
                awarded.push("level_2".to_string());
            }
        if profile.level >= 3
            && self.unlock_badge("level_3", None).await? {
                awarded.push("level_3".to_string());
            }
        if profile.level >= 5
            && self.unlock_badge("level_5", None).await? {
                awarded.push("level_5".to_string());
            }

        // XP badges
        if profile.total_xp >= 100
            && self.unlock_badge("xp_100", None).await? {
                awarded.push("xp_100".to_string());
            }
        if profile.total_xp >= 500
            && self.unlock_badge("xp_500", None).await? {
                awarded.push("xp_500".to_string());
            }
        if profile.total_xp >= 1000
            && self.unlock_badge("xp_1000", None).await? {
                awarded.push("xp_1000".to_string());
            }

        // Project cleared
        let sprints = self.get_sprints(project_id).await?;
        if !sprints.is_empty() && sprints.iter().all(|s| s.status == "passed")
            && self.unlock_badge("project_clear", Some(project_id)).await? {
                awarded.push("project_clear".to_string());
            }

        Ok(awarded)
    }

    // Project switching
    pub async fn set_active_project(&self, project_id: &str) -> Result<()> {
        sqlx::query(
            "UPDATE projects SET last_active = datetime('now') WHERE id = ?",
        )
        .bind(project_id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn get_project_by_id(&self, project_id: &str) -> Result<Option<Project>> {
        let project = sqlx::query_as::<_, Project>(
            "SELECT * FROM projects WHERE id = ?",
        )
        .bind(project_id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(project)
    }

    // Knowledge Identity
    pub async fn get_knowledge_id(&self) -> Result<KnowledgeIdentity> {
        let identity = sqlx::query_as::<_, KnowledgeIdentity>(
            "SELECT * FROM knowledge_identity WHERE id = 1",
        )
        .fetch_one(&self.pool)
        .await?;
        Ok(identity)
    }

    pub async fn set_display_name(&self, name: &str) -> Result<()> {
        sqlx::query("UPDATE knowledge_identity SET display_name = ? WHERE id = 1")
            .bind(name)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    // Domains
    pub async fn get_domains(&self) -> Result<Vec<Domain>> {
        let domains = sqlx::query_as::<_, Domain>(
            "SELECT * FROM domains WHERE total_xp > 0 ORDER BY total_xp DESC",
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(domains)
    }

    pub async fn get_all_domains(&self) -> Result<Vec<Domain>> {
        let domains = sqlx::query_as::<_, Domain>("SELECT * FROM domains ORDER BY name")
            .fetch_all(&self.pool)
            .await?;
        Ok(domains)
    }

    pub async fn update_domain_progress(
        &self,
        domain_id: &str,
        xp: i32,
        correct: bool,
    ) -> Result<()> {
        let correct_delta = if correct { 1 } else { 0 };
        sqlx::query(
            "UPDATE domains SET
                total_xp = total_xp + ?,
                questions_seen = questions_seen + 1,
                questions_correct = questions_correct + ?,
                mastery_level = CASE
                    WHEN total_xp + ? >= 500 THEN 5
                    WHEN total_xp + ? >= 300 THEN 4
                    WHEN total_xp + ? >= 150 THEN 3
                    WHEN total_xp + ? >= 75 THEN 2
                    WHEN total_xp + ? >= 25 THEN 1
                    ELSE 0
                END
            WHERE id = ?",
        )
        .bind(xp)
        .bind(correct_delta)
        .bind(xp)
        .bind(xp)
        .bind(xp)
        .bind(xp)
        .bind(xp)
        .bind(domain_id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    // Domain Connections
    pub async fn record_domain_connection(&self, domain_a: &str, domain_b: &str) -> Result<()> {
        // Ensure alphabetical order for consistent storage
        let (a, b) = if domain_a < domain_b {
            (domain_a, domain_b)
        } else {
            (domain_b, domain_a)
        };
        sqlx::query(
            "INSERT INTO domain_connections (domain_a, domain_b, strength)
            VALUES (?, ?, 1)
            ON CONFLICT(domain_a, domain_b) DO UPDATE SET strength = strength + 1",
        )
        .bind(a)
        .bind(b)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn get_domain_connections(&self) -> Result<Vec<DomainConnection>> {
        let connections = sqlx::query_as::<_, DomainConnection>(
            "SELECT * FROM domain_connections ORDER BY strength DESC",
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(connections)
    }

    // Collected Questions
    #[allow(clippy::too_many_arguments)]
    pub async fn collect_question(
        &self,
        project_id: &str,
        sprint_number: i32,
        question_number: i32,
        question_text: &str,
        correct_answer: &str,
        user_answer: &str,
        tier: &str,
        xp_earned: i32,
        domains: &[String],
    ) -> Result<()> {
        let domains_json = serde_json::to_string(domains)?;
        sqlx::query(
            "INSERT OR REPLACE INTO collected_questions
            (project_id, sprint_number, question_number, question_text, correct_answer,
             user_answer, tier, xp_earned, domains_json)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(project_id)
        .bind(sprint_number)
        .bind(question_number)
        .bind(question_text)
        .bind(correct_answer)
        .bind(user_answer)
        .bind(tier)
        .bind(xp_earned)
        .bind(domains_json)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn get_collected_questions(&self) -> Result<Vec<CollectedQuestion>> {
        let questions = sqlx::query_as::<_, CollectedQuestion>(
            "SELECT * FROM collected_questions ORDER BY collected_at DESC",
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(questions)
    }

    pub async fn count_collected(&self) -> Result<i64> {
        let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM collected_questions")
            .fetch_one(&self.pool)
            .await?;
        Ok(count.0)
    }

    // Achievements
    pub async fn get_achievements(&self) -> Result<Vec<Achievement>> {
        let achievements = sqlx::query_as::<_, Achievement>(
            "SELECT * FROM achievements WHERE unlocked_at IS NOT NULL ORDER BY unlocked_at DESC",
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(achievements)
    }

    pub async fn unlock_achievement(
        &self,
        achievement_id: &str,
        context: Option<&str>,
    ) -> Result<bool> {
        // Check if already unlocked
        let existing: Option<Achievement> = sqlx::query_as(
            "SELECT * FROM achievements WHERE id = ? AND unlocked_at IS NOT NULL",
        )
        .bind(achievement_id)
        .fetch_optional(&self.pool)
        .await?;

        if existing.is_some() {
            return Ok(false);
        }

        sqlx::query(
            "UPDATE achievements SET unlocked_at = datetime('now'), context_json = ? WHERE id = ?",
        )
        .bind(context)
        .bind(achievement_id)
        .execute(&self.pool)
        .await?;

        Ok(true)
    }

    pub async fn check_collection_achievements(&self) -> Result<Vec<String>> {
        let mut unlocked = Vec::new();
        let count = self.count_collected().await?;

        if count >= 10
            && self.unlock_achievement("collector_10", None).await? {
                unlocked.push("collector_10".to_string());
            }
        if count >= 50
            && self.unlock_achievement("collector_50", None).await? {
                unlocked.push("collector_50".to_string());
            }
        if count >= 100
            && self.unlock_achievement("collector_100", None).await? {
                unlocked.push("collector_100".to_string());
            }

        // Check domain master
        let domains = self.get_domains().await?;
        for d in domains {
            if d.mastery_level >= 5 {
                if self
                    .unlock_achievement("domain_master", Some(&d.id))
                    .await?
                {
                    unlocked.push("domain_master".to_string());
                }
                break;
            }
        }

        // Check bridge builder
        let connections = self.get_domain_connections().await?;
        if connections.len() >= 5
            && self.unlock_achievement("bridge_builder", None).await? {
                unlocked.push("bridge_builder".to_string());
            }

        Ok(unlocked)
    }

    // Settings
    pub async fn get_setting(&self, key: &str) -> Result<Option<String>> {
        let setting: Option<Setting> =
            sqlx::query_as("SELECT * FROM settings WHERE key = ?")
                .bind(key)
                .fetch_optional(&self.pool)
                .await?;
        Ok(setting.map(|s| s.value))
    }

    pub async fn set_setting(&self, key: &str, value: &str) -> Result<()> {
        sqlx::query(
            "INSERT OR REPLACE INTO settings (key, value) VALUES (?, ?)",
        )
        .bind(key)
        .bind(value)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn is_sound_enabled(&self) -> Result<bool> {
        let val = self.get_setting("sound_enabled").await?;
        Ok(val.map(|v| v == "true").unwrap_or(true))
    }

    pub async fn is_fast_answer_enabled(&self) -> Result<bool> {
        let val = self.get_setting("fast_answer_mode").await?;
        Ok(val.map(|v| v == "true").unwrap_or(true))
    }

    // ============================================
    // Spaced Repetition Methods (Phase 3)
    // ============================================

    /// Add a question to spaced repetition review queue
    pub async fn add_review_item(
        &self,
        project_id: &str,
        sprint_number: i32,
        question_number: i32,
        question_text: &str,
        correct_answer: &str,
        domain: &str,
    ) -> Result<()> {
        sqlx::query(
            "INSERT OR IGNORE INTO review_items
            (project_id, sprint_number, question_number, question_text, correct_answer, domain)
            VALUES (?, ?, ?, ?, ?, ?)",
        )
        .bind(project_id)
        .bind(sprint_number)
        .bind(question_number)
        .bind(question_text)
        .bind(correct_answer)
        .bind(domain)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    /// Get items due for review
    pub async fn get_due_reviews(&self, limit: i32) -> Result<Vec<ReviewItemRow>> {
        let items = sqlx::query_as::<_, ReviewItemRow>(
            "SELECT * FROM review_items WHERE next_review <= datetime('now')
            ORDER BY next_review ASC LIMIT ?",
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;
        Ok(items)
    }

    /// Update review item after a review
    pub async fn update_review_item(
        &self,
        id: i64,
        easiness_factor: f64,
        repetition_count: i32,
        interval_days: i32,
        next_review: &str,
        was_correct: bool,
    ) -> Result<()> {
        let (correct_delta, wrong_delta, streak_update) = if was_correct {
            (1, 0, "streak = streak + 1")
        } else {
            (0, 1, "streak = 0")
        };

        let query = format!(
            "UPDATE review_items SET
                easiness_factor = ?,
                repetition_count = ?,
                interval_days = ?,
                next_review = ?,
                last_reviewed = datetime('now'),
                times_correct = times_correct + ?,
                times_wrong = times_wrong + ?,
                {}
            WHERE id = ?",
            streak_update
        );

        sqlx::query(&query)
            .bind(easiness_factor)
            .bind(repetition_count)
            .bind(interval_days)
            .bind(next_review)
            .bind(correct_delta)
            .bind(wrong_delta)
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    /// Get review statistics
    pub async fn get_review_stats(&self) -> Result<ReviewStatsRow> {
        let stats = sqlx::query_as::<_, ReviewStatsRow>(
            "SELECT
                COUNT(*) as total_items,
                SUM(CASE WHEN next_review <= datetime('now') THEN 1 ELSE 0 END) as due_now,
                SUM(times_correct) as total_correct,
                SUM(times_wrong) as total_wrong,
                AVG(easiness_factor) as avg_easiness,
                MAX(streak) as max_streak
            FROM review_items",
        )
        .fetch_one(&self.pool)
        .await?;
        Ok(stats)
    }

    // ============================================
    // Domain Catalog Methods (Phase 3)
    // ============================================

    /// Add question to domain catalog
    #[allow(clippy::too_many_arguments)]
    pub async fn add_to_domain_catalog(
        &self,
        domain: &str,
        question_id: &str,
        question_text: &str,
        correct_answer: &str,
        source_project: &str,
        source_sprint: i32,
        tier: &str,
        difficulty: &str,
        tags: &[String],
    ) -> Result<()> {
        let tags_json = serde_json::to_string(tags)?;
        sqlx::query(
            "INSERT INTO domain_catalog
            (domain, question_id, question_text, correct_answer, source_project, source_sprint, tier, difficulty, tags_json)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
            ON CONFLICT(question_id) DO UPDATE SET
                times_seen = times_seen + 1,
                last_seen = datetime('now')",
        )
        .bind(domain)
        .bind(question_id)
        .bind(question_text)
        .bind(correct_answer)
        .bind(source_project)
        .bind(source_sprint)
        .bind(tier)
        .bind(difficulty)
        .bind(tags_json)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    /// Update domain catalog entry after answer
    pub async fn update_catalog_entry(&self, question_id: &str, was_correct: bool) -> Result<()> {
        let correct_delta = if was_correct { 1 } else { 0 };
        sqlx::query(
            "UPDATE domain_catalog SET
                times_seen = times_seen + 1,
                times_correct = times_correct + ?,
                last_seen = datetime('now')
            WHERE question_id = ?",
        )
        .bind(correct_delta)
        .bind(question_id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    /// Get domain catalog entries for a specific domain
    pub async fn get_domain_catalog(&self, domain: &str) -> Result<Vec<DomainCatalogRow>> {
        let entries = sqlx::query_as::<_, DomainCatalogRow>(
            "SELECT * FROM domain_catalog WHERE domain = ? ORDER BY times_seen DESC",
        )
        .bind(domain)
        .fetch_all(&self.pool)
        .await?;
        Ok(entries)
    }

    /// Get all domain stats
    pub async fn get_domain_catalog_stats(&self) -> Result<Vec<DomainCatalogStatsRow>> {
        let stats = sqlx::query_as::<_, DomainCatalogStatsRow>(
            "SELECT
                domain,
                COUNT(*) as question_count,
                SUM(times_seen) as total_attempts,
                SUM(times_correct) as total_correct,
                CASE WHEN SUM(times_seen) > 0
                    THEN CAST(SUM(times_correct) AS REAL) / SUM(times_seen) * 100
                    ELSE 0
                END as accuracy
            FROM domain_catalog
            GROUP BY domain
            ORDER BY question_count DESC",
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(stats)
    }

    /// Export domain catalog to JSON
    pub async fn export_domain_catalog(&self) -> Result<String> {
        let entries = sqlx::query_as::<_, DomainCatalogRow>(
            "SELECT * FROM domain_catalog ORDER BY domain, question_id",
        )
        .fetch_all(&self.pool)
        .await?;

        serde_json::to_string_pretty(&entries).map_err(|e| anyhow::anyhow!("{}", e))
    }

    // ============================================
    // Adaptive Difficulty Methods (Phase 4)
    // ============================================

    /// Update difficulty profile after answering a question
    pub async fn update_difficulty_profile(
        &self,
        domain: &str,
        tier: &str,
        was_correct: bool,
    ) -> Result<()> {
        let tier_col = match tier.to_uppercase().as_str() {
            "RECALL" => "recall",
            "COMPREHENSION" => "comprehension",
            "APPLICATION" => "application",
            "ANALYSIS" => "analysis",
            _ => "recall",
        };

        // First ensure the row exists
        sqlx::query("INSERT OR IGNORE INTO difficulty_profile (domain) VALUES (?)")
            .bind(domain)
            .execute(&self.pool)
            .await?;

        // Update the appropriate tier
        let count_col = format!("{}_count", tier_col);
        let accuracy_col = format!("{}_accuracy", tier_col);

        let correct_val = if was_correct { 1.0 } else { 0.0 };

        let query = format!(
            "UPDATE difficulty_profile SET
                {count_col} = {count_col} + 1,
                {accuracy_col} = ({accuracy_col} * {count_col} + ?) / ({count_col} + 1),
                consecutive_correct = CASE WHEN ? = 1 THEN consecutive_correct + 1 ELSE 0 END,
                consecutive_wrong = CASE WHEN ? = 0 THEN consecutive_wrong + 1 ELSE 0 END,
                recommended_level = CASE
                    WHEN consecutive_correct >= 5 THEN MIN(recommended_level + 1, 5)
                    WHEN consecutive_wrong >= 3 THEN MAX(recommended_level - 1, 1)
                    ELSE recommended_level
                END,
                last_updated = datetime('now')
            WHERE domain = ?",
            count_col = count_col,
            accuracy_col = accuracy_col
        );

        sqlx::query(&query)
            .bind(correct_val)
            .bind(if was_correct { 1 } else { 0 })
            .bind(if was_correct { 1 } else { 0 })
            .bind(domain)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    /// Get recommended difficulty level for a domain
    pub async fn get_recommended_difficulty(&self, domain: &str) -> Result<i32> {
        let result: Option<(i32,)> = sqlx::query_as(
            "SELECT recommended_level FROM difficulty_profile WHERE domain = ?",
        )
        .bind(domain)
        .fetch_optional(&self.pool)
        .await?;

        Ok(result.map(|r| r.0).unwrap_or(2)) // Default to medium difficulty
    }

    // ============================================
    // Review Session Methods
    // ============================================

    /// Start a new review session, returns the session ID
    pub async fn start_review_session(&self) -> Result<i64> {
        let result = sqlx::query(
            "INSERT INTO review_sessions (started_at) VALUES (datetime('now'))",
        )
        .execute(&self.pool)
        .await?;
        Ok(result.last_insert_rowid())
    }

    /// End a review session with final stats
    pub async fn end_review_session(
        &self,
        session_id: i64,
        items_reviewed: i32,
        items_correct: i32,
        domains_json: &str,
        xp_earned: i32,
    ) -> Result<()> {
        sqlx::query(
            "UPDATE review_sessions SET
                ended_at = datetime('now'),
                items_reviewed = ?,
                items_correct = ?,
                domains_covered = ?,
                xp_earned = ?
            WHERE id = ?",
        )
        .bind(items_reviewed)
        .bind(items_correct)
        .bind(domains_json)
        .bind(xp_earned)
        .bind(session_id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    /// Get full difficulty profile for a domain
    pub async fn get_difficulty_profile(&self, domain: &str) -> Result<Option<DifficultyProfileRow>> {
        let profile = sqlx::query_as::<_, DifficultyProfileRow>(
            "SELECT * FROM difficulty_profile WHERE domain = ?",
        )
        .bind(domain)
        .fetch_optional(&self.pool)
        .await?;
        Ok(profile)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    /// Returns (Database, TempDir). The TempDir must be kept alive for the
    /// duration of the test so the SQLite file is not deleted.
    async fn setup_db() -> (Database, TempDir) {
        let dir = TempDir::new().unwrap();
        let db_path = dir.path().join("test.db");
        let db = Database::new(&db_path).await.unwrap();
        db.init().await.unwrap();
        (db, dir)
    }

    async fn setup_db_with_project() -> (Database, Project, TempDir) {
        let (db, dir) = setup_db().await;
        let project = db
            .get_or_create_project("/tmp/test-project", "test-project")
            .await
            .unwrap();
        (db, project, dir)
    }

    // ---- Profile tests ----

    #[tokio::test]
    async fn test_get_profile_returns_defaults_after_init() {
        let (db, _dir) = setup_db().await;
        let profile = db.get_profile().await.unwrap();
        assert_eq!(profile.id, 1);
        assert_eq!(profile.total_xp, 0);
        assert_eq!(profile.level, 1);
        assert_eq!(profile.current_streak, 0);
        assert_eq!(profile.best_streak, 0);
        assert_eq!(profile.sprints_passed, 0);
    }

    #[tokio::test]
    async fn test_update_profile_xp_adds_xp() {
        let (db, _dir) = setup_db().await;
        let profile = db.update_profile_xp(25).await.unwrap();
        assert_eq!(profile.total_xp, 25);
    }

    #[tokio::test]
    async fn test_update_profile_xp_triggers_level_up() {
        let (db, _dir) = setup_db().await;
        // Level 1 needs 50 XP to level up
        let profile = db.update_profile_xp(50).await.unwrap();
        assert_eq!(profile.level, 2);
    }

    #[tokio::test]
    async fn test_update_streak_increments_on_pass() {
        let (db, _dir) = setup_db().await;
        let streak = db.update_streak(true).await.unwrap();
        assert_eq!(streak, 1);
        let streak = db.update_streak(true).await.unwrap();
        assert_eq!(streak, 2);
    }

    #[tokio::test]
    async fn test_update_streak_resets_on_fail() {
        let (db, _dir) = setup_db().await;
        db.update_streak(true).await.unwrap();
        db.update_streak(true).await.unwrap();
        let streak = db.update_streak(false).await.unwrap();
        assert_eq!(streak, 0);
    }

    #[tokio::test]
    async fn test_update_streak_tracks_best() {
        let (db, _dir) = setup_db().await;
        db.update_streak(true).await.unwrap();
        db.update_streak(true).await.unwrap();
        db.update_streak(true).await.unwrap();
        db.update_streak(false).await.unwrap();
        let profile = db.get_profile().await.unwrap();
        assert_eq!(profile.best_streak, 3);
        assert_eq!(profile.current_streak, 0);
    }

    // ---- Project tests ----

    #[tokio::test]
    async fn test_get_or_create_project_creates_new() {
        let (db, _dir) = setup_db().await;
        let project = db
            .get_or_create_project("/tmp/my-project", "my-project")
            .await
            .unwrap();
        assert_eq!(project.name, "my-project");
        assert_eq!(project.path, "/tmp/my-project");
        assert!(!project.id.is_empty());
    }

    #[tokio::test]
    async fn test_get_or_create_project_returns_same_on_second_call() {
        let (db, _dir) = setup_db().await;
        let p1 = db
            .get_or_create_project("/tmp/my-project", "my-project")
            .await
            .unwrap();
        let p2 = db
            .get_or_create_project("/tmp/my-project", "my-project")
            .await
            .unwrap();
        assert_eq!(p1.id, p2.id);
    }

    #[tokio::test]
    async fn test_list_projects_empty_initially() {
        let (db, _dir) = setup_db().await;
        let projects = db.list_projects().await.unwrap();
        assert!(projects.is_empty());
    }

    #[tokio::test]
    async fn test_list_projects_populated_after_add() {
        let (db, _dir) = setup_db().await;
        db.get_or_create_project("/tmp/p1", "p1").await.unwrap();
        db.get_or_create_project("/tmp/p2", "p2").await.unwrap();
        let projects = db.list_projects().await.unwrap();
        assert_eq!(projects.len(), 2);
    }

    #[tokio::test]
    async fn test_get_project_by_id() {
        let (db, project, _dir) = setup_db_with_project().await;
        let found = db.get_project_by_id(&project.id).await.unwrap();
        assert!(found.is_some());
        assert_eq!(found.unwrap().name, "test-project");
    }

    #[tokio::test]
    async fn test_get_project_by_id_not_found() {
        let (db, _dir) = setup_db().await;
        let found = db.get_project_by_id("nonexistent").await.unwrap();
        assert!(found.is_none());
    }

    // ---- Debt tests ----

    #[tokio::test]
    async fn test_get_debt_returns_zero_initially() {
        let (db, project, _dir) = setup_db_with_project().await;
        let debt = db.get_debt(&project.id).await.unwrap();
        assert_eq!(debt, 0);
    }

    #[tokio::test]
    async fn test_add_debt_increments_correctly() {
        let (db, project, _dir) = setup_db_with_project().await;
        let debt = db
            .add_debt(&project.id, "concept_explained", 1, None)
            .await
            .unwrap();
        assert_eq!(debt, 1);
        let debt = db
            .add_debt(&project.id, "architecture_decision", 2, Some("chose X"))
            .await
            .unwrap();
        assert_eq!(debt, 3);
    }

    #[tokio::test]
    async fn test_clear_debt_decrements() {
        let (db, project, _dir) = setup_db_with_project().await;
        db.add_debt(&project.id, "action", 5, None).await.unwrap();
        let debt = db.clear_debt(&project.id, 3).await.unwrap();
        assert_eq!(debt, 2);
    }

    #[tokio::test]
    async fn test_clear_debt_floors_at_zero() {
        let (db, project, _dir) = setup_db_with_project().await;
        db.add_debt(&project.id, "action", 3, None).await.unwrap();
        let debt = db.clear_debt(&project.id, 10).await.unwrap();
        assert_eq!(debt, 0);
    }

    // ---- Sprint tests ----

    fn make_test_sprint(project_id: &str, number: i32, topic: &str) -> Sprint {
        Sprint {
            id: 0,
            project_id: project_id.to_string(),
            sprint_number: number,
            topic: topic.to_string(),
            questions_json: "[]".to_string(),
            answer_key_json: "{}".to_string(),
            status: "pending".to_string(),
            best_score: None,
            attempts: 0,
            xp_available: 30,
            xp_earned: 0,
            created_at: Utc::now(),
            passed_at: None,
            sprint_id: None,
            source_project_name: None,
        }
    }

    #[tokio::test]
    async fn test_upsert_sprint_insert() {
        let (db, project, _dir) = setup_db_with_project().await;
        let sprint = make_test_sprint(&project.id, 1, "Basics");
        db.upsert_sprint(&sprint).await.unwrap();

        let found = db.get_sprint(&project.id, 1).await.unwrap();
        assert!(found.is_some());
        assert_eq!(found.unwrap().topic, "Basics");
    }

    #[tokio::test]
    async fn test_upsert_sprint_update() {
        let (db, project, _dir) = setup_db_with_project().await;
        let sprint = make_test_sprint(&project.id, 1, "Basics");
        db.upsert_sprint(&sprint).await.unwrap();

        let mut updated = make_test_sprint(&project.id, 1, "Advanced Basics");
        updated.xp_available = 50;
        db.upsert_sprint(&updated).await.unwrap();

        let found = db.get_sprint(&project.id, 1).await.unwrap().unwrap();
        assert_eq!(found.topic, "Advanced Basics");
        assert_eq!(found.xp_available, 50);
    }

    #[tokio::test]
    async fn test_get_sprint_returns_none_if_missing() {
        let (db, project, _dir) = setup_db_with_project().await;
        let found = db.get_sprint(&project.id, 99).await.unwrap();
        assert!(found.is_none());
    }

    #[tokio::test]
    async fn test_get_sprints_returns_multiple() {
        let (db, project, _dir) = setup_db_with_project().await;
        db.upsert_sprint(&make_test_sprint(&project.id, 1, "First"))
            .await
            .unwrap();
        db.upsert_sprint(&make_test_sprint(&project.id, 2, "Second"))
            .await
            .unwrap();
        db.upsert_sprint(&make_test_sprint(&project.id, 3, "Third"))
            .await
            .unwrap();

        let sprints = db.get_sprints(&project.id).await.unwrap();
        assert_eq!(sprints.len(), 3);
        assert_eq!(sprints[0].sprint_number, 1);
        assert_eq!(sprints[2].sprint_number, 3);
    }

    // ---- Attempt recording tests ----

    #[tokio::test]
    async fn test_record_sprint_attempt_logged() {
        let (db, project, _dir) = setup_db_with_project().await;
        db.upsert_sprint(&make_test_sprint(&project.id, 1, "Test"))
            .await
            .unwrap();
        db.record_sprint_attempt(&project.id, 1, 66, true, 20)
            .await
            .unwrap();

        let history = db.get_project_history(&project.id, 10).await.unwrap();
        assert_eq!(history.len(), 1);
        assert_eq!(history[0].score_percent, 66);
        assert!(history[0].passed);
        assert_eq!(history[0].xp_earned, 20);
    }

    #[tokio::test]
    async fn test_record_sprint_attempt_updates_sprint_status() {
        let (db, project, _dir) = setup_db_with_project().await;
        db.upsert_sprint(&make_test_sprint(&project.id, 1, "Test"))
            .await
            .unwrap();
        db.record_sprint_attempt(&project.id, 1, 100, true, 30)
            .await
            .unwrap();

        let sprint = db.get_sprint(&project.id, 1).await.unwrap().unwrap();
        assert_eq!(sprint.status, "passed");
        assert_eq!(sprint.best_score, Some(100));
        assert_eq!(sprint.attempts, 1);
    }

    #[tokio::test]
    async fn test_record_sprint_attempt_fail_keeps_pending() {
        let (db, project, _dir) = setup_db_with_project().await;
        db.upsert_sprint(&make_test_sprint(&project.id, 1, "Test"))
            .await
            .unwrap();
        db.record_sprint_attempt(&project.id, 1, 33, false, 0)
            .await
            .unwrap();

        let sprint = db.get_sprint(&project.id, 1).await.unwrap().unwrap();
        assert_eq!(sprint.status, "pending");
        assert_eq!(sprint.best_score, Some(33));
    }

    // ---- Settings tests ----

    #[tokio::test]
    async fn test_set_and_get_setting_roundtrip() {
        let (db, _dir) = setup_db().await;
        db.set_setting("theme", "dark").await.unwrap();
        let value = db.get_setting("theme").await.unwrap();
        assert_eq!(value, Some("dark".to_string()));
    }

    #[tokio::test]
    async fn test_get_setting_returns_none_for_unknown() {
        let (db, _dir) = setup_db().await;
        let value = db.get_setting("nonexistent_key").await.unwrap();
        assert!(value.is_none());
    }

    #[tokio::test]
    async fn test_set_setting_overwrites() {
        let (db, _dir) = setup_db().await;
        db.set_setting("theme", "light").await.unwrap();
        db.set_setting("theme", "dark").await.unwrap();
        let value = db.get_setting("theme").await.unwrap();
        assert_eq!(value, Some("dark".to_string()));
    }

    #[tokio::test]
    async fn test_is_sound_enabled_default_true() {
        let (db, _dir) = setup_db().await;
        let enabled = db.is_sound_enabled().await.unwrap();
        assert!(enabled);
    }

    // ---- Knowledge Identity tests ----

    #[tokio::test]
    async fn test_get_knowledge_id_after_init() {
        let (db, _dir) = setup_db().await;
        let identity = db.get_knowledge_id().await.unwrap();
        assert_eq!(identity.id, 1);
        assert!(!identity.knowledge_id.is_empty());
    }

    #[tokio::test]
    async fn test_set_display_name_retrievable() {
        let (db, _dir) = setup_db().await;
        db.set_display_name("TestUser").await.unwrap();
        let identity = db.get_knowledge_id().await.unwrap();
        assert_eq!(identity.display_name, Some("TestUser".to_string()));
    }

    // ---- Question stats tests ----

    #[tokio::test]
    async fn test_update_question_stats_correct() {
        let (db, _dir) = setup_db().await;
        db.update_question_stats(true).await.unwrap();
        db.update_question_stats(true).await.unwrap();
        let profile = db.get_profile().await.unwrap();
        assert_eq!(profile.questions_passed, 2);
        assert_eq!(profile.questions_attempted, 2);
        assert_eq!(profile.current_combo, 2);
    }

    #[tokio::test]
    async fn test_update_question_stats_wrong_resets_combo() {
        let (db, _dir) = setup_db().await;
        db.update_question_stats(true).await.unwrap();
        db.update_question_stats(true).await.unwrap();
        db.update_question_stats(false).await.unwrap();
        let profile = db.get_profile().await.unwrap();
        assert_eq!(profile.questions_passed, 2);
        assert_eq!(profile.questions_attempted, 3);
        assert_eq!(profile.current_combo, 0);
        assert_eq!(profile.best_combo, 2);
    }

    // ---- History tests ----

    #[tokio::test]
    async fn test_get_history_returns_across_projects() {
        let (db, _dir) = setup_db().await;
        let p1 = db
            .get_or_create_project("/tmp/p1", "p1")
            .await
            .unwrap();
        let p2 = db
            .get_or_create_project("/tmp/p2", "p2")
            .await
            .unwrap();
        db.upsert_sprint(&make_test_sprint(&p1.id, 1, "T1"))
            .await
            .unwrap();
        db.upsert_sprint(&make_test_sprint(&p2.id, 1, "T2"))
            .await
            .unwrap();
        db.record_sprint_attempt(&p1.id, 1, 80, true, 20)
            .await
            .unwrap();
        db.record_sprint_attempt(&p2.id, 1, 60, true, 15)
            .await
            .unwrap();

        let history = db.get_history(10).await.unwrap();
        assert_eq!(history.len(), 2);
    }
}

// Row types for database queries
#[derive(Debug, Clone, sqlx::FromRow, serde::Serialize, serde::Deserialize)]
pub struct ReviewItemRow {
    pub id: i64,
    pub project_id: String,
    pub sprint_number: i32,
    pub question_number: i32,
    pub question_text: String,
    pub correct_answer: String,
    pub domain: String,
    pub easiness_factor: f64,
    pub repetition_count: i32,
    pub interval_days: i32,
    pub next_review: DateTime<Utc>,
    pub last_reviewed: Option<DateTime<Utc>>,
    pub times_correct: i32,
    pub times_wrong: i32,
    pub streak: i32,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct ReviewStatsRow {
    pub total_items: i64,
    pub due_now: i64,
    pub total_correct: i64,
    pub total_wrong: i64,
    pub avg_easiness: Option<f64>,
    pub max_streak: Option<i32>,
}

#[derive(Debug, Clone, sqlx::FromRow, serde::Serialize, serde::Deserialize)]
pub struct DomainCatalogRow {
    pub id: i64,
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
    pub tags_json: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct DomainCatalogStatsRow {
    pub domain: String,
    pub question_count: i64,
    pub total_attempts: i64,
    pub total_correct: i64,
    pub accuracy: f64,
}

#[derive(Debug, Clone, sqlx::FromRow, serde::Serialize)]
pub struct DifficultyProfileRow {
    pub id: i64,
    pub domain: String,
    pub recall_accuracy: f64,
    pub comprehension_accuracy: f64,
    pub application_accuracy: f64,
    pub analysis_accuracy: f64,
    pub recall_count: i32,
    pub comprehension_count: i32,
    pub application_count: i32,
    pub analysis_count: i32,
    pub recommended_level: i32,
    pub consecutive_correct: i32,
    pub consecutive_wrong: i32,
    pub last_updated: DateTime<Utc>,
}
