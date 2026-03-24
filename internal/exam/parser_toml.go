package exam

import (
	"encoding/json"
	"fmt"
	"os"
	"strings"

	"github.com/BurntSushi/toml"
	"github.com/loljeah/exambuilder/internal/db"
)

// ExamFile represents the TOML v2.1 exam format with domain support
type ExamFile struct {
	Meta    ExamMeta `toml:"meta"`
	Sprints []Sprint `toml:"sprint"`
}

// ExamMeta contains exam-level metadata including domains
type ExamMeta struct {
	Project       string              `toml:"project"`
	Version       string              `toml:"version"`
	Generated     string              `toml:"generated"`
	VoiceReady    bool                `toml:"voice_ready"`
	PassThreshold int                 `toml:"pass_threshold"`
	TotalXP       int                 `toml:"total_xp"`
	ContentType   string              `toml:"content_type"`
	Domains       []Domain            `toml:"domain"`
	Achievements  []DomainAchievement `toml:"achievement"`
	DomainLevels  []DomainLevelDef    `toml:"domain_levels"`
}

// Domain represents a knowledge domain within the exam
type Domain struct {
	ID          string `toml:"id"`
	Name        string `toml:"name"`
	Description string `toml:"description"`
	Color       string `toml:"color"`
	Icon        string `toml:"icon"`
	Sprints     []int  `toml:"sprints"`
	TotalXP     int    `toml:"total_xp"`
}

// DomainAchievement represents an auto-generated achievement for a domain
type DomainAchievement struct {
	ID          string `toml:"id"`
	Domain      string `toml:"domain"`
	Name        string `toml:"name"`
	Description string `toml:"description"`
	Condition   string `toml:"condition"`
	XPReward    int    `toml:"xp_reward"`
	Icon        string `toml:"icon"`
}

// DomainLevelDef defines XP thresholds and titles for domain leveling
type DomainLevelDef struct {
	Domain string   `toml:"domain"`
	Levels []int    `toml:"levels"`
	Titles []string `toml:"titles"`
}

// Sprint represents a single sprint with questions
type Sprint struct {
	Number          int        `toml:"number"`
	Topic           string     `toml:"topic"`
	Domain          string     `toml:"domain"`
	Subdomain       string     `toml:"subdomain"`
	TargetMinutes   int        `toml:"target_minutes"`
	VoiceCompatible bool       `toml:"voice_compatible"`
	Questions       []Question `toml:"question"`
}

// Question represents a single exam question
type Question struct {
	Number      int      `toml:"number"`
	Tier        string   `toml:"tier"`
	Difficulty  int      `toml:"difficulty"`
	XP          int      `toml:"xp"`
	Text        string   `toml:"text"`
	Code        string   `toml:"code"`
	Options     []string `toml:"options"`
	Answer      int      `toml:"answer"`       // For single choice (0-3)
	Answers     []int    `toml:"answers"`      // For multi choice [0, 2] means A and C
	Type        string   `toml:"type"`         // "single" (default) or "multi"
	Hint        string   `toml:"hint"`
	Explanation string   `toml:"explanation"`
}

// ParseExamTOML parses a TOML exam file
func ParseExamTOML(path string) (*ExamFile, error) {
	var exam ExamFile
	if _, err := toml.DecodeFile(path, &exam); err != nil {
		return nil, fmt.Errorf("parse TOML: %w", err)
	}

	if err := validateExam(&exam); err != nil {
		return nil, fmt.Errorf("validate: %w", err)
	}

	return &exam, nil
}

// ParseExamTOMLContent parses TOML content from string
func ParseExamTOMLContent(content string) (*ExamFile, error) {
	var exam ExamFile
	if _, err := toml.Decode(content, &exam); err != nil {
		return nil, fmt.Errorf("parse TOML: %w", err)
	}

	if err := validateExam(&exam); err != nil {
		return nil, fmt.Errorf("validate: %w", err)
	}

	return &exam, nil
}

// validateExam checks all exam constraints
func validateExam(e *ExamFile) error {
	if e.Meta.Version == "" {
		return fmt.Errorf("meta.version is required")
	}
	if e.Meta.Project == "" {
		return fmt.Errorf("meta.project is required")
	}
	if e.Meta.PassThreshold <= 0 || e.Meta.PassThreshold > 100 {
		return fmt.Errorf("meta.pass_threshold must be 1-100")
	}

	for _, s := range e.Sprints {
		if err := validateSprint(&s); err != nil {
			return fmt.Errorf("sprint %d: %w", s.Number, err)
		}
	}

	return nil
}

func validateSprint(s *Sprint) error {
	if len(s.Questions) < 3 || len(s.Questions) > 5 {
		return fmt.Errorf("must have 3-5 questions, got %d", len(s.Questions))
	}

	if s.Topic == "" {
		return fmt.Errorf("topic is required")
	}

	// Q1 must be easy (difficulty 1)
	if len(s.Questions) > 0 && s.Questions[0].Difficulty != 1 {
		return fmt.Errorf("Q1 must be difficulty 1 (easy win), got %d", s.Questions[0].Difficulty)
	}

	for i, q := range s.Questions {
		if err := validateQuestion(&q, i+1); err != nil {
			return fmt.Errorf("Q%d: %w", q.Number, err)
		}
	}

	return nil
}

