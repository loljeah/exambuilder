package llm

import (
	"context"
	"database/sql"
	"encoding/json"
	"fmt"
	"sync"
	"time"

	"github.com/loljeah/exambuilder/internal/db"
	"github.com/loljeah/exambuilder/internal/exam"
	"github.com/loljeah/exambuilder/internal/gamification"
)

// GenerationType defines what kind of content to generate
type GenerationType string

const (
	GenSprint       GenerationType = "sprint"
	GenCustomSprint GenerationType = "custom_sprint"
	GenExam         GenerationType = "exam"
	GenChallenge    GenerationType = "challenge"
)

// GenerationGate describes what a user can generate for a domain
type GenerationGate struct {
	DomainID      string
	DomainLevel   int
	CanSprint     bool
	CanCustom     bool
	CanExam       bool
	CanChallenge  bool
	FirstFreeUsed bool
	SprintCost    int
	CustomCost    int
	ExamCost      int
	ChallengeCost int
}

// GenerationResult holds the output of a generation
type GenerationResult struct {
	GenerationID int64
	SprintIDs    []int
	CoinsSpent   int
	Status       string
}

// Generator orchestrates LLM-based exam generation
type Generator struct {
	client      *OllamaClient
	sqlDB       *sql.DB
	examDB      *db.DB
	maxRetries  int
	mu          sync.Mutex // Prevent concurrent generations
	lastGenTime time.Time  // Rate limiting: track last generation
}

const genCooldown = 10 * time.Second // Minimum time between generation requests

// NewGenerator creates a new Generator
func NewGenerator(client *OllamaClient, sqlDB *sql.DB, examDB *db.DB, maxRetries int) *Generator {
	if maxRetries <= 0 {
		maxRetries = 2
	}
	return &Generator{
		client:     client,
		sqlDB:      sqlDB,
		examDB:     examDB,
		maxRetries: maxRetries,
	}
}

// GetGenerationGate returns what generation types are available for a domain
func (g *Generator) GetGenerationGate(projectID, domainID string) (*GenerationGate, error) {
	// Get domain level
	level := 0
	err := g.sqlDB.QueryRow(`
		SELECT COALESCE(level, 0) FROM domain_levels
		WHERE project_id = ? AND domain_id = ?
	`, projectID, domainID).Scan(&level)
	if err != nil && err != sql.ErrNoRows {
		return nil, err
	}

	// Check first-free status
	firstFreeUsed := false
	var ffu int
	err = g.sqlDB.QueryRow(`
		SELECT first_free_used FROM domain_generation_tracking
		WHERE project_id = ? AND domain_id = ?
	`, projectID, domainID).Scan(&ffu)
	if err == nil {
		firstFreeUsed = ffu > 0
	}

	gate := &GenerationGate{
		DomainID:      domainID,
		DomainLevel:   level,
		CanSprint:     level >= 3,
		CanCustom:     level >= 5,
		CanExam:       level >= 8,
		CanChallenge:  level >= 10,
		FirstFreeUsed: firstFreeUsed,
		SprintCost:    50,
		CustomCost:    75,
		ExamCost:      200,
		ChallengeCost: 100,
	}

	return gate, nil
}

