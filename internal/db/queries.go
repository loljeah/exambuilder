package db

import (
	"crypto/sha256"
	"encoding/hex"
	"fmt"
	"path/filepath"
	"time"
)

// ProjectHash generates short hash from path
func ProjectHash(path string) string {
	h := sha256.Sum256([]byte(path))
	return hex.EncodeToString(h[:])[:8]
}

// GetOrCreateProject finds or creates a project by path
func (d *DB) GetOrCreateProject(path string) (*Project, error) {
	absPath, err := filepath.Abs(path)
	if err != nil {
		return nil, err
	}

	id := ProjectHash(absPath)
	fullHash := hex.EncodeToString(sha256.New().Sum([]byte(absPath)))
	name := filepath.Base(absPath)

	_, err = d.Exec(`
		INSERT INTO projects (id, full_hash, path, name)
		VALUES (?, ?, ?, ?)
		ON CONFLICT(path) DO UPDATE SET last_active = datetime('now')
	`, id, fullHash, absPath, name)
	if err != nil {
		return nil, err
	}

	return d.GetProject(id)
}

// GetProject retrieves a project by ID
func (d *DB) GetProject(id string) (*Project, error) {
	row := d.QueryRow(`
		SELECT id, full_hash, path, name, created_at, last_active
		FROM projects WHERE id = ?
	`, id)

	p := &Project{}
	var createdAt, lastActive string
	err := row.Scan(&p.ID, &p.FullHash, &p.Path, &p.Name, &createdAt, &lastActive)
	if err != nil {
		return nil, err
	}

	p.CreatedAt, _ = time.Parse("2006-01-02 15:04:05", createdAt)
	p.LastActive, _ = time.Parse("2006-01-02 15:04:05", lastActive)
	return p, nil
}

// DeleteProject removes a project and all related data
func (d *DB) DeleteProject(projectID string) error {
	tx, err := d.Begin()
	if err != nil {
		return err
	}
	defer tx.Rollback()

	// Delete in order to respect foreign keys
	tables := []string{
		"DELETE FROM domain_achievements WHERE project_id = ?",
		"DELETE FROM domain_levels WHERE project_id = ?",
		"DELETE FROM domains WHERE project_id = ?",
		"DELETE FROM attempts WHERE sprint_id IN (SELECT id FROM sprints WHERE project_id = ?)",
		"DELETE FROM question_stats WHERE sprint_id IN (SELECT id FROM sprints WHERE project_id = ?)",
		"DELETE FROM study_notes WHERE sprint_id IN (SELECT id FROM sprints WHERE project_id = ?)",
		"DELETE FROM study_notes WHERE knowledge_item_id IN (SELECT id FROM knowledge_items WHERE project_id = ?)",
		"DELETE FROM study_notes WHERE project_id = ?",
		"DELETE FROM journal WHERE sprint_id IN (SELECT id FROM sprints WHERE project_id = ?)",
		"DELETE FROM journal WHERE project_id = ?",
		"DELETE FROM knowledge_items WHERE project_id = ?",
		"DELETE FROM sprints WHERE project_id = ?",
		"DELETE FROM debt_log WHERE project_id = ?",
		"DELETE FROM debt_current WHERE project_id = ?",
		"DELETE FROM badges WHERE project_id = ?",
		"DELETE FROM tagged_items WHERE item_type = 'project' AND item_id = ?",
		"DELETE FROM projects WHERE id = ?",
	}

	for _, query := range tables {
		if _, err := tx.Exec(query, projectID); err != nil {
			// Ignore errors for tables that might not exist
			continue
		}
	}

	return tx.Commit()
}

// ListProjects returns all projects
func (d *DB) ListProjects() ([]*Project, error) {
	rows, err := d.Query(`
		SELECT id, full_hash, path, name, created_at, last_active
		FROM projects ORDER BY last_active DESC
	`)
	if err != nil {
		return nil, err
	}
	defer rows.Close()

	var projects []*Project
	for rows.Next() {
		p := &Project{}
		var createdAt, lastActive string
		if err := rows.Scan(&p.ID, &p.FullHash, &p.Path, &p.Name, &createdAt, &lastActive); err != nil {
			return nil, err
		}
		p.CreatedAt, _ = time.Parse("2006-01-02 15:04:05", createdAt)
		p.LastActive, _ = time.Parse("2006-01-02 15:04:05", lastActive)
		projects = append(projects, p)
	}
	return projects, nil
}

