package daemon

import (
	"bufio"
	"encoding/json"
	"fmt"
	"log"
	"net"
	"os"
	"strconv"
	"strings"
	"sync"

	"github.com/loljeah/exambuilder/internal/config"
	"github.com/loljeah/exambuilder/internal/db"
	"github.com/loljeah/exambuilder/internal/exam"
	"github.com/loljeah/exambuilder/internal/voice"
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
	s.listener = listener

	log.Printf("listening on %s", s.cfg.General.SocketPath)

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

	reader := bufio.NewReader(conn)
	line, err := reader.ReadString('\n')
	if err != nil {
		return
	}

	line = strings.TrimSpace(line)
	parts := strings.SplitN(line, " ", 2)
	cmd := parts[0]
	args := ""
	if len(parts) > 1 {
		args = parts[1]
	}

	response := s.handleCommand(cmd, args)
	fmt.Fprintf(conn, "%s\n", response)
}

func (s *Server) handleCommand(cmd, args string) string {
	s.mu.Lock()
	defer s.mu.Unlock()

	switch cmd {
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
	case "quit":
		go func() {
			s.Stop()
			os.Exit(0)
		}()
		return "OK"
	default:
		return "ERR unknown command"
	}
}

func (s *Server) cmdStatus() string {
	profile, err := s.db.GetProfile()
	if err != nil {
		return "ERR " + err.Error()
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
			return "ERR " + err.Error()
		}
		return fmt.Sprintf("OK %s %s", p.ID, p.Name)
	}

	// Set active project
	p, err := s.db.GetOrCreateProject(args)
	if err != nil {
		return "ERR " + err.Error()
	}
	s.activeProject = p.ID
	return fmt.Sprintf("OK %s %s", p.ID, p.Name)
}

func (s *Server) cmdProjects() string {
	projects, err := s.db.ListProjects()
	if err != nil {
		return "ERR " + err.Error()
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
		return "ERR " + err.Error()
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
		return "ERR " + err.Error()
	}

	var lines []string
	for _, sp := range sprints {
		score := 0
		if sp.BestScore != nil {
			score = *sp.BestScore
		}
		lines = append(lines, fmt.Sprintf("%d %s %s score=%d attempts=%d",
			sp.SprintNumber, sp.Topic, sp.Status, score, sp.Attempts))
	}
	return "OK\n" + strings.Join(lines, "\n")
}

func (s *Server) cmdSprint(args string) string {
	if s.activeProject == "" {
		return "ERR no active project"
	}

	num, err := strconv.Atoi(args)
	if err != nil {
		return "ERR invalid sprint number"
	}

	sprint, err := s.db.GetSprint(s.activeProject, num)
	if err != nil {
		return "ERR " + err.Error()
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
	if err != nil {
		return "ERR invalid sprint number"
	}

	var answers []string
	if err := json.Unmarshal([]byte(parts[1]), &answers); err != nil {
		return "ERR invalid answers JSON"
	}

	sprint, err := s.db.GetSprint(s.activeProject, num)
	if err != nil {
		return "ERR " + err.Error()
	}

	result, err := exam.GradeSprint(sprint, answers, s.cfg.Grading.PassThreshold)
	if err != nil {
		return "ERR " + err.Error()
	}

	// Record attempt
	answersJSON := exam.AnswersToJSON(answers)
	if err := s.db.RecordAttempt(sprint.ID, answersJSON, result.ScorePercent, result.Passed, result.XPEarned); err != nil {
		return "ERR " + err.Error()
	}

	// Update profile
	streakDelta := 0
	if result.Passed {
		streakDelta = 1
		// Clear debt
		s.db.ClearDebt(s.activeProject, s.cfg.KnowledgeDebt.DebtPerSprintCleared)
	}
	s.db.UpdateProfile(result.XPEarned, streakDelta, result.Passed)

	// Return result as JSON
	resultJSON, _ := json.Marshal(result)
	return "OK " + string(resultJSON)
}

func (s *Server) cmdProfile() string {
	profile, err := s.db.GetProfile()
	if err != nil {
		return "ERR " + err.Error()
	}
	return fmt.Sprintf("OK level=%d xp=%d streak=%d best_streak=%d sprints=%d",
		profile.Level, profile.TotalXP, profile.CurrentStreak, profile.BestStreak, profile.SprintsPassed)
}

func (s *Server) cmdSpeak(args string) string {
	if err := s.voice.Speak(args); err != nil {
		return "ERR " + err.Error()
	}
	return "OK"
}
