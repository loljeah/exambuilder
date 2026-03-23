package exam

import (
	"encoding/json"
	"regexp"
	"strconv"
	"strings"

	"github.com/loljeah/exambuilder/internal/db"
)

var (
	sprintHeaderRe   = regexp.MustCompile(`(?m)^## Sprint (\d+):\s*(.+)$`)
	// Match both formats:
	// ### Q1. [RECALL] ⭐⭐ — 10 XP
	// ### Q1. [RECALL] Easy — 10 XP
	questionHeaderRe = regexp.MustCompile(`(?m)^### Q(\d+)\.\s*\[([^\]]+)\]\s*([^—]+)—\s*(\d+)\s*XP`)
	optionRe         = regexp.MustCompile(`(?m)^- ([A-D])\)\s*(.+)$`)
	// Match various answer key formats:
	// **Q1**: B
	// **Q1. Answer: B**
	// Q1: B
	answerKeyRe      = regexp.MustCompile(`(?m)^\*?\*?Q(\d+)[.\s]*(?:Answer)?[:\s]*\*?\*?\s*([A-D])`)
	codeBlockRe      = regexp.MustCompile("(?s)```[a-z]*\n(.+?)\n```")
)

type ParsedSprint struct {
	Number    int
	Topic     string
	Questions []db.Question
	Answers   []string
}

// ParseExamFile parses an exam markdown file into sprints
func ParseExamFile(content string) ([]ParsedSprint, error) {
	// Split by sprint headers
	sprintMatches := sprintHeaderRe.FindAllStringSubmatchIndex(content, -1)
	if len(sprintMatches) == 0 {
		return nil, nil
	}

	var sprints []ParsedSprint

	for i, match := range sprintMatches {
		sprintNum, _ := strconv.Atoi(content[match[2]:match[3]])
		topic := strings.TrimSpace(content[match[4]:match[5]])

		// Get sprint content (until next sprint or answer key)
		start := match[1]
		end := len(content)
		if i+1 < len(sprintMatches) {
			end = sprintMatches[i+1][0]
		}
		// Also stop at answer key (handle both formats)
		if answerKeyIdx := strings.Index(content[start:end], "## 🔑 Answer Key"); answerKeyIdx != -1 {
			end = start + answerKeyIdx
		} else if answerKeyIdx := strings.Index(content[start:end], "## Answer Key"); answerKeyIdx != -1 {
			end = start + answerKeyIdx
		}

		sprintContent := content[start:end]

		sprint := ParsedSprint{
			Number: sprintNum,
			Topic:  topic,
		}

		// Parse questions
		sprint.Questions = parseQuestions(sprintContent)
		sprints = append(sprints, sprint)
	}

	// Parse answer key (handle both "## 🔑 Answer Key" and "## Answer Key")
	answerKeyIdx := strings.Index(content, "## 🔑 Answer Key")
	if answerKeyIdx == -1 {
		answerKeyIdx = strings.Index(content, "## Answer Key")
	}
	if answerKeyIdx != -1 {
		answerKeyContent := content[answerKeyIdx:]
		// Parse answers per sprint
		sprintAnswers := parseAnswerKeyBySprint(answerKeyContent)

		// Assign answers to sprints
		for i := range sprints {
			sprintNum := sprints[i].Number
			if answers, ok := sprintAnswers[sprintNum]; ok {
				sprints[i].Answers = answers
				for j := range sprints[i].Questions {
					if j < len(answers) {
						sprints[i].Questions[j].CorrectIdx = letterToIdx(answers[j])
					}
				}
			}
		}
	}

	return sprints, nil
}

