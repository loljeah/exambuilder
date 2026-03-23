package gamification

import (
	"database/sql"
	"fmt"
	"time"
)

// Achievement represents an unlockable achievement
type Achievement struct {
	ID               string
	Name             string
	Description      string
	Category         string
	Icon             string
	RewardCoins      int
	Secret           bool
	RequirementType  string
	RequirementValue int
	Unlocked         bool
	UnlockedAt       string
	Progress         int // Current progress toward requirement
}

// GetAllAchievements returns all achievements with unlock status
func GetAllAchievements(db *sql.DB) ([]Achievement, error) {
	rows, err := db.Query(`
		SELECT a.id, a.name, a.description, a.category, a.icon, a.reward_coins,
		       a.secret, a.requirement_type, a.requirement_value,
		       CASE WHEN u.achievement_id IS NOT NULL THEN 1 ELSE 0 END as unlocked,
		       COALESCE(u.unlocked_at, '')
		FROM achievements a
		LEFT JOIN unlocked_achievements u ON a.id = u.achievement_id
		ORDER BY a.category, a.requirement_value
	`)
	if err != nil {
		return nil, err
	}
	defer rows.Close()

	var achievements []Achievement
	for rows.Next() {
		var a Achievement
		var secret, unlocked int
		err := rows.Scan(&a.ID, &a.Name, &a.Description, &a.Category, &a.Icon,
			&a.RewardCoins, &secret, &a.RequirementType, &a.RequirementValue,
			&unlocked, &a.UnlockedAt)
		if err != nil {
			return nil, err
		}
		a.Secret = secret == 1
		a.Unlocked = unlocked == 1
		achievements = append(achievements, a)
	}
	return achievements, rows.Err()
}

// GetUnlockedAchievements returns only unlocked achievements
func GetUnlockedAchievements(db *sql.DB) ([]Achievement, error) {
	all, err := GetAllAchievements(db)
	if err != nil {
		return nil, err
	}

	var unlocked []Achievement
	for _, a := range all {
		if a.Unlocked {
			unlocked = append(unlocked, a)
		}
	}
	return unlocked, nil
}

// CheckAndUnlockAchievements checks all achievements and unlocks any that are now earned
// Returns list of newly unlocked achievements
func CheckAndUnlockAchievements(db *sql.DB, stats *PlayerStats) ([]Achievement, error) {
	achievements, err := GetAllAchievements(db)
	if err != nil {
		return nil, err
	}

	var newlyUnlocked []Achievement

	for _, a := range achievements {
		if a.Unlocked {
			continue
		}

		progress := getProgressForRequirement(stats, a.RequirementType)
		if progress >= a.RequirementValue {
			// Unlock it!
			if err := unlockAchievement(db, a.ID, a.RewardCoins); err != nil {
				continue // Skip on error
			}
			a.Unlocked = true
			a.UnlockedAt = time.Now().Format("2006-01-02 15:04:05")
			newlyUnlocked = append(newlyUnlocked, a)
		}
	}

	return newlyUnlocked, nil
}

// PlayerStats holds current player statistics for achievement checking
type PlayerStats struct {
	SprintsTotal    int
	PerfectSprints  int
	CurrentStreak   int
	BestStreak      int
	InventoryCount  int
	EquippedSlots   int
	OwnsLegendary   bool
	PurchaseCount   int
	SpeedSprint     bool // Completed under 2 min
	NightSprint     bool // 2-5 AM
	EarlySprint     bool // Before 6 AM
	HolidaySprint   bool
	ComebackDays    int // Days since last activity before return
}

func getProgressForRequirement(stats *PlayerStats, reqType string) int {
	switch reqType {
	case "sprints_total":
		return stats.SprintsTotal
	case "perfect_sprints":
		return stats.PerfectSprints
	case "streak":
		if stats.CurrentStreak > stats.BestStreak {
			return stats.CurrentStreak
		}
		return stats.BestStreak
	case "inventory_count":
		return stats.InventoryCount
	case "equipped_slots":
		return stats.EquippedSlots
	case "legendary_owned":
		if stats.OwnsLegendary {
			return 1
		}
		return 0
	case "purchases":
		return stats.PurchaseCount
	case "speed_sprint":
		if stats.SpeedSprint {
			return 1
		}
		return 0
	case "night_sprint":
		if stats.NightSprint {
			return 1
		}
		return 0
	case "early_sprint":
		if stats.EarlySprint {
			return 1
		}
		return 0
	case "holiday_sprint":
		if stats.HolidaySprint {
			return 1
		}
		return 0
	case "comeback":
		if stats.ComebackDays >= 30 {
			return 1
		}
		return 0
	default:
		return 0
	}
}

