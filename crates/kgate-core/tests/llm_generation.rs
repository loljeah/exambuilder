//! Integration tests for LLM-powered exam generation
//!
//! All tests use MockLlmClient — no actual API calls are made.

use kgate_core::anthropic::MockLlmClient;
use kgate_core::analyzer::{
    CodeElement, Complexity, ElementType, ProjectAnalysis,
};
use kgate_core::llm_generator::{GenerationConfig, LlmGenerator};

fn test_element(name: &str, complexity: Complexity, domain: &str) -> CodeElement {
    CodeElement {
        element_type: ElementType::Function,
        name: name.to_string(),
        file_path: format!("src/{}.rs", name),
        line_number: 1,
        context: format!("pub fn {}() {{}}", name),
        complexity,
        domains: vec![domain.to_string()],
    }
}

fn valid_llm_response() -> String {
    r#"QUESTION: What does the `connect` function return?
A) A database connection
B) An error message
C) A file handle
D) A thread handle
ANSWER: A
HINT: Think about what 'connect' means in a database context
EXPLANATION: The connect function establishes and returns a database connection.
TIER: RECALL
XP: 10

---

QUESTION: Why is `process_data` marked as async?
A) It runs in a separate thread
B) It performs I/O operations that can be awaited
C) It cannot return errors
D) It is called from main
ANSWER: B
HINT: async enables non-blocking I/O
EXPLANATION: Async functions can perform I/O without blocking the thread.
TIER: COMPREHENSION
XP: 15

---

QUESTION: How would you handle an error from `analyze_code`?
A) Ignore it
B) Use the ? operator or match on Result
C) Call panic!
D) Restart the program
ANSWER: B
HINT: Rust has a standard way to propagate errors
EXPLANATION: The ? operator propagates errors, and match allows handling specific cases.
TIER: APPLICATION
XP: 20"#
        .to_string()
}

fn test_analysis(elements: Vec<CodeElement>) -> ProjectAnalysis {
    ProjectAnalysis {
        project_path: "/tmp/test_project".into(),
        project_name: "test_project".to_string(),
        elements,
        detected_languages: vec!["rust".to_string()],
        detected_frameworks: vec![],
        suggested_sprints: vec![],
    }
}

#[tokio::test]
async fn test_full_pipeline_mock_llm() {
    let mock = MockLlmClient::with_single_response(&valid_llm_response());
    let generator = LlmGenerator::new(Box::new(mock), false);

    let analysis = test_analysis(vec![
        test_element("connect", Complexity::Simple, "rust"),
        test_element("process_data", Complexity::Medium, "rust"),
        test_element("analyze_code", Complexity::Complex, "rust"),
    ]);

    let config = GenerationConfig::default();
    let result = generator
        .generate_exam(&analysis, &config, |_| {})
        .await
        .unwrap();

    assert!(!result.sprints.is_empty(), "Should produce at least one sprint");
    assert!(result.total_questions > 0, "Should produce questions");
    assert!(result.total_xp > 0, "Should accumulate XP");
}

#[tokio::test]
async fn test_malformed_response_falls_back_to_templates() {
    let mock = MockLlmClient::with_single_response("This is not a valid question format at all.");
    let generator = LlmGenerator::new(Box::new(mock), true);

    let analysis = test_analysis(vec![
        test_element("fn_a", Complexity::Simple, "rust"),
        test_element("fn_b", Complexity::Medium, "rust"),
    ]);

    let config = GenerationConfig::default();
    let result = generator
        .generate_exam(&analysis, &config, |_| {})
        .await
        .unwrap();

    // Should fall back to template generation
    assert!(!result.sprints.is_empty(), "Fallback should produce sprints");
    assert!(
        !result.warnings.is_empty(),
        "Should have warnings about LLM failure"
    );
}

#[tokio::test]
async fn test_generated_exam_markdown_roundtrip() {
    // Generate questions via mock, format as markdown, parse back
    let mock = MockLlmClient::with_single_response(&valid_llm_response());
    let generator = LlmGenerator::new(Box::new(mock), false);

    let analysis = test_analysis(vec![
        test_element("connect", Complexity::Simple, "rust"),
        test_element("process_data", Complexity::Medium, "rust"),
        test_element("analyze_code", Complexity::Complex, "rust"),
    ]);

    let config = GenerationConfig::default();
    let result = generator
        .generate_exam(&analysis, &config, |_| {})
        .await
        .unwrap();

    // Build markdown (mimicking what the CLI does)
    let llm_analysis = ProjectAnalysis {
        project_path: analysis.project_path.clone(),
        project_name: analysis.project_name.clone(),
        elements: analysis.elements.clone(),
        detected_languages: analysis.detected_languages.clone(),
        detected_frameworks: analysis.detected_frameworks.clone(),
        suggested_sprints: result.sprints,
    };

    let markdown = build_test_markdown(&llm_analysis);

    // Parse the markdown back
    let parsed = kgate_core::parser::parse_exam_file(&markdown);
    assert!(parsed.is_ok(), "Generated markdown should be parseable back");
    let exam = parsed.unwrap();

    // Verify structure
    assert!(
        !exam.sprints.is_empty(),
        "Parsed exam should have sprints"
    );
    for sprint in &exam.sprints {
        assert!(!sprint.questions.is_empty(), "Each sprint should have questions");
        for q in &sprint.questions {
            assert_eq!(q.options.len(), 4, "Each question should have 4 options");
            assert!(
                matches!(q.answer, 'A' | 'B' | 'C' | 'D'),
                "Answer should be A-D"
            );
        }
    }
}