// GetDebt returns current debt for a project
func (d *DB) GetDebt(projectID string) (int, error) {
	var total int
	err := d.QueryRow("SELECT COALESCE(total, 0) FROM debt_current WHERE project_id = ?", projectID).Scan(&total)
	if err != nil {
		return 0, nil // No debt record = 0 debt
	}
	return total, nil
}

// AddDebt adds debt and logs it
func (d *DB) AddDebt(projectID, action string, weight int, description string) error {
	tx, err := d.Begin()
	if err != nil {
		return err
	}
	defer tx.Rollback()

	// Log the debt entry
	_, err = tx.Exec(`
		INSERT INTO debt_log (project_id, action, weight, description)
		VALUES (?, ?, ?, ?)
	`, projectID, action, weight, description)
	if err != nil {
		return err
	}

	// Update current debt
	_, err = tx.Exec(`
		INSERT INTO debt_current (project_id, total)
		VALUES (?, ?)
		ON CONFLICT(project_id) DO UPDATE SET
			total = total + ?,
			last_updated = datetime('now')
	`, projectID, weight, weight)
	if err != nil {
		return err
	}

	if err := tx.Commit(); err != nil {
		return err
	}

	// Log to journal
	d.LogEvent(EventDebtAdded, &projectID, nil, map[string]interface{}{
		"action":      action,
		"weight":      weight,
		"description": description,
	})

	return nil
}

// ClearDebt reduces debt by amount
func (d *DB) ClearDebt(projectID string, amount int) error {
	_, err := d.Exec(`
		UPDATE debt_current
		SET total = MAX(0, total - ?), last_updated = datetime('now')
		WHERE project_id = ?
	`, amount, projectID)
	if err != nil {
		return err
	}

	// Log to journal
	d.LogEvent(EventDebtCleared, &projectID, nil, map[string]interface{}{
		"amount": amount,
	})

	return nil
}

// GetProfile returns the global profile
func (d *DB) GetProfile() (*Profile, error) {
	row := d.QueryRow(`
		SELECT total_xp, level, current_streak, best_streak, sprints_passed, last_activity
		FROM profile WHERE id = 1
	`)

	p := &Profile{}
	var lastActivity *string
	err := row.Scan(&p.TotalXP, &p.Level, &p.CurrentStreak, &p.BestStreak, &p.SprintsPassed, &lastActivity)
	if err != nil {
		return nil, err
	}

	if lastActivity != nil {
		t, _ := time.Parse("2006-01-02 15:04:05", *lastActivity)
		p.LastActivity = &t
	}
	return p, nil
}

// UpdateProfile updates XP, level, and streak
func (d *DB) UpdateProfile(xpDelta, streakDelta int, sprintPassed bool) error {
	// Get current state for detecting level ups and streak changes
	oldProfile, _ := d.GetProfile()

	_, err := d.Exec(`
		UPDATE profile SET
			total_xp = total_xp + ?,
			level = 1 + (total_xp + ?) / 100,
			current_streak = CASE
				WHEN ? > 0 THEN current_streak + ?
				WHEN ? < 0 THEN 0
				ELSE current_streak
			END,
			best_streak = MAX(best_streak, current_streak + ?),
			sprints_passed = sprints_passed + CASE WHEN ? THEN 1 ELSE 0 END,
			last_activity = datetime('now')
		WHERE id = 1
	`, xpDelta, xpDelta, streakDelta, streakDelta, streakDelta, streakDelta, sprintPassed)
	if err != nil {
		return err
	}

	// Log events for level ups and streak changes
	if oldProfile != nil {
		newProfile, _ := d.GetProfile()
		if newProfile != nil {
			// Check for level up
			if newProfile.Level > oldProfile.Level {
				d.LogEvent(EventLevelUp, nil, nil, map[string]interface{}{
					"old_level": oldProfile.Level,
					"new_level": newProfile.Level,
					"total_xp":  newProfile.TotalXP,
				})
			}

			// Log streak update
			if streakDelta > 0 {
				d.LogEvent(EventStreakUpdated, nil, nil, map[string]interface{}{
					"streak":      newProfile.CurrentStreak,
					"best_streak": newProfile.BestStreak,
				})
			} else if streakDelta < 0 && oldProfile.CurrentStreak > 0 {
				d.LogEvent(EventStreakBroken, nil, nil, map[string]interface{}{
					"lost_streak": oldProfile.CurrentStreak,
				})
			}
		}
	}

	return nil
}

