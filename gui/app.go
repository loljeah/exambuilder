package main

import (
	"context"
	"database/sql"
	"encoding/json"
	"fmt"
	"os"
	"path/filepath"
	"time"

	"github.com/loljeah/exambuilder/internal/config"
	"github.com/loljeah/exambuilder/internal/db"
	"github.com/loljeah/exambuilder/internal/gamification"
)

// App struct holds the application state
type App struct {
	ctx    context.Context
	cfg    *config.Config
	db     *db.DB
	sqlDB  *sql.DB
	active string // Active project ID
}

// NewApp creates a new App application struct
func NewApp() *App {
	return &App{}
}

// startup is called when the app starts
func (a *App) startup(ctx context.Context) {
	a.ctx = ctx

	// Load config
	cfg, err := config.Load()
	if err != nil {
		fmt.Println("Config load failed:", err)
	}
	a.cfg = cfg

	// Open database
	home, _ := os.UserHomeDir()
	dbPath := filepath.Join(home, ".local/share/kgate/kgate.db")
	migrationsDir := filepath.Join(home, ".local/share/kgate/migrations")

	database, err := db.Open(dbPath, migrationsDir)
	if err != nil {
		fmt.Println("DB open failed:", err)
		return
	}
	a.db = database
	a.sqlDB = database.DB
}

// shutdown is called when the app closes
func (a *App) shutdown(ctx context.Context) {
	if a.db != nil {
		a.db.Close()
	}
}

// ============================================================================
// Dashboard
// ============================================================================

type DashboardData struct {
	Profile        ProfileData        `json:"profile"`
	Avatar         AvatarData         `json:"avatar"`
	Wallet         WalletData         `json:"wallet"`
	DailyLogin     DailyLoginData     `json:"daily_login"`
	Challenges     []ChallengeData    `json:"challenges"`
	WeeklyGoals    []WeeklyGoalData   `json:"weekly_goals"`
	ReviewDue      int                `json:"review_due"`
	ActiveProject  *ProjectData       `json:"active_project"`
	PendingSprints int                `json:"pending_sprints"`
}

func (a *App) GetDashboardData() DashboardData {
	data := DashboardData{}

	// Profile
	if profile, err := a.db.GetProfile(); err == nil {
		data.Profile = ProfileData{
			Level:         profile.Level,
			TotalXP:       profile.TotalXP,
			CurrentStreak: profile.CurrentStreak,
			BestStreak:    profile.BestStreak,
			SprintsPassed: profile.SprintsPassed,
		}
	}

	// Avatar
	if avatar, err := gamification.GetAvatar(a.sqlDB); err == nil {
		data.Avatar = AvatarData{
			CreatureType: avatar.CreatureType,
			Name:         avatar.Name,
			Mood:         avatar.Mood,
			XPMultiplier: avatar.XPMultiplier,
		}
	}

	// Wallet
	if wallet, err := gamification.GetWallet(a.sqlDB); err == nil {
		data.Wallet = WalletData{
			Coins:         wallet.Coins,
			LifetimeCoins: wallet.LifetimeCoins,
		}
	}

	// Daily login
	if dl, err := gamification.GetDailyLogin(a.sqlDB); err == nil {
		data.DailyLogin = DailyLoginData{
			CurrentDay:  dl.CurrentDay,
			TotalClaims: dl.TotalClaims,
			CanClaim:    dl.CanClaim,
		}
	}

	// Challenges
	if challenges, err := gamification.GetDailyChallenges(a.sqlDB); err == nil {
		for _, c := range challenges {
			data.Challenges = append(data.Challenges, ChallengeData{
				ID:          c.ID,
				Description: c.Description,
				Target:      c.Target,
				Progress:    c.Progress,
				RewardCoins: c.RewardCoins,
				Completed:   c.Completed,
				Claimed:     c.Claimed,
			})
		}
	}

	// Weekly goals
	if goals, err := gamification.GetWeeklyGoals(a.sqlDB); err == nil {
		for _, g := range goals {
			data.WeeklyGoals = append(data.WeeklyGoals, WeeklyGoalData{
				ID:          g.ID,
				Description: g.Description,
				Target:      g.Target,
				Progress:    g.Progress,
				RewardCoins: g.RewardCoins,
				Completed:   g.Completed,
				Claimed:     g.Claimed,
			})
		}
	}

	// Review items due
	if a.active != "" {
		if items, err := a.db.GetKnowledgeItemsForReview(a.active, 100); err == nil {
			data.ReviewDue = len(items)
		}
	}

	// Active project
	if a.active != "" {
		if p, err := a.db.GetProject(a.active); err == nil {
			data.ActiveProject = &ProjectData{
				ID:   p.ID,
				Name: p.Name,
				Path: p.Path,
			}
		}
		if sprints, err := a.db.GetSprints(a.active); err == nil {
			for _, s := range sprints {
				if s.Status == "pending" {
					data.PendingSprints++
				}
			}
		}
	}

	return data
}

