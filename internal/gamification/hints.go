package gamification

import (
	"database/sql"
	"encoding/json"
	"fmt"

	"github.com/loljeah/exambuilder/internal/db"
)

// HintTokenBalance represents the user's hint token state
type HintTokenBalance struct {
	Tokens         int
	LifetimeTokens int
}

// HintPack defines a purchasable hint token pack
type HintPack struct {
	Tier    string
	Tokens  int
	Cost    int
}

var hintPacks = map[string]HintPack{
	"small":  {Tier: "small", Tokens: 3, Cost: 30},
	"medium": {Tier: "medium", Tokens: 10, Cost: 80},
	"large":  {Tier: "large", Tokens: 25, Cost: 150},
}

// GetHintTokens retrieves the current hint token balance
func GetHintTokens(sqlDB *sql.DB) (*HintTokenBalance, error) {
	balance := &HintTokenBalance{}
	err := sqlDB.QueryRow(`SELECT tokens, lifetime_tokens FROM hint_tokens WHERE id = 'default'`).
		Scan(&balance.Tokens, &balance.LifetimeTokens)
	if err != nil {
		return nil, err
	}
	return balance, nil
}

// PurchaseHintTokens buys a hint pack using coins
func PurchaseHintTokens(sqlDB *sql.DB, tier string) error {
	pack, ok := hintPacks[tier]
	if !ok {
		return fmt.Errorf("unknown hint pack tier: %s", tier)
	}

	tx, err := sqlDB.Begin()
	if err != nil {
		return err
	}
	defer tx.Rollback()

	// Check coin balance
	var balance int
	err = tx.QueryRow(`SELECT coins FROM wallet WHERE id = 'default'`).Scan(&balance)
	if err != nil {
		return err
	}
	if balance < pack.Cost {
		return fmt.Errorf("insufficient coins: have %d, need %d", balance, pack.Cost)
	}

	// Deduct coins
	_, err = tx.Exec(`UPDATE wallet SET coins = coins - ? WHERE id = 'default'`, pack.Cost)
	if err != nil {
		return err
	}

	newBalance := balance - pack.Cost

	// Log coin transaction
	_, err = tx.Exec(`
		INSERT INTO coin_transactions (amount, reason, reference_id, balance_after)
		VALUES (?, ?, ?, ?)
	`, -pack.Cost, ReasonHintPurchase, fmt.Sprintf("hint_%s", tier), newBalance)
	if err != nil {
		return err
	}

	// Add hint tokens
	_, err = tx.Exec(`
		UPDATE hint_tokens SET
			tokens = tokens + ?,
			lifetime_tokens = lifetime_tokens + ?
		WHERE id = 'default'
	`, pack.Tokens, pack.Tokens)
	if err != nil {
		return err
	}

	return tx.Commit()
}

// UseHintToken spends one hint token and returns the hint text for a question
func UseHintToken(sqlDB *sql.DB, examDB *db.DB, projectID string, sprintNum int, questionNum int) (string, error) {
	tx, err := sqlDB.Begin()
	if err != nil {
		return "", err
	}
	defer tx.Rollback()

	// Check if hint already used for this question
	var count int
	err = tx.QueryRow(`
		SELECT COUNT(*) FROM hint_usage
		WHERE project_id = ? AND sprint_number = ? AND question_number = ?
	`, projectID, sprintNum, questionNum).Scan(&count)
	if err != nil {
		return "", err
	}
	if count > 0 {
		return "", fmt.Errorf("hint already used for this question")
	}

	// Check token balance
	var tokens int
	err = tx.QueryRow(`SELECT tokens FROM hint_tokens WHERE id = 'default'`).Scan(&tokens)
	if err != nil {
		return "", err
	}
	if tokens < 1 {
		return "", fmt.Errorf("no hint tokens available")
	}

	// Deduct token
	_, err = tx.Exec(`UPDATE hint_tokens SET tokens = tokens - 1 WHERE id = 'default'`)
	if err != nil {
		return "", err
	}

	// Record usage
	_, err = tx.Exec(`
		INSERT INTO hint_usage (project_id, sprint_number, question_number)
		VALUES (?, ?, ?)
	`, projectID, sprintNum, questionNum)
	if err != nil {
		return "", err
	}

	if err := tx.Commit(); err != nil {
		return "", err
	}

	// Retrieve hint text from sprint answer key
	sprint, err := examDB.GetSprint(projectID, sprintNum)
	if err != nil {
		return "", fmt.Errorf("sprint not found: %w", err)
	}

	var answerKey db.AnswerKey
	if err := json.Unmarshal([]byte(sprint.AnswerKeyJSON), &answerKey); err != nil {
		return "", fmt.Errorf("failed to parse answer key: %w", err)
	}

	// questionNum is 1-based from the frontend
	idx := questionNum - 1
	if idx < 0 || idx >= len(answerKey.Hints) {
		return "", fmt.Errorf("question number %d out of range", questionNum)
	}

	return answerKey.Hints[idx], nil
}

// GetHintUsageForSprint returns question numbers that have used hints in a sprint
func GetHintUsageForSprint(sqlDB *sql.DB, projectID string, sprintNum int) ([]int, error) {
	rows, err := sqlDB.Query(`
		SELECT question_number FROM hint_usage
		WHERE project_id = ? AND sprint_number = ?
		ORDER BY question_number
	`, projectID, sprintNum)
	if err != nil {
		return nil, err
	}
	defer rows.Close()

	var used []int
	for rows.Next() {
		var qn int
		if err := rows.Scan(&qn); err != nil {
			return nil, err
		}
		used = append(used, qn)
	}
	return used, rows.Err()
}

// GetHintPacks returns available hint packs for display
func GetHintPacks() []HintPack {
	return []HintPack{
		hintPacks["small"],
		hintPacks["medium"],
		hintPacks["large"],
	}
}
