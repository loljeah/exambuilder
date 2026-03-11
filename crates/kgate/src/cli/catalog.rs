use std::path::PathBuf;

use anyhow::Result;
use console::style;

use kgate_core::Database;

use crate::{CatalogCommands, HarvestCommands};

fn catalog_path() -> PathBuf {
    crate::kgate_dir().join("catalog.json")
}

pub async fn cmd_catalog(action: CatalogCommands) -> Result<()> {
    let db = crate::db_path();
    if !db.exists() {
        println!(
            "{} Not initialized. Run {} first.",
            style("✗").red(),
            style("kgate init").yellow()
        );
        return Ok(());
    }

    let database = Database::new(&db).await?;

    match action {
        CatalogCommands::List => {
            let stats = database.get_domain_catalog_stats().await?;

            if stats.is_empty() {
                println!("No questions in catalog yet.");
                println!("Pass exam sprints to collect questions!");
                return Ok(());
            }

            println!("{}", style("Domain Catalog").cyan().bold());
            println!();

            for stat in stats {
                let accuracy_color = if stat.accuracy >= 80.0 {
                    style(format!("{:.0}%", stat.accuracy)).green()
                } else if stat.accuracy >= 60.0 {
                    style(format!("{:.0}%", stat.accuracy)).yellow()
                } else {
                    style(format!("{:.0}%", stat.accuracy)).red()
                };

                println!(
                    "  {:12} {:4} questions | {} accuracy",
                    style(&stat.domain).cyan(),
                    stat.question_count,
                    accuracy_color
                );
            }
        }

        CatalogCommands::Show { domain } => {
            let entries = database.get_domain_catalog(&domain).await?;

            if entries.is_empty() {
                println!("No questions found for domain: {}", domain);
                return Ok(());
            }

            println!(
                "{} {} questions",
                style(format!("Domain: {}", domain)).cyan().bold(),
                entries.len()
            );
            println!();

            for entry in entries.iter().take(20) {
                let accuracy = if entry.times_seen > 0 {
                    (entry.times_correct as f64 / entry.times_seen as f64) * 100.0
                } else {
                    0.0
                };

                println!(
                    "  [{}] {} — {:.0}% ({}/{})",
                    entry.tier,
                    if entry.question_text.len() > 50 {
                        format!("{}...", &entry.question_text[..50])
                    } else {
                        entry.question_text.clone()
                    },
                    accuracy,
                    entry.times_correct,
                    entry.times_seen
                );
            }

            if entries.len() > 20 {
                println!("  ... and {} more", entries.len() - 20);
            }
        }

        CatalogCommands::Export { output } => {
            let output_path = output.unwrap_or_else(|| {
                crate::kgate_dir().join("catalog.json")
            });

            let json = database.export_domain_catalog().await?;
            std::fs::write(&output_path, &json)?;

            println!(
                "{} Exported catalog to {}",
                style("✓").green(),
                output_path.display()
            );
        }

        CatalogCommands::Stats => {
            let stats = database.get_domain_catalog_stats().await?;
            let review_stats = database.get_review_stats().await?;

            println!("{}", style("Catalog Statistics").cyan().bold());
            println!();

            let total_questions: i64 = stats.iter().map(|s| s.question_count).sum();
            let total_attempts: i64 = stats.iter().map(|s| s.total_attempts).sum();
            let total_correct: i64 = stats.iter().map(|s| s.total_correct).sum();

            println!("  Total questions: {}", total_questions);
            println!("  Total attempts:  {}", total_attempts);
            println!(
                "  Overall accuracy: {:.1}%",
                if total_attempts > 0 {
                    (total_correct as f64 / total_attempts as f64) * 100.0
                } else {
                    0.0
                }
            );
            println!("  Domains covered: {}", stats.len());
            println!();

            println!("{}", style("Review Queue").cyan().bold());
            println!("  Items in queue: {}", review_stats.total_items);
            println!("  Due now: {}", review_stats.due_now);
            println!(
                "  Average EF: {:.2}",
                review_stats.avg_easiness.unwrap_or(2.5)
            );
            println!(
                "  Longest streak: {}",
                review_stats.max_streak.unwrap_or(0)
            );
        }
    }

    Ok(())
}

pub async fn cmd_grade(answer: &str, concepts: &str) -> Result<()> {
    use kgate_core::{AnswerKey, LocalGrader};

    // Parse concepts from comma-separated string
    let key_concepts: Vec<&str> = concepts
        .split(',')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .collect();

    if key_concepts.is_empty() {
        println!(
            "{} Please provide at least one concept to grade against",
            style("✗").red()
        );
        return Ok(());
    }

    let answer_key = AnswerKey::new(key_concepts);
    let grader = LocalGrader::new();
    let result = grader.grade(answer, &answer_key);

    // Display results
    println!();
    println!("{}", style("LLM Grading Result").cyan().bold());
    println!();

    // Score visualization
    let score_bar = match result.score {
        3 => style("███").green(),
        2 => style("██░").yellow(),
        1 => style("█░░").red(),
        _ => style("░░░").dim(),
    };

    let score_label = match result.score {
        3 => style("COMPLETE").green().bold(),
        2 => style("PARTIAL").yellow().bold(),
        1 => style("SURFACE").red().bold(),
        _ => style("INCORRECT").red().bold(),
    };

    println!(
        "  Score: {} {}/3 — {}",
        score_bar, result.score, score_label
    );
    println!("  XP Multiplier: {}%", (result.xp_multiplier() * 100.0) as i32);
    println!("  Confidence: {:.0}%", result.confidence * 100.0);
    println!();

    // Feedback
    println!("  {}", style("Feedback:").bold());
    println!("    {}", result.feedback);
    println!();

    // Matched concepts
    if !result.matched_concepts.is_empty() {
        println!("  {} Matched:", style("✓").green());
        for concept in &result.matched_concepts {
            println!("    • {}", style(concept).green());
        }
    }

    // Missing concepts
    if !result.missing_concepts.is_empty() {
        println!("  {} Missing:", style("○").yellow());
        for concept in &result.missing_concepts {
            println!("    • {}", style(concept).yellow());
        }
    }

    println!();

    // Pass/fail indicator
    if result.passed() {
        println!(
            "  {} This answer would {} the question",
            style("✓").green(),
            style("PASS").green().bold()
        );
    } else {
        println!(
            "  {} This answer would {} the question",
            style("✗").red(),
            style("FAIL").red().bold()
        );
    }

    Ok(())
}

