package db

import (
	"encoding/json"
	"os"
	"time"

	"github.com/google/uuid"
)

var currentSessionID string

// InitSession creates a new session and returns its ID
func (d *DB) InitSession(version string) (string, error) {
	currentSessionID = uuid.New().String()

	hostname, _ := os.Hostname()
	username := os.Getenv("USER")

	_, err := d.Exec(`
		INSERT INTO sessions (id, hostname, username, version)
		VALUES (?, ?, ?, ?)
	`, currentSessionID, hostname, username, version)
	if err != nil {
		return "", err
	}

	// Log daemon start event
	d.LogEvent(EventDaemonStart, nil, nil, map[string]interface{}{
		"version":  version,
		"hostname": hostname,
	})

	return currentSessionID, nil
}

// EndSession marks the current session as ended
func (d *DB) EndSession() error {
	if currentSessionID == "" {
		return nil
	}

	// Log daemon stop event
	d.LogEvent(EventDaemonStop, nil, nil, nil)

	_, err := d.Exec(`
		UPDATE sessions SET
			ended_at = datetime('now'),
			duration_seconds = CAST((julianday('now') - julianday(started_at)) * 86400 AS INTEGER)
		WHERE id = ?
	`, currentSessionID)

	return err
}

// GetSessionID returns the current session ID
func GetSessionID() string {
	return currentSessionID
}

// LogEvent records an event in the journal
func (d *DB) LogEvent(eventType string, projectID *string, sprintID *int64, data map[string]interface{}) error {
	var dataJSON *string
	if data != nil {
		b, _ := json.Marshal(data)
		s := string(b)
		dataJSON = &s
	}

	_, err := d.Exec(`
		INSERT INTO journal (event_type, project_id, sprint_id, data_json, session_id)
		VALUES (?, ?, ?, ?, ?)
	`, eventType, projectID, sprintID, dataJSON, currentSessionID)

	// Update session stats
	if currentSessionID != "" {
		d.Exec(`UPDATE sessions SET commands_received = commands_received + 1 WHERE id = ?`, currentSessionID)

		if eventType == EventSprintCompleted {
			d.Exec(`UPDATE sessions SET sprints_taken = sprints_taken + 1 WHERE id = ?`, currentSessionID)
		}
		if eventType == EventSprintPassed {
			d.Exec(`UPDATE sessions SET sprints_passed = sprints_passed + 1 WHERE id = ?`, currentSessionID)
			if data != nil {
				if xp, ok := data["xp_earned"].(int); ok {
					d.Exec(`UPDATE sessions SET xp_earned = xp_earned + ? WHERE id = ?`, xp, currentSessionID)
				}
			}
		}
	}

	// Update daily stats
	d.updateDailyStats(eventType, data)

	return err
}

// updateDailyStats updates the daily statistics
func (d *DB) updateDailyStats(eventType string, data map[string]interface{}) {
	today := time.Now().Format("2006-01-02")

	// Ensure row exists
	d.Exec(`
		INSERT INTO daily_stats (date, first_activity, last_activity)
		VALUES (?, datetime('now'), datetime('now'))
		ON CONFLICT(date) DO UPDATE SET last_activity = datetime('now')
	`, today)

	switch eventType {
	case EventDaemonStart:
		d.Exec(`UPDATE daily_stats SET sessions_count = sessions_count + 1 WHERE date = ?`, today)
	case EventSprintCompleted:
		d.Exec(`UPDATE daily_stats SET sprints_attempted = sprints_attempted + 1 WHERE date = ?`, today)
	case EventSprintPassed:
		d.Exec(`UPDATE daily_stats SET sprints_passed = sprints_passed + 1 WHERE date = ?`, today)
		if data != nil {
			if xp, ok := data["xp_earned"].(int); ok {
				d.Exec(`UPDATE daily_stats SET xp_earned = xp_earned + ? WHERE date = ?`, xp, today)
			}
		}
	case EventQuestionAnswered:
		d.Exec(`UPDATE daily_stats SET questions_answered = questions_answered + 1 WHERE date = ?`, today)
		if data != nil {
			if correct, ok := data["correct"].(bool); ok && correct {
				d.Exec(`UPDATE daily_stats SET questions_correct = questions_correct + 1 WHERE date = ?`, today)
			}
		}
	case EventDebtAdded:
		if data != nil {
			if weight, ok := data["weight"].(int); ok {
				d.Exec(`UPDATE daily_stats SET debt_added = debt_added + ? WHERE date = ?`, weight, today)
			}
		}
	case EventDebtCleared:
		if data != nil {
			if amount, ok := data["amount"].(int); ok {
				d.Exec(`UPDATE daily_stats SET debt_cleared = debt_cleared + ? WHERE date = ?`, amount, today)
			}
		}
	case EventStreakUpdated:
		if data != nil {
			if streak, ok := data["streak"].(int); ok {
				d.Exec(`UPDATE daily_stats SET streak_at_end = ? WHERE date = ?`, streak, today)
			}
		}
	}

	d.Exec(`UPDATE daily_stats SET commands_count = commands_count + 1 WHERE date = ?`, today)
}