func validateQuestion(q *Question, expectedNum int) error {
	if q.Number != expectedNum {
		return fmt.Errorf("expected number %d, got %d", expectedNum, q.Number)
	}

	validTiers := map[string]bool{
		"RECALL":        true,
		"COMPREHENSION": true,
		"APPLICATION":   true,
		"ANALYSIS":      true,
	}
	if !validTiers[q.Tier] {
		return fmt.Errorf("invalid tier %q", q.Tier)
	}

	if q.Difficulty < 1 || q.Difficulty > 3 {
		return fmt.Errorf("difficulty must be 1-3, got %d", q.Difficulty)
	}

	if q.XP <= 0 {
		return fmt.Errorf("xp must be positive")
	}

	if q.Text == "" {
		return fmt.Errorf("text is required")
	}

	if len(q.Options) != 4 {
		return fmt.Errorf("must have exactly 4 options, got %d", len(q.Options))
	}

	if q.Answer < 0 || q.Answer > 3 {
		return fmt.Errorf("answer must be 0-3, got %d", q.Answer)
	}

	// Check code line count
	if q.Code != "" {
		lines := strings.Count(q.Code, "\n") + 1
		if lines > 8 {
			return fmt.Errorf("code exceeds 8 lines (%d)", lines)
		}
	}

	if q.Hint == "" {
		return fmt.Errorf("hint is required")
	}

	if q.Explanation == "" {
		return fmt.Errorf("explanation is required")
	}

	return nil
}

// ToDBSprints converts parsed exam to database format
func (e *ExamFile) ToDBSprints(projectID string) ([]*db.Sprint, error) {
	var sprints []*db.Sprint

	for _, s := range e.Sprints {
		dbSprint, err := sprintToDBSprint(&s, projectID)
		if err != nil {
			return nil, fmt.Errorf("sprint %d: %w", s.Number, err)
		}
		sprints = append(sprints, dbSprint)
	}

	return sprints, nil
}

func sprintToDBSprint(s *Sprint, projectID string) (*db.Sprint, error) {
	// Convert questions to db format
	var questions []db.Question
	var answers []string
	var hints []string
	var explanations []string
	totalXP := 0

	for _, q := range s.Questions {
		// Determine question type
		qType := q.Type
		if qType == "" {
			qType = "single" // default
		}

		dbQ := db.Question{
			Number:  q.Number,
			Tier:    q.Tier,
			Stars:   q.Difficulty,
			XP:      q.XP,
			Text:    q.Text,
			Code:    q.Code,
			Options: q.Options,
			Type:    qType,
		}

		// Handle single vs multi choice
		var answerStr string
		if qType == "multi" && len(q.Answers) > 0 {
			// Multi-choice: store all correct indices
			dbQ.CorrectIdxs = q.Answers
			dbQ.CorrectIdx = q.Answers[0] // first for compatibility
			// Convert to letters like "A,C"
			var letters []string
			for _, idx := range q.Answers {
				letters = append(letters, string(rune('A'+idx)))
			}
			answerStr = strings.Join(letters, ",")
		} else {
			// Single choice
			dbQ.CorrectIdx = q.Answer
			answerStr = string(rune('A' + q.Answer))
		}

		questions = append(questions, dbQ)
		answers = append(answers, answerStr)
		hints = append(hints, q.Hint)
		explanations = append(explanations, q.Explanation)
		totalXP += q.XP
	}

	// Serialize to JSON
	questionsJSON, err := jsonMarshal(questions)
	if err != nil {
		return nil, err
	}

	answerKey := db.AnswerKey{
		Answers:      answers,
		Hints:        hints,
		Explanations: explanations,
	}
	answerKeyJSON, err := jsonMarshal(answerKey)
	if err != nil {
		return nil, err
	}

	return &db.Sprint{
		ProjectID:     projectID,
		SprintNumber:  s.Number,
		Topic:         s.Topic,
		DomainID:      s.Domain,
		SubdomainID:   s.Subdomain,
		QuestionsJSON: string(questionsJSON),
		AnswerKeyJSON: string(answerKeyJSON),
		XPAvailable:   totalXP,
		Status:        "pending",
	}, nil
}

// GetDomains returns the domain definitions from the exam
func (e *ExamFile) GetDomains() []Domain {
	return e.Meta.Domains
}

// GetDomainAchievements returns the auto-generated achievements
func (e *ExamFile) GetDomainAchievements() []DomainAchievement {
	return e.Meta.Achievements
}

// GetDomainLevels returns the level definitions
func (e *ExamFile) GetDomainLevels() []DomainLevelDef {
	return e.Meta.DomainLevels
}

// IsExamTOML checks if file is TOML format (vs legacy markdown)
func IsExamTOML(path string) bool {
	return strings.HasSuffix(path, ".toml")
}

// LoadExamFile loads either TOML or legacy markdown format
func LoadExamFile(path string) ([]ParsedSprint, error) {
	if IsExamTOML(path) {
		exam, err := ParseExamTOML(path)
		if err != nil {
			return nil, err
		}
		return examToLegacyFormat(exam), nil
	}

	// Legacy markdown format
	content, err := os.ReadFile(path)
	if err != nil {
		return nil, err
	}
	return ParseExamFile(string(content))
}

// examToLegacyFormat converts TOML exam to legacy ParsedSprint format
// for compatibility with existing code
func examToLegacyFormat(e *ExamFile) []ParsedSprint {
	var sprints []ParsedSprint

	for _, s := range e.Sprints {
		ps := ParsedSprint{
			Number: s.Number,
			Topic:  s.Topic,
		}

		for _, q := range s.Questions {
			ps.Questions = append(ps.Questions, db.Question{
				Number:     q.Number,
				Tier:       q.Tier,
				Stars:      q.Difficulty,
				XP:         q.XP,
				Text:       q.Text,
				Code:       q.Code,
				Options:    q.Options,
				CorrectIdx: q.Answer,
			})
			ps.Answers = append(ps.Answers, string(rune('A'+q.Answer)))
		}

		sprints = append(sprints, ps)
	}

	return sprints
}

// Helper for JSON marshaling
func jsonMarshal(v interface{}) ([]byte, error) {
	return json.Marshal(v)
}
