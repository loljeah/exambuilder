package db

import (
	"crypto/sha256"
	"encoding/hex"
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
		SELECT id, project_id, sprint_number, topic, questions_json, answer_key_json,
		       status, best_score, attempts, xp_available, xp_earned, created_at, passed_at
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
		if err := rows.Scan(&s.ID, &s.ProjectID, &s.SprintNumber, &s.Topic,
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
		SELECT id, project_id, sprint_number, topic, questions_json, answer_key_json,
		       status, best_score, attempts, xp_available, xp_earned, created_at, passed_at
		FROM sprints WHERE project_id = ? AND sprint_number = ?
	`, projectID, sprintNum)

	s := &Sprint{}
	var createdAt string
	var passedAt *string
	err := row.Scan(&s.ID, &s.ProjectID, &s.SprintNumber, &s.Topic,
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
		INSERT INTO sprints (project_id, sprint_number, topic, questions_json, answer_key_json, xp_available)
		VALUES (?, ?, ?, ?, ?, ?)
		ON CONFLICT(project_id, sprint_number) DO UPDATE SET
			topic = excluded.topic,
			questions_json = excluded.questions_json,
			answer_key_json = excluded.answer_key_json,
			xp_available = excluded.xp_available
	`, s.ProjectID, s.SprintNumber, s.Topic, s.QuestionsJSON, s.AnswerKeyJSON, s.XPAvailable)
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
