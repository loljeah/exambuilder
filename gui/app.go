package main

import (
	"context"
	"database/sql"
	"encoding/json"
	"fmt"
	"os"
	"path/filepath"
	"sort"
	"strings"
	"sync"
	"time"

	"github.com/loljeah/exambuilder/internal/config"
	"github.com/loljeah/exambuilder/internal/db"
	"github.com/loljeah/exambuilder/internal/exam"
	"github.com/loljeah/exambuilder/internal/gamification"
	"github.com/loljeah/exambuilder/internal/llm"
	"github.com/loljeah/exambuilder/internal/voice"
)

// App struct holds the application state
type App struct {
	ctx    context.Context
	cfg    *config.Config
	db     *db.DB
	sqlDB  *sql.DB
	voice  *voice.Client
	gen    *llm.Generator
	mu     sync.RWMutex // Protects active field
	active string       // Active project ID
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

	// Initialize voice client
	a.voice = voice.NewClient(cfg)

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

	// Initialize LLM generator
	ollamaClient := llm.NewClient(cfg.Ollama.BaseURL, cfg.Ollama.Model, cfg.Ollama.TimeoutSeconds)
	a.gen = llm.NewGenerator(ollamaClient, a.sqlDB, a.db, cfg.Ollama.MaxRetries)
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
	a.mu.RLock()
	activeProject := a.active
	a.mu.RUnlock()

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
	if activeProject != "" {
		if items, err := a.db.GetKnowledgeItemsForReview(activeProject, 100); err == nil {
			data.ReviewDue = len(items)
		}
	}

	// Active project
	if activeProject != "" {
		if p, err := a.db.GetProject(activeProject); err == nil {
			data.ActiveProject = &ProjectData{
				ID:   p.ID,
				Name: p.Name,
				Path: p.Path,
			}
		}
		if sprints, err := a.db.GetSprints(activeProject); err == nil {
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
	XPEarned     int    `json:"xp_earned"`
	DomainID     string `json:"domain_id"`
}

type DomainData struct {
	ID             string `json:"id"`
	DomainID       string `json:"domain_id"`
	Name           string `json:"name"`
	Description    string `json:"description"`
	Color          string `json:"color"`
	Icon           string `json:"icon"`
	TotalXP        int    `json:"total_xp"`
	EarnedXP       int    `json:"earned_xp"`
	Level          int    `json:"level"`
	LevelTitle     string `json:"level_title"`
	NextLevelXP    int    `json:"next_level_xp"`
	SprintsTotal   int    `json:"sprints_total"`
	SprintsPassed  int    `json:"sprints_passed"`
	SprintsPerfect int    `json:"sprints_perfect"`
	ProgressPct    int    `json:"progress_pct"`
}

type DomainAchievementData struct {
	ID          string  `json:"id"`
	Name        string  `json:"name"`
	Description string  `json:"description"`
	Icon        string  `json:"icon"`
	XPReward    int     `json:"xp_reward"`
	Unlocked    bool    `json:"unlocked"`
	UnlockedAt  *string `json:"unlocked_at"`
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

// AddProject adds a new project by path
func (a *App) AddProject(path string) error {
	// Expand ~ to home directory
	if strings.HasPrefix(path, "~/") {
		home, err := os.UserHomeDir()
		if err != nil {
			return fmt.Errorf("get home directory: %w", err)
		}
		path = filepath.Join(home, path[2:])
	}

	// Make path absolute
	absPath, err := filepath.Abs(path)
	if err != nil {
		return fmt.Errorf("get absolute path: %w", err)
	}

	// Verify path exists
	if _, err := os.Stat(absPath); os.IsNotExist(err) {
		return fmt.Errorf("path does not exist: %s", absPath)
	}

	_, err = a.db.GetOrCreateProject(absPath)
	return err
}

// RemoveProject removes a project and all its associated data
func (a *App) RemoveProject(projectID string) error {
	return a.db.DeleteProject(projectID)
}

// ScanAndImportExams scans a project for exam_*.toml and exam_*.md files and imports them
func (a *App) ScanAndImportExams(projectID string) (string, error) {
	project, err := a.db.GetProject(projectID)
	if err != nil {
		return "", err
	}

	// Look for exam files (TOML preferred, then markdown)
	var matches []string

	// First try TOML (v2.0 format)
	tomlPattern := filepath.Join(project.Path, "exam_*.toml")
	tomlMatches, _ := filepath.Glob(tomlPattern)
	matches = append(matches, tomlMatches...)

	// Then try markdown (legacy format)
	mdPattern := filepath.Join(project.Path, "exam_*.md")
	mdMatches, _ := filepath.Glob(mdPattern)
	matches = append(matches, mdMatches...)

	if len(matches) == 0 {
		return "No exam_*.toml or exam_*.md files found in " + project.Path, nil
	}

	totalSprints := 0
	var errors []string

	for _, examFile := range matches {
		var parsedSprints []exam.ParsedSprint

		if strings.HasSuffix(examFile, ".toml") {
			// TOML v2.0 format
			examData, err := exam.ParseExamTOML(examFile)
			if err != nil {
				errors = append(errors, fmt.Sprintf("%s: %v", filepath.Base(examFile), err))
				continue
			}

			// Convert to DB sprints directly
			dbSprints, err := examData.ToDBSprints(projectID)
			if err != nil {
				errors = append(errors, fmt.Sprintf("%s: convert error: %v", filepath.Base(examFile), err))
				continue
			}

			for _, dbSprint := range dbSprints {
				if err := a.db.UpsertSprint(dbSprint); err != nil {
					errors = append(errors, fmt.Sprintf("sprint %d: db error: %v", dbSprint.SprintNumber, err))
					continue
				}
				totalSprints++
			}

			// Import domains from TOML
			for _, domain := range examData.GetDomains() {
				// Calculate total XP for domain from sprints
				domainTotalXP := domain.TotalXP
				sprintsInDomain := 0
				for _, s := range dbSprints {
					if s.DomainID == domain.ID {
						sprintsInDomain++
						if domainTotalXP == 0 {
							domainTotalXP += s.XPAvailable
						}
					}
				}

				dbDomain := &db.Domain{
					ID:           fmt.Sprintf("%s_%s", project.ID, domain.ID),
					ProjectID:    project.ID,
					DomainID:     domain.ID,
					Name:         domain.Name,
					Description:  domain.Description,
					Color:        domain.Color,
					Icon:         domain.Icon,
					TotalXP:      domainTotalXP,
					SprintsTotal: sprintsInDomain,
				}
				if err := a.db.UpsertDomain(dbDomain); err != nil {
					errors = append(errors, fmt.Sprintf("domain %s: db error: %v", domain.ID, err))
				}
			}

			// Import domain levels from TOML
			for _, dl := range examData.GetDomainLevels() {
				for i, threshold := range dl.Levels {
					title := "Level " + fmt.Sprintf("%d", i+1)
					if i < len(dl.Titles) {
						title = dl.Titles[i]
					}
					dbLevel := &db.DomainLevel{
						ProjectID:   project.ID,
						DomainID:    dl.Domain,
						Level:       i + 1,
						XPThreshold: threshold,
						Title:       title,
					}
					if err := a.db.UpsertDomainLevel(dbLevel); err != nil {
						errors = append(errors, fmt.Sprintf("domain level %s/%d: db error: %v", dl.Domain, i+1, err))
					}
				}
			}

			// Import domain achievements from TOML
			for _, ach := range examData.GetDomainAchievements() {
				dbAch := &db.DomainAchievement{
					ID:          fmt.Sprintf("%s_%s", project.ID, ach.ID),
					ProjectID:   project.ID,
					DomainID:    ach.Domain,
					Name:        ach.Name,
					Description: ach.Description,
					Condition:   ach.Condition,
					XPReward:    ach.XPReward,
					Icon:        ach.Icon,
				}
				if err := a.db.UpsertDomainAchievement(dbAch); err != nil {
					errors = append(errors, fmt.Sprintf("domain achievement %s: db error: %v", ach.ID, err))
				}
			}

			continue
		}

		// Legacy markdown format
		content, err := os.ReadFile(examFile)
		if err != nil {
			errors = append(errors, fmt.Sprintf("%s: read error: %v", filepath.Base(examFile), err))
			continue
		}

		parsedSprints, err = exam.ParseExamFile(string(content))
		if err != nil {
			errors = append(errors, fmt.Sprintf("%s: parse error: %v", filepath.Base(examFile), err))
			continue
		}

		if len(parsedSprints) == 0 {
			errors = append(errors, fmt.Sprintf("%s: no sprints found", filepath.Base(examFile)))
			continue
		}

		for _, ps := range parsedSprints {
			dbSprint, err := ps.ToDBSprint(projectID)
			if err != nil {
				errors = append(errors, fmt.Sprintf("sprint %d: %v", ps.Number, err))
				continue
			}

			if err := a.db.UpsertSprint(dbSprint); err != nil {
				errors = append(errors, fmt.Sprintf("sprint %d: db error: %v", ps.Number, err))
				continue
			}
			totalSprints++
		}
	}

	result := fmt.Sprintf("Imported %d sprints from %d file(s)", totalSprints, len(matches))
	if len(errors) > 0 {
		result += fmt.Sprintf(" with %d warnings: %s", len(errors), strings.Join(errors, "; "))
	}
	return result, nil
}

func (a *App) SetActiveProject(projectID string) error {
	a.mu.Lock()
	a.active = projectID
	a.mu.Unlock()
	return nil
}

func (a *App) GetActiveProject() *ProjectData {
	a.mu.RLock()
	activeProject := a.active
	a.mu.RUnlock()

	if activeProject == "" {
		return nil
	}
	p, err := a.db.GetProject(activeProject)
	if err != nil {
		return nil
	}
	return &ProjectData{ID: p.ID, Name: p.Name, Path: p.Path}
}

func (a *App) GetSprints() []SprintData {
	a.mu.RLock()
	activeProject := a.active
	a.mu.RUnlock()

	if activeProject == "" {
		return nil
	}
	sprints, err := a.db.GetSprints(activeProject)
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
			XPEarned:     s.XPEarned,
			DomainID:     s.DomainID,
		})
	}
	return result
}