// GetSprints returns all sprints for a project
func (d *DB) GetSprints(projectID string) ([]*Sprint, error) {
	rows, err := d.Query(`
		SELECT id, project_id, sprint_number, topic, COALESCE(domain_id, ''), COALESCE(subdomain_id, ''),
		       questions_json, answer_key_json, status, best_score, attempts,
		       xp_available, xp_earned, created_at, passed_at
		FROM sprints WHERE project_id = ? ORDER BY sprint_number
	`, projectID)
	if err != nil {
		return nil, err
	}
	defer rows.Close()

	var sprints []*Sprint
	for rows.Next() {
		s := &Sprint{}
		var createdAt string
		var passedAt *string
		if err := rows.Scan(&s.ID, &s.ProjectID, &s.SprintNumber, &s.Topic, &s.DomainID, &s.SubdomainID,
			&s.QuestionsJSON, &s.AnswerKeyJSON, &s.Status, &s.BestScore,
			&s.Attempts, &s.XPAvailable, &s.XPEarned, &createdAt, &passedAt); err != nil {
			return nil, err
		}
		s.CreatedAt, _ = time.Parse("2006-01-02 15:04:05", createdAt)
		if passedAt != nil {
			t, _ := time.Parse("2006-01-02 15:04:05", *passedAt)
			s.PassedAt = &t
		}
		sprints = append(sprints, s)
	}
	return sprints, nil
}

// GetSprint returns a specific sprint
func (d *DB) GetSprint(projectID string, sprintNum int) (*Sprint, error) {
	row := d.QueryRow(`
		SELECT id, project_id, sprint_number, topic, COALESCE(domain_id, ''), COALESCE(subdomain_id, ''),
		       questions_json, answer_key_json, status, best_score, attempts,
		       xp_available, xp_earned, created_at, passed_at
		FROM sprints WHERE project_id = ? AND sprint_number = ?
	`, projectID, sprintNum)

	s := &Sprint{}
	var createdAt string
	var passedAt *string
	err := row.Scan(&s.ID, &s.ProjectID, &s.SprintNumber, &s.Topic, &s.DomainID, &s.SubdomainID,
		&s.QuestionsJSON, &s.AnswerKeyJSON, &s.Status, &s.BestScore,
		&s.Attempts, &s.XPAvailable, &s.XPEarned, &createdAt, &passedAt)
	if err != nil {
		return nil, err
	}
	s.CreatedAt, _ = time.Parse("2006-01-02 15:04:05", createdAt)
	if passedAt != nil {
		t, _ := time.Parse("2006-01-02 15:04:05", *passedAt)
		s.PassedAt = &t
	}
	return s, nil
}

// UpsertSprint creates or updates a sprint
func (d *DB) UpsertSprint(s *Sprint) error {
	_, err := d.Exec(`
		INSERT INTO sprints (project_id, sprint_number, topic, domain_id, subdomain_id, questions_json, answer_key_json, xp_available)
		VALUES (?, ?, ?, ?, ?, ?, ?, ?)
		ON CONFLICT(project_id, sprint_number) DO UPDATE SET
			topic = excluded.topic,
			domain_id = excluded.domain_id,
			subdomain_id = excluded.subdomain_id,
			questions_json = excluded.questions_json,
			answer_key_json = excluded.answer_key_json,
			xp_available = excluded.xp_available
	`, s.ProjectID, s.SprintNumber, s.Topic, s.DomainID, s.SubdomainID, s.QuestionsJSON, s.AnswerKeyJSON, s.XPAvailable)
	return err
}

