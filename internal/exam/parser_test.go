package exam

import (
	"testing"
)

func TestParseExamFile_BasicSprint(t *testing.T) {
	content := `# Exam: Test Project

## Sprint 1: Basic Concepts

### Q1. [RECALL] ⭐ — 10 XP

What is 2+2?

- A) 3
- B) 4
- C) 5
- D) 6

### Q2. [COMPREHENSION] ⭐⭐ — 15 XP

Which is a fruit?

- A) Carrot
- B) Apple
- C) Potato
- D) Broccoli

---

## 🔑 Answer Key

### Sprint 1

**Q1**: B
**Q2**: B
`

	sprints, err := ParseExamFile(content)
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}

	if len(sprints) != 1 {
		t.Fatalf("expected 1 sprint, got %d", len(sprints))
	}

	sprint := sprints[0]
	if sprint.Number != 1 {
		t.Errorf("expected sprint number 1, got %d", sprint.Number)
	}
	if sprint.Topic != "Basic Concepts" {
		t.Errorf("expected topic 'Basic Concepts', got '%s'", sprint.Topic)
	}
	if len(sprint.Questions) != 2 {
		t.Errorf("expected 2 questions, got %d", len(sprint.Questions))
	}

	// Check first question
	q1 := sprint.Questions[0]
	if q1.Number != 1 {
		t.Errorf("Q1: expected number 1, got %d", q1.Number)
	}
	if q1.Tier != "RECALL" {
		t.Errorf("Q1: expected tier RECALL, got %s", q1.Tier)
	}
	if q1.Stars != 1 {
		t.Errorf("Q1: expected 1 star, got %d", q1.Stars)
	}
	if q1.XP != 10 {
		t.Errorf("Q1: expected 10 XP, got %d", q1.XP)
	}
	if len(q1.Options) != 4 {
		t.Errorf("Q1: expected 4 options, got %d", len(q1.Options))
	}
	if q1.CorrectIdx != 1 { // B = index 1
		t.Errorf("Q1: expected correct idx 1, got %d", q1.CorrectIdx)
	}

	// Check second question
	q2 := sprint.Questions[1]
	if q2.Stars != 2 {
		t.Errorf("Q2: expected 2 stars, got %d", q2.Stars)
	}
	if q2.XP != 15 {
		t.Errorf("Q2: expected 15 XP, got %d", q2.XP)
	}
}

func TestParseExamFile_MultipleSprints(t *testing.T) {
	content := `# Exam

## Sprint 1: Topic A

### Q1. [RECALL] Easy — 10 XP

Question 1?

- A) A1
- B) B1
- C) C1
- D) D1

## Sprint 2: Topic B

### Q1. [APPLICATION] Hard — 20 XP

Question 2?

- A) A2
- B) B2
- C) C2
- D) D2

## 🔑 Answer Key

### Sprint 1

**Q1**: A

### Sprint 2

**Q1**: C
`

	sprints, err := ParseExamFile(content)
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}

	if len(sprints) != 2 {
		t.Fatalf("expected 2 sprints, got %d", len(sprints))
	}

	if sprints[0].Topic != "Topic A" {
		t.Errorf("Sprint 1 topic: expected 'Topic A', got '%s'", sprints[0].Topic)
	}
	if sprints[1].Topic != "Topic B" {
		t.Errorf("Sprint 2 topic: expected 'Topic B', got '%s'", sprints[1].Topic)
	}

	// Check answers assigned correctly
	if len(sprints[0].Answers) != 1 || sprints[0].Answers[0] != "A" {
		t.Errorf("Sprint 1 answers incorrect: %v", sprints[0].Answers)
	}
	if len(sprints[1].Answers) != 1 || sprints[1].Answers[0] != "C" {
		t.Errorf("Sprint 2 answers incorrect: %v", sprints[1].Answers)
	}
}

func TestParseExamFile_CodeBlock(t *testing.T) {
	content := `## Sprint 1: Code Questions

### Q1. [APPLICATION] ⭐⭐ — 15 XP

What does this function return?

` + "```go\n" + `func add(a, b int) int {
    return a + b
}
` + "```\n" + `

- A) The sum
- B) The product
- C) An error
- D) Nothing

## 🔑 Answer Key

### Sprint 1

**Q1**: A
`

	sprints, err := ParseExamFile(content)
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}

	if len(sprints) == 0 || len(sprints[0].Questions) == 0 {
		t.Fatal("no questions parsed")
	}

	q := sprints[0].Questions[0]
	if q.Code == "" {
		t.Error("expected code block to be extracted")
	}
	if q.Code != "func add(a, b int) int {\n    return a + b\n}" {
		t.Errorf("code block content incorrect: '%s'", q.Code)
	}
}