// GetDomains returns all knowledge domains for the active project
func (a *App) GetDomains() []DomainData {
	a.mu.RLock()
	activeProject := a.active
	a.mu.RUnlock()

	if activeProject == "" {
		return nil
	}

	domains, err := a.db.GetDomains(activeProject)
	if err != nil {
		return nil
	}

	var result []DomainData
	for _, d := range domains {
		// Get current level title and next level XP
		levelTitle := "Novice"
		nextLevelXP := 100
		levels, _ := a.db.GetDomainLevels(activeProject, d.DomainID)
		for _, lvl := range levels {
			if lvl.Level == d.Level {
				levelTitle = lvl.Title
			}
			if lvl.Level == d.Level+1 {
				nextLevelXP = lvl.XPThreshold
			}
		}

		progressPct := 0
		if d.TotalXP > 0 {
			progressPct = (d.EarnedXP * 100) / d.TotalXP
		}

		result = append(result, DomainData{
			ID:             d.ID,
			DomainID:       d.DomainID,
			Name:           d.Name,
			Description:    d.Description,
			Color:          d.Color,
			Icon:           d.Icon,
			TotalXP:        d.TotalXP,
			EarnedXP:       d.EarnedXP,
			Level:          d.Level,
			LevelTitle:     levelTitle,
			NextLevelXP:    nextLevelXP,
			SprintsTotal:   d.SprintsTotal,
			SprintsPassed:  d.SprintsPassed,
			SprintsPerfect: d.SprintsPerfect,
			ProgressPct:    progressPct,
		})
	}
	return result
}

