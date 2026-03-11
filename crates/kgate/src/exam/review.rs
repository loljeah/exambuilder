use anyhow::Result;
use console::style;
use console::Term;
use crossterm::event::{self, Event};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use dialoguer::{theme::ColorfulTheme, Select};
use std::time::{Duration, Instant};

use kgate_core::{Database, ReviewItem, ReviewQuality, SpacedRepetitionEngine};
use kgate_core::db::ReviewItemRow;

use crate::sound;

use super::helpers::{read_number_key, recover_original_question};

/// Interactive spaced repetition review session
pub async fn cmd_review_session(db: &Database, limit: i32) -> Result<()> {
    let due_items = db.get_due_reviews(limit).await?;

    if due_items.is_empty() {
        let stats = db.get_review_stats().await?;
        println!("{} No reviews due right now!", style("✓").green());
        if stats.total_items > 0 {
            let accuracy = if stats.total_correct + stats.total_wrong > 0 {
                (stats.total_correct as f64 / (stats.total_correct + stats.total_wrong) as f64 * 100.0) as i32
            } else {
                0
            };
            println!("  {} items in queue | {}% accuracy | best streak: {}",
                stats.total_items,
                accuracy,
                stats.max_streak.unwrap_or(0),
            );
        }
        return Ok(());
    }

    let term = Term::stdout();
    term.clear_screen()?;

    println!(
        "{} {} items due for review\n",
        style("📚").cyan(),
        style(due_items.len()).bold()
    );

    let fast_mode = db.is_fast_answer_enabled().await.unwrap_or(true);
    let sound_enabled = db.is_sound_enabled().await.unwrap_or(true);

    let session_id = db.start_review_session().await?;
    let mut items_correct = 0;
    let mut items_reviewed = 0;
    let mut domains_seen: Vec<String> = Vec::new();

    for (i, item) in due_items.iter().enumerate() {
        println!(
            "{} [{}/{}] {}",
            style(format!("Review {}.", i + 1)).bold(),
            style(&item.domain).cyan(),
            style(format!("streak:{}", item.streak)).dim(),
            style(format!("EF:{:.1}", item.easiness_factor)).dim(),
        );

        // Try to recover original MC question
        let original_q = recover_original_question(db, item).await;

        let (is_correct, response_time_ms) = if let Some(ref q) = original_q {
            if !q.options.is_empty() {
                // MC mode: show question with options
                println!("{}\n", q.text);

                if let Some(ref code) = q.code_snippet {
                    println!("{}", style("┌─ Code ─────────────────────────────────").dim());
                    for line in code.lines() {
                        if line.starts_with("```") {
                            continue;
                        }
                        println!("{} {}", style("│").dim(), line);
                    }
                    println!("{}\n", style("└────────────────────────────────────────").dim());
                }

                let options: Vec<&str> = q.options.iter().map(|s| s.as_str()).collect();

                for (idx, opt) in options.iter().enumerate() {
                    println!("  {} {}", style(format!("{})", idx + 1)).cyan().bold(), opt);
                }
                println!();

                let start = Instant::now();

                let answer = if fast_mode && options.len() <= 4 {
                    println!("{}", style("Press 1-4 to answer...").dim());
                    let selection = read_number_key(options.len())?;
                    (b'A' + selection as u8) as char
                } else {
                    let selection = Select::with_theme(&ColorfulTheme::default())
                        .items(&options)
                        .default(0)
                        .interact()?;
                    (b'A' + selection as u8) as char
                };

                let elapsed_ms = start.elapsed().as_millis() as u64;
                let correct = answer == q.answer;

                if correct {
                    println!("  {}", style("Correct!").green());
                    if let Some(ref explanation) = q.explanation {
                        println!("  {}", style(explanation).dim());
                    }
                } else {
                    let correct_idx = (q.answer as u8 - b'A') as usize;
                    let correct_text = q.options.get(correct_idx).map(|s| s.as_str()).unwrap_or("?");
                    println!("  {} Answer: {} — {}", style("✗").red(), style(q.answer).green().bold(), correct_text);
                    if let Some(ref explanation) = q.explanation {
                        println!("  {}", style(explanation).dim());
                    }
                }

                (correct, elapsed_ms)
            } else {
                // Flashcard fallback (no MC options)
                flashcard_review(item)?
            }
        } else {
            // Flashcard fallback (sprint data unavailable)
            flashcard_review(item)?
        };

        // Play sound
        if sound_enabled {
            if is_correct {
                sound::play_correct();
            } else {
                sound::play_wrong();
            }
        }

        // SM-2 calculation
        let quality = ReviewQuality::from_score(is_correct, response_time_ms);
        let review_item = ReviewItem {
            id: item.id,
            project_id: item.project_id.clone(),
            sprint_number: item.sprint_number,
            question_number: item.question_number,
            question_text: item.question_text.clone(),
            correct_answer: item.correct_answer.clone(),
            domain: item.domain.clone(),
            easiness_factor: item.easiness_factor,
            repetition_count: item.repetition_count,
            interval_days: item.interval_days,
            next_review: item.next_review,
            last_reviewed: item.last_reviewed,
            times_correct: item.times_correct,
            times_wrong: item.times_wrong,
            streak: item.streak,
        };

        let update = SpacedRepetitionEngine::calculate_next_review(&review_item, quality);
        let next_review_str = update.next_review.format("%Y-%m-%d %H:%M:%S").to_string();

        db.update_review_item(
            item.id,
            update.easiness_factor,
            update.repetition_count,
            update.interval_days,
            &next_review_str,
            update.was_correct,
        ).await?;

        // Show next review interval
        let interval_display = if update.interval_days == 1 {
            "tomorrow".to_string()
        } else {
            format!("in {} days", update.interval_days)
        };
        println!("  {} Next review: {}", style("→").dim(), style(&interval_display).dim());
        println!();

        items_reviewed += 1;
        if is_correct {
            items_correct += 1;
        }
        if !domains_seen.contains(&item.domain) {
            domains_seen.push(item.domain.clone());
        }
    }

    // Session summary
    println!("{}", style("─".repeat(40)).dim());
    let accuracy = if items_reviewed > 0 {
        (items_correct as f64 / items_reviewed as f64 * 100.0) as i32
    } else {
        0
    };

    let accuracy_style = if accuracy >= 80 {
        style(format!("{}%", accuracy)).green()
    } else if accuracy >= 60 {
        style(format!("{}%", accuracy)).yellow()
    } else {
        style(format!("{}%", accuracy)).red()
    };

    println!(
        "\n{} Review complete: {}/{} correct ({})",
        style("📊").cyan(),
        items_correct,
        items_reviewed,
        accuracy_style,
    );
    println!(
        "  Domains: {}",
        domains_seen.join(", ")
    );

    // End session in DB
    let domains_json = serde_json::to_string(&domains_seen)?;
    db.end_review_session(session_id, items_reviewed, items_correct, &domains_json, 0).await?;

    Ok(())
}

