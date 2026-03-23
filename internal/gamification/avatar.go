package gamification

import (
	"database/sql"
	"time"
)

// Creature types
const (
	CreatureCat     = "cat"
	CreatureSlime   = "slime"
	CreatureOctopus = "octopus"
	CreatureSnail   = "snail"
)

// Mood states
const (
	MoodHappy   = "happy"   // Active today, +15% XP
	MoodContent = "content" // Active yesterday, +5% XP
	MoodNeutral = "neutral" // 2-3 days inactive
	MoodSad     = "sad"     // 4-6 days inactive
	MoodLonely  = "lonely"  // 7+ days inactive
)

// Avatar represents the user's companion
type Avatar struct {
	ID             string
	CreatureType   string
	Name           string
	Mood           string
	LastActiveDate string
	CreatedAt      string
}

// GetAvatar retrieves the user's avatar
func GetAvatar(db *sql.DB) (*Avatar, error) {
	avatar := &Avatar{}
	err := db.QueryRow(`
		SELECT id, creature_type, name, last_active_date, created_at
		FROM avatar WHERE id = 'default'
	`).Scan(&avatar.ID, &avatar.CreatureType, &avatar.Name, &avatar.LastActiveDate, &avatar.CreatedAt)
	if err != nil {
		return nil, err
	}
	avatar.Mood = CalculateMood(avatar.LastActiveDate)
	return avatar, nil
}

// SetCreatureType changes the avatar's creature
func SetCreatureType(db *sql.DB, creatureType string) error {
	validTypes := map[string]bool{
		CreatureCat:     true,
		CreatureSlime:   true,
		CreatureOctopus: true,
		CreatureSnail:   true,
	}
	if !validTypes[creatureType] {
		return sql.ErrNoRows // Invalid type
	}

	_, err := db.Exec(`UPDATE avatar SET creature_type = ? WHERE id = 'default'`, creatureType)
	return err
}

// SetAvatarName renames the avatar
func SetAvatarName(db *sql.DB, name string) error {
	if name == "" || len(name) > 20 {
		return sql.ErrNoRows
	}
	_, err := db.Exec(`UPDATE avatar SET name = ? WHERE id = 'default'`, name)
	return err
}

// UpdateLastActive marks the avatar as active today
func UpdateLastActive(db *sql.DB) error {
	today := time.Now().Format("2006-01-02")
	_, err := db.Exec(`UPDATE avatar SET last_active_date = ? WHERE id = 'default'`, today)
	return err
}

// CalculateMood determines mood based on last active date
func CalculateMood(lastActiveDate string) string {
	if lastActiveDate == "" {
		return MoodNeutral
	}

	lastActive, err := time.Parse("2006-01-02", lastActiveDate)
	if err != nil {
		return MoodNeutral
	}

	today := time.Now().Truncate(24 * time.Hour)
	lastActive = lastActive.Truncate(24 * time.Hour)
	daysSince := int(today.Sub(lastActive).Hours() / 24)

	switch {
	case daysSince == 0:
		return MoodHappy
	case daysSince == 1:
		return MoodContent
	case daysSince <= 3:
		return MoodNeutral
	case daysSince <= 6:
		return MoodSad
	default:
		return MoodLonely
	}
}

// GetXPMultiplier returns the XP bonus based on mood
func GetXPMultiplier(mood string) float64 {
	switch mood {
	case MoodHappy:
		return 1.15
	case MoodContent:
		return 1.05
	default:
		return 1.0
	}
}

// ValidCreatureTypes returns all valid creature types
func ValidCreatureTypes() []string {
	return []string{CreatureCat, CreatureSlime, CreatureOctopus, CreatureSnail}
}