func TestParseExamFile_TextDifficulty(t *testing.T) {
	content := `## Sprint 1: Difficulty Parsing

### Q1. [RECALL] Easy — 10 XP

Easy question?

- A) A
- B) B
- C) C
- D) D

### Q2. [COMPREHENSION] Medium — 15 XP

Medium question?

- A) A
- B) B
- C) C
- D) D

### Q3. [ANALYSIS] Challenge — 20 XP

Hard question?

- A) A
- B) B
- C) C
- D) D

## 🔑 Answer Key

### Sprint 1

**Q1**: A
**Q2**: B
**Q3**: C
`

	sprints, err := ParseExamFile(content)
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}

	if len(sprints) == 0 || len(sprints[0].Questions) != 3 {
		t.Fatalf("expected 3 questions, got %d", len(sprints[0].Questions))
	}

	tests := []struct {
		idx           int
		expectedStars int
	}{
		{0, 1}, // Easy = 1 star
		{1, 2}, // Medium = 2 stars
		{2, 3}, // Challenge/Hard = 3 stars
	}

	for _, tt := range tests {
		if sprints[0].Questions[tt.idx].Stars != tt.expectedStars {
			t.Errorf("Q%d: expected %d stars, got %d",
				tt.idx+1, tt.expectedStars, sprints[0].Questions[tt.idx].Stars)
		}
	}
}

func TestParseExamFile_EmptyContent(t *testing.T) {
	sprints, err := ParseExamFile("")
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}
	if len(sprints) != 0 {
		t.Errorf("expected 0 sprints for empty content, got %d", len(sprints))
	}
}

func TestParseExamFile_NoAnswerKey(t *testing.T) {
	content := `## Sprint 1: No Answers

### Q1. [RECALL] ⭐ — 10 XP

Question?

- A) A
- B) B
- C) C
- D) D
`

	sprints, err := ParseExamFile(content)
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}

	if len(sprints) != 1 {
		t.Fatalf("expected 1 sprint, got %d", len(sprints))
	}

	if len(sprints[0].Answers) != 0 {
		t.Errorf("expected no answers without answer key, got %v", sprints[0].Answers)
	}
}

func TestParseExamFile_LegacyAnswerKey(t *testing.T) {
	content := `## Sprint 1: Legacy Format

### Q1. [RECALL] ⭐ — 10 XP

Question?

- A) A
- B) B
- C) C
- D) D

## Answer Key

**Q1**: B
`

	sprints, err := ParseExamFile(content)
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}

	if len(sprints) == 0 {
		t.Fatal("no sprints parsed")
	}

	// Legacy format should still parse
	if sprints[0].Questions[0].CorrectIdx != 1 {
		t.Errorf("expected correct idx 1 (B), got %d", sprints[0].Questions[0].CorrectIdx)
	}
}

func TestDifficultyToStars(t *testing.T) {
	tests := []struct {
		input    string
		expected int
	}{
		{"Easy", 1},
		{"easy", 1},
		{"EASY", 1},
		{"Medium", 2},
		{"medium", 2},
		{"Challenge", 3},
		{"Hard", 3},
		{"hard", 3},
		{"⭐", 1},
		{"⭐⭐", 2},
		{"⭐⭐⭐", 3},
		{"unknown", 1}, // Default to 1
		{"", 1},
	}

	for _, tt := range tests {
		result := difficultyToStars(tt.input)
		if result != tt.expected {
			t.Errorf("difficultyToStars(%q) = %d, expected %d", tt.input, result, tt.expected)
		}
	}
}

func TestLetterToIdx(t *testing.T) {
	tests := []struct {
		input    string
		expected int
	}{
		{"A", 0},
		{"a", 0},
		{"B", 1},
		{"b", 1},
		{"C", 2},
		{"c", 2},
		{"D", 3},
		{"d", 3},
		{"E", -1},
		{"", -1},
		{"AB", -1},
	}

	for _, tt := range tests {
		result := letterToIdx(tt.input)
		if result != tt.expected {
			t.Errorf("letterToIdx(%q) = %d, expected %d", tt.input, result, tt.expected)
		}
	}
}

