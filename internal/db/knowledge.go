package db

import (
	"crypto/sha256"
	"encoding/hex"
	"encoding/json"
	"time"
)

// ============================================================================
// KNOWLEDGE ITEMS
// ============================================================================

// GetOrCreateKnowledgeItem finds or creates a knowledge item
func (d *DB) GetOrCreateKnowledgeItem(projectID, concept, category, tier string, sprintID *int64, questionNum *int) (*KnowledgeItem, error) {
	// Try to get existing
	row := d.QueryRow(`
		SELECT id, project_id, sprint_id, question_number, concept, category, tier,
		       status, times_seen, times_correct, times_incorrect, mastery_score,
		       next_review, ease_factor, interval_days,
		       first_seen, last_seen, last_correct, mastered_at
		FROM knowledge_items
		WHERE project_id = ? AND concept = ?
	`, projectID, concept)

	k := &KnowledgeItem{}
	var nextReview, firstSeen, lastSeen, lastCorrect, masteredAt *string

	err := row.Scan(
		&k.ID, &k.ProjectID, &k.SprintID, &k.QuestionNumber, &k.Concept, &k.Category, &k.Tier,
		&k.Status, &k.TimesSeen, &k.TimesCorrect, &k.TimesIncorrect, &k.MasteryScore,
		&nextReview, &k.EaseFactor, &k.IntervalDays,
		&firstSeen, &lastSeen, &lastCorrect, &masteredAt,
	)

	if err == nil {
		// Parse times
		if nextReview != nil {
			t, _ := time.Parse("2006-01-02 15:04:05", *nextReview)
			k.NextReview = &t
		}
		if firstSeen != nil {
			t, _ := time.Parse("2006-01-02 15:04:05", *firstSeen)
			k.FirstSeen = &t
		}
		if lastSeen != nil {
			t, _ := time.Parse("2006-01-02 15:04:05", *lastSeen)
			k.LastSeen = &t
		}
		if lastCorrect != nil {
			t, _ := time.Parse("2006-01-02 15:04:05", *lastCorrect)
			k.LastCorrect = &t
		}
		if masteredAt != nil {
			t, _ := time.Parse("2006-01-02 15:04:05", *masteredAt)
			k.MasteredAt = &t
		}
		return k, nil
	}

	// Create new
	result, err := d.Exec(`
		INSERT INTO knowledge_items (project_id, sprint_id, question_number, concept, category, tier, first_seen)
		VALUES (?, ?, ?, ?, ?, ?, datetime('now'))
	`, projectID, sprintID, questionNum, concept, category, tier)
	if err != nil {
		return nil, err
	}

	id, _ := result.LastInsertId()
	return d.GetKnowledgeItem(id)
}

// GetKnowledgeItem retrieves a knowledge item by ID
func (d *DB) GetKnowledgeItem(id int64) (*KnowledgeItem, error) {
	row := d.QueryRow(`
		SELECT id, project_id, sprint_id, question_number, concept, category, tier,
		       status, times_seen, times_correct, times_incorrect, mastery_score,
		       next_review, ease_factor, interval_days,
		       first_seen, last_seen, last_correct, mastered_at
		FROM knowledge_items WHERE id = ?
	`, id)

	k := &KnowledgeItem{}
	var nextReview, firstSeen, lastSeen, lastCorrect, masteredAt *string

	err := row.Scan(
		&k.ID, &k.ProjectID, &k.SprintID, &k.QuestionNumber, &k.Concept, &k.Category, &k.Tier,
		&k.Status, &k.TimesSeen, &k.TimesCorrect, &k.TimesIncorrect, &k.MasteryScore,
		&nextReview, &k.EaseFactor, &k.IntervalDays,
		&firstSeen, &lastSeen, &lastCorrect, &masteredAt,
	)
	if err != nil {
		return nil, err
	}

	// Parse times
	if nextReview != nil {
		t, _ := time.Parse("2006-01-02 15:04:05", *nextReview)
		k.NextReview = &t
	}
	if firstSeen != nil {
		t, _ := time.Parse("2006-01-02 15:04:05", *firstSeen)
		k.FirstSeen = &t
	}
	if lastSeen != nil {
		t, _ := time.Parse("2006-01-02 15:04:05", *lastSeen)
		k.LastSeen = &t
	}
	if lastCorrect != nil {
		t, _ := time.Parse("2006-01-02 15:04:05", *lastCorrect)
		k.LastCorrect = &t
	}
	if masteredAt != nil {
		t, _ := time.Parse("2006-01-02 15:04:05", *masteredAt)
		k.MasteredAt = &t
	}

	return k, nil
}

// RecordKnowledgeAttempt records a correct/incorrect attempt on a knowledge item
func (d *DB) RecordKnowledgeAttempt(id int64, correct bool) error {
	// Update counters
	if correct {
		_, err := d.Exec(`
			UPDATE knowledge_items SET
				times_seen = times_seen + 1,
				times_correct = times_correct + 1,
				last_seen = datetime('now'),
				last_correct = datetime('now')
			WHERE id = ?
		`, id)
		if err != nil {
			return err
		}
	} else {
		_, err := d.Exec(`
			UPDATE knowledge_items SET
				times_seen = times_seen + 1,
				times_incorrect = times_incorrect + 1,
				last_seen = datetime('now')
			WHERE id = ?
		`, id)
		if err != nil {
			return err
		}
	}

	// Update mastery score and status
	return d.updateKnowledgeMastery(id)
}

