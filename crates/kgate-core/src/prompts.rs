use anyhow::{anyhow, Result};

use crate::analyzer::GeneratedQuestion;

/// Context for generating a question about a specific code element
pub struct PromptContext {
    pub project_name: String,
    pub file_path: String,
    pub code_snippet: String,
    pub element_type: String,
    pub element_name: String,
    pub language: String,
    pub surrounding_context: String,
    pub domain: String,
    pub target_tier: String,
    pub target_difficulty: String,
}

/// A constructed prompt ready to send to the LLM
pub struct QuestionPrompt {
    pub system_prompt: String,
    pub user_prompt: String,
}

/// Build the system prompt for question generation
pub fn system_prompt() -> &'static str {
    r#"You are a technical exam question generator for developers.
Generate multiple-choice questions that test UNDERSTANDING, not memorization.

Rules:
- One question per code element
- 4 options (A-D), exactly one correct
- Distractors must be plausible (common misconceptions, off-by-one errors, similar-sounding concepts)
- Question must reference the actual code shown
- Keep question text under 3 lines
- Keep code snippets under 5 lines
- No "all of the above" or "none of the above"
- Output in this exact format per question:

QUESTION: <question text>
CODE: <optional code snippet, or NONE if not needed>
A) <option>
B) <option>
C) <option>
D) <option>
ANSWER: <letter>
HINT: <one-line hint for wrong answers>
EXPLANATION: <2-3 sentence explanation>
TIER: <RECALL|COMPREHENSION|APPLICATION|ANALYSIS>
XP: <10|15|20|25>

Separate multiple questions with a blank line and "---"."#
}

/// Build a prompt for generating questions about a single code element
pub fn build_question_prompt(ctx: &PromptContext) -> QuestionPrompt {
    let user_prompt = format!(
        r#"Generate a {difficulty} {tier} question about this {element_type} from the {language} project "{project}".

Element: `{name}` in `{file}`

```{language}
{code}
```

{context}

Domain: {domain}
Difficulty: {difficulty}
Tier: {tier}"#,
        difficulty = ctx.target_difficulty,
        tier = ctx.target_tier,
        element_type = ctx.element_type,
        language = ctx.language,
        project = ctx.project_name,
        name = ctx.element_name,
        file = ctx.file_path,
        code = ctx.code_snippet,
        context = if ctx.surrounding_context.is_empty() {
            String::new()
        } else {
            format!("Context:\n{}", ctx.surrounding_context)
        },
        domain = ctx.domain,
    );

    QuestionPrompt {
        system_prompt: system_prompt().to_string(),
        user_prompt,
    }
}

/// Build a batch prompt for generating multiple questions in one API call
pub fn build_batch_prompt(contexts: &[PromptContext]) -> QuestionPrompt {
    let mut elements = String::new();

    for (i, ctx) in contexts.iter().enumerate() {
        elements.push_str(&format!(
            "\n--- Element {} ---\n\
             Type: {} | Name: `{}` | File: `{}`\n\
             Language: {} | Domain: {} | Difficulty: {} | Tier: {}\n\
             ```{}\n{}\n```\n",
            i + 1,
            ctx.element_type,
            ctx.element_name,
            ctx.file_path,
            ctx.language,
            ctx.domain,
            ctx.target_difficulty,
            ctx.target_tier,
            ctx.language,
            ctx.code_snippet,
        ));
    }

    let user_prompt = format!(
        "Generate exactly {} multiple-choice questions, one per element below.\n\
         Use the exact format specified in your instructions.\n\
         Separate questions with \"---\".\n\
         {}",
        contexts.len(),
        elements
    );

    QuestionPrompt {
        system_prompt: system_prompt().to_string(),
        user_prompt,
    }
}

/// Parse an LLM response into structured GeneratedQuestion objects
pub fn parse_llm_response(response: &str, default_source_file: &str) -> Result<Vec<GeneratedQuestion>> {
    let mut questions = Vec::new();

    // Split by "---" separator (allowing variations)
    let sections: Vec<&str> = response.split("---").collect();

    for section in sections {
        let section = section.trim();
        if section.is_empty() {
            continue;
        }

        if let Some(q) = parse_single_question(section, default_source_file) {
            questions.push(q);
        }
    }

    if questions.is_empty() {
        // Try parsing the whole response as a single question
        if let Some(q) = parse_single_question(response.trim(), default_source_file) {
            questions.push(q);
        }
    }

    if questions.is_empty() {
        return Err(anyhow!("No valid questions found in LLM response"));
    }

    Ok(questions)
}