pub async fn cmd_harvest(action: HarvestCommands) -> Result<()> {
    use kgate_core::{Harvester, QuestionCatalog};

    // Load existing catalog or create new
    let catalog = if catalog_path().exists() {
        QuestionCatalog::load(&catalog_path()).unwrap_or_default()
    } else {
        QuestionCatalog::new()
    };

    let mut harvester = Harvester::with_catalog(catalog);

    match action {
        HarvestCommands::Add { path } => {
            let abs_path = std::fs::canonicalize(&path)?;
            println!(
                "{} Harvesting questions from: {}",
                style("🌱").green(),
                abs_path.display()
            );

            let result = harvester.harvest(&abs_path);

            if result.success {
                println!(
                    "  {} Scanned {} elements",
                    style("✓").green(),
                    result.files_scanned
                );
                println!(
                    "  {} Generated {} questions",
                    style("✓").green(),
                    result.questions_generated
                );
                println!(
                    "  Languages: {}",
                    result.languages.join(", ")
                );

                // Save catalog
                let catalog = harvester.into_catalog();
                catalog.save(&catalog_path())?;

                println!();
                println!(
                    "{} Catalog saved: {} total questions",
                    style("📚").cyan(),
                    catalog.total_questions
                );
            } else {
                println!(
                    "{} Harvest failed: {}",
                    style("✗").red(),
                    result.error.unwrap_or("Unknown error".to_string())
                );
            }
        }

        HarvestCommands::All => {
            let db = crate::db_path();
            if !db.exists() {
                println!(
                    "{} Not initialized. Run {} first.",
                    style("✗").red(),
                    style("kgate init").yellow()
                );
                return Ok(());
            }

            let database = Database::new(&db).await?;
            let projects = database.list_projects().await?;

            if projects.is_empty() {
                println!("No projects found. Add projects with: kgate scan ~/gitZ --import");
                return Ok(());
            }

            println!(
                "{} Harvesting from {} projects...",
                style("🌱").green(),
                projects.len()
            );
            println!();

            let mut total_questions = 0;

            for proj in &projects {
                let path = PathBuf::from(&proj.path);
                if path.exists() {
                    print!("  {} {}... ", style("→").dim(), proj.name);
                    let result = harvester.harvest(&path);

                    if result.success {
                        println!(
                            "{} {} Qs",
                            style("✓").green(),
                            result.questions_generated
                        );
                        total_questions += result.questions_generated;
                    } else {
                        println!("{} skip", style("○").dim());
                    }
                }
            }

            // Save catalog
            let catalog = harvester.into_catalog();
            catalog.save(&catalog_path())?;

            println!();
            println!(
                "{} Harvested {} new questions. Catalog total: {}",
                style("🌳").green(),
                total_questions,
                catalog.total_questions
            );
        }

        HarvestCommands::Tree => {
            if !catalog_path().exists() {
                println!("No catalog found. Run: kgate harvest add <path>");
                return Ok(());
            }

            let catalog = QuestionCatalog::load(&catalog_path())?;
            println!();
            println!("{}", catalog.tree_view());
        }

        HarvestCommands::Stats => {
            if !catalog_path().exists() {
                println!("No catalog found. Run: kgate harvest add <path>");
                return Ok(());
            }

            let catalog = QuestionCatalog::load(&catalog_path())?;

            println!();
            println!("{}", style("Harvest Catalog Statistics").cyan().bold());
            println!();
            println!("  Total questions: {}", catalog.total_questions);
            println!("  Total domains:   {}", catalog.total_domains);
            println!(
                "  Last harvest:    {}",
                catalog.last_harvest
                    .map(|d| d.format("%Y-%m-%d %H:%M").to_string())
                    .unwrap_or("Never".to_string())
            );
            println!("  Catalog version: {}", catalog.version);
            println!();

            // Domain breakdown
            println!("  {}", style("By Domain:").bold());
            for domain in &catalog.root.children {
                println!(
                    "    {:12} {:4} questions ({} XP)",
                    style(&domain.name).cyan(),
                    domain.metadata.question_count,
                    domain.metadata.total_xp
                );
            }
        }

        HarvestCommands::Export { output } => {
            if !catalog_path().exists() {
                println!("No catalog found. Run: kgate harvest add <path>");
                return Ok(());
            }

            let catalog = QuestionCatalog::load(&catalog_path())?;
            let output_path = output.unwrap_or_else(|| crate::kgate_dir().join("catalog_export.json"));

            catalog.save(&output_path)?;

            println!(
                "{} Exported catalog ({} questions) to: {}",
                style("✓").green(),
                catalog.total_questions,
                output_path.display()
            );
        }
    }

    Ok(())
}
