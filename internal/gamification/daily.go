package gamification

import (
	"database/sql"
	"fmt"
	"math/rand"
	"time"
)

// DailyLogin tracks the 7-day login reward cycle
type DailyLogin struct {
	CurrentDay  int
	LastClaim   string
	TotalClaims int
	CanClaim    bool
}

// Daily reward amounts (day 1-7)
var dailyRewards = []int{10, 15, 25, 40, 60, 85, 120}

// Challenge types
const (
	ChallengeCompleteSprints = "complete_sprints"
	ChallengeScore80         = "score_80"
	ChallengePerfect         = "perfect"
	ChallengeReview          = "review"
	ChallengeVoice           = "voice"
	ChallengeAnyActivity     = "any_activity"
)

// Challenge represents a daily challenge
type Challenge struct {
	ID          int
	Date        string
	Type        string
	Description string
	Target      int
	Progress    int
	RewardCoins int
	Completed   bool
	Claimed     bool
}

// WeeklyGoal represents a weekly goal
type WeeklyGoal struct {
	ID          int
	WeekStart   string
	GoalType    string
	Description string
	Target      int
	Progress    int
	RewardCoins int
	Completed   bool
	Claimed     bool
}

// GetDailyLogin returns current login status
func GetDailyLogin(db *sql.DB) (*DailyLogin, error) {
	dl := &DailyLogin{}
	var lastClaim sql.NullString

	err := db.QueryRow(`
		SELECT current_day, last_claim_date, total_claims
		FROM daily_login WHERE id = 'default'
	`).Scan(&dl.CurrentDay, &lastClaim, &dl.TotalClaims)
	if err != nil {
		return nil, err
	}

	dl.LastClaim = lastClaim.String
	today := time.Now().Format("2006-01-02")
	dl.CanClaim = dl.LastClaim != today

	return dl, nil
}

// ClaimDailyReward claims today's reward, returns coins earned
func ClaimDailyReward(db *sql.DB) (int, error) {
	dl, err := GetDailyLogin(db)
	if err != nil {
		return 0, err
	}

	if !dl.CanClaim {
		return 0, fmt.Errorf("already claimed today")
	}

	today := time.Now().Format("2006-01-02")

	// Check if streak broken (more than 1 day since last claim)
	nextDay := dl.CurrentDay + 1
	if dl.LastClaim != "" {
		lastClaim, _ := time.Parse("2006-01-02", dl.LastClaim)
		yesterday := time.Now().AddDate(0, 0, -1).Format("2006-01-02")
		if dl.LastClaim != yesterday && dl.LastClaim != today {
			// Streak broken, reset to day 1
			nextDay = 1
			_ = lastClaim // suppress unused
		}
	} else {
		nextDay = 1
	}

	// Wrap around after day 7
	if nextDay > 7 {
		nextDay = 1
	}

	reward := dailyRewards[nextDay-1]

	tx, err := db.Begin()
	if err != nil {
		return 0, err
	}
	defer tx.Rollback()

	// Update daily login
	_, err = tx.Exec(`
		UPDATE daily_login SET
			current_day = ?,
			last_claim_date = ?,
			total_claims = total_claims + 1
		WHERE id = 'default'
	`, nextDay, today)
	if err != nil {
		return 0, err
	}

	// Add coins
	var balance int
	err = tx.QueryRow(`SELECT coins FROM wallet WHERE id = 'default'`).Scan(&balance)
	if err != nil {
		return 0, err
	}

	newBalance := balance + reward
	_, err = tx.Exec(`
		UPDATE wallet SET coins = ?, lifetime_coins = lifetime_coins + ?
		WHERE id = 'default'
	`, newBalance, reward)
	if err != nil {
		return 0, err
	}

	// Log transaction
	_, err = tx.Exec(`
		INSERT INTO coin_transactions (amount, reason, reference_id, balance_after)
		VALUES (?, ?, ?, ?)
	`, reward, ReasonDailyReward, fmt.Sprintf("day_%d", nextDay), newBalance)
	if err != nil {
		return 0, err
	}

	return reward, tx.Commit()
}

