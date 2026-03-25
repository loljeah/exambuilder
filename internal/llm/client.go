package llm

import (
	"bytes"
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

// Generate sends a prompt to Ollama and returns the full response text
func (c *OllamaClient) Generate(system, prompt string) (string, error) {
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

	client := &http.Client{Timeout: c.timeout}
	resp, err := client.Post(c.baseURL+"/api/generate", "application/json", bytes.NewReader(jsonBody))
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
	client := &http.Client{Timeout: 3 * time.Second}
	resp, err := client.Get(c.baseURL + "/api/tags")
	if err != nil {
		return false
	}
	defer resp.Body.Close()
	return resp.StatusCode == http.StatusOK
}

// ListModels returns available models from Ollama
func (c *OllamaClient) ListModels() ([]string, error) {
	client := &http.Client{Timeout: 5 * time.Second}
	resp, err := client.Get(c.baseURL + "/api/tags")
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
