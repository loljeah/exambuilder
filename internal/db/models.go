package db

import "time"

// ============================================================================
// CORE MODELS
// ============================================================================

type Project struct {
	ID          string
	FullHash    string
	Path        string
	Name        string
	ContentType string // code, medical, legal, scientific, technical, study, other
	CreatedAt   time.Time
	LastActive  time.Time
}

type Sprint struct {
	ID            int64
	ProjectID     string
	SprintNumber  int
	Topic         string
	DomainID      string // References domain.id
	SubdomainID   string // Optional finer categorization
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

// ============================================================================
// DOMAIN TRACKING
// ============================================================================

type Domain struct {
	ID            string // project_id + "_" + domain_id
	ProjectID     string
	DomainID      string
	Name          string
	Description   string
	Color         string
	Icon          string
	TotalXP       int
	EarnedXP      int
	Level         int
	SprintsTotal  int
	SprintsPassed int
	SprintsPerfect int
	CreatedAt     time.Time
	UpdatedAt     time.Time
}

type DomainLevel struct {
	ID          int64
	ProjectID   string
	DomainID    string
	Level       int
	XPThreshold int
	Title       string
}

type DomainAchievement struct {
	ID          string // project_id + "_" + achievement_id
	ProjectID   string
	DomainID    string
	Name        string
	Description string
	Condition   string
	XPReward    int
	Icon        string
	Unlocked    bool
	UnlockedAt  *time.Time
}

type Subdomain struct {
	ID          string // project_id + "_" + domain_id + "_" + subdomain_id
	ProjectID   string
	DomainID    string
	SubdomainID string
	Name        string
	TotalXP     int
	EarnedXP    int
}

type DomainStats struct {
	ID               int64
	ProjectID        string
	DomainID         string
	QuestionsTotal   int
	QuestionsCorrect int
	TimeSpentSeconds int
	CurrentStreak    int
	BestStreak       int
	LastActivity     *time.Time
}

// LevelUpResult contains info about domain level changes
type LevelUpResult struct {
	OldLevel  int
	NewLevel  int
	LeveledUp bool
	NewTitle  string
}

type Profile struct {
	TotalXP       int
	Level         int
	CurrentStreak int
	BestStreak    int
	SprintsPassed int
	LastActivity  *time.Time
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

// ============================================================================
// DEBT TRACKING
// ============================================================================

type DebtEntry struct {
	ID          int64
	ProjectID   string
	Action      string
	Weight      int
	Description string
	Timestamp   time.Time
}

// ============================================================================
// JOURNAL & EVENTS
// ============================================================================

type JournalEntry struct {
	ID        int64
	Timestamp time.Time
	EventType string
	ProjectID *string
	SprintID  *int64
	DataJSON  string
	SessionID string
}

// Event types
const (
	EventDaemonStart      = "daemon_start"
	EventDaemonStop       = "daemon_stop"
	EventProjectActivated = "project_activated"
	EventProjectCreated   = "project_created"
	EventExamImported     = "exam_imported"
	EventExamUpdated      = "exam_updated"
	EventSprintStarted    = "sprint_started"
	EventSprintCompleted  = "sprint_completed"
	EventSprintPassed     = "sprint_passed"
	EventSprintFailed     = "sprint_failed"
	EventQuestionAnswered = "question_answered"
	EventDebtAdded        = "debt_added"
	EventDebtCleared      = "debt_cleared"
	EventLevelUp          = "level_up"
	EventStreakUpdated    = "streak_updated"
	EventStreakBroken     = "streak_broken"
	EventBadgeUnlocked    = "badge_unlocked"
	EventVoiceModeUsed    = "voice_mode_used"
	EventMilestoneReached = "milestone_reached"
)

// ============================================================================
// KNOWLEDGE TRACKING
// ============================================================================

type KnowledgeItem struct {
	ID             int64
	ProjectID      string
	SprintID       *int64
	QuestionNumber *int
	Concept        string
	Category       string
	Tier           string

	// Learning status
	Status         string  // unseen, learning, mastered
	TimesSeen      int
	TimesCorrect   int
	TimesIncorrect int
	MasteryScore   float64

	// Spaced repetition
	NextReview   *time.Time
	EaseFactor   float64
	IntervalDays int

	// Timestamps
	FirstSeen   *time.Time
	LastSeen    *time.Time
	LastCorrect *time.Time
	MasteredAt  *time.Time
}

const (
	KnowledgeStatusUnseen   = "unseen"
	KnowledgeStatusLearning = "learning"
	KnowledgeStatusMastered = "mastered"
)

// ============================================================================
// QUESTION ANALYTICS
// ============================================================================

type QuestionStats struct {
	ID             int64
	SprintID       int64
	QuestionNumber int
	QuestionHash   string

	// Stats
	TimesShown     int
	TimesCorrect   int
	TimesIncorrect int
	TimesSkipped   int

	// Timing
	AvgResponseTimeMs  *int
	FastestResponseMs  *int
	SlowestResponseMs  *int
	WrongAnswersJSON   string // {"A": 5, "C": 2}

	// Timestamps
	FirstShown  *time.Time
	LastShown   *time.Time
	LastCorrect *time.Time
}

// ============================================================================
// SESSIONS
// ============================================================================

type Session struct {
	ID               string
	StartedAt        time.Time
	EndedAt          *time.Time
	DurationSeconds  *int
	CommandsReceived int
	SprintsTaken     int
	SprintsPassed    int
	XPEarned         int
	Hostname         string
	Username         string
	Version          string
}

// ============================================================================
// DAILY STATS
// ============================================================================

type DailyStats struct {
	Date string // YYYY-MM-DD

	// Activity
	SessionsCount  int
	ActiveMinutes  int
	CommandsCount  int

	// Learning
	SprintsAttempted  int
	SprintsPassed     int
	QuestionsAnswered int
	QuestionsCorrect  int

	// Progress
	XPEarned    int
	DebtAdded   int
	DebtCleared int
	StreakAtEnd int

	// Timestamps
	FirstActivity *time.Time
	LastActivity  *time.Time
}

// ============================================================================
// GOALS
// ============================================================================

type WeeklyGoal struct {
	ID        int64
	WeekStart string // YYYY-MM-DD (Monday)

	TargetSprints int
	TargetXP      int
	TargetStreak  int

	ActualSprints int
	ActualXP      int
	MaxStreak     int

	Completed   bool
	CompletedAt *time.Time
}

// ============================================================================
// NOTES
// ============================================================================

type StudyNote struct {
	ID              int64
	ProjectID       *string
	SprintID        *int64
	QuestionNumber  *int
	KnowledgeItemID *int64
	Note            string
	CreatedAt       time.Time
	UpdatedAt       time.Time
}

// ============================================================================
// BADGES & MILESTONES
// ============================================================================

type Badge struct {
	ID          string
	Name        string
	Description string
	Icon        string
	Rarity      string
	UnlockedAt  *time.Time
	ProjectID   *string
}

type Milestone struct {
	ID           string
	Name         string
	Description  string
	Category     string // xp, streak, sprints, mastery, special
	Threshold    int
	Icon         string
	CurrentValue int
	Unlocked     bool
	UnlockedAt   *time.Time
	DisplayOrder int
	Hidden       bool
}

// ============================================================================
// SETTINGS & TAGS
// ============================================================================

type Setting struct {
	Key       string
	Value     string
	UpdatedAt time.Time
}

type Tag struct {
	ID        int64
	Name      string
	Color     string
	CreatedAt time.Time
}

type TaggedItem struct {
	TagID    int64
	ItemType string // project, sprint, knowledge_item
	ItemID   string
	TaggedAt time.Time
}

// ============================================================================
// EXPORT
// ============================================================================

type ExportHistory struct {
	ID           int64
	ExportedAt   time.Time
	ExportType   string // full, incremental, journal_only
	FilePath     string
	FileHash     string
	RecordsCount int
	SizeBytes    int
}

// ============================================================================
// QUESTION TYPES (JSON parsing)
// ============================================================================

type Question struct {
	Number      int      `json:"number"`
	Tier        string   `json:"tier"`
	Stars       int      `json:"stars"`
	XP          int      `json:"xp"`
	Text        string   `json:"text"`
	Code        string   `json:"code,omitempty"`
	Options     []string `json:"options"`
	CorrectIdx  int      `json:"correct_idx"`            // For single choice
	CorrectIdxs []int    `json:"correct_idxs,omitempty"` // For multi choice
	Type        string   `json:"type,omitempty"`         // "single" (default) or "multi"
}

type AnswerKey struct {
	Answers      []string `json:"answers"`      // ["B", "A", "C"]
	Hints        []string `json:"hints"`        // Hints per question
	Explanations []string `json:"explanations"` // Full explanations per question
}
