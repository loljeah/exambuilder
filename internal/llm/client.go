package llm

import (
	"bufio"
	"bytes"
	"context"
	"encoding/json"
	"fmt"
	"io"
	"net/http"
	"strings"
	"time"
)

// OllamaClient communicates with the Ollama HTTP API
type OllamaClient struct {
	baseURL string
	model   string
	timeout time.Duration
}

// NewClient creates a new Ollama client
func NewClient(baseURL, model string, timeoutSeconds int) *OllamaClient {
	if baseURL == "" {
		baseURL = "http://localhost:11434"
	}
	if model == "" {
		model = "llama3.1:8b"
	}
	if timeoutSeconds <= 0 {
		timeoutSeconds = 120
	}
	return &OllamaClient{
		baseURL: strings.TrimRight(baseURL, "/"),
		model:   model,
		timeout: time.Duration(timeoutSeconds) * time.Second,
	}
}

// generateRequest is the Ollama /api/generate request body
type generateRequest struct {
	Model  string `json:"model"`
	Prompt string `json:"prompt"`
	System string `json:"system,omitempty"`
	Stream bool   `json:"stream"`
}

// generateResponse is the Ollama /api/generate response
type generateResponse struct {
	Response string `json:"response"`
	Done     bool   `json:"done"`
}

// modelInfo represents a model from /api/tags
type modelInfo struct {
	Name string `json:"name"`
}

// tagsResponse is the Ollama /api/tags response
type tagsResponse struct {
	Models []modelInfo `json:"models"`
}

// Generate sends a prompt to Ollama and returns the full response text.
// The context allows cancellation (e.g., on app shutdown).
func (c *OllamaClient) Generate(ctx context.Context, system, prompt string) (string, error) {
	body := generateRequest{
		Model:  c.model,
		Prompt: prompt,
		System: system,
		Stream: false,
	}

	jsonBody, err := json.Marshal(body)
	if err != nil {
		return "", fmt.Errorf("marshal request: %w", err)
	}

	req, err := http.NewRequestWithContext(ctx, http.MethodPost, c.baseURL+"/api/generate", bytes.NewReader(jsonBody))
	if err != nil {
		return "", fmt.Errorf("create request: %w", err)
	}
	req.Header.Set("Content-Type", "application/json")

	client := &http.Client{Timeout: c.timeout}
	resp, err := client.Do(req)
	if err != nil {
		return "", fmt.Errorf("ollama request failed: %w", err)
	}
	defer resp.Body.Close()

	if resp.StatusCode != http.StatusOK {
		respBody, _ := io.ReadAll(resp.Body)
		return "", fmt.Errorf("ollama returned status %d: %s", resp.StatusCode, string(respBody))
	}

	respBody, err := io.ReadAll(resp.Body)
	if err != nil {
		return "", fmt.Errorf("read response: %w", err)
	}

	var result generateResponse
	if err := json.Unmarshal(respBody, &result); err != nil {
		return "", fmt.Errorf("unmarshal response: %w", err)
	}

	return result.Response, nil
}

// IsAvailable checks if Ollama is reachable
func (c *OllamaClient) IsAvailable() bool {
	ctx, cancel := context.WithTimeout(context.Background(), 3*time.Second)
	defer cancel()
	req, err := http.NewRequestWithContext(ctx, http.MethodGet, c.baseURL+"/api/tags", nil)
	if err != nil {
		return false
	}
	resp, err := http.DefaultClient.Do(req)
	if err != nil {
		return false
	}
	defer resp.Body.Close()
	return resp.StatusCode == http.StatusOK
}

// ListModels returns available models from Ollama
func (c *OllamaClient) ListModels() ([]string, error) {
	ctx, cancel := context.WithTimeout(context.Background(), 5*time.Second)
	defer cancel()
	req, err := http.NewRequestWithContext(ctx, http.MethodGet, c.baseURL+"/api/tags", nil)
	if err != nil {
		return nil, fmt.Errorf("create request: %w", err)
	}
	resp, err := http.DefaultClient.Do(req)
	if err != nil {
		return nil, fmt.Errorf("list models: %w", err)
	}
	defer resp.Body.Close()

	if resp.StatusCode != http.StatusOK {
		return nil, fmt.Errorf("ollama returned status %d", resp.StatusCode)
	}

	body, err := io.ReadAll(resp.Body)
	if err != nil {
		return nil, fmt.Errorf("read response: %w", err)
	}

	var tags tagsResponse
	if err := json.Unmarshal(body, &tags); err != nil {
		return nil, fmt.Errorf("unmarshal tags: %w", err)
	}

	var models []string
	for _, m := range tags.Models {
		models = append(models, m.Name)
	}
	return models, nil
}