// RecordAttempt records an exam attempt
func (d *DB) RecordAttempt(sprintID int64, answersJSON string, score int, passed bool, xpEarned int) error {
	tx, err := d.Begin()
	if err != nil {
		return err
	}
	defer tx.Rollback()

	// Insert attempt
	_, err = tx.Exec(`
		INSERT INTO attempts (sprint_id, answers_json, score, passed, xp_earned)
		VALUES (?, ?, ?, ?, ?)
	`, sprintID, answersJSON, score, passed, xpEarned)
	if err != nil {
		return err
	}

	// Update sprint
	if passed {
		_, err = tx.Exec(`
			UPDATE sprints SET
				status = 'passed',
				best_score = MAX(COALESCE(best_score, 0), ?),
				attempts = attempts + 1,
				xp_earned = xp_earned + ?,
				passed_at = datetime('now')
			WHERE id = ?
		`, score, xpEarned, sprintID)
	} else {
		_, err = tx.Exec(`
			UPDATE sprints SET
				best_score = MAX(COALESCE(best_score, 0), ?),
				attempts = attempts + 1
			WHERE id = ?
		`, score, sprintID)
	}
	if err != nil {
		return err
	}

	return tx.Commit()
}

// UpdateSprintAttempt updates sprint after an attempt
func (d *DB) UpdateSprintAttempt(sprintID int64, status string, bestScore *int, answersJSON string) error {
	if bestScore != nil {
		_, err := d.Exec(`
			UPDATE sprints SET
				status = ?,
				best_score = ?,
				attempts = attempts + 1,
				passed_at = CASE WHEN ? = 'passed' THEN datetime('now') ELSE passed_at END
			WHERE id = ?
		`, status, *bestScore, status, sprintID)
		return err
	}

	_, err := d.Exec(`
		UPDATE sprints SET
			status = ?,
			attempts = attempts + 1,
			passed_at = CASE WHEN ? = 'passed' THEN datetime('now') ELSE passed_at END
		WHERE id = ?
	`, status, status, sprintID)
	return err
}

// AddXP adds XP to the global profile
func (d *DB) AddXP(amount int) error {
	_, err := d.Exec(`
		UPDATE profile SET
			total_xp = total_xp + ?,
			level = 1 + (total_xp + ?) / 100,
			last_activity = datetime('now')
		WHERE id = 1
	`, amount, amount)
	return err
}

// RecordSprintAttempt records sprint attempt in daily stats
func (d *DB) RecordSprintAttempt(passed bool, correctCount, totalCount, xpEarned int) {
	today := time.Now().Format("2006-01-02")

	// Ensure row exists
	d.Exec(`
		INSERT INTO daily_stats (date, first_activity, last_activity)
		VALUES (?, datetime('now'), datetime('now'))
		ON CONFLICT(date) DO UPDATE SET last_activity = datetime('now')
	`, today)

	// Update stats
	d.Exec(`
		UPDATE daily_stats SET
			sprints_attempted = sprints_attempted + 1,
			questions_answered = questions_answered + ?,
			questions_correct = questions_correct + ?
		WHERE date = ?
	`, totalCount, correctCount, today)

	if passed {
		d.Exec(`
			UPDATE daily_stats SET
				sprints_passed = sprints_passed + 1,
				xp_earned = xp_earned + ?
			WHERE date = ?
		`, xpEarned, today)
	}
}

// ============================================================================
// SETTINGS
// ============================================================================

// GetSetting retrieves a setting by key
func (d *DB) GetSetting(key string) (string, error) {
	var value string
	err := d.QueryRow(`SELECT value FROM settings WHERE key = ?`, key).Scan(&value)
	if err != nil {
		return "", err
	}
	return value, nil
}

// SetSetting stores a setting
func (d *DB) SetSetting(key, value string) error {
	_, err := d.Exec(`
		INSERT INTO settings (key, value, updated_at)
		VALUES (?, ?, datetime('now'))
		ON CONFLICT(key) DO UPDATE SET value = ?, updated_at = datetime('now')
	`, key, value, value)
	return err
}

