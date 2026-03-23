package main

import (
	"bufio"
	"encoding/json"
	"fmt"
	"net"
	"os"
	"path/filepath"
	"strconv"
	"strings"
	"time"

	"github.com/loljeah/exambuilder/internal/config"
	"github.com/loljeah/exambuilder/internal/db"
	"github.com/loljeah/exambuilder/internal/exam"
)

func main() {
	if len(os.Args) < 2 {
		printUsage()
		os.Exit(1)
	}

	cfg, err := config.Load()
	if err != nil {
		fmt.Fprintf(os.Stderr, "warning: config load failed: %v\n", err)
	}

	cmd := os.Args[1]
	args := os.Args[2:]

	switch cmd {
	case "status":
		cmdStatus(cfg)
	case "project":
		cmdProject(cfg, args)
	case "projects":
		cmdProjects(cfg)
	case "debt":
		cmdDebt(cfg)
	case "sprints":
		cmdSprints(cfg)
	case "take":
		cmdTake(cfg, args)
	case "profile":
		cmdProfile(cfg)
	case "import":
		cmdImport(cfg, args)
	case "quit":
		cmdQuit(cfg)
	case "help", "-h", "--help":
		printUsage()
	default:
		fmt.Fprintf(os.Stderr, "unknown command: %s\n", cmd)
		printUsage()
		os.Exit(1)
	}
}

func printUsage() {
	fmt.Println(`kgatectl - Knowledge Gate CLI

Usage:
  kgatectl <command> [args]

Commands:
  status              Show current status (debt, profile, project)
  project [path]      Get or set active project
  projects            List all projects
  debt                Show debt details
  sprints             List sprints for active project
  take <sprint>       Take an exam interactively
  profile             Show profile (XP, level, streak)
  quit                Stop the daemon

Options:
  --voice             Enable voice mode (TTS reads questions)
  --voice-full        Full voice mode (TTS + STT for answers)`)
}

func sendCommand(cfg *config.Config, cmd string) string {
	conn, err := net.Dial("unix", cfg.General.SocketPath)
	if err != nil {
		fmt.Fprintf(os.Stderr, "error: cannot connect to daemon at %s\n", cfg.General.SocketPath)
		fmt.Fprintf(os.Stderr, "start it with: kgate-daemon\n")
		os.Exit(1)
	}
	defer conn.Close()

	fmt.Fprintf(conn, "%s\n", cmd)

	// Set reasonable read timeout
	conn.SetReadDeadline(time.Now().Add(10 * time.Second))

	// Read all response data
	var response strings.Builder
	buf := make([]byte, 4096)
	for {
		n, err := conn.Read(buf)
		if n > 0 {
			response.Write(buf[:n])
			// Check if we got complete response (ends with newline)
			if buf[n-1] == '\n' {
				break
			}
		}
		if err != nil {
			break
		}
	}
	return strings.TrimSpace(response.String())
}

func cmdStatus(cfg *config.Config) {
	resp := sendCommand(cfg, "status")
	if !strings.HasPrefix(resp, "OK ") {
		fmt.Println(resp)
		return
	}

	// Parse: OK project=name debt=5/10 level=2 xp=150 streak=3 pending=2
	parts := strings.Fields(resp[3:])
	data := make(map[string]string)
	for _, p := range parts {
		kv := strings.SplitN(p, "=", 2)
		if len(kv) == 2 {
			data[kv[0]] = kv[1]
		}
	}

	fmt.Println("╭─────────────────────────────────────╮")
	fmt.Println("│  Knowledge Gate                     │")
	fmt.Println("├─────────────────────────────────────┤")
	fmt.Printf("│  Project: %-26s │\n", data["project"])
	fmt.Printf("│  Debt: %-29s │\n", data["debt"])
	fmt.Printf("│  Level %s • %s XP • 🔥%s            │\n", data["level"], data["xp"], data["streak"])
	fmt.Printf("│  Pending: %s sprints                 │\n", data["pending"])
	fmt.Println("╰─────────────────────────────────────╯")
}

func cmdProject(cfg *config.Config, args []string) {
	cmd := "project"
	if len(args) > 0 {
		cmd += " " + args[0]
	}
	resp := sendCommand(cfg, cmd)
	fmt.Println(resp)
}

