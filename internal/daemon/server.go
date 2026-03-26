package daemon

import (
	"bufio"
	"encoding/json"
	"fmt"
	"net"
	"os"
	"path/filepath"
	"strconv"
	"strings"
	"sync"
	"time"

	"github.com/loljeah/exambuilder/internal/config"
	"github.com/loljeah/exambuilder/internal/db"
	"github.com/loljeah/exambuilder/internal/exam"
	"github.com/loljeah/exambuilder/internal/gamification"
	"github.com/loljeah/exambuilder/internal/logging"
	"github.com/loljeah/exambuilder/internal/voice"
)

const (
	socketReadTimeout = 5 * time.Second
	maxCommandLength  = 8192
)

type Server struct {
	cfg      *config.Config
	db       *db.DB
	voice    *voice.Client
	listener net.Listener
	mu       sync.Mutex

	// Current state
	activeProject string
}

func NewServer(cfg *config.Config, database *db.DB) *Server {
	return &Server{
		cfg:   cfg,
		db:    database,
		voice: voice.NewClient(cfg),
	}
}

func (s *Server) Start() error {
	// Remove stale socket
	os.Remove(s.cfg.General.SocketPath)

	listener, err := net.Listen("unix", s.cfg.General.SocketPath)
	if err != nil {
		return err
	}
	// Restrict socket to owner only (security)
	os.Chmod(s.cfg.General.SocketPath, 0700)
	s.listener = listener

	logging.Info("listening", "socket", s.cfg.General.SocketPath)

	for {
		conn, err := listener.Accept()
		if err != nil {
			return err
		}
		go s.handleConnection(conn)
	}
}

func (s *Server) Stop() {
	if s.listener != nil {
		s.listener.Close()
	}
	os.Remove(s.cfg.General.SocketPath)
}

func (s *Server) handleConnection(conn net.Conn) {
	defer conn.Close()

	// Set read deadline to prevent DoS from slow/stalled clients
	conn.SetReadDeadline(time.Now().Add(socketReadTimeout))

	reader := bufio.NewReader(conn)
	line, err := reader.ReadString('\n')
	if err != nil {
		logging.Info("connection read error: %v", err)
		return
	}

	// Limit command length
	if len(line) > maxCommandLength {
		fmt.Fprintf(conn, "ERR command too long\n")
		return
	}

	line = strings.TrimSpace(line)

	// Security: reject commands with embedded newlines (protocol injection)
	if strings.ContainsAny(line, "\r\n") {
		fmt.Fprintf(conn, "ERR invalid command\n")
		return
	}

	parts := strings.SplitN(line, " ", 2)
	cmd := parts[0]
	args := ""
	if len(parts) > 1 {
		args = parts[1]
	}

	// Clear read deadline for response
	conn.SetReadDeadline(time.Time{})

	response := s.handleCommand(cmd, args)
	fmt.Fprintf(conn, "%s\n", response)
}

func (s *Server) handleCommand(cmd, args string) string {
	s.mu.Lock()
	defer s.mu.Unlock()

	switch cmd {
	case "health":
		return s.cmdHealth()
	case "status":
		return s.cmdStatus()
	case "project":
		return s.cmdProject(args)
	case "projects":
		return s.cmdProjects()
	case "debt":
		return s.cmdDebt()
	case "sprints":
		return s.cmdSprints()
	case "sprint":
		return s.cmdSprint(args)
	case "grade":
		return s.cmdGrade(args)
	case "profile":
		return s.cmdProfile()
	case "speak":
		return s.cmdSpeak(args)
	case "import":
		return s.cmdImport(args)
	// P1 Analytics commands
	case "review":
		return s.cmdReview(args)
	case "stats":
		return s.cmdStats(args)
	case "journal":
		return s.cmdJournal(args)
	case "hard":
		return s.cmdHard(args)
	case "knowledge":
		return s.cmdKnowledge()
	// Gamification commands
	case "avatar":
		return s.cmdAvatar(args)
	case "wallet":
		return s.cmdWallet()
	case "daily":
		return s.cmdDaily()
	case "challenges":
		return s.cmdChallenges()
	case "goals":
		return s.cmdGoals()
	case "achievements":
		return s.cmdAchievements()
	case "quit":
		go func() {
			s.db.EndSession()
			s.Stop()
			time.Sleep(100 * time.Millisecond) // Let writes complete
			os.Exit(0)
		}()
		return "OK"
	default:
		return "ERR unknown command"
	}
}

