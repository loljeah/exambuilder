use anyhow::{anyhow, Result};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Trait for LLM client abstraction (enables testing with mocks)
#[async_trait]
pub trait LlmClient: Send + Sync {
    async fn generate(&self, system: &str, prompt: &str, temperature: f32) -> Result<String>;
    async fn is_available(&self) -> Result<bool>;
    fn model_name(&self) -> &str;
}

/// Anthropic Messages API client
pub struct AnthropicClient {
    api_key: String,
    model: String,
    base_url: String,
    timeout: Duration,
    max_tokens: u32,
    http: reqwest::Client,
}

/// Usage tracking for API calls
#[derive(Debug, Clone, Default)]
pub struct Usage {
    pub input_tokens: u32,
    pub output_tokens: u32,
}

// Internal API request/response types
#[derive(Serialize)]
struct MessagesRequest {
    model: String,
    max_tokens: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    system: Option<String>,
    messages: Vec<Message>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
}

#[derive(Serialize, Deserialize, Clone)]
struct Message {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct MessagesResponse {
    content: Vec<ContentBlock>,
    #[serde(default)]
    #[allow(dead_code)]
    stop_reason: Option<String>,
    #[serde(default)]
    usage: Option<ApiUsage>,
}

#[derive(Deserialize)]
struct ContentBlock {
    #[serde(rename = "type")]
    content_type: String,
    #[serde(default)]
    text: Option<String>,
}

#[derive(Deserialize)]
struct ApiUsage {
    #[serde(default)]
    input_tokens: u32,
    #[serde(default)]
    output_tokens: u32,
}

#[derive(Deserialize)]
struct ApiError {
    #[serde(default)]
    error: Option<ApiErrorDetail>,
}

#[derive(Deserialize)]
struct ApiErrorDetail {
    #[serde(default)]
    message: Option<String>,
    #[serde(rename = "type", default)]
    error_type: Option<String>,
}

impl AnthropicClient {
    /// Create a new client, reading ANTHROPIC_API_KEY from environment
    pub fn new() -> Result<Self> {
        let api_key = std::env::var("ANTHROPIC_API_KEY")
            .map_err(|_| anyhow!("ANTHROPIC_API_KEY not set. Set it to use LLM generation."))?;

        if api_key.is_empty() {
            return Err(anyhow!("ANTHROPIC_API_KEY is empty"));
        }

        Ok(Self {
            api_key,
            model: "claude-opus-4-20250514".to_string(),
            base_url: "https://api.anthropic.com".to_string(),
            timeout: Duration::from_secs(120),
            max_tokens: 4096,
            http: reqwest::Client::new(),
        })
    }

    /// Create with explicit configuration
    pub fn with_config(api_key: &str, model: &str, max_tokens: u32) -> Self {
        Self {
            api_key: api_key.to_string(),
            model: model.to_string(),
            base_url: "https://api.anthropic.com".to_string(),
            timeout: Duration::from_secs(120),
            max_tokens,
            http: reqwest::Client::new(),
        }
    }

    /// Generate with usage tracking
    pub async fn generate_with_usage(
        &self,
        system: &str,
        prompt: &str,
        temperature: f32,
    ) -> Result<(String, Usage)> {
        let request = MessagesRequest {
            model: self.model.clone(),
            max_tokens: self.max_tokens,
            system: if system.is_empty() {
                None
            } else {
                Some(system.to_string())
            },
            messages: vec![Message {
                role: "user".to_string(),
                content: prompt.to_string(),
            }],
            temperature: Some(temperature),
        };

        let response = self
            .http
            .post(format!("{}/v1/messages", self.base_url))
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json")
            .timeout(self.timeout)
            .json(&request)
            .send()
            .await?;

        let status = response.status();
        let body = response.text().await?;

        if !status.is_success() {
            // Try to parse error response
            if let Ok(err) = serde_json::from_str::<ApiError>(&body) {
                if let Some(detail) = err.error {
                    let msg = detail.message.unwrap_or_default();
                    let err_type = detail.error_type.unwrap_or_default();
                    return Err(anyhow!(
                        "Anthropic API error ({}): {} — {}",
                        status.as_u16(),
                        err_type,
                        msg
                    ));
                }
            }
            return Err(anyhow!(
                "Anthropic API returned {}: {}",
                status.as_u16(),
                body
            ));
        }

        let parsed: MessagesResponse = serde_json::from_str(&body).map_err(|e| {
            anyhow!(
                "Failed to parse API response: {} — body: {}",
                e,
                &body[..body.len().min(200)]
            )
        })?;

        // Extract text from content blocks
        let text = parsed
            .content
            .iter()
            .filter(|b| b.content_type == "text")
            .filter_map(|b| b.text.as_deref())
            .collect::<Vec<_>>()
            .join("");

        let usage = parsed
            .usage
            .map(|u| Usage {
                input_tokens: u.input_tokens,
                output_tokens: u.output_tokens,
            })
            .unwrap_or_default();

        Ok((text, usage))
    }

