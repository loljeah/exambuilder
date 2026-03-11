use std::path::PathBuf;

use anyhow::Result;
use console::style;

pub async fn cmd_generate(path: &PathBuf, output: Option<PathBuf>) -> Result<()> {
    use kgate_core::CodebaseAnalyzer;

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

    if analysis.suggested_sprints.is_empty() {
        println!(
            "{} Not enough code elements to generate exam. Add more code!",
            style("⚠").yellow()
        );
        return Ok(());
    }

    // Generate exam markdown
    let exam_content = generate_exam_markdown(&analysis);

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
        "  {} sprints, {} questions total",
        analysis.suggested_sprints.len(),
        analysis.suggested_sprints.iter().map(|s| s.questions.len()).sum::<usize>()
    );
    println!();
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