// cmdHealth returns detailed health status
func (s *Server) cmdHealth() string {
	// Basic health check - verify DB is accessible
	_, err := s.db.GetProfile()
	if err != nil {
		logging.Info("health check failed: db error: %v", err)
		return "ERR unhealthy"
	}
	return "OK"
}

func (s *Server) cmdStatus() string {
	profile, err := s.db.GetProfile()
	if err != nil {
		logging.Info("status: get profile failed: %v", err)
		return "ERR internal error"
	}

	debt := 0
	projectName := "(none)"
	pendingSprints := 0

	if s.activeProject != "" {
		debt, _ = s.db.GetDebt(s.activeProject)
		if p, err := s.db.GetProject(s.activeProject); err == nil {
			projectName = p.Name
		}
		if sprints, err := s.db.GetSprints(s.activeProject); err == nil {
			for _, sp := range sprints {
				if sp.Status == "pending" {
					pendingSprints++
				}
			}
		}
	}

	return fmt.Sprintf("OK project=%s debt=%d/%d level=%d xp=%d streak=%d pending=%d",
		projectName, debt, s.cfg.KnowledgeDebt.Threshold,
		profile.Level, profile.TotalXP, profile.CurrentStreak, pendingSprints)
}

func (s *Server) cmdProject(args string) string {
	if args == "" {
		if s.activeProject == "" {
			return "OK (none)"
		}
		p, err := s.db.GetProject(s.activeProject)
		if err != nil {
			logging.Info("project: get project failed: %v", err)
			return "ERR project not found"
		}
		return fmt.Sprintf("OK %s %s", p.ID, p.Name)
	}

	// Security: validate path doesn't contain newlines
	if strings.ContainsAny(args, "\r\n") {
		return "ERR invalid path"
	}

	// Set active project
	p, err := s.db.GetOrCreateProject(args)
	if err != nil {
		logging.Info("project: create project failed: %v", err)
		return "ERR failed to create project"
	}
	s.activeProject = p.ID

	// Log project activation
	s.db.LogEvent(db.EventProjectActivated, &p.ID, nil, map[string]interface{}{
		"name": p.Name,
		"path": p.Path,
	})

	return fmt.Sprintf("OK %s %s", p.ID, p.Name)
}

func (s *Server) cmdProjects() string {
	projects, err := s.db.ListProjects()
	if err != nil {
		logging.Info("projects: list failed: %v", err)
		return "ERR failed to list projects"
	}

	var lines []string
	for _, p := range projects {
		lines = append(lines, fmt.Sprintf("%s %s", p.ID, p.Name))
	}
	return "OK\n" + strings.Join(lines, "\n")
}

func (s *Server) cmdDebt() string {
	if s.activeProject == "" {
		return "ERR no active project"
	}
	debt, err := s.db.GetDebt(s.activeProject)
	if err != nil {
		logging.Info("debt: get debt failed: %v", err)
		return "ERR failed to get debt"
	}
	locked := debt >= s.cfg.KnowledgeDebt.Threshold
	return fmt.Sprintf("OK %d/%d locked=%v", debt, s.cfg.KnowledgeDebt.Threshold, locked)
}

func (s *Server) cmdSprints() string {
	if s.activeProject == "" {
		return "ERR no active project"
	}

	sprints, err := s.db.GetSprints(s.activeProject)
	if err != nil {
		logging.Info("sprints: get sprints failed: %v", err)
		return "ERR failed to get sprints"
	}

	var lines []string
	for _, sp := range sprints {
		score := 0
		if sp.BestScore != nil {
			score = *sp.BestScore
		}
		// Use tab separator for easier parsing
		lines = append(lines, fmt.Sprintf("%d\t%s\t%s\t%d\t%d",
			sp.SprintNumber, sp.Topic, sp.Status, score, sp.Attempts))
	}
	return "OK\n" + strings.Join(lines, "\n")
}

func (s *Server) cmdSprint(args string) string {
	if s.activeProject == "" {
		return "ERR no active project"
	}

	num, err := strconv.Atoi(args)
	if err != nil || num < 1 {
		return "ERR invalid sprint number"
	}

	sprint, err := s.db.GetSprint(s.activeProject, num)
	if err != nil {
		logging.Info("sprint: get sprint %d failed: %v", num, err)
		return "ERR sprint not found"
	}

	// Return questions as JSON
	return "OK " + sprint.QuestionsJSON
}

