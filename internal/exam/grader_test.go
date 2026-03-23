package exam

import (
	"testing"

	"github.com/loljeah/exambuilder/internal/db"
)

func TestNormalizeAnswer(t *testing.T) {
	tests := []struct {
		input    string
		expected string
	}{
		// Direct letters
		{"A", "A"},
		{"a", "A"},
		{"B", "B"},
		{"b", "B"},
		{"C", "C"},
		{"c", "C"},
		{"D", "D"},
		{"d", "D"},

		// With parenthesis/period
		{"A)", "A"},
		{"B.", "B"},
		{"C,", "C"},

		// Option prefix
		{"A option", "A"},
		{"ANSWER B", "A"}, // Contains B but starts with A, so A wins

		// Numbers
		{"1", "A"},
		{"2", "B"},
		{"3", "C"},
		{"4", "D"},

		// Ordinals
		{"FIRST", "A"},
		{"SECOND", "B"},
		{"THIRD", "C"},
		{"FOURTH", "D"},

		// NATO phonetic
		{"ALPHA", "A"},
		{"BRAVO", "B"},
		{"CHARLIE", "C"},
		{"DELTA", "D"},

		// Phonetic sounds
		{"BEE", "B"},
		{"SEE", "C"},
		{"DEE", "D"},

		// With whitespace
		{"  A  ", "A"},
		{" b ", "B"},

		// Mixed case
		{"Alpha", "A"},
		{"bravo", "B"},
		{"Charlie", "C"},
		{"delta", "D"},

		// Unknown - returned as-is (uppercased)
		{"unknown", "UNKNOWN"},
		{"xyz", "XYZ"},
	}

	for _, tt := range tests {
		result := normalizeAnswer(tt.input)
		if result != tt.expected {
			t.Errorf("normalizeAnswer(%q) = %q, expected %q", tt.input, result, tt.expected)
		}
	}
}

func TestGradeSprint_AllCorrect(t *testing.T) {
	sprint := &db.Sprint{
		SprintNumber:  1,
		Topic:         "Test",
		QuestionsJSON: `[{"number":1,"tier":"RECALL","stars":1,"xp":10},{"number":2,"tier":"RECALL","stars":1,"xp":15},{"number":3,"tier":"RECALL","stars":2,"xp":20}]`,
		AnswerKeyJSON: `{"answers":["A","B","C"]}`,
		XPAvailable:   45,
		Attempts:      0,
	}

	result, err := GradeSprint(sprint, []string{"A", "B", "C"}, 60)
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}

	if !result.Passed {
		t.Error("expected passed=true")
	}
	if result.ScorePercent != 100 {
		t.Errorf("expected 100%%, got %d%%", result.ScorePercent)
	}
	if result.CorrectCount != 3 {
		t.Errorf("expected 3 correct, got %d", result.CorrectCount)
	}
	if result.XPEarned != 45 {
		t.Errorf("expected 45 XP, got %d", result.XPEarned)
	}
}

func TestGradeSprint_AllWrong(t *testing.T) {
	sprint := &db.Sprint{
		SprintNumber:  1,
		Topic:         "Test",
		QuestionsJSON: `[{"number":1,"tier":"RECALL","stars":1,"xp":10},{"number":2,"tier":"RECALL","stars":1,"xp":15},{"number":3,"tier":"RECALL","stars":2,"xp":20}]`,
		AnswerKeyJSON: `{"answers":["A","B","C"]}`,
		XPAvailable:   45,
		Attempts:      0,
	}

	result, err := GradeSprint(sprint, []string{"D", "D", "D"}, 60)
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}

	if result.Passed {
		t.Error("expected passed=false")
	}
	if result.ScorePercent != 0 {
		t.Errorf("expected 0%%, got %d%%", result.ScorePercent)
	}
	if result.CorrectCount != 0 {
		t.Errorf("expected 0 correct, got %d", result.CorrectCount)
	}
	if result.XPEarned != 0 {
		t.Errorf("expected 0 XP, got %d", result.XPEarned)
	}
}

func TestGradeSprint_PassThreshold(t *testing.T) {
	sprint := &db.Sprint{
		SprintNumber:  1,
		Topic:         "Test",
		QuestionsJSON: `[{"number":1,"tier":"RECALL","stars":1,"xp":10},{"number":2,"tier":"RECALL","stars":1,"xp":10},{"number":3,"tier":"RECALL","stars":1,"xp":10}]`,
		AnswerKeyJSON: `{"answers":["A","B","C"]}`,
		XPAvailable:   30,
		Attempts:      0,
	}

	tests := []struct {
		answers   []string
		threshold int
		passed    bool
		score     int
	}{
		// 2/3 = 66%
		{[]string{"A", "B", "D"}, 60, true, 66},
		{[]string{"A", "B", "D"}, 67, false, 66},
		// 1/3 = 33%
		{[]string{"A", "D", "D"}, 60, false, 33},
		{[]string{"A", "D", "D"}, 30, true, 33},
		// 3/3 = 100%
		{[]string{"A", "B", "C"}, 100, true, 100},
	}

	for _, tt := range tests {
		result, err := GradeSprint(sprint, tt.answers, tt.threshold)
		if err != nil {
			t.Fatalf("unexpected error: %v", err)
		}

		if result.Passed != tt.passed {
			t.Errorf("answers=%v threshold=%d: passed=%v, expected=%v",
				tt.answers, tt.threshold, result.Passed, tt.passed)
		}
		if result.ScorePercent != tt.score {
			t.Errorf("answers=%v: score=%d%%, expected=%d%%",
				tt.answers, result.ScorePercent, tt.score)
		}
	}
}

