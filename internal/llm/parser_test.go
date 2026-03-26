package llm

import (
	"testing"
)

func TestExtractTOML_FencedBlock(t *testing.T) {
	raw := `Here is the exam content:

` + "```toml" + `
[meta]
title = "Test Exam"
domain = "go"

[[questions]]
text = "What is Go?"
` + "```" + `

Hope this helps!`

	result, err := ExtractTOML(raw)
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}

	if !contains(result, "[meta]") {
		t.Errorf("expected [meta] in result, got: %s", result)
	}
	if !contains(result, "[[questions]]") {
		t.Errorf("expected [[questions]] in result, got: %s", result)
	}
	if contains(result, "Hope this helps") {
		t.Error("result should not contain text outside the fence")
	}
}

func TestExtractTOML_RawTOML(t *testing.T) {
	raw := `[meta]
title = "Test Exam"
domain = "go"

[[questions]]
text = "What is Go?"
`

	result, err := ExtractTOML(raw)
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}

	if !contains(result, "[meta]") {
		t.Errorf("expected [meta] in result, got: %s", result)
	}
}

func TestExtractTOML_NoTOML(t *testing.T) {
	raw := "This is just a regular response with no TOML content."

	_, err := ExtractTOML(raw)
	if err == nil {
		t.Error("expected error for input without TOML")
	}
}

func TestExtractTOML_EmptyInput(t *testing.T) {
	_, err := ExtractTOML("")
	if err == nil {
		t.Error("expected error for empty input")
	}
}

func TestExtractTOML_MultipleFences(t *testing.T) {
	raw := "First fence:\n```toml\n[meta]\ntitle = \"First\"\n```\n\nSecond fence:\n```toml\n[meta]\ntitle = \"Second\"\n```"

	result, err := ExtractTOML(raw)
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}

	// Should extract the first fence
	if !contains(result, "First") {
		t.Errorf("expected first fence content, got: %s", result)
	}
}

func TestExtractTOML_FencePreferredOverRaw(t *testing.T) {
	raw := "[meta]\ntitle = \"Raw\"\n\n```toml\n[meta]\ntitle = \"Fenced\"\n```"

	result, err := ExtractTOML(raw)
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}

	// Fenced content should be preferred
	if !contains(result, "Fenced") {
		t.Errorf("expected fenced content preferred, got: %s", result)
	}
}

func TestExtractTOML_WhitespaceHandling(t *testing.T) {
	raw := "\n\n  ```toml\n  [meta]\n  title = \"Test\"\n  ```\n\n"

	result, err := ExtractTOML(raw)
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}

	if !contains(result, "[meta]") {
		t.Errorf("expected [meta] in result, got: %s", result)
	}
}

func contains(s, substr string) bool {
	return len(s) >= len(substr) && containsCheck(s, substr)
}

func containsCheck(s, substr string) bool {
	for i := 0; i <= len(s)-len(substr); i++ {
		if s[i:i+len(substr)] == substr {
			return true
		}
	}
	return false
}