func (s *Server) cmdGrade(args string) string {
	if s.activeProject == "" {
		return "ERR no active project"
	}

	// Parse: <sprint_num> <answers JSON>
	parts := strings.SplitN(args, " ", 2)
	if len(parts) != 2 {
		return "ERR usage: grade <sprint> <answers_json>"
	}

	num, err := strconv.Atoi(parts[0])
	if err != nil || num < 1 {
		return "ERR invalid sprint number"
	}

	var answers []string
	if err := json.Unmarshal([]byte(parts[1]), &answers); err != nil {
		return "ERR invalid answers JSON"
	}

	sprint, err := s.db.GetSprint(s.activeProject, num)
	if err != nil {
		logging.Info("grade: get sprint %d failed: %v", num, err)
		return "ERR sprint not found"
	}

	result, err := exam.GradeSprint(sprint, answers, s.cfg.Grading.PassThreshold)
	if err != nil {
		logging.Info("grade: grading failed: %v", err)
		return "ERR grading failed"
	}

	// Record attempt
	answersJSON := exam.AnswersToJSON(answers)
	if err := s.db.RecordAttempt(sprint.ID, answersJSON, result.ScorePercent, result.Passed, result.XPEarned); err != nil {
		logging.Info("grade: record attempt failed: %v", err)
		return "ERR failed to record attempt"
	}

	// Update profile
	streakDelta := 0
	if result.Passed {
		streakDelta = 1
		// Clear debt
		s.db.ClearDebt(s.activeProject, s.cfg.KnowledgeDebt.DebtPerSprintCleared)
	}
	s.db.UpdateProfile(result.XPEarned, streakDelta, result.Passed)

	// Log sprint completion event
	s.db.LogEvent(db.EventSprintCompleted, &s.activeProject, &sprint.ID, map[string]interface{}{
		"sprint_number": num,
		"score":         result.ScorePercent,
		"correct":       result.CorrectCount,
		"total":         result.TotalQuestions,
	})

	// Log pass/fail event
	if result.Passed {
		s.db.LogEvent(db.EventSprintPassed, &s.activeProject, &sprint.ID, map[string]interface{}{
			"xp_earned": result.XPEarned,
			"score":     result.ScorePercent,
		})
	} else {
		s.db.LogEvent(db.EventSprintFailed, &s.activeProject, &sprint.ID, map[string]interface{}{
			"score": result.ScorePercent,
		})
	}

	// Record per-question stats for analytics
	for i, answer := range answers {
		correct := i < len(result.QuestionResults) && result.QuestionResults[i].Correct
		s.db.RecordQuestionAttempt(sprint.ID, i+1, "", correct, 0, answer)

		// Log individual question answered event
		s.db.LogEvent(db.EventQuestionAnswered, &s.activeProject, &sprint.ID, map[string]interface{}{
			"question_number": i + 1,
			"correct":         correct,
			"answer":          answer,
		})
	}

	// Return result as JSON
	resultJSON, _ := json.Marshal(result)
	return "OK " + string(resultJSON)
}

func (s *Server) cmdProfile() string {
	profile, err := s.db.GetProfile()
	if err != nil {
		logging.Info("profile: get profile failed: %v", err)
		return "ERR internal error"
	}
	return fmt.Sprintf("OK level=%d xp=%d streak=%d best_streak=%d sprints=%d",
		profile.Level, profile.TotalXP, profile.CurrentStreak, profile.BestStreak, profile.SprintsPassed)
}

func (s *Server) cmdSpeak(args string) string {
	if err := s.voice.Speak(args); err != nil {
		logging.Info("speak: voice failed: %v", err)
		return "ERR voice unavailable"
	}
	return "OK"
}