func TestGradeSprint_NormalizedAnswers(t *testing.T) {
	sprint := &db.Sprint{
		SprintNumber:  1,
		Topic:         "Test",
		QuestionsJSON: `[{"number":1,"tier":"RECALL","stars":1,"xp":10}]`,
		AnswerKeyJSON: `{"answers":["B"]}`,
		XPAvailable:   10,
		Attempts:      0,
	}

	// All these should match "B"
	correctVariants := []string{
		"B", "b",
		"2", "SECOND",
		"BRAVO", "bravo",
		"BEE",
		"B)",
	}

	for _, answer := range correctVariants {
		result, err := GradeSprint(sprint, []string{answer}, 60)
		if err != nil {
			t.Fatalf("unexpected error: %v", err)
		}

		if !result.QuestionResults[0].Correct {
			t.Errorf("answer %q should be correct (normalized to B)", answer)
		}
	}
}

func TestGradeSprint_PartialAnswers(t *testing.T) {
	sprint := &db.Sprint{
		SprintNumber:  1,
		Topic:         "Test",
		QuestionsJSON: `[{"number":1,"tier":"RECALL","stars":1,"xp":10},{"number":2,"tier":"RECALL","stars":1,"xp":10},{"number":3,"tier":"RECALL","stars":1,"xp":10}]`,
		AnswerKeyJSON: `{"answers":["A","B","C"]}`,
		XPAvailable:   30,
		Attempts:      0,
	}

	// Only 2 answers provided for 3 questions
	result, err := GradeSprint(sprint, []string{"A", "B"}, 60)
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}

	if result.CorrectCount != 2 {
		t.Errorf("expected 2 correct, got %d", result.CorrectCount)
	}
	if result.TotalQuestions != 3 {
		t.Errorf("expected 3 total questions, got %d", result.TotalQuestions)
	}
	// 2/3 = 66%
	if result.ScorePercent != 66 {
		t.Errorf("expected 66%%, got %d%%", result.ScorePercent)
	}
}

func TestGradeSprint_QuestionResults(t *testing.T) {
	sprint := &db.Sprint{
		SprintNumber:  1,
		Topic:         "Test",
		QuestionsJSON: `[{"number":1,"tier":"RECALL","stars":1,"xp":10},{"number":2,"tier":"RECALL","stars":1,"xp":15}]`,
		AnswerKeyJSON: `{"answers":["A","B"]}`,
		XPAvailable:   25,
		Attempts:      2, // Already attempted twice
	}

	result, err := GradeSprint(sprint, []string{"A", "C"}, 60)
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}

	if len(result.QuestionResults) != 2 {
		t.Fatalf("expected 2 question results, got %d", len(result.QuestionResults))
	}

	// First question: correct
	qr1 := result.QuestionResults[0]
	if !qr1.Correct {
		t.Error("Q1 should be correct")
	}
	if qr1.UserAnswer != "A" {
		t.Errorf("Q1 UserAnswer: expected A, got %s", qr1.UserAnswer)
	}
	if qr1.RightAnswer != "A" {
		t.Errorf("Q1 RightAnswer: expected A, got %s", qr1.RightAnswer)
	}
	if qr1.XPEarned != 10 {
		t.Errorf("Q1 XPEarned: expected 10, got %d", qr1.XPEarned)
	}

	// Second question: wrong
	qr2 := result.QuestionResults[1]
	if qr2.Correct {
		t.Error("Q2 should be incorrect")
	}
	if qr2.UserAnswer != "C" {
		t.Errorf("Q2 UserAnswer: expected C, got %s", qr2.UserAnswer)
	}
	if qr2.RightAnswer != "B" {
		t.Errorf("Q2 RightAnswer: expected B, got %s", qr2.RightAnswer)
	}
	if qr2.XPEarned != 0 {
		t.Errorf("Q2 XPEarned: expected 0, got %d", qr2.XPEarned)
	}

	// Check attempt number
	if result.AttemptNumber != 3 {
		t.Errorf("AttemptNumber: expected 3, got %d", result.AttemptNumber)
	}
}

func TestGradeSprint_CaseInsensitive(t *testing.T) {
	sprint := &db.Sprint{
		SprintNumber:  1,
		Topic:         "Test",
		QuestionsJSON: `[{"number":1,"tier":"RECALL","stars":1,"xp":10}]`,
		AnswerKeyJSON: `{"answers":["b"]}`, // lowercase in answer key
		XPAvailable:   10,
		Attempts:      0,
	}

	result, err := GradeSprint(sprint, []string{"B"}, 60) // uppercase answer
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}

	if !result.QuestionResults[0].Correct {
		t.Error("answer should be correct (case insensitive)")
	}
}

func TestAnswersToJSON(t *testing.T) {
	tests := []struct {
		input    []string
		expected string
	}{
		{[]string{"A", "B", "C"}, `["A","B","C"]`},
		{[]string{}, `[]`},
		{nil, `null`},
		{[]string{"A"}, `["A"]`},
	}

	for _, tt := range tests {
		result := AnswersToJSON(tt.input)
		if result != tt.expected {
			t.Errorf("AnswersToJSON(%v) = %q, expected %q", tt.input, result, tt.expected)
		}
	}
}

func TestGradeSprint_EmptyQuestions(t *testing.T) {
	sprint := &db.Sprint{
		SprintNumber:  1,
		Topic:         "Empty",
		QuestionsJSON: `[]`,
		AnswerKeyJSON: `{"answers":[]}`,
		XPAvailable:   0,
		Attempts:      0,
	}

	result, err := GradeSprint(sprint, []string{}, 60)
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}

	if result.TotalQuestions != 0 {
		t.Errorf("expected 0 total questions, got %d", result.TotalQuestions)
	}
	if result.ScorePercent != 0 {
		t.Errorf("expected 0%% for empty quiz, got %d%%", result.ScorePercent)
	}
}