fn parse_single_question(text: &str, default_source: &str) -> Option<GeneratedQuestion> {
    let lines: Vec<&str> = text.lines().collect();

    let mut question_text = String::new();
    let mut code_snippet = None;
    let mut options: Vec<String> = Vec::new();
    let mut answer = 'A';
    let mut hint = String::new();
    let mut explanation = String::new();
    let mut tier = "RECALL".to_string();
    let mut xp = 10;

    let mut in_code = false;
    let mut code_lines: Vec<String> = Vec::new();

    for line in &lines {
        let line = line.trim();

        // Handle code blocks
        if line.starts_with("```") {
            if in_code {
                in_code = false;
                if !code_lines.is_empty() {
                    code_snippet = Some(code_lines.join("\n"));
                    code_lines.clear();
                }
            } else {
                in_code = true;
            }
            continue;
        }

        if in_code {
            code_lines.push(line.to_string());
            continue;
        }

        // Parse labeled fields
        if let Some(text) = line.strip_prefix("QUESTION:") {
            question_text = text.trim().to_string();
        } else if let Some(code) = line.strip_prefix("CODE:") {
            let code = code.trim();
            if code != "NONE" && !code.is_empty() {
                code_snippet = Some(code.to_string());
            }
        } else if line.starts_with("A)") || line.starts_with("A.") {
            options.push(format!("A) {}", line[2..].trim()));
        } else if line.starts_with("B)") || line.starts_with("B.") {
            options.push(format!("B) {}", line[2..].trim()));
        } else if line.starts_with("C)") || line.starts_with("C.") {
            options.push(format!("C) {}", line[2..].trim()));
        } else if line.starts_with("D)") || line.starts_with("D.") {
            options.push(format!("D) {}", line[2..].trim()));
        } else if let Some(ans) = line.strip_prefix("ANSWER:") {
            let ans = ans.trim();
            if let Some(c) = ans.chars().next() {
                if c.is_ascii_uppercase() {
                    answer = c;
                }
            }
        } else if let Some(h) = line.strip_prefix("HINT:") {
            hint = h.trim().to_string();
        } else if let Some(e) = line.strip_prefix("EXPLANATION:") {
            explanation = e.trim().to_string();
        } else if let Some(t) = line.strip_prefix("TIER:") {
            tier = t.trim().to_string();
        } else if let Some(x) = line.strip_prefix("XP:") {
            if let Ok(v) = x.trim().parse::<i32>() {
                xp = v;
            }
        }
    }

    // Validate minimum fields
    if question_text.is_empty() || options.len() < 4 {
        return None;
    }

    let difficulty = match tier.as_str() {
        "RECALL" => "Easy",
        "COMPREHENSION" => "Medium",
        "APPLICATION" => "Challenge",
        "ANALYSIS" => "Boss",
        _ => "Medium",
    }.to_string();

    Some(GeneratedQuestion {
        question_text,
        options,
        correct_answer: answer,
        tier,
        difficulty,
        xp,
        hint,
        explanation,
        code_snippet,
        source_file: default_source.to_string(),
        source_line: 0,
        domains: vec![],
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_system_prompt_not_empty() {
        let sp = system_prompt();
        assert!(!sp.is_empty());
        assert!(sp.contains("multiple-choice"));
        assert!(sp.contains("QUESTION:"));
        assert!(sp.contains("ANSWER:"));
    }

    #[test]
    fn test_build_question_prompt() {
        let ctx = PromptContext {
            project_name: "myproject".to_string(),
            file_path: "src/main.rs".to_string(),
            code_snippet: "fn main() {}".to_string(),
            element_type: "function".to_string(),
            element_name: "main".to_string(),
            language: "rust".to_string(),
            surrounding_context: String::new(),
            domain: "rust".to_string(),
            target_tier: "RECALL".to_string(),
            target_difficulty: "Easy".to_string(),
        };
        let prompt = build_question_prompt(&ctx);
        assert!(prompt.system_prompt.contains("multiple-choice"));
        assert!(prompt.user_prompt.contains("main"));
        assert!(prompt.user_prompt.contains("myproject"));
        assert!(prompt.user_prompt.contains("fn main()"));
    }

    #[test]
    fn test_build_batch_prompt() {
        let contexts = vec![
            PromptContext {
                project_name: "proj".to_string(),
                file_path: "a.rs".to_string(),
                code_snippet: "fn a() {}".to_string(),
                element_type: "function".to_string(),
                element_name: "a".to_string(),
                language: "rust".to_string(),
                surrounding_context: String::new(),
                domain: "rust".to_string(),
                target_tier: "RECALL".to_string(),
                target_difficulty: "Easy".to_string(),
            },
            PromptContext {
                project_name: "proj".to_string(),
                file_path: "b.rs".to_string(),
                code_snippet: "fn b() {}".to_string(),
                element_type: "function".to_string(),
                element_name: "b".to_string(),
                language: "rust".to_string(),
                surrounding_context: String::new(),
                domain: "rust".to_string(),
                target_tier: "COMPREHENSION".to_string(),
                target_difficulty: "Medium".to_string(),
            },
        ];
        let prompt = build_batch_prompt(&contexts);
        assert!(prompt.user_prompt.contains("exactly 2"));
        assert!(prompt.user_prompt.contains("Element 1"));
        assert!(prompt.user_prompt.contains("Element 2"));
    }

    #[test]
    fn test_parse_single_question() {
        let response = r#"QUESTION: What does the `main` function do in Rust?
A) Starts the program
B) Compiles the code
C) Imports libraries
D) Defines a module
ANSWER: A
HINT: Every Rust program starts here
EXPLANATION: The main function is the entry point of every Rust binary.
TIER: RECALL
XP: 10"#;
        let questions = parse_llm_response(response, "src/main.rs").unwrap();
        assert_eq!(questions.len(), 1);
        let q = &questions[0];
        assert!(q.question_text.contains("main"));
        assert_eq!(q.options.len(), 4);
        assert_eq!(q.correct_answer, 'A');
        assert_eq!(q.tier, "RECALL");
        assert_eq!(q.xp, 10);
        assert!(!q.hint.is_empty());
        assert!(!q.explanation.is_empty());
    }

    #[test]
    fn test_parse_multiple_questions() {
        let response = r#"QUESTION: What is a struct?
A) A data type
B) A function
C) A loop
D) A variable
ANSWER: A
HINT: Think data
EXPLANATION: A struct groups related data.
TIER: RECALL
XP: 10