/// Flashcard-style review when MC options are unavailable
fn flashcard_review(item: &ReviewItemRow) -> Result<(bool, u64)> {
    println!("{}\n", item.question_text);
    println!("{}", style("Press any key to reveal answer...").dim());

    let start = Instant::now();

    // Wait for keypress to reveal
    enable_raw_mode()?;
    loop {
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(_) = event::read()? {
                break;
            }
        }
    }
    disable_raw_mode()?;

    let elapsed_ms = start.elapsed().as_millis() as u64;

    println!("\n  Answer: {}\n", style(&item.correct_answer).green().bold());
    println!("  How well did you know it?");
    println!("  {} Knew it", style("1)").cyan().bold());
    println!("  {} Sort of", style("2)").cyan().bold());
    println!("  {} Didn't know", style("3)").cyan().bold());
    println!();

    let rating = read_number_key(3)?;

    let (is_correct, adjusted_time) = match rating {
        0 => (true, 2000u64),    // "Knew it" → Perfect
        1 => (true, 7000u64),    // "Sort of" → CorrectHesitation
        _ => (false, 0u64),      // "Didn't know" → IncorrectRemembered
    };

    let _ = elapsed_ms; // response time from reveal, not used for flashcard rating
    Ok((is_correct, adjusted_time))
}
