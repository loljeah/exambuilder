use std::path::PathBuf;

use anyhow::Result;
use console::style;
use indicatif::{ProgressBar, ProgressStyle};
use kgate_core::LlmClient;

pub async fn cmd_generate(
    path: &PathBuf,
    output: Option<PathBuf>,
    force_llm: bool,
    force_templates: bool,
    dry_run: bool,
    model_override: Option<String>,
) -> Result<()> {
    use kgate_core::{AnthropicClient, CodebaseAnalyzer, GenerationConfig, LlmGenerator};

    let abs_path = std::fs::canonicalize(path)?;
    println!(
        "{} Analyzing codebase: {}",
        style("🔍").cyan(),
        abs_path.display()
    );

    let analyzer = CodebaseAnalyzer::new();
    let analysis = analyzer.analyze(&abs_path)?;

    println!(
        "  Found {} code elements in {} files",
        analysis.elements.len(),
        analysis.detected_languages.len()
    );
    println!(
        "  Languages: {}",
        analysis.detected_languages.join(", ")
    );
    if !analysis.detected_frameworks.is_empty() {
        println!(
            "  Frameworks: {}",
            analysis.detected_frameworks.join(", ")
        );
    }

    if analysis.elements.is_empty() {
        println!(
            "{} Not enough code elements to generate exam. Add more code!",
            style("⚠").yellow()
        );
        return Ok(());
    }

    // Determine generation mode
    let use_llm = if force_templates {
        false
    } else if force_llm {
        true
    } else {
        // Auto-detect: use LLM if API key is set
        std::env::var("ANTHROPIC_API_KEY").is_ok()
    };

    let exam_content = if use_llm {
        // LLM-powered generation
        let client = if let Some(ref model) = model_override {
            let key = std::env::var("ANTHROPIC_API_KEY")
                .map_err(|_| anyhow::anyhow!("ANTHROPIC_API_KEY not set. Required for --llm mode."))?;
            AnthropicClient::with_config(&key, model, 4096)
        } else {
            AnthropicClient::new()?
        };

        let model_name = client.model_name().to_string();
        println!(
            "  Mode: {} ({})",
            style("LLM").green().bold(),
            style(&model_name).dim()
        );

        let generator = LlmGenerator::new(Box::new(client), !force_llm);
        let config = GenerationConfig::default();

        let pb = ProgressBar::new(config.max_sprints as u64);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("{msg} [{bar:30.cyan/blue}] {pos}/{len}")
                .unwrap_or_else(|_| ProgressStyle::default_bar())
                .progress_chars("━░"),
        );

        let result = generator
            .generate_exam(&analysis, &config, |p| {
                pb.set_message(p.phase.clone());
                pb.set_position(p.current as u64);
                pb.set_length(p.total as u64);
            })
            .await?;

        pb.finish_and_clear();

        for warning in &result.warnings {
            println!("  {} {}", style("⚠").yellow(), warning);
        }

        if result.sprints.is_empty() {
            println!(
                "{} No questions could be generated. Try --templates mode.",
                style("⚠").yellow()
            );
            return Ok(());
        }

        let sprint_suggestions = result.sprints;
        let total_xp = result.total_xp;
        let total_questions = result.total_questions;
        let usage = result.total_usage;

        // Build a temporary analysis with LLM sprints for markdown generation
        let llm_analysis = kgate_core::ProjectAnalysis {
            project_path: analysis.project_path.clone(),
            project_name: analysis.project_name.clone(),
            elements: analysis.elements.clone(),
            detected_languages: analysis.detected_languages.clone(),
            detected_frameworks: analysis.detected_frameworks.clone(),
            suggested_sprints: sprint_suggestions,
        };

        let content = generate_exam_markdown(&llm_analysis);

        println!(
            "  {} sprints, {} questions, {} XP total",
            llm_analysis.suggested_sprints.len(),
            total_questions,
            total_xp,
        );
        if usage.input_tokens > 0 || usage.output_tokens > 0 {
            println!(
                "  Tokens: {} input, {} output",
                usage.input_tokens, usage.output_tokens,
            );
        }

        content
    } else {
        // Template-based generation (original behavior)
        println!("  Mode: {}", style("Templates").yellow().bold());

        if analysis.suggested_sprints.is_empty() {
            println!(
                "{} Not enough code elements to generate exam. Add more code!",
                style("⚠").yellow()
            );
            return Ok(());
        }

        generate_exam_markdown(&analysis)
    };

    if dry_run {
        println!();
        println!("{}", style("--- Dry Run Preview ---").dim());
        // Show first 50 lines
        for (i, line) in exam_content.lines().enumerate() {
            if i >= 50 {
                println!("  {} (truncated, {} total lines)", style("...").dim(), exam_content.lines().count());
                break;
            }
            println!("  {}", line);
        }
        println!("{}", style("--- End Preview ---").dim());
        println!();
        println!(
            "  {} Use without --dry-run to write the file.",
            style("ℹ").blue()
        );
        return Ok(());
    }

    // Determine output path
    let output_path = output.unwrap_or_else(|| {
        abs_path.join(format!("exam_{}.md", analysis.project_name))
    });

    std::fs::write(&output_path, &exam_content)?;

    println!();
    println!(
        "{} Generated exam: {}",
        style("✓").green(),
        output_path.display()
    );
    println!(
        "  Import with: {}",
        style(format!("kgate scan {} --import", abs_path.display())).yellow()
    );

    Ok(())
}

fn generate_exam_markdown(analysis: &kgate_core::ProjectAnalysis) -> String {
    let mut md = String::new();

    // Header
    md.push_str(&format!("# Exam: {}\n", analysis.project_name));
    md.push_str(&format!("# Generated: {}\n", chrono::Utc::now().format("%Y-%m-%d")));
    md.push_str(&format!("# Languages: {}\n", analysis.detected_languages.join(", ")));
    md.push_str("# Pass: 60% per sprint | Retakes: unlimited\n");
    md.push_str("# Voice-Ready: yes\n\n");
    md.push_str("---\n\n");

    // Sprints
    for (i, sprint) in analysis.suggested_sprints.iter().enumerate() {
        md.push_str(&format!("## Sprint {}: {}\n", i + 1, sprint.topic));
        md.push_str(&format!(
            "⏱️ Target: 3 min | 🎯 Pass: 60% | ⚡ {} XP\n",
            sprint.total_xp
        ));
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

    // Answer key
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
            md.push_str(&format!("Full: {}\n", q.explanation));
            md.push_str(&format!("📁 `{}:{}`\n\n", q.source_file, q.source_line));
        }
    }

    md
}