---

QUESTION: What does `impl` do?
A) Imports a module
B) Implements methods
C) Creates a variable
D) Defines a loop
ANSWER: B
HINT: Methods on types
EXPLANATION: impl blocks define methods for a type.
TIER: COMPREHENSION
XP: 15"#;
        let questions = parse_llm_response(response, "src/lib.rs").unwrap();
        assert_eq!(questions.len(), 2);
        assert_eq!(questions[0].correct_answer, 'A');
        assert_eq!(questions[1].correct_answer, 'B');
        assert_eq!(questions[1].xp, 15);
    }

    #[test]
    fn test_parse_with_code_snippet() {
        let response = r#"QUESTION: What does this function return?
```rust
fn add(a: i32, b: i32) -> i32 {
    a + b
}
```
A) The sum of a and b
B) The product of a and b
C) Nothing
D) A string
ANSWER: A
HINT: Look at the operator
EXPLANATION: The + operator adds two numbers.
TIER: COMPREHENSION
XP: 15"#;
        let questions = parse_llm_response(response, "src/math.rs").unwrap();
        assert_eq!(questions.len(), 1);
        assert!(questions[0].code_snippet.is_some());
        let code = questions[0].code_snippet.as_ref().unwrap();
        assert!(code.contains("fn add"));
    }

    #[test]
    fn test_parse_malformed_response_returns_error() {
        let response = "This is just random text with no question structure at all.";
        let result = parse_llm_response(response, "test.rs");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_partial_question_missing_options() {
        let response = "QUESTION: What is Rust?\nANSWER: A";
        let result = parse_llm_response(response, "test.rs");
        assert!(result.is_err()); // Missing options
    }
}