// SetModel changes the active model
func (c *OllamaClient) SetModel(model string) {
	if model != "" {
		c.model = model
	}
}

// Model returns the current model name
func (c *OllamaClient) Model() string {
	return c.model
}

// BaseURL returns the client's base URL
func (c *OllamaClient) BaseURL() string {
	return c.baseURL
}

// PullProgress represents the progress of a model pull operation
type PullProgress struct {
	Status    string  `json:"status"`
	Digest    string  `json:"digest,omitempty"`
	Total     int64   `json:"total,omitempty"`
	Completed int64   `json:"completed,omitempty"`
	Percent   float64 `json:"percent"`
}

// pullRequest is the Ollama /api/pull request body
type pullRequest struct {
	Name   string `json:"name"`
	Stream bool   `json:"stream"`
}

// PullModel pulls a model from Ollama, sending progress updates to the channel.
// The channel is closed when the pull completes or errors.
// The context allows cancellation of the download.
func (c *OllamaClient) PullModel(ctx context.Context, model string, progressCh chan<- PullProgress) error {
	defer close(progressCh)

	body := pullRequest{Name: model, Stream: true}
	jsonBody, err := json.Marshal(body)
	if err != nil {
		return fmt.Errorf("marshal pull request: %w", err)
	}

	req, err := http.NewRequestWithContext(ctx, http.MethodPost, c.baseURL+"/api/pull", bytes.NewReader(jsonBody))
	if err != nil {
		return fmt.Errorf("create request: %w", err)
	}
	req.Header.Set("Content-Type", "application/json")

	// Use a long timeout for large model downloads
	client := &http.Client{Timeout: 30 * time.Minute}
	resp, err := client.Do(req)
	if err != nil {
		return fmt.Errorf("pull request failed: %w", err)
	}
	defer resp.Body.Close()

	if resp.StatusCode != http.StatusOK {
		respBody, _ := io.ReadAll(resp.Body)
		return fmt.Errorf("pull returned status %d: %s", resp.StatusCode, string(respBody))
	}

	// Ollama streams NDJSON lines
	scanner := bufio.NewScanner(resp.Body)
	scanner.Buffer(make([]byte, 64*1024), 64*1024)

	for scanner.Scan() {
		// Check for cancellation between lines
		select {
		case <-ctx.Done():
			return ctx.Err()
		default:
		}

		line := scanner.Bytes()
		if len(line) == 0 {
			continue
		}

		var p PullProgress
		if err := json.Unmarshal(line, &p); err != nil {
			continue
		}

		// Compute percentage
		if p.Total > 0 {
			p.Percent = float64(p.Completed) / float64(p.Total) * 100
		}

		progressCh <- p
	}

	return scanner.Err()
}

// TestGenerate sends a minimal prompt to verify the model responds correctly
func (c *OllamaClient) TestGenerate(ctx context.Context) (string, time.Duration, error) {
	start := time.Now()

	body := generateRequest{
		Model:  c.model,
		Prompt: "Reply with exactly one word: OK",
		Stream: false,
	}

	jsonBody, err := json.Marshal(body)
	if err != nil {
		return "", 0, err
	}

	testCtx, cancel := context.WithTimeout(ctx, 30*time.Second)
	defer cancel()

	req, err := http.NewRequestWithContext(testCtx, http.MethodPost, c.baseURL+"/api/generate", bytes.NewReader(jsonBody))
	if err != nil {
		return "", 0, err
	}
	req.Header.Set("Content-Type", "application/json")

	resp, err := http.DefaultClient.Do(req)
	if err != nil {
		return "", 0, fmt.Errorf("test generate failed: %w", err)
	}
	defer resp.Body.Close()

	if resp.StatusCode != http.StatusOK {
		return "", 0, fmt.Errorf("test returned status %d", resp.StatusCode)
	}

	respBody, err := io.ReadAll(resp.Body)
	if err != nil {
		return "", 0, err
	}

	var result generateResponse
	if err := json.Unmarshal(respBody, &result); err != nil {
		return "", 0, err
	}

	return result.Response, time.Since(start), nil
}