// GetSettingBool retrieves a boolean setting (stored as "true"/"false")
func (d *DB) GetSettingBool(key string, defaultVal bool) bool {
	val, err := d.GetSetting(key)
	if err != nil {
		return defaultVal
	}
	return val == "true" || val == "1"
}

// SetSettingBool stores a boolean setting
func (d *DB) SetSettingBool(key string, value bool) error {
	v := "false"
	if value {
		v = "true"
	}
	return d.SetSetting(key, v)
}

// ============================================================================
// DOMAINS
// ============================================================================

// GetDomains returns all domains for a project
func (d *DB) GetDomains(projectID string) ([]Domain, error) {
	rows, err := d.Query(`
		SELECT id, project_id, domain_id, name, description, color, icon,
		       total_xp, earned_xp, level, sprints_total, sprints_passed, sprints_perfect,
		       created_at, updated_at
		FROM domains
		WHERE project_id = ?
		ORDER BY name
	`, projectID)
	if err != nil {
		return nil, err
	}
	defer rows.Close()

	var domains []Domain
	for rows.Next() {
		var dom Domain
		var createdAt, updatedAt string
		err := rows.Scan(&dom.ID, &dom.ProjectID, &dom.DomainID, &dom.Name, &dom.Description,
			&dom.Color, &dom.Icon, &dom.TotalXP, &dom.EarnedXP, &dom.Level,
			&dom.SprintsTotal, &dom.SprintsPassed, &dom.SprintsPerfect,
			&createdAt, &updatedAt)
		if err != nil {
			return nil, err
		}
		dom.CreatedAt, _ = time.Parse("2006-01-02 15:04:05", createdAt)
		dom.UpdatedAt, _ = time.Parse("2006-01-02 15:04:05", updatedAt)
		domains = append(domains, dom)
	}
	return domains, nil
}

// GetDomainLevels returns level definitions for a domain
func (d *DB) GetDomainLevels(projectID, domainID string) ([]DomainLevel, error) {
	rows, err := d.Query(`
		SELECT id, project_id, domain_id, level, xp_threshold, title
		FROM domain_levels
		WHERE project_id = ? AND domain_id = ?
		ORDER BY level
	`, projectID, domainID)
	if err != nil {
		return nil, err
	}
	defer rows.Close()

	var levels []DomainLevel
	for rows.Next() {
		var lvl DomainLevel
		err := rows.Scan(&lvl.ID, &lvl.ProjectID, &lvl.DomainID, &lvl.Level, &lvl.XPThreshold, &lvl.Title)
		if err != nil {
			return nil, err
		}
		levels = append(levels, lvl)
	}
	return levels, nil
}

// GetDomainAchievements returns achievements for a domain
func (d *DB) GetDomainAchievements(projectID, domainID string) ([]DomainAchievement, error) {
	rows, err := d.Query(`
		SELECT id, project_id, domain_id, name, description, condition, xp_reward, icon, unlocked, unlocked_at
		FROM domain_achievements
		WHERE project_id = ? AND domain_id = ?
		ORDER BY xp_reward
	`, projectID, domainID)
	if err != nil {
		return nil, err
	}
	defer rows.Close()

	var achievements []DomainAchievement
	for rows.Next() {
		var ach DomainAchievement
		err := rows.Scan(&ach.ID, &ach.ProjectID, &ach.DomainID, &ach.Name, &ach.Description,
			&ach.Condition, &ach.XPReward, &ach.Icon, &ach.Unlocked, &ach.UnlockedAt)
		if err != nil {
			return nil, err
		}
		achievements = append(achievements, ach)
	}
	return achievements, nil
}

// UpsertDomain inserts or updates a domain
func (d *DB) UpsertDomain(dom *Domain) error {
	_, err := d.Exec(`
		INSERT INTO domains (id, project_id, domain_id, name, description, color, icon, total_xp, sprints_total)
		VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
		ON CONFLICT(project_id, domain_id) DO UPDATE SET
			name = excluded.name,
			description = excluded.description,
			color = excluded.color,
			icon = excluded.icon,
			total_xp = excluded.total_xp,
			sprints_total = excluded.sprints_total,
			updated_at = datetime('now')
	`, dom.ID, dom.ProjectID, dom.DomainID, dom.Name, dom.Description, dom.Color, dom.Icon, dom.TotalXP, dom.SprintsTotal)
	return err
}