// GetJournalEntries returns recent journal entries
func (d *DB) GetJournalEntries(limit int, eventTypes []string) ([]*JournalEntry, error) {
	query := `
		SELECT id, timestamp, event_type, project_id, sprint_id, data_json, session_id
		FROM journal
	`
	args := []interface{}{}

	if len(eventTypes) > 0 {
		query += " WHERE event_type IN (?" + repeatString(",?", len(eventTypes)-1) + ")"
		for _, et := range eventTypes {
			args = append(args, et)
		}
	}

	query += " ORDER BY timestamp DESC LIMIT ?"
	args = append(args, limit)

	rows, err := d.Query(query, args...)
	if err != nil {
		return nil, err
	}
	defer rows.Close()

	var entries []*JournalEntry
	for rows.Next() {
		e := &JournalEntry{}
		var ts string
		var projectID, dataJSON, sessionID *string
		var sprintID *int64

		if err := rows.Scan(&e.ID, &ts, &e.EventType, &projectID, &sprintID, &dataJSON, &sessionID); err != nil {
			return nil, err
		}

		e.Timestamp, _ = time.Parse("2006-01-02 15:04:05", ts)
		if projectID != nil {
			e.ProjectID = projectID
		}
		if sprintID != nil {
			e.SprintID = sprintID
		}
		if dataJSON != nil {
			e.DataJSON = *dataJSON
		}
		if sessionID != nil {
			e.SessionID = *sessionID
		}

		entries = append(entries, e)
	}

	return entries, nil
}

// GetDailyStats returns stats for a date range
func (d *DB) GetDailyStats(startDate, endDate string) ([]*DailyStats, error) {
	rows, err := d.Query(`
		SELECT date, sessions_count, active_minutes, commands_count,
		       sprints_attempted, sprints_passed, questions_answered, questions_correct,
		       xp_earned, debt_added, debt_cleared, streak_at_end,
		       first_activity, last_activity
		FROM daily_stats
		WHERE date >= ? AND date <= ?
		ORDER BY date
	`, startDate, endDate)
	if err != nil {
		return nil, err
	}
	defer rows.Close()

	var stats []*DailyStats
	for rows.Next() {
		s := &DailyStats{}
		var firstAct, lastAct *string

		if err := rows.Scan(
			&s.Date, &s.SessionsCount, &s.ActiveMinutes, &s.CommandsCount,
			&s.SprintsAttempted, &s.SprintsPassed, &s.QuestionsAnswered, &s.QuestionsCorrect,
			&s.XPEarned, &s.DebtAdded, &s.DebtCleared, &s.StreakAtEnd,
			&firstAct, &lastAct,
		); err != nil {
			return nil, err
		}

		if firstAct != nil {
			t, _ := time.Parse("2006-01-02 15:04:05", *firstAct)
			s.FirstActivity = &t
		}
		if lastAct != nil {
			t, _ := time.Parse("2006-01-02 15:04:05", *lastAct)
			s.LastActivity = &t
		}

		stats = append(stats, s)
	}

	return stats, nil
}

func repeatString(s string, n int) string {
	result := ""
	for i := 0; i < n; i++ {
		result += s
	}
	return result
}