// GetDailyChallenges returns today's challenges, generating if needed
func GetDailyChallenges(db *sql.DB) ([]Challenge, error) {
	today := time.Now().Format("2006-01-02")

	// Check if challenges exist for today
	var count int
	db.QueryRow(`SELECT COUNT(*) FROM daily_challenges WHERE date = ?`, today).Scan(&count)

	if count == 0 {
		// Generate new challenges
		if err := generateDailyChallenges(db, today); err != nil {
			return nil, err
		}
	}

	rows, err := db.Query(`
		SELECT id, date, challenge_type, description, target, progress, reward_coins, completed, claimed
		FROM daily_challenges WHERE date = ?
		ORDER BY id
	`, today)
	if err != nil {
		return nil, err
	}
	defer rows.Close()

	var challenges []Challenge
	for rows.Next() {
		var c Challenge
		var completed, claimed int
		err := rows.Scan(&c.ID, &c.Date, &c.Type, &c.Description, &c.Target,
			&c.Progress, &c.RewardCoins, &completed, &claimed)
		if err != nil {
			return nil, err
		}
		c.Completed = completed == 1
		c.Claimed = claimed == 1
		challenges = append(challenges, c)
	}
	return challenges, rows.Err()
}

func generateDailyChallenges(db *sql.DB, date string) error {
	// Define possible challenges
	challengePool := []struct {
		Type        string
		Description string
		Target      int
		Reward      int
	}{
		{ChallengeCompleteSprints, "Complete 2 sprints", 2, 25},
		{ChallengeCompleteSprints, "Complete 3 sprints", 3, 40},
		{ChallengeScore80, "Score 80%+ on a sprint", 1, 30},
		{ChallengePerfect, "Get a perfect 100% score", 1, 50},
		{ChallengeReview, "Review 5 knowledge items", 5, 20},
		{ChallengeAnyActivity, "Complete any sprint", 1, 15},
	}

	// Pick 3 random challenges
	rand.Shuffle(len(challengePool), func(i, j int) {
		challengePool[i], challengePool[j] = challengePool[j], challengePool[i]
	})

	for i := 0; i < 3 && i < len(challengePool); i++ {
		c := challengePool[i]
		_, err := db.Exec(`
			INSERT INTO daily_challenges (date, challenge_type, description, target, reward_coins)
			VALUES (?, ?, ?, ?, ?)
		`, date, c.Type, c.Description, c.Target, c.Reward)
		if err != nil {
			return err
		}
	}
	return nil
}

// UpdateChallengeProgress updates progress for a challenge type
func UpdateChallengeProgress(db *sql.DB, challengeType string, delta int) error {
	today := time.Now().Format("2006-01-02")

	_, err := db.Exec(`
		UPDATE daily_challenges
		SET progress = MIN(progress + ?, target),
		    completed = CASE WHEN progress + ? >= target THEN 1 ELSE completed END
		WHERE date = ? AND challenge_type = ? AND completed = 0
	`, delta, delta, today, challengeType)
	return err
}

// ClaimChallengeReward claims a completed challenge reward
func ClaimChallengeReward(db *sql.DB, challengeID int) (int, error) {
	var completed, claimed, reward int
	err := db.QueryRow(`
		SELECT completed, claimed, reward_coins
		FROM daily_challenges WHERE id = ?
	`, challengeID).Scan(&completed, &claimed, &reward)
	if err != nil {
		return 0, err
	}

	if completed != 1 {
		return 0, fmt.Errorf("challenge not completed")
	}
	if claimed == 1 {
		return 0, fmt.Errorf("already claimed")
	}

	tx, err := db.Begin()
	if err != nil {
		return 0, err
	}
	defer tx.Rollback()

	// Mark claimed
	_, err = tx.Exec(`UPDATE daily_challenges SET claimed = 1 WHERE id = ?`, challengeID)
	if err != nil {
		return 0, err
	}

	// Add coins
	var balance int
	err = tx.QueryRow(`SELECT coins FROM wallet WHERE id = 'default'`).Scan(&balance)
	if err != nil {
		return 0, err
	}

	newBalance := balance + reward
	_, err = tx.Exec(`
		UPDATE wallet SET coins = ?, lifetime_coins = lifetime_coins + ?
		WHERE id = 'default'
	`, newBalance, reward)
	if err != nil {
		return 0, err
	}

	// Log
	_, err = tx.Exec(`
		INSERT INTO coin_transactions (amount, reason, reference_id, balance_after)
		VALUES (?, ?, ?, ?)
	`, reward, ReasonChallenge, fmt.Sprintf("challenge_%d", challengeID), newBalance)
	if err != nil {
		return 0, err
	}

	return reward, tx.Commit()
}

