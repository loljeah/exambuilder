//! LLM-powered exam question generator
//!
//! Uses the Anthropic API (via LlmClient trait) to generate exam questions
//! from analyzed code elements. Falls back to template-based generation if
//! the API is unavailable.

use anyhow::Result;
use std::collections::HashMap;
use std::path::Path;

use crate::analyzer::{
    CodeElement, Complexity, GeneratedQuestion, ProjectAnalysis, SprintSuggestion,
};
use crate::anthropic::{LlmClient, Usage};
use crate::dedup::deduplicate_questions;
use crate::prompts::{build_batch_prompt, parse_llm_response, PromptContext};
use crate::question_validator::{validate_question, Severity};

/// Configuration for LLM-based question generation
#[derive(Debug, Clone)]
pub struct GenerationConfig {
    pub questions_per_sprint: usize,
    pub max_sprints: usize,
    pub temperature: f32,
    pub dedup_threshold: f32,
}

impl Default for GenerationConfig {
    fn default() -> Self {
        Self {
            questions_per_sprint: 3,
            max_sprints: 5,
            temperature: 0.3,
            dedup_threshold: 0.7,
        }
    }
}

/// Progress reporting for generation phases
#[derive(Debug, Clone)]
pub struct GenerationProgress {
    pub phase: String,
    pub current: usize,
    pub total: usize,
    pub message: String,
}

/// Result of a generation run
#[derive(Debug)]
pub struct GenerationResult {
    pub sprints: Vec<SprintSuggestion>,
    pub total_questions: usize,
    pub total_xp: i32,
    pub total_usage: Usage,
    pub warnings: Vec<String>,
}

/// LLM-powered question generator
pub struct LlmGenerator {
    client: Box<dyn LlmClient>,
    fallback_to_templates: bool,
}

impl LlmGenerator {
    pub fn new(client: Box<dyn LlmClient>, fallback_to_templates: bool) -> Self {
        Self {
            client,
            fallback_to_templates,
        }
    }

    /// Generate a full exam from a project analysis
    pub async fn generate_exam(
        &self,
        analysis: &ProjectAnalysis,
        config: &GenerationConfig,
        progress: impl Fn(GenerationProgress),
    ) -> Result<GenerationResult> {
        // Check LLM availability
        let llm_available = match self.client.is_available().await {
            Ok(true) => true,
            Ok(false) => false,
            Err(_) => false,
        };

        if !llm_available && !self.fallback_to_templates {
            return Err(anyhow::anyhow!(
                "LLM not available and fallback disabled. Set ANTHROPIC_API_KEY or use --templates."
            ));
        }

        // Group elements by primary domain
        let domain_groups = group_by_domain(&analysis.elements);

        progress(GenerationProgress {
            phase: "Analyzing".to_string(),
            current: 0,
            total: domain_groups.len(),
            message: format!("{} elements across {} domains", analysis.elements.len(), domain_groups.len()),
        });

        let mut sprints = Vec::new();
        let mut total_usage = Usage::default();
        let mut warnings = Vec::new();
        let mut sprint_idx = 0;

        for (domain, elements) in &domain_groups {
            if sprint_idx >= config.max_sprints {
                break;
            }

            // Select top elements by complexity diversity
            let selected = select_elements(elements, config.questions_per_sprint);
            if selected.len() < 2 {
                continue;
            }

            sprint_idx += 1;
            progress(GenerationProgress {
                phase: "Generating".to_string(),
                current: sprint_idx,
                total: config.max_sprints.min(domain_groups.len()),
                message: format!("Sprint {}/{} — {}", sprint_idx, config.max_sprints.min(domain_groups.len()), domain),
            });

            let questions = if llm_available {
                match self
                    .generate_questions_llm(&selected, &analysis.project_name, config, &mut total_usage)
                    .await
                {
                    Ok(qs) => qs,
                    Err(e) => {
                        warnings.push(format!("LLM failed for domain '{}': {}", domain, e));
                        if self.fallback_to_templates {
                            generate_questions_template(&selected)
                        } else {
                            continue;
                        }
                    }
                }
            } else {
                generate_questions_template(&selected)
            };

            if questions.is_empty() {
                warnings.push(format!("No valid questions generated for domain '{}'", domain));
                continue;
            }

            let total_xp: i32 = questions.iter().map(|q| q.xp).sum();
            let topic = domain_to_topic(domain, &analysis.detected_languages);

            sprints.push(SprintSuggestion {
                topic,
                questions,
                total_xp,
            });
        }

        let total_questions: usize = sprints.iter().map(|s| s.questions.len()).sum();
        let total_xp: i32 = sprints.iter().map(|s| s.total_xp).sum();

        progress(GenerationProgress {
            phase: "Done".to_string(),
            current: sprints.len(),
            total: sprints.len(),
            message: format!("{} sprints, {} questions, {} XP", sprints.len(), total_questions, total_xp),
        });

        Ok(GenerationResult {
            sprints,
            total_questions,
            total_xp,
            total_usage,
            warnings,
        })
    }