// GenerateSprint generates a single sprint for a domain
func (g *Generator) GenerateSprint(ctx context.Context, projectID, domainID, topic string) (*GenerationResult, error) {
	// Enforce rate limit before acquiring the lock
	if err := g.waitForCooldown(ctx); err != nil {
		return nil, err
	}

	g.mu.Lock()
	defer g.mu.Unlock()
	g.lastGenTime = time.Now()

	// Check gate
	gate, err := g.GetGenerationGate(projectID, domainID)
	if err != nil {
		return nil, fmt.Errorf("gate check: %w", err)
	}
	if !gate.CanSprint {
		return nil, fmt.Errorf("domain level %d too low (need 3+)", gate.DomainLevel)
	}

	// Determine cost (first free?)
	cost := gate.SprintCost
	isFree := false
	if !gate.FirstFreeUsed {
		cost = 0
		isFree = true
	}

	// Deduct coins if not free
	if cost > 0 {
		if err := gamification.SpendCoins(g.sqlDB, cost, gamification.ReasonLLMGeneration, fmt.Sprintf("gen_sprint_%s", domainID)); err != nil {
			return nil, fmt.Errorf("insufficient coins: %w", err)
		}
	}

	// Record generation start
	genID, err := g.recordGeneration(projectID, domainID, string(GenSprint), cost)
	if err != nil {
		return nil, err
	}

	// Get domain info and existing topics
	domainName := domainID // Fallback
	var dn string
	if err := g.sqlDB.QueryRow(`SELECT name FROM domains WHERE id = ? AND project_id = ?`, domainID, projectID).Scan(&dn); err == nil {
		domainName = dn
	}

	existingTopics := g.getExistingTopics(projectID, domainID)

	// Get next sprint number
	nextNum, err := NextSprintNumber(g.sqlDB, projectID)
	if err != nil {
		nextNum = 100
	}

	// Get project name
	projectName := projectID
	var pn string
	if err := g.sqlDB.QueryRow(`SELECT name FROM projects WHERE id = ?`, projectID).Scan(&pn); err == nil {
		projectName = pn
	}

	// Build prompt and generate
	systemPrompt, userPrompt := BuildSprintPrompt(projectName, domainID, domainName, topic, nextNum, existingTopics)

	examFile, rawOutput, err := g.generateWithRetries(ctx, systemPrompt, userPrompt)
	if err != nil {
		// Refund coins on failure
		if cost > 0 {
			_ = gamification.AddCoins(g.sqlDB, cost, "generation_refund", fmt.Sprintf("gen_%d", genID))
		}
		g.updateGenerationStatus(genID, "failed", rawOutput, nil)
		return nil, fmt.Errorf("generation failed: %w", err)
	}

	// Insert sprints into database
	dbSprints, err := examFile.ToDBSprints(projectID)
	if err != nil {
		if cost > 0 {
			_ = gamification.AddCoins(g.sqlDB, cost, "generation_refund", fmt.Sprintf("gen_%d", genID))
		}
		g.updateGenerationStatus(genID, "validation_failed", rawOutput, nil)
		return nil, fmt.Errorf("sprint conversion: %w", err)
	}

	var sprintNums []int
	var insertErrors []string
	for _, s := range dbSprints {
		if err := g.examDB.UpsertSprint(s); err != nil {
			insertErrors = append(insertErrors, fmt.Sprintf("sprint %d: %v", s.SprintNumber, err))
			continue
		}
		sprintNums = append(sprintNums, s.SprintNumber)
	}

	// Mark source as generated
	for _, num := range sprintNums {
		_, _ = g.sqlDB.Exec(`UPDATE sprints SET source = 'generated' WHERE project_id = ? AND sprint_number = ?`, projectID, num)
	}

	// Mark first-free as used
	if isFree {
		_, _ = g.sqlDB.Exec(`
			INSERT INTO domain_generation_tracking (project_id, domain_id, first_free_used, total_generations)
			VALUES (?, ?, 1, 1)
			ON CONFLICT(project_id, domain_id) DO UPDATE SET
				first_free_used = 1,
				total_generations = total_generations + 1
		`, projectID, domainID)
	} else {
		_, _ = g.sqlDB.Exec(`
			INSERT INTO domain_generation_tracking (project_id, domain_id, first_free_used, total_generations)
			VALUES (?, ?, 0, 1)
			ON CONFLICT(project_id, domain_id) DO UPDATE SET
				total_generations = total_generations + 1
		`, projectID, domainID)
	}

	status := "completed"
	if len(sprintNums) == 0 && len(insertErrors) > 0 {
		// All sprints failed to insert — refund
		if cost > 0 {
			_ = gamification.AddCoins(g.sqlDB, cost, "generation_refund", fmt.Sprintf("gen_%d", genID))
		}
		status = "insert_failed"
	}
	g.updateGenerationStatus(genID, status, rawOutput, sprintNums)

	return &GenerationResult{
		GenerationID: genID,
		SprintIDs:    sprintNums,
		CoinsSpent:   cost,
		Status:       status,
	}, nil
}

