package llm

import (
	"database/sql"
	"fmt"
	"regexp"
	"strings"

	"github.com/loljeah/exambuilder/internal/exam"
)

// tomlFencePattern matches ```toml ... ``` code blocks
var tomlFencePattern = regexp.MustCompile("(?s)```toml\\s*\n(.*?)```")

// ExtractTOML extracts TOML content from LLM output
// Handles both fenced code blocks and raw TOML output
func ExtractTOML(raw string) (string, error) {
	// Try extracting from code fences first
	matches := tomlFencePattern.FindStringSubmatch(raw)
	if len(matches) >= 2 {
		return strings.TrimSpace(matches[1]), nil
	}

	// Try the whole response as TOML (check for [meta] marker)
	trimmed := strings.TrimSpace(raw)
	if strings.Contains(trimmed, "[meta]") {
		return trimmed, nil
	}

	return "", fmt.Errorf("no valid TOML content found in LLM output")
}

// ValidateGenerated parses and validates LLM-generated TOML exam content
func ValidateGenerated(tomlContent string) (*exam.ExamFile, error) {
	examFile, err := exam.ParseExamTOMLContent(tomlContent)
	if err != nil {
		return nil, fmt.Errorf("validation failed: %w", err)
	}
	return examFile, nil
}

// NextSprintNumber returns the next available sprint number for a project
func NextSprintNumber(db *sql.DB, projectID string) (int, error) {
	var maxNum sql.NullInt64
	err := db.QueryRow(`
		SELECT MAX(sprint_number) FROM sprints WHERE project_id = ?
	`, projectID).Scan(&maxNum)
	if err != nil {
		return 1, nil // Start at 1 if no sprints exist
	}
	if !maxNum.Valid {
		return 1, nil
	}
	return int(maxNum.Int64) + 1, nil
}
