package db

import "time"

type Project struct {
	ID         string
	FullHash   string
	Path       string
	Name       string
	CreatedAt  time.Time
	LastActive time.Time
}

type DebtEntry struct {
	ID          int64
	ProjectID   string
	Action      string
	Weight      int
	Description string
	Timestamp   time.Time
}

type Sprint struct {
	ID            int64
	ProjectID     string
	SprintNumber  int
	Topic         string
	QuestionsJSON string
	AnswerKeyJSON string
	Status        string // pending, passed, failed
	BestScore     *int
	Attempts      int
	XPAvailable   int
	XPEarned      int
	CreatedAt     time.Time
	PassedAt      *time.Time
}

type Profile struct {
	TotalXP       int
	Level         int
	CurrentStreak int
	BestStreak    int
	SprintsPassed int
	LastActivity  *time.Time
}

type Badge struct {
	ID          string
	Name        string
	Description string
	Icon        string
	Rarity      string
	UnlockedAt  *time.Time
	ProjectID   *string
}

type Attempt struct {
	ID          int64
	SprintID    int64
	AnswersJSON string
	Score       int
	Passed      bool
	XPEarned    int
	Timestamp   time.Time
}

// Question types for JSON parsing
type Question struct {
	Number     int      `json:"number"`
	Tier       string   `json:"tier"`
	Stars      int      `json:"stars"`
	XP         int      `json:"xp"`
	Text       string   `json:"text"`
	Code       string   `json:"code,omitempty"`
	Options    []string `json:"options"`
	CorrectIdx int      `json:"correct_idx"`
}

type AnswerKey struct {
	Answers []string `json:"answers"` // ["B", "A", "C"]
}