// GenerateExam generates a full exam (3 sprints) for a domain
func (g *Generator) GenerateExam(ctx context.Context, projectID, domainID string) (*GenerationResult, error) {
	if err := g.waitForCooldown(ctx); err != nil {
		return nil, err
	}

	g.mu.Lock()
	defer g.mu.Unlock()
	g.lastGenTime = time.Now()

	gate, err := g.GetGenerationGate(projectID, domainID)
	if err != nil {
		return nil, fmt.Errorf("gate check: %w", err)
	}
	if !gate.CanExam {
		return nil, fmt.Errorf("domain level %d too low (need 8+)", gate.DomainLevel)
	}

	cost := gate.ExamCost
	if err := gamification.SpendCoins(g.sqlDB, cost, gamification.ReasonLLMGeneration, fmt.Sprintf("gen_exam_%s", domainID)); err != nil {
		return nil, fmt.Errorf("insufficient coins: %w", err)
	}

	genID, err := g.recordGeneration(projectID, domainID, string(GenExam), cost)
	if err != nil {
		return nil, err
	}

	domainName := domainID
	var dn string
	if err := g.sqlDB.QueryRow(`SELECT name FROM domains WHERE id = ? AND project_id = ?`, domainID, projectID).Scan(&dn); err == nil {
		domainName = dn
	}

	existingTopics := g.getExistingTopics(projectID, domainID)
	nextNum, _ := NextSprintNumber(g.sqlDB, projectID)

	projectName := projectID
	var pn string
	if err := g.sqlDB.QueryRow(`SELECT name FROM projects WHERE id = ?`, projectID).Scan(&pn); err == nil {
		projectName = pn
	}

	topics := []string{domainName + " Fundamentals", domainName + " Application", domainName + " Advanced"}
	systemPrompt, userPrompt := BuildExamPrompt(projectName, domainID, domainName, topics, nextNum, existingTopics)

	examFile, rawOutput, err := g.generateWithRetries(ctx, systemPrompt, userPrompt)
	if err != nil {
		_ = gamification.AddCoins(g.sqlDB, cost, "generation_refund", fmt.Sprintf("gen_%d", genID))
		g.updateGenerationStatus(genID, "failed", rawOutput, nil)
		return nil, fmt.Errorf("generation failed: %w", err)
	}

	dbSprints, err := examFile.ToDBSprints(projectID)
	if err != nil {
		_ = gamification.AddCoins(g.sqlDB, cost, "generation_refund", fmt.Sprintf("gen_%d", genID))
		g.updateGenerationStatus(genID, "validation_failed", rawOutput, nil)
		return nil, fmt.Errorf("sprint conversion: %w", err)
	}

	var sprintNums []int
	for _, s := range dbSprints {
		if err := g.examDB.UpsertSprint(s); err != nil {
			continue
		}
		sprintNums = append(sprintNums, s.SprintNumber)
	}

	for _, num := range sprintNums {
		_, _ = g.sqlDB.Exec(`UPDATE sprints SET source = 'generated' WHERE project_id = ? AND sprint_number = ?`, projectID, num)
	}

	_, _ = g.sqlDB.Exec(`
		INSERT INTO domain_generation_tracking (project_id, domain_id, first_free_used, total_generations)
		VALUES (?, ?, 0, 1)
		ON CONFLICT(project_id, domain_id) DO UPDATE SET
			total_generations = total_generations + 1
	`, projectID, domainID)

	g.updateGenerationStatus(genID, "completed", rawOutput, sprintNums)

	return &GenerationResult{
		GenerationID: genID,
		SprintIDs:    sprintNums,
		CoinsSpent:   cost,
		Status:       "completed",
	}, nil
}

