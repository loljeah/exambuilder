package gamification

import (
	"database/sql"
	"fmt"
)

// AchievementSeed defines an achievement to seed
type AchievementSeed struct {
	ID               string
	Name             string
	Description      string
	Category         string
	Icon             string
	RewardCoins      int
	Secret           bool
	RequirementType  string
	RequirementValue int
}

// SeedAchievements populates the achievements table
func SeedAchievements(db *sql.DB) error {
	achievements := getAllAchievements()

	tx, err := db.Begin()
	if err != nil {
		return err
	}
	defer tx.Rollback()

	for _, a := range achievements {
		secret := 0
		if a.Secret {
			secret = 1
		}
		_, err := tx.Exec(`
			INSERT INTO achievements (id, name, description, category, icon, reward_coins, secret, requirement_type, requirement_value)
			VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
			ON CONFLICT(id) DO UPDATE SET
				name = excluded.name,
				description = excluded.description,
				reward_coins = excluded.reward_coins
		`, a.ID, a.Name, a.Description, a.Category, a.Icon, a.RewardCoins, secret, a.RequirementType, a.RequirementValue)
		if err != nil {
			return fmt.Errorf("failed to seed achievement %s: %w", a.ID, err)
		}
	}

	return tx.Commit()
}

func getAllAchievements() []AchievementSeed {
	return []AchievementSeed{
		// ============================================================================
		// SPRINT MILESTONES
		// ============================================================================
		{"first_sprint", "First Steps", "Complete your first sprint", "sprints", "🎯", 25, false, "sprints_total", 1},
		{"sprint_5", "Getting Started", "Complete 5 sprints", "sprints", "📚", 50, false, "sprints_total", 5},
		{"sprint_10", "Dedicated Learner", "Complete 10 sprints", "sprints", "📖", 100, false, "sprints_total", 10},
		{"sprint_25", "Knowledge Seeker", "Complete 25 sprints", "sprints", "🧠", 200, false, "sprints_total", 25},
		{"sprint_50", "Sprint Master", "Complete 50 sprints", "sprints", "🏃", 400, false, "sprints_total", 50},
		{"sprint_100", "Century Club", "Complete 100 sprints", "sprints", "💯", 1000, false, "sprints_total", 100},

		// ============================================================================
		// PERFECT SCORES
		// ============================================================================
		{"first_perfect", "Perfectionist", "Get 100% on a sprint", "perfect", "⭐", 50, false, "perfect_sprints", 1},
		{"perfect_5", "Sharp Mind", "Get 100% on 5 sprints", "perfect", "🌟", 150, false, "perfect_sprints", 5},
		{"perfect_10", "Flawless", "Get 100% on 10 sprints", "perfect", "✨", 300, false, "perfect_sprints", 10},
		{"perfect_25", "Genius", "Get 100% on 25 sprints", "perfect", "🧪", 750, false, "perfect_sprints", 25},

		// ============================================================================
		// STREAKS
		// ============================================================================
		{"streak_3", "On a Roll", "Maintain a 3-day streak", "streak", "🔥", 30, false, "streak", 3},
		{"streak_7", "Weekly Warrior", "Maintain a 7-day streak", "streak", "📅", 100, false, "streak", 7},
		{"streak_14", "Two Week Champion", "Maintain a 14-day streak", "streak", "🏆", 250, false, "streak", 14},
		{"streak_30", "Monthly Master", "Maintain a 30-day streak", "streak", "📆", 500, false, "streak", 30},
		{"streak_100", "Unstoppable", "Maintain a 100-day streak", "streak", "💪", 2000, false, "streak", 100},

		// ============================================================================
		// COLLECTION
		// ============================================================================
		{"first_purchase", "Window Shopper", "Buy your first item", "collection", "🛒", 20, false, "purchases", 1},
		{"collector_10", "Collector", "Own 10 items", "collection", "📦", 100, false, "inventory_count", 10},
		{"collector_25", "Hoarder", "Own 25 items", "collection", "🎁", 250, false, "inventory_count", 25},
		{"collector_50", "Treasure Hunter", "Own 50 items", "collection", "💎", 500, false, "inventory_count", 50},
		{"full_outfit", "Fashionista", "Have all 4 slots equipped", "collection", "👗", 150, false, "equipped_slots", 4},
		{"legendary_owner", "Legendary", "Own a legendary item", "collection", "🌈", 500, false, "legendary_owned", 1},

		// ============================================================================
		// TIME-BASED (Secret achievements)
		// ============================================================================
		{"night_owl", "Night Owl", "Complete a sprint between 2-5 AM", "time", "🦉", 100, true, "night_sprint", 1},
		{"early_bird", "Early Bird", "Complete a sprint before 6 AM", "time", "🐦", 75, true, "early_sprint", 1},
		{"speed_demon", "Speed Demon", "Complete a sprint in under 2 minutes", "time", "⚡", 150, true, "speed_sprint", 1},
		{"holiday_learner", "Holiday Spirit", "Complete a sprint on a holiday", "time", "🎄", 200, true, "holiday_sprint", 1},

		// ============================================================================
		// SPECIAL
		// ============================================================================
		{"comeback", "Welcome Back", "Return after 30+ days away", "special", "👋", 100, true, "comeback", 1},

		// ============================================================================
		// HINT USAGE
		// ============================================================================
		{"first_hint", "Hint Seeker", "Use your first hint token", "learning", "💡", 25, false, "hints_used", 1},
		{"hint_10", "Resourceful", "Use 10 hint tokens", "learning", "🔍", 100, false, "hints_used", 10},
		{"no_hints_pass", "Pure Knowledge", "Pass a sprint without hints while owning tokens", "learning", "🧠", 150, true, "no_hint_pass", 1},

		// ============================================================================
		// LLM GENERATION
		// ============================================================================
		{"first_gen", "Creator", "Generate your first sprint", "special", "⚡", 50, false, "generations", 1},
		{"gen_10", "Content Machine", "Generate 10 sprints", "special", "🏭", 200, false, "generations", 10},
		{"domain_explorer", "Domain Explorer", "Generate content in 3 different domains", "special", "🌍", 300, false, "gen_domains", 3},
	}
}