func (s *Server) cmdImport(args string) string {
	if args == "" {
		return "ERR usage: import <exam_file_path>"
	}

	// Security: validate path doesn't contain newlines
	if strings.ContainsAny(args, "\r\n") {
		return "ERR invalid path"
	}

	// Resolve to absolute path and validate
	absPath, err := filepath.Abs(args)
	if err != nil {
		return "ERR invalid path"
	}

	// Resolve symlinks to get canonical path
	realPath, err := filepath.EvalSymlinks(absPath)
	if err != nil {
		// File may not exist yet, or symlink is broken
		logging.Info("import: symlink resolution failed: %v", err)
		return "ERR file not found"
	}

	// Security: only allow importing from user home directory
	homeDir, err := os.UserHomeDir()
	if err != nil {
		logging.Info("import: failed to get home dir: %v", err)
		return "ERR internal error"
	}

	// Strict validation: must be under user's home directory
	if !strings.HasPrefix(realPath, homeDir+"/") {
		logging.Info("import: path %s not under home directory", realPath)
		return "ERR path not allowed"
	}

	// Validate file exists and is a regular file
	info, err := os.Stat(realPath)
	if err != nil {
		logging.Info("import: stat failed: %v", err)
		return "ERR file not found"
	}
	if !info.Mode().IsRegular() {
		return "ERR not a regular file"
	}

	// Only allow .md files
	if !strings.HasSuffix(strings.ToLower(realPath), ".md") {
		return "ERR only .md files allowed"
	}

	// Size limit: 1MB max
	if info.Size() > 1024*1024 {
		return "ERR file too large (max 1MB)"
	}

	content, err := os.ReadFile(realPath)
	if err != nil {
		logging.Info("import: read failed: %v", err)
		return "ERR failed to read file"
	}

	sprints, err := exam.ParseExamFile(string(content))
	if err != nil {
		logging.Info("import: parse failed: %v", err)
		return "ERR failed to parse exam file"
	}

	if len(sprints) == 0 {
		return "ERR no sprints found in file"
	}

	// Determine project from file path
	projectPath := filepath.Dir(realPath)
	project, err := s.db.GetOrCreateProject(projectPath)
	if err != nil {
		logging.Info("import: create project failed: %v", err)
		return "ERR failed to create project"
	}

	// Import sprints
	imported := 0
	for _, ps := range sprints {
		dbSprint, err := ps.ToDBSprint(project.ID)
		if err != nil {
			logging.Info("import: sprint conversion failed: %v", err)
			continue
		}
		if err := s.db.UpsertSprint(dbSprint); err != nil {
			logging.Info("import: sprint upsert failed: %v", err)
			continue
		}
		imported++
	}

	// Log exam import event
	s.db.LogEvent(db.EventExamImported, &project.ID, nil, map[string]interface{}{
		"file":            realPath,
		"sprints_count":   imported,
		"total_questions": imported * 3,
	})

	return fmt.Sprintf("OK imported %d sprints for %s", imported, project.Name)
}

// ============================================================================
// NEW P1 COMMANDS - Analytics and Knowledge Tracking
// ============================================================================

// cmdReview returns knowledge items due for review
func (s *Server) cmdReview(args string) string {
	if s.activeProject == "" {
		return "ERR no active project"
	}

	limit := 5
	if args != "" {
		if n, err := strconv.Atoi(args); err == nil && n > 0 && n <= 50 {
			limit = n
		}
	}

	items, err := s.db.GetKnowledgeItemsForReview(s.activeProject, limit)
	if err != nil {
		logging.Info("review: get items failed: %v", err)
		return "ERR internal error"
	}

	if len(items) == 0 {
		return "OK no items due for review"
	}

	// Format as JSON for easy parsing
	data, _ := json.Marshal(items)
	return "OK " + string(data)
}

// cmdStats returns daily statistics
func (s *Server) cmdStats(args string) string {
	// Default to today
	date := time.Now().Format("2006-01-02")
	endDate := date

	if args != "" {
		if args == "week" {
			// Last 7 days
			endDate = date
			date = time.Now().AddDate(0, 0, -6).Format("2006-01-02")
		} else {
			// Specific date
			if _, err := time.Parse("2006-01-02", args); err == nil {
				date = args
				endDate = args
			}
		}
	}

	stats, err := s.db.GetDailyStats(date, endDate)
	if err != nil {
		logging.Info("stats: get daily stats failed: %v", err)
		return "ERR internal error"
	}

	if len(stats) == 0 {
		return "OK no stats for this period"
	}

	data, _ := json.Marshal(stats)
	return "OK " + string(data)
}

// cmdJournal returns recent journal entries
func (s *Server) cmdJournal(args string) string {
	limit := 20
	var eventTypes []string

	// Parse args: [limit] [--type event_type]
	parts := strings.Fields(args)
	for i := 0; i < len(parts); i++ {
		if parts[i] == "--type" && i+1 < len(parts) {
			eventTypes = append(eventTypes, parts[i+1])
			i++
		} else if n, err := strconv.Atoi(parts[i]); err == nil && n > 0 && n <= 100 {
			limit = n
		}
	}

	entries, err := s.db.GetJournalEntries(limit, eventTypes)
	if err != nil {
		logging.Info("journal: get entries failed: %v", err)
		return "ERR internal error"
	}

	if len(entries) == 0 {
		return "OK no journal entries"
	}

	data, _ := json.Marshal(entries)
	return "OK " + string(data)
}

// cmdHard returns the hardest questions by accuracy
func (s *Server) cmdHard(args string) string {
	limit := 10
	if args != "" {
		if n, err := strconv.Atoi(args); err == nil && n > 0 && n <= 50 {
			limit = n
		}
	}

	questions, err := s.db.GetHardestQuestions(limit)
	if err != nil {
		logging.Info("hard: get questions failed: %v", err)
		return "ERR internal error"
	}

	if len(questions) == 0 {
		return "OK no question stats yet"
	}

	data, _ := json.Marshal(questions)
	return "OK " + string(data)
}