// waitForCooldown blocks until the rate limit cooldown has elapsed.
func (g *Generator) waitForCooldown(ctx context.Context) error {
	g.mu.Lock()
	wait := genCooldown - time.Since(g.lastGenTime)
	g.mu.Unlock()
	if wait <= 0 {
		return nil
	}
	select {
	case <-ctx.Done():
		return ctx.Err()
	case <-time.After(wait):
		return nil
	}
}

// generateWithRetries attempts generation with retries on validation failure.
// Includes exponential backoff between retries.
func (g *Generator) generateWithRetries(ctx context.Context, systemPrompt, userPrompt string) (*exam.ExamFile, string, error) {
	var lastErr error
	var rawOutput string

	currentSystem := systemPrompt
	currentUser := userPrompt

	for attempt := 0; attempt <= g.maxRetries; attempt++ {
		// Backoff before retries (not before first attempt)
		if attempt > 0 {
			backoff := time.Duration(attempt*2) * time.Second
			select {
			case <-ctx.Done():
				return nil, rawOutput, ctx.Err()
			case <-time.After(backoff):
			}
		}

		rawOutput, lastErr = g.client.Generate(ctx, currentSystem, currentUser)
		if lastErr != nil {
			continue
		}

		tomlContent, err := ExtractTOML(rawOutput)
		if err != nil {
			lastErr = err
			currentSystem, currentUser = BuildRetryPrompt(systemPrompt, userPrompt, err.Error())
			continue
		}

		examFile, err := ValidateGenerated(tomlContent)
		if err != nil {
			lastErr = err
			currentSystem, currentUser = BuildRetryPrompt(systemPrompt, userPrompt, err.Error())
			continue
		}

		return examFile, rawOutput, nil
	}

	return nil, rawOutput, fmt.Errorf("failed after %d attempts: %w", g.maxRetries+1, lastErr)
}

// recordGeneration inserts a generation record
func (g *Generator) recordGeneration(projectID, domainID, genType string, coinsSpent int) (int64, error) {
	result, err := g.sqlDB.Exec(`
		INSERT INTO llm_generations (project_id, domain_id, generation_type, model_used, coins_spent, status)
		VALUES (?, ?, ?, ?, ?, 'generating')
	`, projectID, domainID, genType, g.client.Model(), coinsSpent)
	if err != nil {
		return 0, err
	}
	return result.LastInsertId()
}

// maxRawOutputLen limits stored LLM output to prevent unbounded storage.
const maxRawOutputLen = 64 * 1024 // 64KB

// updateGenerationStatus updates a generation record with results
func (g *Generator) updateGenerationStatus(genID int64, status, rawOutput string, sprintNums []int) {
	// Truncate raw output to prevent unbounded storage
	if len(rawOutput) > maxRawOutputLen {
		rawOutput = rawOutput[:maxRawOutputLen] + "\n... (truncated)"
	}

	sprintJSON := "[]"
	if len(sprintNums) > 0 {
		if b, err := json.Marshal(sprintNums); err == nil {
			sprintJSON = string(b)
		}
	}
	_, _ = g.sqlDB.Exec(`
		UPDATE llm_generations SET status = ?, raw_output = ?, sprint_ids = ? WHERE id = ?
	`, status, rawOutput, sprintJSON, genID)
}

// getExistingTopics returns topics already covered in a domain
func (g *Generator) getExistingTopics(projectID, domainID string) []string {
	rows, err := g.sqlDB.Query(`
		SELECT topic FROM sprints WHERE project_id = ? AND domain_id = ?
	`, projectID, domainID)
	if err != nil {
		return nil
	}
	defer rows.Close()

	var topics []string
	for rows.Next() {
		var t string
		if err := rows.Scan(&t); err == nil {
			topics = append(topics, t)
		}
	}
	return topics
}

// Client returns the underlying Ollama client
func (g *Generator) Client() *OllamaClient {
	return g.client
}