// ============================================================================
// Avatar
// ============================================================================

type AvatarData struct {
	CreatureType string  `json:"creature_type"`
	Name         string  `json:"name"`
	Mood         string  `json:"mood"`
	XPMultiplier float64 `json:"xp_multiplier"`
}

func (a *App) GetAvatar() AvatarData {
	avatar, err := gamification.GetAvatar(a.sqlDB)
	if err != nil {
		return AvatarData{}
	}
	return AvatarData{
		CreatureType: avatar.CreatureType,
		Name:         avatar.Name,
		Mood:         avatar.Mood,
		XPMultiplier: avatar.XPMultiplier,
	}
}

func (a *App) SetCreatureType(creatureType string) error {
	return gamification.SetCreatureType(a.sqlDB, creatureType)
}

func (a *App) SetAvatarName(name string) error {
	return gamification.SetAvatarName(a.sqlDB, name)
}

// ============================================================================
// Wallet
// ============================================================================

type WalletData struct {
	Coins         int `json:"coins"`
	LifetimeCoins int `json:"lifetime_coins"`
}

func (a *App) GetWallet() WalletData {
	wallet, err := gamification.GetWallet(a.sqlDB)
	if err != nil {
		return WalletData{}
	}
	return WalletData{
		Coins:         wallet.Coins,
		LifetimeCoins: wallet.LifetimeCoins,
	}
}

// ============================================================================
// Profile
// ============================================================================

type ProfileData struct {
	Level         int `json:"level"`
	TotalXP       int `json:"total_xp"`
	CurrentStreak int `json:"current_streak"`
	BestStreak    int `json:"best_streak"`
	SprintsPassed int `json:"sprints_passed"`
}

func (a *App) GetProfile() ProfileData {
	profile, err := a.db.GetProfile()
	if err != nil {
		return ProfileData{}
	}
	return ProfileData{
		Level:         profile.Level,
		TotalXP:       profile.TotalXP,
		CurrentStreak: profile.CurrentStreak,
		BestStreak:    profile.BestStreak,
		SprintsPassed: profile.SprintsPassed,
	}
}

// ============================================================================
// Projects & Sprints
// ============================================================================

type ProjectData struct {
	ID   string `json:"id"`
	Name string `json:"name"`
	Path string `json:"path"`
}

type SprintData struct {
	ID           int64  `json:"id"`
	SprintNumber int    `json:"sprint_number"`
	Topic        string `json:"topic"`
	Status       string `json:"status"`
	BestScore    int    `json:"best_score"`
	Attempts     int    `json:"attempts"`
	XPAvailable  int    `json:"xp_available"`
}

func (a *App) GetProjects() []ProjectData {
	projects, err := a.db.ListProjects()
	if err != nil {
		return nil
	}
	var result []ProjectData
	for _, p := range projects {
		result = append(result, ProjectData{
			ID:   p.ID,
			Name: p.Name,
			Path: p.Path,
		})
	}
	return result
}

func (a *App) SetActiveProject(projectID string) error {
	a.active = projectID
	return nil
}

func (a *App) GetActiveProject() *ProjectData {
	if a.active == "" {
		return nil
	}
	p, err := a.db.GetProject(a.active)
	if err != nil {
		return nil
	}
	return &ProjectData{ID: p.ID, Name: p.Name, Path: p.Path}
}

func (a *App) GetSprints() []SprintData {
	if a.active == "" {
		return nil
	}
	sprints, err := a.db.GetSprints(a.active)
	if err != nil {
		return nil
	}
	var result []SprintData
	for _, s := range sprints {
		score := 0
		if s.BestScore != nil {
			score = *s.BestScore
		}
		result = append(result, SprintData{
			ID:           s.ID,
			SprintNumber: s.SprintNumber,
			Topic:        s.Topic,
			Status:       s.Status,
			BestScore:    score,
			Attempts:     s.Attempts,
			XPAvailable:  s.XPAvailable,
		})
	}
	return result
}