// UpsertDomainLevel inserts or updates a domain level
func (d *DB) UpsertDomainLevel(lvl *DomainLevel) error {
	_, err := d.Exec(`
		INSERT INTO domain_levels (project_id, domain_id, level, xp_threshold, title)
		VALUES (?, ?, ?, ?, ?)
		ON CONFLICT(project_id, domain_id, level) DO UPDATE SET
			xp_threshold = excluded.xp_threshold,
			title = excluded.title
	`, lvl.ProjectID, lvl.DomainID, lvl.Level, lvl.XPThreshold, lvl.Title)
	return err
}

// UpsertDomainAchievement inserts or updates a domain achievement
func (d *DB) UpsertDomainAchievement(ach *DomainAchievement) error {
	_, err := d.Exec(`
		INSERT INTO domain_achievements (id, project_id, domain_id, name, description, condition, xp_reward, icon)
		VALUES (?, ?, ?, ?, ?, ?, ?, ?)
		ON CONFLICT(project_id, domain_id, name) DO UPDATE SET
			description = excluded.description,
			condition = excluded.condition,
			xp_reward = excluded.xp_reward,
			icon = excluded.icon
	`, ach.ID, ach.ProjectID, ach.DomainID, ach.Name, ach.Description, ach.Condition, ach.XPReward, ach.Icon)
	return err
}

// AddDomainXP adds XP to a domain and updates level, returns level-up info
func (d *DB) AddDomainXP(projectID, domainID string, xp int) (*LevelUpResult, error) {
	// Get old level (default to 1 if not found)
	var oldLevel int = 1
	_ = d.QueryRow(`SELECT level FROM domains WHERE project_id = ? AND domain_id = ?`, projectID, domainID).Scan(&oldLevel)

	// Add XP
	_, err := d.Exec(`
		UPDATE domains SET
			earned_xp = earned_xp + ?,
			updated_at = datetime('now')
		WHERE project_id = ? AND domain_id = ?
	`, xp, projectID, domainID)
	if err != nil {
		return nil, err
	}

	// Get current XP and calculate new level
	var earnedXP int
	err = d.QueryRow(`SELECT earned_xp FROM domains WHERE project_id = ? AND domain_id = ?`, projectID, domainID).Scan(&earnedXP)
	if err != nil {
		return nil, err
	}

	// Find highest level achieved (no max level - infinite scaling)
	var newLevel int
	err = d.QueryRow(`
		SELECT COALESCE(MAX(level), 1)
		FROM domain_levels
		WHERE project_id = ? AND domain_id = ? AND xp_threshold <= ?
	`, projectID, domainID, earnedXP).Scan(&newLevel)
	if err != nil {
		newLevel = 1
	}

	// Update level
	_, err = d.Exec(`UPDATE domains SET level = ? WHERE project_id = ? AND domain_id = ?`, newLevel, projectID, domainID)
	if err != nil {
		return nil, err
	}

	// Get level title
	newTitle := "Novice"
	d.QueryRow(`SELECT title FROM domain_levels WHERE project_id = ? AND domain_id = ? AND level = ?`,
		projectID, domainID, newLevel).Scan(&newTitle)

	return &LevelUpResult{
		OldLevel:  oldLevel,
		NewLevel:  newLevel,
		LeveledUp: newLevel > oldLevel,
		NewTitle:  newTitle,
	}, nil
}

// RecordDomainSprintComplete updates domain stats after sprint completion
func (d *DB) RecordDomainSprintComplete(projectID, domainID string, passed bool, perfect bool) error {
	if passed {
		_, err := d.Exec(`
			UPDATE domains SET
				sprints_passed = sprints_passed + 1,
				updated_at = datetime('now')
			WHERE project_id = ? AND domain_id = ?
		`, projectID, domainID)
		if err != nil {
			return err
		}
	}

	if perfect {
		_, err := d.Exec(`
			UPDATE domains SET
				sprints_perfect = sprints_perfect + 1
			WHERE project_id = ? AND domain_id = ?
		`, projectID, domainID)
		if err != nil {
			return err
		}
	}

	return nil
}

