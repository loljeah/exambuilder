package exam

import (
	"encoding/json"
	"strings"

	"github.com/loljeah/exambuilder/internal/db"
)

type QuestionResult struct {
	QuestionNum int
	Correct     bool
	UserAnswer  string
	RightAnswer string
	XPEarned    int
}

type SprintResult struct {
	SprintNum       int
	Topic           string
	Passed          bool
	ScorePercent    int
	CorrectCount    int
	TotalQuestions  int
	XPEarned        int
	XPAvailable     int
	AttemptNumber   int
	QuestionResults []QuestionResult
}

// GradeSprint grades answers against a sprint
func GradeSprint(sprint *db.Sprint, answers []string, passThreshold int) (*SprintResult, error) {
	var questions []db.Question
	if err := json.Unmarshal([]byte(sprint.QuestionsJSON), &questions); err != nil {
		return nil, err
	}

	var answerKey db.AnswerKey
	if err := json.Unmarshal([]byte(sprint.AnswerKeyJSON), &answerKey); err != nil {
		return nil, err
	}

	result := &SprintResult{
		SprintNum:      sprint.SprintNumber,
		Topic:          sprint.Topic,
		TotalQuestions: len(questions),
		XPAvailable:    sprint.XPAvailable,
		AttemptNumber:  sprint.Attempts + 1,
	}

	for i, q := range questions {
		qr := QuestionResult{
			QuestionNum: q.Number,
			XPEarned:    0,
		}

		if i < len(answerKey.Answers) {
			qr.RightAnswer = answerKey.Answers[i]
		}

		if i < len(answers) {
			qr.UserAnswer = normalizeAnswer(answers[i])
			qr.Correct = strings.EqualFold(qr.UserAnswer, qr.RightAnswer)
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

	return result, nil
}

// normalizeAnswer converts various answer formats to single letter
func normalizeAnswer(ans string) string {
	ans = strings.TrimSpace(strings.ToUpper(ans))

	// Direct letter
	if len(ans) == 1 && ans >= "A" && ans <= "D" {
		return ans
	}

	// "Option A", "A)", "A.", etc.
	for _, letter := range []string{"A", "B", "C", "D"} {
		if strings.HasPrefix(ans, letter) {
			return letter
		}
	}

	// Spelled out
	switch {
	case strings.Contains(ans, "ALPHA") || ans == "1" || ans == "FIRST":
		return "A"
	case strings.Contains(ans, "BRAVO") || strings.Contains(ans, "BEE") || ans == "2" || ans == "SECOND":
		return "B"
	case strings.Contains(ans, "CHARLIE") || strings.Contains(ans, "SEE") || ans == "3" || ans == "THIRD":
		return "C"
	case strings.Contains(ans, "DELTA") || strings.Contains(ans, "DEE") || ans == "4" || ans == "FOURTH":
		return "D"
	}

	return ans
}

// ToJSON converts answers to JSON for storage
func AnswersToJSON(answers []string) string {
	data, _ := json.Marshal(answers)
	return string(data)
}