// updateKnowledgeMastery recalculates mastery score and updates status
func (d *DB) updateKnowledgeMastery(id int64) error {
	k, err := d.GetKnowledgeItem(id)
	if err != nil {
		return err
	}

	// Calculate mastery score (accuracy weighted by recency)
	if k.TimesSeen == 0 {
		return nil
	}

	accuracy := float64(k.TimesCorrect) / float64(k.TimesSeen)

	// Recency factor (decays if not seen recently)
	recencyFactor := 1.0
	if k.LastSeen != nil {
		daysSince := time.Since(*k.LastSeen).Hours() / 24
		if daysSince > 30 {
			recencyFactor = 0.5
		} else if daysSince > 7 {
			recencyFactor = 0.8
		}
	}

	masteryScore := accuracy * recencyFactor

	// Determine status
	status := KnowledgeStatusLearning
	if k.TimesSeen < 2 {
		status = KnowledgeStatusUnseen
	} else if masteryScore >= 0.8 && k.TimesCorrect >= 3 {
		status = KnowledgeStatusMastered
	}

	// Update spaced repetition (SM-2 algorithm simplified)
	easeFactor := k.EaseFactor
	intervalDays := k.IntervalDays

	if k.TimesCorrect > 0 && k.TimesIncorrect == 0 {
		// Perfect record - increase interval
		intervalDays = int(float64(intervalDays) * easeFactor)
		easeFactor = min(2.5, easeFactor+0.1)
	} else if float64(k.TimesIncorrect)/float64(k.TimesSeen) > 0.3 {
		// Too many mistakes - reset
		intervalDays = 1
		easeFactor = max(1.3, easeFactor-0.2)
	}

	nextReview := time.Now().AddDate(0, 0, intervalDays)

	// Check if newly mastered
	newlyMastered := status == KnowledgeStatusMastered && k.Status != KnowledgeStatusMastered

	// Update using CASE to avoid SQL string concatenation
	_, err = d.Exec(`
		UPDATE knowledge_items SET
			mastery_score = ?,
			status = ?,
			ease_factor = ?,
			interval_days = ?,
			next_review = ?,
			mastered_at = CASE
				WHEN ? AND mastered_at IS NULL THEN datetime('now')
				ELSE mastered_at
			END
		WHERE id = ?
	`, masteryScore, status, easeFactor, intervalDays, nextReview.Format("2006-01-02 15:04:05"), newlyMastered, id)

	return err
}

// GetKnowledgeItemsForReview returns items due for review
func (d *DB) GetKnowledgeItemsForReview(projectID string, limit int) ([]*KnowledgeItem, error) {
	rows, err := d.Query(`
		SELECT id, project_id, concept, category, tier, status, mastery_score
		FROM knowledge_items
		WHERE project_id = ? AND next_review <= datetime('now')
		ORDER BY next_review ASC
		LIMIT ?
	`, projectID, limit)
	if err != nil {
		return nil, err
	}
	defer rows.Close()

	var items []*KnowledgeItem
	for rows.Next() {
		k := &KnowledgeItem{}
		if err := rows.Scan(&k.ID, &k.ProjectID, &k.Concept, &k.Category, &k.Tier, &k.Status, &k.MasteryScore); err != nil {
			return nil, err
		}
		items = append(items, k)
	}

	return items, nil
}

// GetKnowledgeStats returns aggregate knowledge stats for a project
func (d *DB) GetKnowledgeStats(projectID string) (map[string]int, error) {
	stats := map[string]int{
		"total":    0,
		"unseen":   0,
		"learning": 0,
		"mastered": 0,
	}

	rows, err := d.Query(`
		SELECT status, COUNT(*) FROM knowledge_items
		WHERE project_id = ?
		GROUP BY status
	`, projectID)
	if err != nil {
		return stats, err
	}
	defer rows.Close()

	for rows.Next() {
		var status string
		var count int
		if err := rows.Scan(&status, &count); err != nil {
			return stats, err
		}
		stats[status] = count
		stats["total"] += count
	}

	return stats, nil
}

// ============================================================================
// QUESTION STATS
// ============================================================================

// QuestionHash generates a hash of question text for deduplication
func QuestionHash(text string) string {
	h := sha256.Sum256([]byte(text))
	return hex.EncodeToString(h[:])[:16]
}

