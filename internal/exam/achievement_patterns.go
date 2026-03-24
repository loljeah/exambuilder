package exam

// AchievementPattern defines a themed progression tier system
type AchievementPattern struct {
	ID          string
	Name        string
	Description string
	Tiers       []AchievementTier
}

// AchievementTier defines a single tier in a progression pattern
type AchievementTier struct {
	Level       int
	Title       string
	Icon        string
	XPThreshold int
	Condition   string // Achievement condition like "domain_level >= 3"
	XPReward    int
}

// GetAchievementPatterns returns all predefined thematic progression patterns
func GetAchievementPatterns() []AchievementPattern {
	return []AchievementPattern{
		// Career/Job progression
		{
			ID:          "career",
			Name:        "Career Path",
			Description: "Progress through professional ranks",
			Tiers: []AchievementTier{
				{Level: 1, Title: "Intern", Icon: "🎓", XPThreshold: 0, Condition: "domain_sprints_passed >= 1", XPReward: 10},
				{Level: 2, Title: "Junior", Icon: "👨‍💼", XPThreshold: 50, Condition: "domain_level >= 2", XPReward: 20},
				{Level: 3, Title: "Mid-Level", Icon: "💼", XPThreshold: 150, Condition: "domain_level >= 3", XPReward: 30},
				{Level: 4, Title: "Senior", Icon: "👔", XPThreshold: 300, Condition: "domain_level >= 4", XPReward: 50},
				{Level: 5, Title: "Lead", Icon: "🎯", XPThreshold: 500, Condition: "domain_level >= 5", XPReward: 75},
				{Level: 6, Title: "Principal", Icon: "🏆", XPThreshold: 750, Condition: "domain_level >= 6", XPReward: 100},
				{Level: 7, Title: "Distinguished", Icon: "👑", XPThreshold: 1000, Condition: "all_sprints_perfect", XPReward: 200},
			},
		},
		// Butterfly lifecycle
		{
			ID:          "butterfly",
			Name:        "Metamorphosis",
			Description: "Transform like a butterfly",
			Tiers: []AchievementTier{
				{Level: 1, Title: "Egg", Icon: "🥚", XPThreshold: 0, Condition: "first_sprint_passed", XPReward: 10},
				{Level: 2, Title: "Caterpillar", Icon: "🐛", XPThreshold: 50, Condition: "domain_level >= 2", XPReward: 20},
				{Level: 3, Title: "Chrysalis", Icon: "🪺", XPThreshold: 150, Condition: "domain_level >= 3", XPReward: 35},
				{Level: 4, Title: "Butterfly", Icon: "🦋", XPThreshold: 300, Condition: "domain_level >= 4", XPReward: 50},
				{Level: 5, Title: "Monarch", Icon: "🦋", XPThreshold: 500, Condition: "all_sprints_passed", XPReward: 100},
			},
		},
		// Skill mastery (classic RPG)
		{
			ID:          "mastery",
			Name:        "Skill Mastery",
			Description: "Master your craft",
			Tiers: []AchievementTier{
				{Level: 1, Title: "Novice", Icon: "📖", XPThreshold: 0, Condition: "first_sprint_passed", XPReward: 10},
				{Level: 2, Title: "Apprentice", Icon: "🔨", XPThreshold: 50, Condition: "domain_level >= 2", XPReward: 20},
				{Level: 3, Title: "Journeyman", Icon: "⚒️", XPThreshold: 150, Condition: "domain_level >= 3", XPReward: 35},
				{Level: 4, Title: "Expert", Icon: "🔧", XPThreshold: 300, Condition: "domain_level >= 4", XPReward: 50},
				{Level: 5, Title: "Master", Icon: "⚔️", XPThreshold: 500, Condition: "domain_level >= 5", XPReward: 75},
				{Level: 6, Title: "Grandmaster", Icon: "👑", XPThreshold: 1000, Condition: "all_sprints_perfect", XPReward: 150},
			},
		},
		// Dragon growth
		{
			ID:          "dragon",
			Name:        "Dragon's Path",
			Description: "Grow from hatchling to ancient wyrm",
			Tiers: []AchievementTier{
				{Level: 1, Title: "Hatchling", Icon: "🥚", XPThreshold: 0, Condition: "first_sprint_passed", XPReward: 10},
				{Level: 2, Title: "Wyrmling", Icon: "🐉", XPThreshold: 50, Condition: "domain_level >= 2", XPReward: 20},
				{Level: 3, Title: "Young Dragon", Icon: "🐲", XPThreshold: 150, Condition: "domain_level >= 3", XPReward: 35},
				{Level: 4, Title: "Adult Dragon", Icon: "🔥", XPThreshold: 300, Condition: "domain_level >= 4", XPReward: 50},
				{Level: 5, Title: "Elder Dragon", Icon: "⚡", XPThreshold: 500, Condition: "domain_level >= 5", XPReward: 75},
				{Level: 6, Title: "Ancient Wyrm", Icon: "🌟", XPThreshold: 1000, Condition: "all_sprints_perfect", XPReward: 150},
			},
		},
		// Academic ranks
		{
			ID:          "academic",
			Name:        "Academic Journey",
			Description: "Climb the academic ladder",
			Tiers: []AchievementTier{
				{Level: 1, Title: "Student", Icon: "📚", XPThreshold: 0, Condition: "first_sprint_passed", XPReward: 10},
				{Level: 2, Title: "Graduate", Icon: "🎓", XPThreshold: 50, Condition: "domain_level >= 2", XPReward: 20},
				{Level: 3, Title: "Researcher", Icon: "🔬", XPThreshold: 150, Condition: "domain_level >= 3", XPReward: 35},
				{Level: 4, Title: "Doctor", Icon: "🎓", XPThreshold: 300, Condition: "domain_level >= 4", XPReward: 50},
				{Level: 5, Title: "Professor", Icon: "👨‍🏫", XPThreshold: 500, Condition: "domain_level >= 5", XPReward: 75},
				{Level: 6, Title: "Dean", Icon: "🏛️", XPThreshold: 1000, Condition: "all_sprints_perfect", XPReward: 150},
			},
		},
		// Military ranks
		{
			ID:          "military",
			Name:        "Military Ranks",
			Description: "Rise through the ranks",
			Tiers: []AchievementTier{
				{Level: 1, Title: "Recruit", Icon: "🪖", XPThreshold: 0, Condition: "first_sprint_passed", XPReward: 10},
				{Level: 2, Title: "Private", Icon: "🎖️", XPThreshold: 50, Condition: "domain_level >= 2", XPReward: 20},
				{Level: 3, Title: "Sergeant", Icon: "⭐", XPThreshold: 150, Condition: "domain_level >= 3", XPReward: 35},
				{Level: 4, Title: "Lieutenant", Icon: "⭐⭐", XPThreshold: 300, Condition: "domain_level >= 4", XPReward: 50},
				{Level: 5, Title: "Captain", Icon: "🎖️", XPThreshold: 500, Condition: "domain_level >= 5", XPReward: 75},
				{Level: 6, Title: "Commander", Icon: "⚔️", XPThreshold: 750, Condition: "domain_level >= 6", XPReward: 100},
				{Level: 7, Title: "General", Icon: "👑", XPThreshold: 1000, Condition: "all_sprints_perfect", XPReward: 200},
			},
		},
		// Ninja/Stealth progression
		{
			ID:          "ninja",
			Name:        "Way of the Shadow",
			Description: "Master the ninja arts",
			Tiers: []AchievementTier{
				{Level: 1, Title: "Genin", Icon: "🥷", XPThreshold: 0, Condition: "first_sprint_passed", XPReward: 10},
				{Level: 2, Title: "Chunin", Icon: "🌀", XPThreshold: 50, Condition: "domain_level >= 2", XPReward: 20},
				{Level: 3, Title: "Jonin", Icon: "⚡", XPThreshold: 150, Condition: "domain_level >= 3", XPReward: 35},
				{Level: 4, Title: "Anbu", Icon: "🎭", XPThreshold: 300, Condition: "domain_level >= 4", XPReward: 50},
				{Level: 5, Title: "Hokage", Icon: "🔥", XPThreshold: 500, Condition: "all_sprints_perfect", XPReward: 100},
			},
		},
		// Space exploration
		{
			ID:          "space",
			Name:        "Starbound",
			Description: "Journey through the cosmos",
			Tiers: []AchievementTier{
				{Level: 1, Title: "Cadet", Icon: "🚀", XPThreshold: 0, Condition: "first_sprint_passed", XPReward: 10},
				{Level: 2, Title: "Pilot", Icon: "🛸", XPThreshold: 50, Condition: "domain_level >= 2", XPReward: 20},
				{Level: 3, Title: "Navigator", Icon: "🌟", XPThreshold: 150, Condition: "domain_level >= 3", XPReward: 35},
				{Level: 4, Title: "Commander", Icon: "🌠", XPThreshold: 300, Condition: "domain_level >= 4", XPReward: 50},
				{Level: 5, Title: "Admiral", Icon: "🌌", XPThreshold: 500, Condition: "domain_level >= 5", XPReward: 75},
				{Level: 6, Title: "Starmaster", Icon: "✨", XPThreshold: 1000, Condition: "all_sprints_perfect", XPReward: 150},
			},
		},
	}
}