    /// Generate questions for a set of elements using the LLM
    async fn generate_questions_llm(
        &self,
        elements: &[&CodeElement],
        project_name: &str,
        config: &GenerationConfig,
        total_usage: &mut Usage,
    ) -> Result<Vec<GeneratedQuestion>> {
        // Build prompt contexts
        let contexts: Vec<PromptContext> = elements
            .iter()
            .map(|elem| element_to_prompt_context(elem, project_name))
            .collect();

        let prompt = build_batch_prompt(&contexts);

        // Call LLM
        let response = self
            .client
            .generate(&prompt.system_prompt, &prompt.user_prompt, config.temperature)
            .await?;

        // Track usage if available (MockLlmClient won't have real usage)
        total_usage.input_tokens += 0; // Usage tracked at client level if needed
        total_usage.output_tokens += 0;

        // Parse response
        let default_source = elements
            .first()
            .map(|e| e.file_path.as_str())
            .unwrap_or("unknown");

        let mut questions = parse_llm_response(&response, default_source)?;

        // Set domains from source elements
        for (i, q) in questions.iter_mut().enumerate() {
            if let Some(elem) = elements.get(i) {
                q.domains.clone_from(&elem.domains);
                q.source_file.clone_from(&elem.file_path);
                q.source_line = elem.line_number;
            }
        }

        // Validate and filter
        let mut valid_questions = Vec::new();
        for q in questions {
            let result = validate_question(&q);
            if result.valid {
                valid_questions.push(q);
            } else {
                let errors: Vec<String> = result
                    .issues
                    .iter()
                    .filter(|i| i.severity == Severity::Error)
                    .map(|i| i.message.clone())
                    .collect();
                eprintln!("  Skipping invalid question: {}", errors.join(", "));
            }
        }

        // Deduplicate
        deduplicate_questions(&mut valid_questions, config.dedup_threshold);

        // Limit to requested count
        valid_questions.truncate(config.questions_per_sprint);

        Ok(valid_questions)
    }
}

/// Generate template-based questions (fallback when LLM unavailable)
fn generate_questions_template(elements: &[&CodeElement]) -> Vec<GeneratedQuestion> {
    use crate::analyzer::CodebaseAnalyzer;
    let analyzer = CodebaseAnalyzer::new();
    elements
        .iter()
        .enumerate()
        .take(3)
        .map(|(i, elem)| analyzer.element_to_question(elem, i + 1))
        .collect()
}

/// Group code elements by their primary domain
fn group_by_domain(elements: &[CodeElement]) -> Vec<(String, Vec<&CodeElement>)> {
    let mut map: HashMap<String, Vec<&CodeElement>> = HashMap::new();

    for elem in elements {
        let domain = elem.domains.first().cloned().unwrap_or_else(|| "general".to_string());
        map.entry(domain).or_default().push(elem);
    }

    // Sort by number of elements descending (most populated domains first)
    let mut pairs: Vec<_> = map.into_iter().collect();
    pairs.sort_by(|a, b| b.1.len().cmp(&a.1.len()));
    pairs
}

/// Select elements with complexity diversity for a sprint
fn select_elements<'a>(elements: &[&'a CodeElement], count: usize) -> Vec<&'a CodeElement> {
    if elements.len() <= count {
        return elements.to_vec();
    }

    let mut selected = Vec::new();

    // Try to get one of each complexity level
    let mut by_complexity: HashMap<&str, Vec<&&CodeElement>> = HashMap::new();
    for elem in elements {
        let key = match elem.complexity {
            Complexity::Simple => "simple",
            Complexity::Medium => "medium",
            Complexity::Complex => "complex",
        };
        by_complexity.entry(key).or_default().push(elem);
    }

    // Pick one from each complexity (easy -> medium -> hard ordering for sprint rhythm)
    for key in &["simple", "medium", "complex"] {
        if selected.len() >= count {
            break;
        }
        if let Some(elems) = by_complexity.get(key) {
            if let Some(elem) = elems.first() {
                selected.push(**elem);
            }
        }
    }

    // Fill remaining from whatever's available
    for elem in elements {
        if selected.len() >= count {
            break;
        }
        if !selected.iter().any(|s| std::ptr::eq(*s, *elem)) {
            selected.push(elem);
        }
    }

    selected
}