func cmdProjects(cfg *config.Config) {
	resp := sendCommand(cfg, "projects")
	fmt.Println(resp)
}

func cmdDebt(cfg *config.Config) {
	resp := sendCommand(cfg, "debt")
	fmt.Println(resp)
}

func cmdSprints(cfg *config.Config) {
	resp := sendCommand(cfg, "sprints")
	if !strings.HasPrefix(resp, "OK") {
		fmt.Println(resp)
		return
	}

	lines := strings.Split(resp, "\n")
	fmt.Println("Sprints:")
	for _, line := range lines[1:] {
		if line == "" {
			continue
		}
		// Format: num\ttopic\tstatus\tscore\tattempts
		parts := strings.Split(line, "\t")
		if len(parts) >= 3 {
			num := parts[0]
			topic := parts[1]
			status := parts[2]
			icon := "⬜"
			if status == "passed" {
				icon = "✓"
			} else if status == "pending" {
				icon = "○"
			}
			fmt.Printf("  %s Sprint %s: %s\n", icon, num, topic)
		}
	}
}

func cmdProfile(cfg *config.Config) {
	resp := sendCommand(cfg, "profile")
	fmt.Println(resp)
}

func cmdQuit(cfg *config.Config) {
	resp := sendCommand(cfg, "quit")
	fmt.Println(resp)
}

func cmdImport(cfg *config.Config, args []string) {
	if len(args) < 1 {
		fmt.Println("usage: kgatectl import <exam_file.md>")
		os.Exit(1)
	}

	// Get absolute path
	path := args[0]
	if !filepath.IsAbs(path) {
		cwd, _ := os.Getwd()
		path = filepath.Join(cwd, path)
	}

	resp := sendCommand(cfg, "import "+path)
	fmt.Println(resp)
}

func cmdTake(cfg *config.Config, args []string) {
	if len(args) < 1 {
		fmt.Println("usage: kgatectl take <sprint_number>")
		os.Exit(1)
	}

	sprintNum, err := strconv.Atoi(args[0])
	if err != nil {
		fmt.Println("invalid sprint number")
		os.Exit(1)
	}

	// Check for voice flags
	voiceMode := false
	voiceFull := false
	for _, a := range args[1:] {
		if a == "--voice" {
			voiceMode = true
		}
		if a == "--voice-full" {
			voiceFull = true
			voiceMode = true
		}
	}

	// Get sprint questions
	resp := sendCommand(cfg, fmt.Sprintf("sprint %d", sprintNum))
	if !strings.HasPrefix(resp, "OK ") {
		fmt.Println(resp)
		return
	}

	questionsJSON := resp[3:]
	var questions []db.Question
	if err := json.Unmarshal([]byte(questionsJSON), &questions); err != nil {
		fmt.Printf("error parsing questions: %v\n", err)
		return
	}

	if len(questions) == 0 {
		fmt.Println("No questions in this sprint")
		return
	}

	fmt.Printf("\n━━━ Sprint %d ━━━\n\n", sprintNum)

	// Collect answers
	answers := make([]string, len(questions))
	reader := bufio.NewReader(os.Stdin)

	for i, q := range questions {
		// Voice: speak question
		if voiceMode {
			speakQuestion(cfg, &q, i+1)
		}

		// Display question
		fmt.Printf("Q%d. [%s] %s — %d XP\n", q.Number, q.Tier, strings.Repeat("⭐", q.Stars), q.XP)
		fmt.Println(q.Text)
		if q.Code != "" {
			fmt.Printf("\n```\n%s\n```\n", q.Code)
		}
		fmt.Println()
		for j, opt := range q.Options {
			fmt.Printf("  %c) %s\n", 'A'+j, opt)
		}
		fmt.Print("\nYour answer: ")

		var answer string
		if voiceFull {
			// STT via moonshine-daemon
			answer = listenForAnswer(cfg)
			fmt.Printf("%s\n", answer)
		} else {
			answer, _ = reader.ReadString('\n')
		}
		answers[i] = normalizeAnswer(strings.TrimSpace(answer))
		fmt.Println()
	}

	// Grade
	answersJSON, _ := json.Marshal(answers)
	resp = sendCommand(cfg, fmt.Sprintf("grade %d %s", sprintNum, string(answersJSON)))
	if !strings.HasPrefix(resp, "OK ") {
		fmt.Println(resp)
		return
	}

	var result exam.SprintResult
	if err := json.Unmarshal([]byte(resp[3:]), &result); err != nil {
		fmt.Printf("error parsing result: %v\n", err)
		return
	}

	// Display results
	fmt.Println("━━━ Results ━━━")
	fmt.Printf("Score: %d%% (%d/%d)\n", result.ScorePercent, result.CorrectCount, result.TotalQuestions)
	fmt.Printf("XP earned: %d\n", result.XPEarned)

	if result.Passed {
		fmt.Println("\n✓ SPRINT PASSED!")
	} else {
		fmt.Println("\n✗ Not passed. Try again!")
	}

	// Voice: announce result
	if voiceMode {
		speakResult(cfg, result.Passed, result.ScorePercent, result.XPEarned)
	}
}