func unlockAchievement(db *sql.DB, achievementID string, rewardCoins int) error {
	tx, err := db.Begin()
	if err != nil {
		return err
	}
	defer tx.Rollback()

	// Record unlock
	_, err = tx.Exec(`
		INSERT OR IGNORE INTO unlocked_achievements (achievement_id)
		VALUES (?)
	`, achievementID)
	if err != nil {
		return err
	}

	// Award coins
	if rewardCoins > 0 {
		var balance int
		err = tx.QueryRow(`SELECT coins FROM wallet WHERE id = 'default'`).Scan(&balance)
		if err != nil {
			return err
		}

		newBalance := balance + rewardCoins
		_, err = tx.Exec(`
			UPDATE wallet SET coins = ?, lifetime_coins = lifetime_coins + ?
			WHERE id = 'default'
		`, newBalance, rewardCoins)
		if err != nil {
			return err
		}

		_, err = tx.Exec(`
			INSERT INTO coin_transactions (amount, reason, reference_id, balance_after)
			VALUES (?, ?, ?, ?)
		`, rewardCoins, ReasonAchievement, achievementID, newBalance)
		if err != nil {
			return err
		}
	}

	return tx.Commit()
}

// GetAchievementProgress returns progress toward a specific achievement
func GetAchievementProgress(db *sql.DB, achievementID string, stats *PlayerStats) (int, int, error) {
	var reqType string
	var reqValue int

	err := db.QueryRow(`
		SELECT requirement_type, requirement_value
		FROM achievements WHERE id = ?
	`, achievementID).Scan(&reqType, &reqValue)
	if err != nil {
		return 0, 0, err
	}

	progress := getProgressForRequirement(stats, reqType)
	return progress, reqValue, nil
}

// GatherPlayerStats collects all stats needed for achievement checking
func GatherPlayerStats(db *sql.DB) (*PlayerStats, error) {
	stats := &PlayerStats{}

	// Get from profile
	db.QueryRow(`
		SELECT sprints_passed, current_streak, best_streak
		FROM profile WHERE id = 1
	`).Scan(&stats.SprintsTotal, &stats.CurrentStreak, &stats.BestStreak)

	// Count perfect sprints (100% scores)
	db.QueryRow(`
		SELECT COUNT(*) FROM sprints WHERE best_score = 100
	`).Scan(&stats.PerfectSprints)

	// Inventory count
	db.QueryRow(`SELECT COUNT(*) FROM inventory`).Scan(&stats.InventoryCount)

	// Equipped slots
	stats.EquippedSlots = GetEquippedCount(db)

	// Legendary ownership
	stats.OwnsLegendary = OwnsLegendary(db)

	// Purchase count
	db.QueryRow(`SELECT COUNT(*) FROM purchase_history`).Scan(&stats.PurchaseCount)

	// Time-based checks happen at sprint completion, not here

	return stats, nil
}

// IsNightOwlTime checks if current time is 2-5 AM
func IsNightOwlTime() bool {
	hour := time.Now().Hour()
	return hour >= 2 && hour < 5
}

// IsEarlyBirdTime checks if current time is before 6 AM
func IsEarlyBirdTime() bool {
	return time.Now().Hour() < 6
}

// IsHoliday checks if today is a major holiday (simplified)
func IsHoliday() bool {
	now := time.Now()
	month := now.Month()
	day := now.Day()

	// Major US holidays (simplified)
	holidays := map[string]bool{
		"1-1":   true, // New Year's
		"7-4":   true, // Independence Day
		"12-25": true, // Christmas
		"12-31": true, // New Year's Eve
		"11-28": true, // Thanksgiving (approximate)
	}

	key := fmt.Sprintf("%d-%d", month, day)
	return holidays[key]
}

// GetAchievementCount returns unlocked/total counts
func GetAchievementCount(db *sql.DB) (int, int, error) {
	var unlocked, total int
	err := db.QueryRow(`SELECT COUNT(*) FROM unlocked_achievements`).Scan(&unlocked)
	if err != nil {
		return 0, 0, err
	}
	err = db.QueryRow(`SELECT COUNT(*) FROM achievements`).Scan(&total)
	if err != nil {
		return 0, 0, err
	}
	return unlocked, total, nil
}