/// Convert a CodeElement to a PromptContext for the LLM
fn element_to_prompt_context(elem: &CodeElement, project_name: &str) -> PromptContext {
    let language = elem
        .domains
        .first()
        .cloned()
        .unwrap_or_else(|| "code".to_string());

    let (tier, difficulty) = match elem.complexity {
        Complexity::Simple => ("RECALL", "Easy"),
        Complexity::Medium => ("COMPREHENSION", "Medium"),
        Complexity::Complex => ("APPLICATION", "Challenge"),
    };

    PromptContext {
        project_name: project_name.to_string(),
        file_path: elem.file_path.clone(),
        code_snippet: elem.context.clone(),
        element_type: format!("{:?}", elem.element_type).to_lowercase(),
        element_name: elem.name.clone(),
        language,
        surrounding_context: String::new(),
        domain: elem.domains.join(", "),
        target_tier: tier.to_string(),
        target_difficulty: difficulty.to_string(),
    }
}

/// Read surrounding context from a file for richer prompts
pub fn read_element_context(path: &Path, line: usize, context_lines: usize) -> Result<String> {
    let content = std::fs::read_to_string(path)?;
    let lines: Vec<&str> = content.lines().collect();
    let start = line.saturating_sub(context_lines + 1);
    let end = (line + context_lines).min(lines.len());
    Ok(lines[start..end].join("\n"))
}

/// Map domain name to human-readable sprint topic
fn domain_to_topic(domain: &str, languages: &[String]) -> String {
    match domain {
        "rust" => "Rust Fundamentals".to_string(),
        "nix" => "NixOS Configuration".to_string(),
        "python" => "Python Basics".to_string(),
        "docker" => "Docker & Containers".to_string(),
        "networking" => "Networking Concepts".to_string(),
        "security" => "Security Practices".to_string(),
        "architecture" => "Code Architecture".to_string(),
        "devops" => "DevOps & Infrastructure".to_string(),
        "testing" => "Testing Practices".to_string(),
        _ => {
            if let Some(lang) = languages.first() {
                format!("{} Fundamentals", lang.to_uppercase())
            } else {
                "Project Basics".to_string()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::analyzer::{CodeElement, Complexity, ElementType};
    use crate::anthropic::MockLlmClient;

    fn test_element(name: &str, complexity: Complexity) -> CodeElement {
        CodeElement {
            element_type: ElementType::Function,
            name: name.to_string(),
            file_path: "src/test.rs".to_string(),
            line_number: 1,
            context: format!("pub fn {}() {{}}", name),
            complexity,
            domains: vec!["rust".to_string()],
        }
    }

    fn mock_llm_response() -> String {
        r#"QUESTION: What does the `hello` function do?
A) Prints a greeting
B) Returns an error
C) Creates a thread
D) Opens a file
ANSWER: A
HINT: Look at the function name
EXPLANATION: The hello function prints a greeting message.
TIER: RECALL
XP: 10

---

QUESTION: What does the `process` function do?
A) Reads input
B) Processes data asynchronously
C) Writes to disk
D) Sends a network request
ANSWER: B
HINT: Check the async keyword
EXPLANATION: The process function handles data processing asynchronously.
TIER: COMPREHENSION
XP: 15

---