func speakQuestion(cfg *config.Config, q *db.Question, num int) {
	text := fmt.Sprintf("Question %d. %s. ", num, q.Text)
	for i, opt := range q.Options {
		text += fmt.Sprintf("Option %c: %s. ", 'A'+i, opt)
	}
	sendCommand(cfg, "speak "+text)
}

func speakResult(cfg *config.Config, passed bool, score, xp int) {
	var text string
	if passed {
		text = fmt.Sprintf("Sprint passed! %d percent correct. You earned %d XP.", score, xp)
	} else {
		text = fmt.Sprintf("Not passed. %d percent. Try again when ready.", score)
	}
	sendCommand(cfg, "speak "+text)
}

// listenForAnswer uses moonshine-daemon for STT input
func listenForAnswer(cfg *config.Config) string {
	// Send beep to indicate ready for input
	sendCommand(cfg, "speak Ready")

	// Connect to moonshine-daemon and toggle recording
	conn, err := net.Dial("unix", cfg.Voice.MoonshineSocket)
	if err != nil {
		fmt.Fprintf(os.Stderr, "moonshine not available: %v\n", err)
		return ""
	}
	defer conn.Close()

	// Start recording
	fmt.Fprintf(conn, "toggle\n")

	// Wait for transcription result (blocking)
	var response strings.Builder
	buf := make([]byte, 4096)
	for {
		conn.SetReadDeadline(time.Now().Add(30 * time.Second)) // 30s max recording
		n, err := conn.Read(buf)
		if n > 0 {
			response.Write(buf[:n])
		}
		if err != nil {
			break
		}
	}

	result := strings.TrimSpace(response.String())
	if strings.HasPrefix(result, "OK ") {
		return result[3:]
	}
	return result
}

// normalizeAnswer converts spoken answers like "alpha", "bravo" to A, B, C, D
func normalizeAnswer(input string) string {
	input = strings.ToUpper(strings.TrimSpace(input))

	// Direct letter match
	if len(input) == 1 && input[0] >= 'A' && input[0] <= 'D' {
		return input
	}

	// NATO phonetic alphabet
	switch {
	case strings.HasPrefix(input, "ALPHA"), strings.HasPrefix(input, "ALFA"):
		return "A"
	case strings.HasPrefix(input, "BRAVO"), strings.HasPrefix(input, "B "):
		return "B"
	case strings.HasPrefix(input, "CHARLIE"), strings.HasPrefix(input, "C "):
		return "C"
	case strings.HasPrefix(input, "DELTA"), strings.HasPrefix(input, "D "):
		return "D"
	// Common speech variations
	case strings.Contains(input, "OPTION A"), strings.Contains(input, "ANSWER A"):
		return "A"
	case strings.Contains(input, "OPTION B"), strings.Contains(input, "ANSWER B"):
		return "B"
	case strings.Contains(input, "OPTION C"), strings.Contains(input, "ANSWER C"):
		return "C"
	case strings.Contains(input, "OPTION D"), strings.Contains(input, "ANSWER D"):
		return "D"
	case strings.Contains(input, "FIRST"):
		return "A"
	case strings.Contains(input, "SECOND"):
		return "B"
	case strings.Contains(input, "THIRD"):
		return "C"
	case strings.Contains(input, "FOURTH"), strings.Contains(input, "LAST"):
		return "D"
	}

	return input
}