// cmdKnowledge returns knowledge mastery statistics
func (s *Server) cmdKnowledge() string {
	if s.activeProject == "" {
		return "ERR no active project"
	}

	stats, err := s.db.GetKnowledgeStats(s.activeProject)
	if err != nil {
		logging.Info("knowledge: get stats failed: %v", err)
		return "ERR internal error"
	}

	data, _ := json.Marshal(stats)
	return "OK " + string(data)
}

// ============================================================================
// GAMIFICATION COMMANDS
// ============================================================================

// cmdAvatar handles avatar operations: get status or set creature type
func (s *Server) cmdAvatar(args string) string {
	sqlDB := s.db.DB

	if args == "" {
		// Get avatar status
		avatar, err := gamification.GetAvatar(sqlDB)
		if err != nil {
			logging.Info("avatar: get failed: %v", err)
			return "ERR internal error"
		}

		data, _ := json.Marshal(avatar)
		return "OK " + string(data)
	}

	// Set creature type: avatar set <type>
	parts := strings.SplitN(args, " ", 2)
	if parts[0] == "set" && len(parts) > 1 {
		creatureType := parts[1]
		if err := gamification.SetCreatureType(sqlDB, creatureType); err != nil {
			logging.Info("avatar: set type failed: %v", err)
			return "ERR " + err.Error()
		}

		// Log event
		s.db.LogEvent("avatar_changed", nil, nil, map[string]interface{}{
			"creature_type": creatureType,
		})

		return "OK creature set to " + creatureType
	}

	return "ERR usage: avatar [set <cat|slime|octopus|snail>]"
}

// cmdWallet returns current wallet balance and stats
func (s *Server) cmdWallet() string {
	sqlDB := s.db.DB

	wallet, err := gamification.GetWallet(sqlDB)
	if err != nil {
		logging.Info("wallet: get failed: %v", err)
		return "ERR internal error"
	}

	data, _ := json.Marshal(wallet)
	return "OK " + string(data)
}

// cmdDaily handles daily login reward
func (s *Server) cmdDaily() string {
	sqlDB := s.db.DB

	// Get status first
	dl, err := gamification.GetDailyLogin(sqlDB)
	if err != nil {
		logging.Info("daily: get status failed: %v", err)
		return "ERR internal error"
	}

	if !dl.CanClaim {
		// Return status without claiming
		data, _ := json.Marshal(dl)
		return "OK " + string(data)
	}

	// Claim reward
	coins, err := gamification.ClaimDailyReward(sqlDB)
	if err != nil {
		logging.Info("daily: claim failed: %v", err)
		return "ERR " + err.Error()
	}

	// Log event
	s.db.LogEvent("daily_claimed", nil, nil, map[string]interface{}{
		"coins":       coins,
		"current_day": dl.CurrentDay + 1,
	})

	// Get updated status
	dl, _ = gamification.GetDailyLogin(sqlDB)
	response := map[string]interface{}{
		"claimed":      true,
		"coins_earned": coins,
		"status":       dl,
	}
	data, _ := json.Marshal(response)
	return "OK " + string(data)
}

// cmdChallenges returns today's daily challenges
func (s *Server) cmdChallenges() string {
	sqlDB := s.db.DB

	challenges, err := gamification.GetDailyChallenges(sqlDB)
	if err != nil {
		logging.Info("challenges: get failed: %v", err)
		return "ERR internal error"
	}

	data, _ := json.Marshal(challenges)
	return "OK " + string(data)
}

// cmdGoals returns this week's goals
func (s *Server) cmdGoals() string {
	sqlDB := s.db.DB

	goals, err := gamification.GetWeeklyGoals(sqlDB)
	if err != nil {
		logging.Info("goals: get failed: %v", err)
		return "ERR internal error"
	}

	data, _ := json.Marshal(goals)
	return "OK " + string(data)
}

// cmdAchievements returns all achievements with unlock status
func (s *Server) cmdAchievements() string {
	sqlDB := s.db.DB

	achievements, err := gamification.GetAllAchievements(sqlDB)
	if err != nil {
		logging.Info("achievements: get failed: %v", err)
		return "ERR internal error"
	}

	// Also get counts
	unlocked, total, _ := gamification.GetAchievementCount(sqlDB)

	response := map[string]interface{}{
		"achievements": achievements,
		"unlocked":     unlocked,
		"total":        total,
	}
	data, _ := json.Marshal(response)
	return "OK " + string(data)
}