type QuestionData struct {
	Number     int      `json:"number"`
	Tier       string   `json:"tier"`
	Stars      int      `json:"stars"`
	XP         int      `json:"xp"`
	Text       string   `json:"text"`
	Code       string   `json:"code"`
	Options    []string `json:"options"`
	CorrectIdx int      `json:"correct_idx"`
}

func (a *App) GetSprintQuestions(sprintNumber int) []QuestionData {
	if a.active == "" {
		return nil
	}
	sprint, err := a.db.GetSprint(a.active, sprintNumber)
	if err != nil {
		return nil
	}

	// Parse questions JSON
	var questions []db.Question
	if err := json.Unmarshal([]byte(sprint.QuestionsJSON), &questions); err != nil {
		return nil
	}

	var result []QuestionData
	for _, q := range questions {
		result = append(result, QuestionData{
			Number:     q.Number,
			Tier:       q.Tier,
			Stars:      q.Stars,
			XP:         q.XP,
			Text:       q.Text,
			Code:       q.Code,
			Options:    q.Options,
			CorrectIdx: q.CorrectIdx,
		})
	}
	return result
}

// ============================================================================
// Shop
// ============================================================================

type ShopItemData struct {
	ID          string `json:"id"`
	Name        string `json:"name"`
	Description string `json:"description"`
	Slot        string `json:"slot"`
	Price       int    `json:"price"`
	Rarity      string `json:"rarity"`
	UnlockLevel int    `json:"unlock_level"`
	Owned       bool   `json:"owned"`
}

func (a *App) GetShopItems(slot string) []ShopItemData {
	profile, _ := a.db.GetProfile()
	items, err := gamification.GetShopItems(a.sqlDB, slot, profile.Level)
	if err != nil {
		return nil
	}
	var result []ShopItemData
	for _, item := range items {
		result = append(result, ShopItemData{
			ID:          item.ID,
			Name:        item.Name,
			Description: item.Description,
			Slot:        item.Slot,
			Price:       item.Price,
			Rarity:      item.Rarity,
			UnlockLevel: item.UnlockLevel,
			Owned:       item.Owned,
		})
	}
	return result
}

func (a *App) PurchaseItem(itemID string) error {
	profile, _ := a.db.GetProfile()
	return gamification.PurchaseItem(a.sqlDB, itemID, profile.Level)
}

func (a *App) GetInventory() []ShopItemData {
	items, err := gamification.GetOwnedItems(a.sqlDB)
	if err != nil {
		return nil
	}
	var result []ShopItemData
	for _, item := range items {
		result = append(result, ShopItemData{
			ID:          item.ID,
			Name:        item.Name,
			Description: item.Description,
			Slot:        item.Slot,
			Price:       item.Price,
			Rarity:      item.Rarity,
			Owned:       true,
		})
	}
	return result
}

type EquippedData struct {
	HatID        string `json:"hat_id"`
	HeldID       string `json:"held_id"`
	AuraID       string `json:"aura_id"`
	BackgroundID string `json:"background_id"`
}

func (a *App) GetEquipped() EquippedData {
	eq, err := gamification.GetEquipped(a.sqlDB)
	if err != nil {
		return EquippedData{}
	}
	return EquippedData{
		HatID:        eq.HatID,
		HeldID:       eq.HeldID,
		AuraID:       eq.AuraID,
		BackgroundID: eq.BackgroundID,
	}
}

func (a *App) EquipItem(itemID string) error {
	return gamification.EquipItem(a.sqlDB, itemID)
}

func (a *App) UnequipSlot(slot string) error {
	return gamification.UnequipSlot(a.sqlDB, slot)
}

// ============================================================================
// Daily Systems
// ============================================================================

type DailyLoginData struct {
	CurrentDay  int  `json:"current_day"`
	TotalClaims int  `json:"total_claims"`
	CanClaim    bool `json:"can_claim"`
}

func (a *App) GetDailyLogin() DailyLoginData {
	dl, err := gamification.GetDailyLogin(a.sqlDB)
	if err != nil {
		return DailyLoginData{}
	}
	return DailyLoginData{
		CurrentDay:  dl.CurrentDay,
		TotalClaims: dl.TotalClaims,
		CanClaim:    dl.CanClaim,
	}
}

func (a *App) ClaimDailyReward() (int, error) {
	return gamification.ClaimDailyReward(a.sqlDB)
}

type ChallengeData struct {
	ID          int    `json:"id"`
	Description string `json:"description"`
	Target      int    `json:"target"`
	Progress    int    `json:"progress"`
	RewardCoins int    `json:"reward_coins"`
	Completed   bool   `json:"completed"`
	Claimed     bool   `json:"claimed"`
}