// GetPatternByID returns a specific achievement pattern
func GetPatternByID(id string) *AchievementPattern {
	patterns := GetAchievementPatterns()
	for _, p := range patterns {
		if p.ID == id {
			return &p
		}
	}
	return nil
}

// GenerateDomainLevels creates domain level definitions from a pattern
func GenerateDomainLevels(pattern *AchievementPattern, domainID string) DomainLevelDef {
	levels := make([]int, len(pattern.Tiers))
	titles := make([]string, len(pattern.Tiers))

	for i, tier := range pattern.Tiers {
		levels[i] = tier.XPThreshold
		titles[i] = tier.Title
	}

	return DomainLevelDef{
		Domain: domainID,
		Levels: levels,
		Titles: titles,
	}
}

// GenerateDomainAchievements creates domain achievements from a pattern
func GenerateDomainAchievements(pattern *AchievementPattern, domainID string) []DomainAchievement {
	achievements := make([]DomainAchievement, len(pattern.Tiers))

	for i, tier := range pattern.Tiers {
		achievements[i] = DomainAchievement{
			ID:          domainID + "_" + pattern.ID + "_" + tier.Title,
			Domain:      domainID,
			Name:        tier.Title,
			Description: "Reach " + tier.Title + " rank in " + pattern.Name,
			Condition:   tier.Condition,
			XPReward:    tier.XPReward,
			Icon:        tier.Icon,
		}
	}

	return achievements
}