// GetDomainByID returns a single domain
func (d *DB) GetDomainByID(projectID, domainID string) (*Domain, error) {
	row := d.QueryRow(`
		SELECT id, project_id, domain_id, name, description, color, icon,
		       total_xp, earned_xp, level, sprints_total, sprints_passed, sprints_perfect,
		       created_at, updated_at
		FROM domains
		WHERE project_id = ? AND domain_id = ?
	`, projectID, domainID)

	var dom Domain
	var createdAt, updatedAt string
	err := row.Scan(&dom.ID, &dom.ProjectID, &dom.DomainID, &dom.Name, &dom.Description,
		&dom.Color, &dom.Icon, &dom.TotalXP, &dom.EarnedXP, &dom.Level,
		&dom.SprintsTotal, &dom.SprintsPassed, &dom.SprintsPerfect,
		&createdAt, &updatedAt)
	if err != nil {
		return nil, err
	}
	dom.CreatedAt, _ = time.Parse("2006-01-02 15:04:05", createdAt)
	dom.UpdatedAt, _ = time.Parse("2006-01-02 15:04:05", updatedAt)
	return &dom, nil
}

// EvaluateAchievements checks and unlocks achievements for a domain
func (d *DB) EvaluateAchievements(projectID, domainID string) ([]DomainAchievement, error) {
	// Get domain stats
	domain, err := d.GetDomainByID(projectID, domainID)
	if err != nil {
		return nil, err
	}

	// Get all locked achievements for this domain
	achievements, err := d.GetDomainAchievements(projectID, domainID)
	if err != nil {
		return nil, err
	}

	var unlocked []DomainAchievement
	for _, ach := range achievements {
		if ach.Unlocked {
			continue
		}

		// Parse and evaluate condition
		if evaluateCondition(ach.Condition, domain) {
			// Unlock achievement
			if err := d.UnlockAchievement(ach.ID); err == nil {
				ach.Unlocked = true
				unlocked = append(unlocked, ach)
			}
		}
	}
	return unlocked, nil
}

// UnlockAchievement marks an achievement as unlocked
func (d *DB) UnlockAchievement(achievementID string) error {
	_, err := d.Exec(`
		UPDATE domain_achievements
		SET unlocked = 1, unlocked_at = datetime('now')
		WHERE id = ?
	`, achievementID)
	return err
}

// evaluateCondition checks if a domain meets achievement conditions
// Supports conditions like: "domain_sprints_passed >= 1", "domain_level >= 3", "all_sprints_passed"
func evaluateCondition(condition string, domain *Domain) bool {
	// Parse simple conditions
	switch {
	case condition == "all_sprints_passed":
		return domain.SprintsPassed >= domain.SprintsTotal && domain.SprintsTotal > 0
	case condition == "first_sprint_passed":
		return domain.SprintsPassed >= 1
	case condition == "perfect_sprint":
		return domain.SprintsPerfect >= 1
	case condition == "all_sprints_perfect":
		return domain.SprintsPerfect >= domain.SprintsTotal && domain.SprintsTotal > 0
	}

	// Parse numeric conditions like "domain_sprints_passed >= 1"
	var field string
	var op string
	var value int
	_, err := fmt.Sscanf(condition, "%s %s %d", &field, &op, &value)
	if err != nil {
		return false
	}

	var fieldValue int
	switch field {
	case "domain_sprints_passed":
		fieldValue = domain.SprintsPassed
	case "domain_sprints_perfect":
		fieldValue = domain.SprintsPerfect
	case "domain_level":
		fieldValue = domain.Level
	case "domain_xp", "earned_xp":
		fieldValue = domain.EarnedXP
	default:
		return false
	}

	switch op {
	case ">=":
		return fieldValue >= value
	case ">":
		return fieldValue > value
	case "==", "=":
		return fieldValue == value
	case "<=":
		return fieldValue <= value
	case "<":
		return fieldValue < value
	}

	return false
}
