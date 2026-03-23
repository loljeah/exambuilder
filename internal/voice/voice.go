package voice

import (
	"bufio"
	"fmt"
	"net"
	"strings"
	"time"

	"github.com/loljeah/exambuilder/internal/config"
	"github.com/loljeah/exambuilder/internal/db"
)

const (
	connectTimeout = 3 * time.Second
	readTimeout    = 30 * time.Second
)

// Client handles TTS and STT via external daemons
type Client struct {
	cfg *config.Config
}

func NewClient(cfg *config.Config) *Client {
	return &Client{cfg: cfg}
}

// sendCommand sends a command to a Unix socket and returns the response
func sendCommand(socketPath, command string) (string, error) {
	// Use dialer with timeout
	dialer := net.Dialer{Timeout: connectTimeout}
	conn, err := dialer.Dial("unix", socketPath)
	if err != nil {
		return "", fmt.Errorf("connect to %s: %w", socketPath, err)
	}
	defer conn.Close()

	// Set read deadline
	conn.SetDeadline(time.Now().Add(readTimeout))

	_, err = fmt.Fprintf(conn, "%s\n", command)
	if err != nil {
		return "", err
	}

	reader := bufio.NewReader(conn)
	response, err := reader.ReadString('\n')
	if err != nil {
		return "", err
	}

	return strings.TrimSpace(response), nil
}

// Speak sends text to piper-daemon for TTS
func (c *Client) Speak(text string) error {
	resp, err := sendCommand(c.cfg.Voice.PiperDaemonSocket, "speak "+text)
	if err != nil {
		return err
	}
	if !strings.HasPrefix(resp, "OK") {
		return fmt.Errorf("piper-daemon: %s", resp)
	}
	return nil
}

// SpeakBlocking sends text and waits for playback to complete
func (c *Client) SpeakBlocking(text string) error {
	resp, err := sendCommand(c.cfg.Voice.PiperDaemonSocket, "speak-blocking "+text)
	if err != nil {
		return err
	}
	if !strings.HasPrefix(resp, "OK") {
		return fmt.Errorf("piper-daemon: %s", resp)
	}
	return nil
}

// StopSpeech stops current TTS playback
func (c *Client) StopSpeech() error {
	_, err := sendCommand(c.cfg.Voice.PiperDaemonSocket, "stop")
	return err
}

// Listen triggers moonshine-daemon STT and returns transcription
func (c *Client) Listen() (string, error) {
	// Toggle recording on, wait for result
	resp, err := sendCommand(c.cfg.Voice.MoonshineSocket, "toggle")
	if err != nil {
		return "", err
	}
	// moonshine returns the transcription after recording stops
	return resp, nil
}

// IsPiperAvailable checks if piper-daemon is running
func (c *Client) IsPiperAvailable() bool {
	resp, err := sendCommand(c.cfg.Voice.PiperDaemonSocket, "status")
	return err == nil && strings.HasPrefix(resp, "OK")
}

// IsMoonshineAvailable checks if moonshine-daemon is running
func (c *Client) IsMoonshineAvailable() bool {
	resp, err := sendCommand(c.cfg.Voice.MoonshineSocket, "status")
	return err == nil && strings.HasPrefix(resp, "OK")
}

// SpeakQuestion reads a question aloud
func (c *Client) SpeakQuestion(q *db.Question, qNum int) error {
	// Build speech text
	var parts []string

	parts = append(parts, fmt.Sprintf("Question %d.", qNum))
	parts = append(parts, q.Text)

	if q.Code != "" {
		parts = append(parts, "Code snippet follows.")
		// Simplify code for speech
		code := strings.ReplaceAll(q.Code, "\n", ". ")
		code = strings.ReplaceAll(code, "{", " open brace ")
		code = strings.ReplaceAll(code, "}", " close brace ")
		parts = append(parts, code)
	}

	for i, opt := range q.Options {
		letter := string(rune('A' + i))
		parts = append(parts, fmt.Sprintf("Option %s: %s.", letter, opt))
	}

	parts = append(parts, "Your answer?")

	return c.SpeakBlocking(strings.Join(parts, " "))
}

// SpeakResult announces if the answer was correct
func (c *Client) SpeakResult(correct bool, xp int) error {
	if correct {
		return c.SpeakBlocking(fmt.Sprintf("Correct! Plus %d XP.", xp))
	}
	return c.SpeakBlocking("Incorrect.")
}

// SpeakSprintResult announces sprint completion
func (c *Client) SpeakSprintResult(passed bool, score int, xp int) error {
	if passed {
		return c.SpeakBlocking(fmt.Sprintf("Sprint passed! %d percent correct. You earned %d XP.", score, xp))
	}
	return c.SpeakBlocking(fmt.Sprintf("Sprint not passed. %d percent correct. Try again when ready.", score))
}