QUESTION: What does the `analyze` function do?
A) Parses configuration
B) Runs benchmarks
C) Analyzes code elements
D) Generates reports
ANSWER: C
HINT: The name gives it away
EXPLANATION: The analyze function examines code elements and produces analysis results.
TIER: APPLICATION
XP: 20"#
            .to_string()
    }

    #[test]
    fn test_group_by_domain() {
        let elements = vec![
            test_element("a", Complexity::Simple),
            test_element("b", Complexity::Medium),
            {
                let mut e = test_element("c", Complexity::Simple);
                e.domains = vec!["python".to_string()];
                e
            },
        ];

        let groups = group_by_domain(&elements);
        assert_eq!(groups.len(), 2);
        // Rust domain should have 2 elements
        let rust_group = groups.iter().find(|(d, _)| d == "rust").unwrap();
        assert_eq!(rust_group.1.len(), 2);
    }

    #[test]
    fn test_select_elements_diversity() {
        let elements = [
            test_element("simple1", Complexity::Simple),
            test_element("simple2", Complexity::Simple),
            test_element("medium1", Complexity::Medium),
            test_element("complex1", Complexity::Complex),
        ];

        let refs: Vec<&CodeElement> = elements.iter().collect();
        let selected = select_elements(&refs, 3);

        assert_eq!(selected.len(), 3);
        // Should include one of each complexity
        assert!(selected.iter().any(|e| e.complexity == Complexity::Simple));
        assert!(selected.iter().any(|e| e.complexity == Complexity::Medium));
        assert!(selected.iter().any(|e| e.complexity == Complexity::Complex));
    }

    #[test]
    fn test_select_elements_fewer_than_count() {
        let elements = [test_element("only", Complexity::Simple)];
        let refs: Vec<&CodeElement> = elements.iter().collect();
        let selected = select_elements(&refs, 3);
        assert_eq!(selected.len(), 1);
    }

    #[test]
    fn test_element_to_prompt_context() {
        let elem = test_element("my_func", Complexity::Medium);
        let ctx = element_to_prompt_context(&elem, "myproject");

        assert_eq!(ctx.project_name, "myproject");
        assert_eq!(ctx.element_name, "my_func");
        assert_eq!(ctx.target_tier, "COMPREHENSION");
        assert_eq!(ctx.target_difficulty, "Medium");
        assert_eq!(ctx.language, "rust");
    }

    #[test]
    fn test_domain_to_topic() {
        assert_eq!(domain_to_topic("rust", &[]), "Rust Fundamentals");
        assert_eq!(domain_to_topic("nix", &[]), "NixOS Configuration");
        assert_eq!(
            domain_to_topic("unknown", &["go".to_string()]),
            "GO Fundamentals"
        );
    }

    #[tokio::test]
    async fn test_generate_exam_with_mock() {
        let mock = MockLlmClient::with_single_response(&mock_llm_response());
        let generator = LlmGenerator::new(Box::new(mock), false);

        let analysis = ProjectAnalysis {
            project_path: "/tmp/test".into(),
            project_name: "test_project".to_string(),
            elements: vec![
                test_element("hello", Complexity::Simple),
                test_element("process", Complexity::Medium),
                test_element("analyze", Complexity::Complex),
            ],
            detected_languages: vec!["rust".to_string()],
            detected_frameworks: vec![],
            suggested_sprints: vec![],
        };

        let config = GenerationConfig::default();
        let result = generator
            .generate_exam(&analysis, &config, |_| {})
            .await
            .unwrap();

        assert!(!result.sprints.is_empty());
        assert!(result.total_questions > 0);
        assert!(result.total_xp > 0);
    }

    #[tokio::test]
    async fn test_fallback_to_templates() {
        let mock = MockLlmClient::with_single_response("garbage no questions here");
        let generator = LlmGenerator::new(Box::new(mock), true);

        let analysis = ProjectAnalysis {
            project_path: "/tmp/test".into(),
            project_name: "test_project".to_string(),
            elements: vec![
                test_element("fn_a", Complexity::Simple),
                test_element("fn_b", Complexity::Medium),
                test_element("fn_c", Complexity::Complex),
            ],
            detected_languages: vec!["rust".to_string()],
            detected_frameworks: vec![],
            suggested_sprints: vec![],
        };

        let config = GenerationConfig::default();
        let result = generator
            .generate_exam(&analysis, &config, |_| {})
            .await
            .unwrap();

        // Should fall back to template generation
        assert!(!result.sprints.is_empty());
        assert!(!result.warnings.is_empty());
    }

    #[tokio::test]
    async fn test_progress_callback_fires() {
        let mock = MockLlmClient::with_single_response(&mock_llm_response());
        let generator = LlmGenerator::new(Box::new(mock), false);

        let analysis = ProjectAnalysis {
            project_path: "/tmp/test".into(),
            project_name: "test_project".to_string(),
            elements: vec![
                test_element("a", Complexity::Simple),
                test_element("b", Complexity::Medium),
            ],
            detected_languages: vec!["rust".to_string()],
            detected_frameworks: vec![],
            suggested_sprints: vec![],
        };

        let config = GenerationConfig::default();
        let progress_count = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let pc = progress_count.clone();

        let _result = generator
            .generate_exam(&analysis, &config, move |_p| {
                pc.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            })
            .await
            .unwrap();

        assert!(progress_count.load(std::sync::atomic::Ordering::SeqCst) >= 2);
    }

    #[test]
    fn test_read_element_context() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("test.rs");
        std::fs::write(&file, "line1\nline2\nline3\nline4\nline5\nline6\nline7\n").unwrap();

        let ctx = read_element_context(&file, 4, 2).unwrap();
        assert!(ctx.contains("line2"));
        assert!(ctx.contains("line4"));
        assert!(ctx.contains("line5"));
    }
}