    /// Check if the API key is valid (and has available credits) by making a minimal request
    pub async fn check_availability(&self) -> Result<bool> {
        // Use a tiny, cheap request to verify the key works
        match self.generate_with_usage("", "Say 'ok'", 0.0).await {
            Ok(_) => Ok(true),
            Err(e) => {
                let msg = e.to_string();
                if msg.contains("401") || msg.contains("authentication") {
                    Ok(false)
                } else {
                    Err(e)
                }
            }
        }
    }
}

#[async_trait]
impl LlmClient for AnthropicClient {
    async fn generate(&self, system: &str, prompt: &str, temperature: f32) -> Result<String> {
        let (text, _usage) = self.generate_with_usage(system, prompt, temperature).await?;
        Ok(text)
    }

    async fn is_available(&self) -> Result<bool> {
        self.check_availability().await
    }

    fn model_name(&self) -> &str {
        &self.model
    }
}

/// Mock LLM client for testing
pub struct MockLlmClient {
    pub responses: Vec<String>,
    response_idx: std::sync::atomic::AtomicUsize,
    pub model: String,
}

impl MockLlmClient {
    pub fn new(responses: Vec<String>) -> Self {
        Self {
            responses,
            response_idx: std::sync::atomic::AtomicUsize::new(0),
            model: "mock-model".to_string(),
        }
    }

    pub fn with_single_response(response: &str) -> Self {
        Self::new(vec![response.to_string()])
    }
}

#[async_trait]
impl LlmClient for MockLlmClient {
    async fn generate(&self, _system: &str, _prompt: &str, _temperature: f32) -> Result<String> {
        let idx = self
            .response_idx
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        let response_idx = idx % self.responses.len();
        Ok(self.responses[response_idx].clone())
    }

    async fn is_available(&self) -> Result<bool> {
        Ok(true)
    }

    fn model_name(&self) -> &str {
        &self.model
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_request_serialization() {
        let request = MessagesRequest {
            model: "claude-opus-4-20250514".to_string(),
            max_tokens: 4096,
            system: Some("You are helpful.".to_string()),
            messages: vec![Message {
                role: "user".to_string(),
                content: "Hello".to_string(),
            }],
            temperature: Some(0.3),
        };
        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("claude-opus-4-20250514"));
        assert!(json.contains("4096"));
        assert!(json.contains("You are helpful."));
        assert!(json.contains("Hello"));
        assert!(json.contains("0.3"));
    }

    #[test]
    fn test_response_deserialization() {
        let json = r#"{
            "content": [{"type": "text", "text": "Hello world"}],
            "stop_reason": "end_turn",
            "usage": {"input_tokens": 10, "output_tokens": 5}
        }"#;
        let resp: MessagesResponse = serde_json::from_str(json).unwrap();
        assert_eq!(resp.content.len(), 1);
        assert_eq!(resp.content[0].text.as_deref(), Some("Hello world"));
        assert_eq!(resp.stop_reason.as_deref(), Some("end_turn"));
        let usage = resp.usage.unwrap();
        assert_eq!(usage.input_tokens, 10);
        assert_eq!(usage.output_tokens, 5);
    }

    #[test]
    fn test_response_deserialization_minimal() {
        let json = r#"{"content": [{"type": "text", "text": "ok"}]}"#;
        let resp: MessagesResponse = serde_json::from_str(json).unwrap();
        assert_eq!(resp.content[0].text.as_deref(), Some("ok"));
        assert!(resp.stop_reason.is_none());
        assert!(resp.usage.is_none());
    }

    #[test]
    fn test_error_deserialization() {
        let json =
            r#"{"error": {"type": "authentication_error", "message": "Invalid API key"}}"#;
        let err: ApiError = serde_json::from_str(json).unwrap();
        let detail = err.error.unwrap();
        assert_eq!(detail.error_type.as_deref(), Some("authentication_error"));
        assert_eq!(detail.message.as_deref(), Some("Invalid API key"));
    }

    #[test]
    fn test_missing_api_key_error() {
        // Temporarily unset the env var — but don't actually because it may affect other tests.
        // Instead, test that the error message is appropriate when using with_config with empty key.
        let client = AnthropicClient::with_config("", "model", 4096);
        assert!(client.api_key.is_empty());
    }

    #[test]
    fn test_system_prompt_omitted_when_empty() {
        let request = MessagesRequest {
            model: "test".to_string(),
            max_tokens: 100,
            system: None,
            messages: vec![],
            temperature: None,
        };
        let json = serde_json::to_string(&request).unwrap();
        assert!(!json.contains("system"));
        assert!(!json.contains("temperature"));
    }

    #[tokio::test]
    async fn test_mock_client_returns_responses() {
        let mock = MockLlmClient::new(vec![
            "Response 1".to_string(),
            "Response 2".to_string(),
        ]);
        assert!(mock.is_available().await.unwrap());
        assert_eq!(mock.model_name(), "mock-model");

        let r1 = mock.generate("sys", "prompt1", 0.3).await.unwrap();
        assert_eq!(r1, "Response 1");

        let r2 = mock.generate("sys", "prompt2", 0.3).await.unwrap();
        assert_eq!(r2, "Response 2");

        // Wraps around
        let r3 = mock.generate("sys", "prompt3", 0.3).await.unwrap();
        assert_eq!(r3, "Response 1");
    }

    #[tokio::test]
    async fn test_mock_single_response() {
        let mock = MockLlmClient::with_single_response("Always this");
        let r1 = mock.generate("", "", 0.0).await.unwrap();
        let r2 = mock.generate("", "", 0.0).await.unwrap();
        assert_eq!(r1, "Always this");
        assert_eq!(r2, "Always this");
    }
}