// RecordQuestionAttempt records stats for a question attempt
func (d *DB) RecordQuestionAttempt(sprintID int64, questionNum int, questionText string, correct bool, responseTimeMs int, userAnswer string) error {
	hash := QuestionHash(questionText)

	// Ensure row exists
	_, err := d.Exec(`
		INSERT INTO question_stats (sprint_id, question_number, question_hash, first_shown)
		VALUES (?, ?, ?, datetime('now'))
		ON CONFLICT(sprint_id, question_number) DO NOTHING
	`, sprintID, questionNum, hash)
	if err != nil {
		return err
	}

	// Update stats
	if correct {
		_, err = d.Exec(`
			UPDATE question_stats SET
				times_shown = times_shown + 1,
				times_correct = times_correct + 1,
				last_shown = datetime('now'),
				last_correct = datetime('now')
			WHERE sprint_id = ? AND question_number = ?
		`, sprintID, questionNum)
	} else {
		// Get current wrong answers
		var wrongJSON *string
		d.QueryRow(`SELECT wrong_answers_json FROM question_stats WHERE sprint_id = ? AND question_number = ?`,
			sprintID, questionNum).Scan(&wrongJSON)

		wrongAnswers := map[string]int{}
		if wrongJSON != nil {
			json.Unmarshal([]byte(*wrongJSON), &wrongAnswers)
		}
		wrongAnswers[userAnswer]++
		newWrongJSON, _ := json.Marshal(wrongAnswers)

		_, err = d.Exec(`
			UPDATE question_stats SET
				times_shown = times_shown + 1,
				times_incorrect = times_incorrect + 1,
				last_shown = datetime('now'),
				wrong_answers_json = ?
			WHERE sprint_id = ? AND question_number = ?
		`, string(newWrongJSON), sprintID, questionNum)
	}

	// Update response time stats
	if responseTimeMs > 0 {
		d.Exec(`
			UPDATE question_stats SET
				avg_response_time_ms = COALESCE((avg_response_time_ms * (times_shown - 1) + ?) / times_shown, ?),
				fastest_response_ms = CASE WHEN fastest_response_ms IS NULL OR ? < fastest_response_ms THEN ? ELSE fastest_response_ms END,
				slowest_response_ms = CASE WHEN slowest_response_ms IS NULL OR ? > slowest_response_ms THEN ? ELSE slowest_response_ms END
			WHERE sprint_id = ? AND question_number = ?
		`, responseTimeMs, responseTimeMs, responseTimeMs, responseTimeMs, responseTimeMs, responseTimeMs, sprintID, questionNum)
	}

	return err
}

// GetQuestionStats returns stats for all questions in a sprint
func (d *DB) GetQuestionStats(sprintID int64) ([]*QuestionStats, error) {
	rows, err := d.Query(`
		SELECT id, sprint_id, question_number, question_hash,
		       times_shown, times_correct, times_incorrect, times_skipped,
		       avg_response_time_ms, fastest_response_ms, slowest_response_ms,
		       wrong_answers_json, first_shown, last_shown, last_correct
		FROM question_stats
		WHERE sprint_id = ?
		ORDER BY question_number
	`, sprintID)
	if err != nil {
		return nil, err
	}
	defer rows.Close()

	var stats []*QuestionStats
	for rows.Next() {
		s := &QuestionStats{}
		var firstShown, lastShown, lastCorrect *string

		if err := rows.Scan(
			&s.ID, &s.SprintID, &s.QuestionNumber, &s.QuestionHash,
			&s.TimesShown, &s.TimesCorrect, &s.TimesIncorrect, &s.TimesSkipped,
			&s.AvgResponseTimeMs, &s.FastestResponseMs, &s.SlowestResponseMs,
			&s.WrongAnswersJSON, &firstShown, &lastShown, &lastCorrect,
		); err != nil {
			return nil, err
		}

		if firstShown != nil {
			t, _ := time.Parse("2006-01-02 15:04:05", *firstShown)
			s.FirstShown = &t
		}
		if lastShown != nil {
			t, _ := time.Parse("2006-01-02 15:04:05", *lastShown)
			s.LastShown = &t
		}
		if lastCorrect != nil {
			t, _ := time.Parse("2006-01-02 15:04:05", *lastCorrect)
			s.LastCorrect = &t
		}

		stats = append(stats, s)
	}

	return stats, nil
}

// GetHardestQuestions returns questions with lowest accuracy
func (d *DB) GetHardestQuestions(limit int) ([]*QuestionStats, error) {
	rows, err := d.Query(`
		SELECT id, sprint_id, question_number, question_hash,
		       times_shown, times_correct, times_incorrect,
		       CAST(times_correct AS REAL) / times_shown as accuracy
		FROM question_stats
		WHERE times_shown >= 2
		ORDER BY accuracy ASC
		LIMIT ?
	`, limit)
	if err != nil {
		return nil, err
	}
	defer rows.Close()

	var stats []*QuestionStats
	for rows.Next() {
		s := &QuestionStats{}
		var accuracy float64
		if err := rows.Scan(&s.ID, &s.SprintID, &s.QuestionNumber, &s.QuestionHash,
			&s.TimesShown, &s.TimesCorrect, &s.TimesIncorrect, &accuracy); err != nil {
			return nil, err
		}
		stats = append(stats, s)
	}

	return stats, nil
}

func min(a, b float64) float64 {
	if a < b {
		return a
	}
	return b
}

func max(a, b float64) float64 {
	if a > b {
		return a
	}
	return b
}