func parseQuestions(content string) []db.Question {
	var questions []db.Question

	qMatches := questionHeaderRe.FindAllStringSubmatchIndex(content, -1)

	for i, match := range qMatches {
		qNum, _ := strconv.Atoi(content[match[2]:match[3]])
		tier := content[match[4]:match[5]]
		difficultyStr := strings.TrimSpace(content[match[6]:match[7]])
		// Convert difficulty string to stars count
		stars := difficultyToStars(difficultyStr)
		xp, _ := strconv.Atoi(content[match[8]:match[9]])

		// Get question content until next question
		start := match[1]
		end := len(content)
		if i+1 < len(qMatches) {
			end = qMatches[i+1][0]
		}
		qContent := content[start:end]

		q := db.Question{
			Number: qNum,
			Tier:   tier,
			Stars:  stars,
			XP:     xp,
		}

		// Extract code block if present
		if codeMatch := codeBlockRe.FindStringSubmatch(qContent); len(codeMatch) > 1 {
			q.Code = strings.TrimSpace(codeMatch[1])
			// Remove code block from content for text parsing
			qContent = codeBlockRe.ReplaceAllString(qContent, "")
		}

		// Extract question text (first non-empty paragraph after header)
		lines := strings.Split(qContent, "\n")
		var textLines []string
		for _, line := range lines {
			line = strings.TrimSpace(line)
			if line == "" || strings.HasPrefix(line, "- ") || strings.HasPrefix(line, "🎙️") {
				continue
			}
			if strings.HasPrefix(line, "###") {
				continue
			}
			textLines = append(textLines, line)
			break
		}
		q.Text = strings.Join(textLines, " ")

		// Extract options
		optMatches := optionRe.FindAllStringSubmatch(qContent, -1)
		for _, optMatch := range optMatches {
			q.Options = append(q.Options, strings.TrimSpace(optMatch[2]))
		}

		questions = append(questions, q)
	}

	return questions
}

// parseAnswerKeyBySprint parses answer key section into map of sprint -> answers
func parseAnswerKeyBySprint(content string) map[int][]string {
	result := make(map[int][]string)
	sprintRe := regexp.MustCompile(`(?m)^### Sprint (\d+)`)

	// Find all sprint sections in the answer key
	sprintMatches := sprintRe.FindAllStringSubmatchIndex(content, -1)
	if len(sprintMatches) == 0 {
		// No sprint headers - parse as single sprint (legacy format)
		matches := answerKeyRe.FindAllStringSubmatch(content, -1)
		var answers []string
		for _, m := range matches {
			answers = append(answers, m[2])
		}
		if len(answers) > 0 {
			result[1] = answers
		}
		return result
	}

	for i, match := range sprintMatches {
		sprintNum, _ := strconv.Atoi(content[match[2]:match[3]])

		// Get content until next sprint section
		start := match[1]
		end := len(content)
		if i+1 < len(sprintMatches) {
			end = sprintMatches[i+1][0]
		}
		sprintContent := content[start:end]

		// Parse answers for this sprint
		matches := answerKeyRe.FindAllStringSubmatch(sprintContent, -1)
		var answers []string
		for _, m := range matches {
			answers = append(answers, m[2])
		}
		if len(answers) > 0 {
			result[sprintNum] = answers
		}
	}

	return result
}

func letterToIdx(letter string) int {
	switch strings.ToUpper(letter) {
	case "A":
		return 0
	case "B":
		return 1
	case "C":
		return 2
	case "D":
		return 3
	default:
		return -1
	}
}

// difficultyToStars converts difficulty text to star count
func difficultyToStars(s string) int {
	s = strings.ToLower(strings.TrimSpace(s))
	// Count star emojis if present
	starCount := strings.Count(s, "⭐")
	if starCount > 0 {
		return starCount
	}
	// Map text difficulty to stars
	switch {
	case strings.Contains(s, "easy"):
		return 1
	case strings.Contains(s, "medium"):
		return 2
	case strings.Contains(s, "challenge"), strings.Contains(s, "hard"):
		return 3
	default:
		return 1
	}
}

// ToDBSprint converts a parsed sprint to DB format
func (ps *ParsedSprint) ToDBSprint(projectID string) (*db.Sprint, error) {
	questionsJSON, err := json.Marshal(ps.Questions)
	if err != nil {
		return nil, err
	}

	answerKey := db.AnswerKey{Answers: ps.Answers}
	answerKeyJSON, err := json.Marshal(answerKey)
	if err != nil {
		return nil, err
	}

	// Calculate total XP
	totalXP := 0
	for _, q := range ps.Questions {
		totalXP += q.XP
	}

	return &db.Sprint{
		ProjectID:     projectID,
		SprintNumber:  ps.Number,
		Topic:         ps.Topic,
		QuestionsJSON: string(questionsJSON),
		AnswerKeyJSON: string(answerKeyJSON),
		XPAvailable:   totalXP,
		Status:        "pending",
	}, nil
}