func (a *App) GetDailyChallenges() []ChallengeData {
	challenges, err := gamification.GetDailyChallenges(a.sqlDB)
	if err != nil {
		return nil
	}
	var result []ChallengeData
	for _, c := range challenges {
		result = append(result, ChallengeData{
			ID:          c.ID,
			Description: c.Description,
			Target:      c.Target,
			Progress:    c.Progress,
			RewardCoins: c.RewardCoins,
			Completed:   c.Completed,
			Claimed:     c.Claimed,
		})
	}
	return result
}

func (a *App) ClaimChallengeReward(challengeID int) (int, error) {
	return gamification.ClaimChallengeReward(a.sqlDB, challengeID)
}

type WeeklyGoalData struct {
	ID          int    `json:"id"`
	Description string `json:"description"`
	Target      int    `json:"target"`
	Progress    int    `json:"progress"`
	RewardCoins int    `json:"reward_coins"`
	Completed   bool   `json:"completed"`
	Claimed     bool   `json:"claimed"`
}

func (a *App) GetWeeklyGoals() []WeeklyGoalData {
	goals, err := gamification.GetWeeklyGoals(a.sqlDB)
	if err != nil {
		return nil
	}
	var result []WeeklyGoalData
	for _, g := range goals {
		result = append(result, WeeklyGoalData{
			ID:          g.ID,
			Description: g.Description,
			Target:      g.Target,
			Progress:    g.Progress,
			RewardCoins: g.RewardCoins,
			Completed:   g.Completed,
			Claimed:     g.Claimed,
		})
	}
	return result
}

func (a *App) ClaimWeeklyGoalReward(goalID int) (int, error) {
	return gamification.ClaimWeeklyGoalReward(a.sqlDB, goalID)
}

// ============================================================================
// Achievements
// ============================================================================

type AchievementData struct {
	ID          string `json:"id"`
	Name        string `json:"name"`
	Description string `json:"description"`
	Category    string `json:"category"`
	Icon        string `json:"icon"`
	RewardCoins int    `json:"reward_coins"`
	Secret      bool   `json:"secret"`
	Unlocked    bool   `json:"unlocked"`
	UnlockedAt  string `json:"unlocked_at"`
}

func (a *App) GetAchievements() []AchievementData {
	achievements, err := gamification.GetAllAchievements(a.sqlDB)
	if err != nil {
		return nil
	}
	var result []AchievementData
	for _, ach := range achievements {
		result = append(result, AchievementData{
			ID:          ach.ID,
			Name:        ach.Name,
			Description: ach.Description,
			Category:    ach.Category,
			Icon:        ach.Icon,
			RewardCoins: ach.RewardCoins,
			Secret:      ach.Secret,
			Unlocked:    ach.Unlocked,
			UnlockedAt:  ach.UnlockedAt,
		})
	}
	return result
}

func (a *App) GetAchievementCounts() (int, int) {
	unlocked, total, _ := gamification.GetAchievementCount(a.sqlDB)
	return unlocked, total
}

// ============================================================================
// Stats
// ============================================================================

type DailyStatsData struct {
	Date              string `json:"date"`
	SessionsCount     int    `json:"sessions_count"`
	SprintsAttempted  int    `json:"sprints_attempted"`
	SprintsPassed     int    `json:"sprints_passed"`
	QuestionsAnswered int    `json:"questions_answered"`
	QuestionsCorrect  int    `json:"questions_correct"`
	XPEarned          int    `json:"xp_earned"`
}

func (a *App) GetStats(period string) []DailyStatsData {
	endDate := time.Now().Format("2006-01-02")
	startDate := endDate

	switch period {
	case "week":
		startDate = time.Now().AddDate(0, 0, -6).Format("2006-01-02")
	case "month":
		startDate = time.Now().AddDate(0, -1, 0).Format("2006-01-02")
	}

	stats, err := a.db.GetDailyStats(startDate, endDate)
	if err != nil {
		return nil
	}

	var result []DailyStatsData
	for _, s := range stats {
		result = append(result, DailyStatsData{
			Date:              s.Date,
			SessionsCount:     s.SessionsCount,
			SprintsAttempted:  s.SprintsAttempted,
			SprintsPassed:     s.SprintsPassed,
			QuestionsAnswered: s.QuestionsAnswered,
			QuestionsCorrect:  s.QuestionsCorrect,
			XPEarned:          s.XPEarned,
		})
	}
	return result
}