// GetWeeklyGoals returns this week's goals, generating if needed
func GetWeeklyGoals(db *sql.DB) ([]WeeklyGoal, error) {
	weekStart := getWeekStart(time.Now())

	// Check if goals exist
	var count int
	db.QueryRow(`SELECT COUNT(*) FROM weekly_goals WHERE week_start = ?`, weekStart).Scan(&count)

	if count == 0 {
		if err := generateWeeklyGoals(db, weekStart); err != nil {
			return nil, err
		}
	}

	rows, err := db.Query(`
		SELECT id, week_start, goal_type, description, target, progress, reward_coins, completed, claimed
		FROM weekly_goals WHERE week_start = ?
		ORDER BY id
	`, weekStart)
	if err != nil {
		return nil, err
	}
	defer rows.Close()

	var goals []WeeklyGoal
	for rows.Next() {
		var g WeeklyGoal
		var completed, claimed int
		err := rows.Scan(&g.ID, &g.WeekStart, &g.GoalType, &g.Description, &g.Target,
			&g.Progress, &g.RewardCoins, &completed, &claimed)
		if err != nil {
			return nil, err
		}
		g.Completed = completed == 1
		g.Claimed = claimed == 1
		goals = append(goals, g)
	}
	return goals, rows.Err()
}

func generateWeeklyGoals(db *sql.DB, weekStart string) error {
	goals := []struct {
		Type        string
		Description string
		Target      int
		Reward      int
	}{
		{"sprints", "Complete 10 sprints", 10, 100},
		{"streak", "Maintain 7-day streak", 7, 75},
		{"perfect", "Get 3 perfect scores", 3, 150},
		{"mastery", "Master 5 knowledge items", 5, 125},
	}

	for _, g := range goals {
		_, err := db.Exec(`
			INSERT INTO weekly_goals (week_start, goal_type, description, target, reward_coins)
			VALUES (?, ?, ?, ?, ?)
		`, weekStart, g.Type, g.Description, g.Target, g.Reward)
		if err != nil {
			return err
		}
	}
	return nil
}

// UpdateWeeklyGoalProgress updates progress for a goal type
func UpdateWeeklyGoalProgress(db *sql.DB, goalType string, delta int) error {
	weekStart := getWeekStart(time.Now())

	_, err := db.Exec(`
		UPDATE weekly_goals
		SET progress = MIN(progress + ?, target),
		    completed = CASE WHEN progress + ? >= target THEN 1 ELSE completed END
		WHERE week_start = ? AND goal_type = ? AND completed = 0
	`, delta, delta, weekStart, goalType)
	return err
}

// ClaimWeeklyGoalReward claims a completed weekly goal
func ClaimWeeklyGoalReward(db *sql.DB, goalID int) (int, error) {
	var completed, claimed, reward int
	err := db.QueryRow(`
		SELECT completed, claimed, reward_coins
		FROM weekly_goals WHERE id = ?
	`, goalID).Scan(&completed, &claimed, &reward)
	if err != nil {
		return 0, err
	}

	if completed != 1 {
		return 0, fmt.Errorf("goal not completed")
	}
	if claimed == 1 {
		return 0, fmt.Errorf("already claimed")
	}

	tx, err := db.Begin()
	if err != nil {
		return 0, err
	}
	defer tx.Rollback()

	_, err = tx.Exec(`UPDATE weekly_goals SET claimed = 1 WHERE id = ?`, goalID)
	if err != nil {
		return 0, err
	}

	var balance int
	err = tx.QueryRow(`SELECT coins FROM wallet WHERE id = 'default'`).Scan(&balance)
	if err != nil {
		return 0, err
	}

	newBalance := balance + reward
	_, err = tx.Exec(`
		UPDATE wallet SET coins = ?, lifetime_coins = lifetime_coins + ?
		WHERE id = 'default'
	`, newBalance, reward)
	if err != nil {
		return 0, err
	}

	_, err = tx.Exec(`
		INSERT INTO coin_transactions (amount, reason, reference_id, balance_after)
		VALUES (?, ?, ?, ?)
	`, reward, ReasonWeeklyGoal, fmt.Sprintf("goal_%d", goalID), newBalance)
	if err != nil {
		return 0, err
	}

	return reward, tx.Commit()
}

// getWeekStart returns the Monday of the given week
func getWeekStart(t time.Time) string {
	weekday := int(t.Weekday())
	if weekday == 0 {
		weekday = 7 // Sunday
	}
	monday := t.AddDate(0, 0, -(weekday - 1))
	return monday.Format("2006-01-02")
}