// GetDomainAchievements returns achievements for a specific domain
func (a *App) GetDomainAchievements(domainID string) []DomainAchievementData {
	a.mu.RLock()
	activeProject := a.active
	a.mu.RUnlock()

	if activeProject == "" {
		return nil
	}

	achievements, err := a.db.GetDomainAchievements(activeProject, domainID)
	if err != nil {
		return nil
	}

	var result []DomainAchievementData
	for _, ach := range achievements {
		var unlockedAt *string
		if ach.UnlockedAt != nil {
			s := ach.UnlockedAt.Format("2006-01-02")
			unlockedAt = &s
		}
		result = append(result, DomainAchievementData{
			ID:          ach.ID,
			Name:        ach.Name,
			Description: ach.Description,
			Icon:        ach.Icon,
			XPReward:    ach.XPReward,
			Unlocked:    ach.Unlocked,
			UnlockedAt:  unlockedAt,
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
	a.mu.RLock()
	activeProject := a.active
	a.mu.RUnlock()

	if activeProject == "" {
		return nil
	}
	sprint, err := a.db.GetSprint(activeProject, sprintNumber)
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
	profile, err := a.db.GetProfile()
	if err != nil || profile == nil {
		return nil
	}
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
	profile, err := a.db.GetProfile()
	if err != nil || profile == nil {
		return fmt.Errorf("failed to get profile")
	}
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

// ============================================================================
// Exam Taking / Sprint Submission
// ============================================================================

type SprintResultData struct {
	SprintNum            int                     `json:"sprint_num"`
	Topic                string                  `json:"topic"`
	Passed               bool                    `json:"passed"`
	ScorePercent         int                     `json:"score_percent"`
	CorrectCount         int                     `json:"correct_count"`
	TotalQuestions       int                     `json:"total_questions"`
	XPEarned             int                     `json:"xp_earned"`
	XPAvailable          int                     `json:"xp_available"`
	AttemptNumber        int                     `json:"attempt_number"`
	CoinsEarned          int                     `json:"coins_earned"`
	QuestionResults      []QuestionResultData    `json:"question_results"`
	DomainLevelUp        bool                    `json:"domain_level_up"`
	DomainNewLevel       int                     `json:"domain_new_level"`
	DomainNewTitle       string                  `json:"domain_new_title"`
	DomainName           string                  `json:"domain_name"`
	UnlockedAchievements []UnlockedAchievementData `json:"unlocked_achievements"`
}

type UnlockedAchievementData struct {
	ID       string `json:"id"`
	Name     string `json:"name"`
	Icon     string `json:"icon"`
	XPReward int    `json:"xp_reward"`
}

type QuestionResultData struct {
	QuestionNum int    `json:"question_num"`
	Correct     bool   `json:"correct"`
	UserAnswer  string `json:"user_answer"`
	RightAnswer string `json:"right_answer"`
	XPEarned    int    `json:"xp_earned"`
}

func (a *App) SubmitSprintAnswers(sprintNumber int, answers []string) (*SprintResultData, error) {
	a.mu.RLock()
	activeProject := a.active
	a.mu.RUnlock()

	if activeProject == "" {
		return nil, fmt.Errorf("no active project")
	}

	// Get sprint
	sprint, err := a.db.GetSprint(activeProject, sprintNumber)
	if err != nil {
		return nil, fmt.Errorf("sprint not found: %w", err)
	}

	// Grade the sprint
	passThreshold := 60 // 2/3 correct (60%)
	if a.cfg != nil && a.cfg.Grading.PassThreshold > 0 {
		passThreshold = a.cfg.Grading.PassThreshold
	}

	// Parse questions for grading
	var questions []db.Question
	if err := json.Unmarshal([]byte(sprint.QuestionsJSON), &questions); err != nil {
		return nil, err
	}

	// Parse answer key
	var answerKey db.AnswerKey
	if err := json.Unmarshal([]byte(sprint.AnswerKeyJSON), &answerKey); err != nil {
		return nil, err
	}

	// Grade each question
	result := &SprintResultData{
		SprintNum:      sprint.SprintNumber,
		Topic:          sprint.Topic,
		TotalQuestions: len(questions),
		XPAvailable:    sprint.XPAvailable,
		AttemptNumber:  sprint.Attempts + 1,
	}

	for i, q := range questions {
		qr := QuestionResultData{
			QuestionNum: q.Number,
			XPEarned:    0,
		}

		if i < len(answerKey.Answers) {
			qr.RightAnswer = answerKey.Answers[i]
		}

		if i < len(answers) {
			qr.UserAnswer = normalizeAnswer(answers[i])
			// Normalize both sides and compare (handles case and multi-choice ordering)
			qr.Correct = normalizeAnswer(qr.UserAnswer) == normalizeAnswer(qr.RightAnswer)
			if qr.Correct {
				qr.XPEarned = q.XP
				result.CorrectCount++
				result.XPEarned += q.XP
			}
		}

		result.QuestionResults = append(result.QuestionResults, qr)
	}

	if result.TotalQuestions > 0 {
		result.ScorePercent = (result.CorrectCount * 100) / result.TotalQuestions
	}
	result.Passed = result.ScorePercent >= passThreshold

	// Calculate coins earned (bonus for passing, per-question coins)
	if result.Passed {
		result.CoinsEarned = 10 + (result.CorrectCount * 2) // Base + per correct
	} else {
		result.CoinsEarned = result.CorrectCount // Just per correct if not passed
	}

	// Update sprint status in database
	newStatus := sprint.Status
	if result.Passed && sprint.Status != "passed" {
		newStatus = "passed"
	}

	bestScore := sprint.BestScore
	if bestScore == nil || result.ScorePercent > *bestScore {
		bestScore = &result.ScorePercent
	}

	answersJSON, _ := json.Marshal(answers)
	if err := a.db.UpdateSprintAttempt(sprint.ID, newStatus, bestScore, string(answersJSON)); err != nil {
		return nil, err
	}

	// Award XP and coins
	isFirstPass := result.Passed && sprint.Status != "passed"
	xpToAward := 0
	if isFirstPass {
		xpToAward = result.XPEarned
	} else if sprint.Status == "passed" {
		// Already passed - no XP on replay
		result.XPEarned = 0
	}

	if xpToAward > 0 {
		a.db.AddXP(xpToAward)
	}

	// Coins: full reward on first pass, reduced on replay
	coinsToAward := result.CoinsEarned
	if sprint.Status == "passed" {
		coinsToAward = result.CorrectCount
	}
	if coinsToAward > 0 {
		gamification.AddCoins(a.sqlDB, coinsToAward, "sprint_completed", fmt.Sprintf("sprint_%d", sprintNumber))
	}
	result.CoinsEarned = coinsToAward

	// Domain XP and level tracking
	if sprint.DomainID != "" && xpToAward > 0 {
		levelUp, err := a.db.AddDomainXP(activeProject, sprint.DomainID, xpToAward)
		if err == nil && levelUp != nil && levelUp.LeveledUp {
			result.DomainLevelUp = true
			result.DomainNewLevel = levelUp.NewLevel
			result.DomainNewTitle = levelUp.NewTitle
			// Get domain name
			if dom, err := a.db.GetDomainByID(activeProject, sprint.DomainID); err == nil {
				result.DomainName = dom.Name
			}
		}
	}

	// Update domain stats on first pass
	if isFirstPass && sprint.DomainID != "" {
		perfect := result.ScorePercent == 100
		a.db.RecordDomainSprintComplete(activeProject, sprint.DomainID, true, perfect)

		// Check domain achievements
		unlocked, _ := a.db.EvaluateAchievements(activeProject, sprint.DomainID)
		for _, ach := range unlocked {
			result.UnlockedAchievements = append(result.UnlockedAchievements, UnlockedAchievementData{
				ID:       ach.ID,
				Name:     ach.Name,
				Icon:     ach.Icon,
				XPReward: ach.XPReward,
			})
			// Award achievement XP
			if ach.XPReward > 0 {
				a.db.AddXP(ach.XPReward)
			}
		}
	}

	// Update daily stats
	a.db.RecordSprintAttempt(result.Passed, result.CorrectCount, result.TotalQuestions, xpToAward)

	// Check global achievements
	stats, _ := gamification.GatherPlayerStats(a.sqlDB)
	if stats != nil {
		gamification.CheckAndUnlockAchievements(a.sqlDB, stats)
	}

	return result, nil
}

// normalizeAnswer converts various answer formats to comparable form
// Handles both single answers (A, B, C, D) and multi-choice (A,C or C,A -> A,C)
func normalizeAnswer(ans string) string {
	ans = strings.TrimSpace(strings.ToUpper(ans))

	// Handle multi-choice: sort comma-separated values for order-independent comparison
	if strings.Contains(ans, ",") {
		parts := strings.Split(ans, ",")
		var letters []string
		for _, part := range parts {
			part = strings.TrimSpace(part)
			if len(part) > 0 {
				// Extract first character (the letter)
				letter := string(part[0])
				if letter >= "A" && letter <= "D" {
					letters = append(letters, letter)
				}
			}
		}
		sort.Strings(letters)
		return strings.Join(letters, ",")
	}

	// Direct single letter
	if len(ans) == 1 && ans >= "A" && ans <= "D" {
		return ans
	}

	// "Option A", "A)", "A.", etc.
	for _, letter := range []string{"A", "B", "C", "D"} {
		if strings.HasPrefix(ans, letter) {
			return letter
		}
	}

	return ans
}

// GetSprintHints returns hints for incorrect answers (after first failed attempt)
func (a *App) GetSprintHints(sprintNumber int) []string {
	a.mu.RLock()
	activeProject := a.active
	a.mu.RUnlock()

	if activeProject == "" {
		return nil
	}

	sprint, err := a.db.GetSprint(activeProject, sprintNumber)
	if err != nil {
		return nil
	}

	var answerKey db.AnswerKey
	if err := json.Unmarshal([]byte(sprint.AnswerKeyJSON), &answerKey); err != nil {
		return nil
	}

	return answerKey.Hints
}

// GetSprintExplanations returns full explanations (after second failed attempt)
func (a *App) GetSprintExplanations(sprintNumber int) []string {
	a.mu.RLock()
	activeProject := a.active
	a.mu.RUnlock()

	if activeProject == "" {
		return nil
	}

	sprint, err := a.db.GetSprint(activeProject, sprintNumber)
	if err != nil {
		return nil
	}

	var answerKey db.AnswerKey
	if err := json.Unmarshal([]byte(sprint.AnswerKeyJSON), &answerKey); err != nil {
		return nil
	}

	return answerKey.Explanations
}

// ============================================================================
// Stats
// ============================================================================

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

// ============================================================================
// Knowledge Base Catalogue
// ============================================================================

type KnowledgeQuestionData struct {
	SprintNumber int      `json:"sprint_number"`
	SprintTopic  string   `json:"sprint_topic"`
	QuestionNum  int      `json:"question_num"`
	Tier         string   `json:"tier"`
	Difficulty   int      `json:"difficulty"`
	XP           int      `json:"xp"`
	Text         string   `json:"text"`
	Code         string   `json:"code"`
	Options      []string `json:"options"`
	CorrectIdx   int      `json:"correct_idx"`
	DomainID     string   `json:"domain_id"`
	DomainName   string   `json:"domain_name"`
	Hint         string   `json:"hint"`
	Explanation  string   `json:"explanation"`
	// User stats
	TimesAnswered int  `json:"times_answered"`
	TimesCorrect  int  `json:"times_correct"`
	LastAnswered  *string `json:"last_answered"`
	Mastered      bool `json:"mastered"`
}

// GetKnowledgeBase returns all questions organized by domain
func (a *App) GetKnowledgeBase() []KnowledgeQuestionData {
	a.mu.RLock()
	activeProject := a.active
	a.mu.RUnlock()

	if activeProject == "" {
		return nil
	}

	// Get all sprints
	sprints, err := a.db.GetSprints(activeProject)
	if err != nil {
		return nil
	}

	// Get domains for names
	domainNames := make(map[string]string)
	if domains, err := a.db.GetDomains(activeProject); err == nil {
		for _, d := range domains {
			domainNames[d.DomainID] = d.Name
		}
	}

	var result []KnowledgeQuestionData

	for _, sprint := range sprints {
		// Parse questions
		var questions []db.Question
		if err := json.Unmarshal([]byte(sprint.QuestionsJSON), &questions); err != nil {
			continue
		}

		// Parse answer key for hints/explanations
		var answerKey db.AnswerKey
		json.Unmarshal([]byte(sprint.AnswerKeyJSON), &answerKey)

		for i, q := range questions {
			kq := KnowledgeQuestionData{
				SprintNumber: sprint.SprintNumber,
				SprintTopic:  sprint.Topic,
				QuestionNum:  q.Number,
				Tier:         q.Tier,
				Difficulty:   q.Stars,
				XP:           q.XP,
				Text:         q.Text,
				Code:         q.Code,
				Options:      q.Options,
				CorrectIdx:   q.CorrectIdx,
				DomainID:     sprint.DomainID,
				DomainName:   domainNames[sprint.DomainID],
			}

			// Add hint and explanation if available
			if i < len(answerKey.Hints) {
				kq.Hint = answerKey.Hints[i]
			}
			if i < len(answerKey.Explanations) {
				kq.Explanation = answerKey.Explanations[i]
			}

			// TODO: Get question stats from question_stats table
			// For now, mark as mastered if sprint is passed
			kq.Mastered = sprint.Status == "passed"

			result = append(result, kq)
		}
	}

	return result
}

// GetKnowledgeByDomain returns questions filtered by domain
func (a *App) GetKnowledgeByDomain(domainID string) []KnowledgeQuestionData {
	all := a.GetKnowledgeBase()
	if domainID == "" {
		return all
	}

	var filtered []KnowledgeQuestionData
	for _, q := range all {
		if q.DomainID == domainID {
			filtered = append(filtered, q)
		}
	}
	return filtered
}

// ============================================================================
// Voice/TTS
// ============================================================================

// SpeakText sends text to piper-daemon for TTS (non-blocking)
func (a *App) SpeakText(text string) error {
	if a.voice == nil {
		return fmt.Errorf("voice client not initialized")
	}
	return a.voice.Speak(text)
}

// SpeakTextBlocking sends text to piper-daemon and waits for completion
func (a *App) SpeakTextBlocking(text string) error {
	if a.voice == nil {
		return fmt.Errorf("voice client not initialized")
	}
	return a.voice.SpeakBlocking(text)
}

// StopSpeech stops current TTS playback
func (a *App) StopSpeech() error {
	if a.voice == nil {
		return nil
	}
	return a.voice.StopSpeech()
}

// IsPiperAvailable checks if piper-daemon is running
func (a *App) IsPiperAvailable() bool {
	if a.voice == nil {
		return false
	}
	return a.voice.IsPiperAvailable()
}

// GetQuestionSpeechText returns the text that will be spoken for a question
// Used by frontend to sync typewriter effect with speech
func (a *App) GetQuestionSpeechText(sprintNumber int, questionIndex int) string {
	a.mu.RLock()
	activeProject := a.active
	a.mu.RUnlock()

	if activeProject == "" {
		return ""
	}

	sprint, err := a.db.GetSprint(activeProject, sprintNumber)
	if err != nil {
		return ""
	}

	var questions []db.Question
	if err := json.Unmarshal([]byte(sprint.QuestionsJSON), &questions); err != nil {
		return ""
	}

	if questionIndex < 0 || questionIndex >= len(questions) {
		return ""
	}

	q := &questions[questionIndex]
	return a.voice.GetQuestionSpeechText(q, questionIndex+1)
}

// SpeakQuestion reads a specific question aloud
func (a *App) SpeakQuestion(sprintNumber int, questionIndex int) error {
	a.mu.RLock()
	activeProject := a.active
	a.mu.RUnlock()

	if activeProject == "" {
		return fmt.Errorf("no active project")
	}

	sprint, err := a.db.GetSprint(activeProject, sprintNumber)
	if err != nil {
		return err
	}

	var questions []db.Question
	if err := json.Unmarshal([]byte(sprint.QuestionsJSON), &questions); err != nil {
		return err
	}

	if questionIndex < 0 || questionIndex >= len(questions) {
		return fmt.Errorf("invalid question index")
	}

	q := &questions[questionIndex]
	return a.voice.SpeakQuestion(q, questionIndex+1)
}

// SpeakSprintResult announces sprint completion via TTS
func (a *App) SpeakSprintResult(passed bool, scorePercent int, xpEarned int) error {
	if a.voice == nil {
		return nil
	}
	return a.voice.SpeakSprintResult(passed, scorePercent, xpEarned)
}

// ============================================================================
// Hint Tokens
// ============================================================================

// HintTokenData represents hint token balance for the frontend
type HintTokenData struct {
	Tokens         int `json:"tokens"`
	LifetimeTokens int `json:"lifetime_tokens"`
}

// HintPackData represents a purchasable hint pack for the frontend
type HintPackData struct {
	Tier   string `json:"tier"`
	Tokens int    `json:"tokens"`
	Cost   int    `json:"cost"`
}

// GetHintTokenBalance returns the current hint token balance
func (a *App) GetHintTokenBalance() *HintTokenData {
	if a.sqlDB == nil {
		return &HintTokenData{}
	}
	balance, err := gamification.GetHintTokens(a.sqlDB)
	if err != nil {
		return &HintTokenData{}
	}
	return &HintTokenData{
		Tokens:         balance.Tokens,
		LifetimeTokens: balance.LifetimeTokens,
	}
}

// GetHintPacks returns available hint packs for purchase
func (a *App) GetHintPacks() []HintPackData {
	packs := gamification.GetHintPacks()
	var result []HintPackData
	for _, p := range packs {
		result = append(result, HintPackData{
			Tier:   p.Tier,
			Tokens: p.Tokens,
			Cost:   p.Cost,
		})
	}
	return result
}

// PurchaseHintTokens buys a hint pack with coins
func (a *App) PurchaseHintTokens(tier string) error {
	if a.sqlDB == nil {
		return fmt.Errorf("database not initialized")
	}
	return gamification.PurchaseHintTokens(a.sqlDB, tier)
}

// UseHintToken spends one hint token and returns the hint for a question
func (a *App) UseHintToken(sprintNumber int, questionNumber int) (string, error) {
	a.mu.RLock()
	activeProject := a.active
	a.mu.RUnlock()

	if activeProject == "" {
		return "", fmt.Errorf("no active project")
	}
	if a.sqlDB == nil {
		return "", fmt.Errorf("database not initialized")
	}

	return gamification.UseHintToken(a.sqlDB, a.db, activeProject, sprintNumber, questionNumber)
}

// GetUsedHintsForSprint returns question numbers that had hints used in a sprint
func (a *App) GetUsedHintsForSprint(sprintNumber int) []int {
	a.mu.RLock()
	activeProject := a.active
	a.mu.RUnlock()

	if activeProject == "" || a.sqlDB == nil {
		return nil
	}

	used, err := gamification.GetHintUsageForSprint(a.sqlDB, activeProject, sprintNumber)
	if err != nil {
		return nil
	}
	return used
}

// ============================================================================
// LLM Generation
// ============================================================================

// GenerationGateData represents generation availability for a domain
type GenerationGateData struct {
	DomainID      string `json:"domain_id"`
	DomainLevel   int    `json:"domain_level"`
	CanSprint     bool   `json:"can_sprint"`
	CanCustom     bool   `json:"can_custom"`
	CanExam       bool   `json:"can_exam"`
	CanChallenge  bool   `json:"can_challenge"`
	FirstFreeUsed bool   `json:"first_free_used"`
	SprintCost    int    `json:"sprint_cost"`
	CustomCost    int    `json:"custom_cost"`
	ExamCost      int    `json:"exam_cost"`
	ChallengeCost int    `json:"challenge_cost"`
}

// GenerationResultData represents the result of an LLM generation
type GenerationResultData struct {
	GenerationID int64  `json:"generation_id"`
	SprintIDs    []int  `json:"sprint_ids"`
	CoinsSpent   int    `json:"coins_spent"`
	Status       string `json:"status"`
}

// IsOllamaAvailable checks if the Ollama service is reachable
func (a *App) IsOllamaAvailable() bool {
	if a.gen == nil {
		return false
	}
	return a.gen.Client().IsAvailable()
}

// GetOllamaModels returns available LLM models
func (a *App) GetOllamaModels() []string {
	if a.gen == nil {
		return nil
	}
	models, err := a.gen.Client().ListModels()
	if err != nil {
		return nil
	}
	return models
}

// GetGenerationGate returns what generation types are unlocked for a domain
func (a *App) GetGenerationGate(domainID string) *GenerationGateData {
	a.mu.RLock()
	activeProject := a.active
	a.mu.RUnlock()

	if activeProject == "" || a.gen == nil {
		return &GenerationGateData{DomainID: domainID}
	}

	gate, err := a.gen.GetGenerationGate(activeProject, domainID)
	if err != nil {
		return &GenerationGateData{DomainID: domainID}
	}

	return &GenerationGateData{
		DomainID:      gate.DomainID,
		DomainLevel:   gate.DomainLevel,
		CanSprint:     gate.CanSprint,
		CanCustom:     gate.CanCustom,
		CanExam:       gate.CanExam,
		CanChallenge:  gate.CanChallenge,
		FirstFreeUsed: gate.FirstFreeUsed,
		SprintCost:    gate.SprintCost,
		CustomCost:    gate.CustomCost,
		ExamCost:      gate.ExamCost,
		ChallengeCost: gate.ChallengeCost,
	}
}

// GenerateSprintForDomain generates a new sprint using Ollama
func (a *App) GenerateSprintForDomain(domainID string, topic string) (*GenerationResultData, error) {
	a.mu.RLock()
	activeProject := a.active
	a.mu.RUnlock()

	if activeProject == "" {
		return nil, fmt.Errorf("no active project")
	}
	if a.gen == nil {
		return nil, fmt.Errorf("LLM generator not initialized")
	}

	result, err := a.gen.GenerateSprint(activeProject, domainID, topic)
	if err != nil {
		return nil, err
	}

	return &GenerationResultData{
		GenerationID: result.GenerationID,
		SprintIDs:    result.SprintIDs,
		CoinsSpent:   result.CoinsSpent,
		Status:       result.Status,
	}, nil
}

// GenerateExamForDomain generates a full exam (3 sprints) using Ollama
func (a *App) GenerateExamForDomain(domainID string) (*GenerationResultData, error) {
	a.mu.RLock()
	activeProject := a.active
	a.mu.RUnlock()

	if activeProject == "" {
		return nil, fmt.Errorf("no active project")
	}
	if a.gen == nil {
		return nil, fmt.Errorf("LLM generator not initialized")
	}

	result, err := a.gen.GenerateExam(activeProject, domainID)
	if err != nil {
		return nil, err
	}

	return &GenerationResultData{
		GenerationID: result.GenerationID,
		SprintIDs:    result.SprintIDs,
		CoinsSpent:   result.CoinsSpent,
		Status:       result.Status,
	}, nil
}