#[tokio::test]
async fn test_multi_domain_generation() {
    // Two different domains should produce separate sprints
    let mock = MockLlmClient::with_single_response(&valid_llm_response());
    let generator = LlmGenerator::new(Box::new(mock), false);

    let analysis = test_analysis(vec![
        test_element("rust_fn_1", Complexity::Simple, "rust"),
        test_element("rust_fn_2", Complexity::Medium, "rust"),
        test_element("nix_opt_1", Complexity::Simple, "nix"),
        test_element("nix_opt_2", Complexity::Medium, "nix"),
    ]);

    let config = GenerationConfig::default();
    let result = generator
        .generate_exam(&analysis, &config, |_| {})
        .await
        .unwrap();

    assert!(
        result.sprints.len() >= 2,
        "Two domains should produce at least 2 sprints, got {}",
        result.sprints.len()
    );
}

#[tokio::test]
async fn test_empty_project_no_crash() {
    let mock = MockLlmClient::with_single_response(&valid_llm_response());
    let generator = LlmGenerator::new(Box::new(mock), false);

    let analysis = test_analysis(vec![]);

    let config = GenerationConfig::default();
    let result = generator
        .generate_exam(&analysis, &config, |_| {})
        .await
        .unwrap();

    assert!(result.sprints.is_empty());
    assert_eq!(result.total_questions, 0);
}

#[tokio::test]
async fn test_single_element_skipped() {
    // A domain with only 1 element should be skipped (need >= 2)
    let mock = MockLlmClient::with_single_response(&valid_llm_response());
    let generator = LlmGenerator::new(Box::new(mock), false);

    let analysis = test_analysis(vec![
        test_element("lonely", Complexity::Simple, "rust"),
    ]);

    let config = GenerationConfig::default();
    let result = generator
        .generate_exam(&analysis, &config, |_| {})
        .await
        .unwrap();

    assert!(result.sprints.is_empty(), "Single element should be skipped");
}

#[tokio::test]
async fn test_max_sprints_respected() {
    let mock = MockLlmClient::with_single_response(&valid_llm_response());
    let generator = LlmGenerator::new(Box::new(mock), false);

    // Create 10 domains with 2 elements each
    let mut elements = Vec::new();
    for i in 0..10 {
        let domain = format!("domain_{}", i);
        elements.push(test_element(&format!("fn_{}_a", i), Complexity::Simple, &domain));
        elements.push(test_element(&format!("fn_{}_b", i), Complexity::Medium, &domain));
    }

    let analysis = test_analysis(elements);

    let config = GenerationConfig {
        max_sprints: 3,
        ..GenerationConfig::default()
    };

    let result = generator
        .generate_exam(&analysis, &config, |_| {})
        .await
        .unwrap();

    assert!(
        result.sprints.len() <= 3,
        "Should respect max_sprints=3, got {}",
        result.sprints.len()
    );
}

#[tokio::test]
async fn test_progress_callback_reports_phases() {
    let mock = MockLlmClient::with_single_response(&valid_llm_response());
    let generator = LlmGenerator::new(Box::new(mock), false);

    let analysis = test_analysis(vec![
        test_element("a", Complexity::Simple, "rust"),
        test_element("b", Complexity::Medium, "rust"),
    ]);

    let config = GenerationConfig::default();
    let phases = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
    let phases_clone = phases.clone();

    let _result = generator
        .generate_exam(&analysis, &config, move |p| {
            phases_clone.lock().unwrap().push(p.phase.clone());
        })
        .await
        .unwrap();

    let phases = phases.lock().unwrap();
    assert!(phases.contains(&"Analyzing".to_string()));
    assert!(phases.contains(&"Done".to_string()));
}

// Helper: build markdown from ProjectAnalysis (mirrors CLI generation.rs logic)
fn build_test_markdown(analysis: &ProjectAnalysis) -> String {
    let mut md = String::new();

    md.push_str(&format!("# Exam: {}\n\n", analysis.project_name));
    md.push_str("---\n\n");

    for (i, sprint) in analysis.suggested_sprints.iter().enumerate() {
        md.push_str(&format!("## Sprint {}: {}\n", i + 1, sprint.topic));
        md.push_str("🎙️ Voice-compatible: yes\n\n");

        for (j, q) in sprint.questions.iter().enumerate() {
            md.push_str(&format!(
                "### Q{}. [{}] {} — {} XP\n",
                j + 1,
                q.tier,
                q.difficulty,
                q.xp
            ));
            md.push_str(&format!("{}\n\n", q.question_text));

            if let Some(ref code) = q.code_snippet {
                md.push_str("```\n");
                md.push_str(code);
                if !code.ends_with('\n') {
                    md.push('\n');
                }
                md.push_str("```\n\n");
            }

            for opt in &q.options {
                md.push_str(&format!("- {}\n", opt));
            }
            md.push('\n');
        }

        md.push_str("---\n\n");
    }

    md.push_str("## 🔑 Answer Key\n\n");

    for (i, sprint) in analysis.suggested_sprints.iter().enumerate() {
        md.push_str(&format!("### Sprint {}\n\n", i + 1));

        for (j, q) in sprint.questions.iter().enumerate() {
            md.push_str(&format!(
                "**Q{}. Answer: {}** — {} XP\n",
                j + 1,
                q.correct_answer,
                q.xp
            ));
            md.push_str(&format!("Hint: {}\n", q.hint));
            md.push_str(&format!("Full: {}\n\n", q.explanation));
        }
    }

    md
}
