package gamification

import (
	"database/sql"
	"fmt"
)

// Wallet represents the user's coin balance
type Wallet struct {
	Coins         int
	LifetimeCoins int
}

// CoinTransaction represents a coin change
type CoinTransaction struct {
	ID           int
	Amount       int
	Reason       string
	ReferenceID  string
	BalanceAfter int
	Timestamp    string
}

// Coin earning reasons
const (
	ReasonSprintPass    = "sprint_pass"
	ReasonSprintPerfect = "sprint_perfect"
	ReasonStreakBonus   = "streak_bonus"
	ReasonDailyReward   = "daily_reward"
	ReasonChallenge     = "challenge"
	ReasonWeeklyGoal    = "weekly_goal"
	ReasonAchievement   = "achievement"
	ReasonPurchase      = "purchase"
)

// GetWallet retrieves the current wallet balance
func GetWallet(db *sql.DB) (*Wallet, error) {
	wallet := &Wallet{}
	err := db.QueryRow(`SELECT coins, lifetime_coins FROM wallet WHERE id = 'default'`).
		Scan(&wallet.Coins, &wallet.LifetimeCoins)
	if err != nil {
		return nil, err
	}
	return wallet, nil
}

// AddCoins adds coins to the wallet and logs the transaction
func AddCoins(db *sql.DB, amount int, reason string, referenceID string) error {
	if amount <= 0 {
		return fmt.Errorf("amount must be positive")
	}

	tx, err := db.Begin()
	if err != nil {
		return err
	}
	defer tx.Rollback()

	// Update wallet
	_, err = tx.Exec(`
		UPDATE wallet SET
			coins = coins + ?,
			lifetime_coins = lifetime_coins + ?
		WHERE id = 'default'
	`, amount, amount)
	if err != nil {
		return err
	}

	// Get new balance
	var newBalance int
	err = tx.QueryRow(`SELECT coins FROM wallet WHERE id = 'default'`).Scan(&newBalance)
	if err != nil {
		return err
	}

	// Log transaction
	_, err = tx.Exec(`
		INSERT INTO coin_transactions (amount, reason, reference_id, balance_after)
		VALUES (?, ?, ?, ?)
	`, amount, reason, referenceID, newBalance)
	if err != nil {
		return err
	}

	return tx.Commit()
}

// SpendCoins removes coins from the wallet for a purchase
func SpendCoins(db *sql.DB, amount int, reason string, referenceID string) error {
	if amount <= 0 {
		return fmt.Errorf("amount must be positive")
	}

	tx, err := db.Begin()
	if err != nil {
		return err
	}
	defer tx.Rollback()

	// Check balance
	var currentBalance int
	err = tx.QueryRow(`SELECT coins FROM wallet WHERE id = 'default'`).Scan(&currentBalance)
	if err != nil {
		return err
	}

	if currentBalance < amount {
		return fmt.Errorf("insufficient coins: have %d, need %d", currentBalance, amount)
	}

	// Deduct
	_, err = tx.Exec(`UPDATE wallet SET coins = coins - ? WHERE id = 'default'`, amount)
	if err != nil {
		return err
	}

	newBalance := currentBalance - amount

	// Log transaction (negative amount)
	_, err = tx.Exec(`
		INSERT INTO coin_transactions (amount, reason, reference_id, balance_after)
		VALUES (?, ?, ?, ?)
	`, -amount, reason, referenceID, newBalance)
	if err != nil {
		return err
	}

	return tx.Commit()
}

// GetBalance returns just the coin count
func GetBalance(db *sql.DB) (int, error) {
	var coins int
	err := db.QueryRow(`SELECT coins FROM wallet WHERE id = 'default'`).Scan(&coins)
	return coins, err
}

// GetRecentTransactions returns recent coin transactions
func GetRecentTransactions(db *sql.DB, limit int) ([]CoinTransaction, error) {
	rows, err := db.Query(`
		SELECT id, amount, reason, COALESCE(reference_id, ''), balance_after, timestamp
		FROM coin_transactions
		ORDER BY timestamp DESC
		LIMIT ?
	`, limit)
	if err != nil {
		return nil, err
	}
	defer rows.Close()

	var transactions []CoinTransaction
	for rows.Next() {
		var tx CoinTransaction
		err := rows.Scan(&tx.ID, &tx.Amount, &tx.Reason, &tx.ReferenceID, &tx.BalanceAfter, &tx.Timestamp)
		if err != nil {
			return nil, err
		}
		transactions = append(transactions, tx)
	}
	return transactions, rows.Err()
}

// CoinsFromSprint calculates coins earned from completing a sprint
func CoinsFromSprint(passed bool, perfect bool, firstTime bool) int {
	if !passed {
		return 0
	}

	coins := 10 // Base for any pass
	if firstTime {
		coins = 50 // First time bonus
	}
	if perfect {
		coins += 25 // Perfect bonus
	}
	return coins
}

// StreakBonus calculates bonus coins from streak
func StreakBonus(streakDays int) int {
	// 5 coins per streak day, cap at 50
	bonus := streakDays * 5
	if bonus > 50 {
		bonus = 50
	}
	return bonus
}
