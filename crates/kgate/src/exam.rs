use anyhow::Result;
use console::{style, Term};
use crossterm::event::{self, Event, KeyCode, KeyEvent};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use dialoguer::{theme::ColorfulTheme, Select};
use std::path::Path;
use std::time::{Duration, Instant};

use kgate_core::{grade_sprint, get_feedback, parse_exam_file, generate_sprint_id, Database, ParsedQuestion, ParsedSprint, ReviewItem, ReviewQuality, SpacedRepetitionEngine, DOMAIN_KEYWORDS};
use kgate_core::db::ReviewItemRow;

use crate::sound;
use crate::voice::{
    config::VoiceConfig,
    tts::{create_tts_with_fallback, format_question_for_speech, TextToSpeech},
};

pub async fn cmd_exam_list(db: &Database, project_id: &str) -> Result<()> {
    let sprints = db.get_sprints(project_id).await?;

    if sprints.is_empty() {
        println!("No sprints loaded yet.");
        println!("Import an exam with: {}", style("kgate exam load <file>").yellow());
        return Ok(());
    }

    println!("{}", style("Sprints:").bold());
    for s in sprints {
        let status = match s.status.as_str() {
            "passed" => style("✓ PASSED").green(),
            "pending" => style("○ pending").dim(),
            _ => style(s.status.as_str()).white(),
        };
        let id_label = s.sprint_id.as_deref().map(|id| format!(" [{}]", id)).unwrap_or_default();
        println!(
            "  {} Sprint {}: {} — {}{}",
            status,
            s.sprint_number,
            s.topic,
            format!("{} XP", s.xp_available),
            style(id_label).dim()
        );
    }

    Ok(())
}

pub async fn cmd_exam_load(db: &Database, project_id: &str, file: &Path) -> Result<()> {
    let content = std::fs::read_to_string(file)?;
    let exam = parse_exam_file(&content)?;

    println!("Loaded exam: {}", style(&exam.project_name).cyan());

    for sprint in &exam.sprints {
        let questions_json = serde_json::to_string(&sprint.questions)?;
        let answers: Vec<char> = sprint.questions.iter().map(|q| q.answer).collect();
        let answer_key_json = serde_json::to_string(&answers)?;

        let sid = generate_sprint_id(project_id, sprint.number, &sprint.topic);

        let s = kgate_core::Sprint {
            id: 0,
            project_id: project_id.to_string(),
            sprint_number: sprint.number,
            topic: sprint.topic.clone(),
            questions_json,
            answer_key_json,
            status: "pending".to_string(),
            best_score: None,
            attempts: 0,
            xp_available: sprint.total_xp,
            xp_earned: 0,
            created_at: chrono::Utc::now(),
            passed_at: None,
            sprint_id: Some(sid.clone()),
            source_project_name: Some(exam.project_name.clone()),
        };

        db.upsert_sprint(&s).await?;
        println!(
            "  {} Sprint {}: {} ({} questions, {} XP) [{}]",
            style("✓").green(),
            sprint.number,
            sprint.topic,
            sprint.questions.len(),
            sprint.total_xp,
            style(&sid).dim()
        );
    }

    Ok(())
}

pub async fn cmd_exam_take(db: &Database, project_id: &str, sprint_number: i32, exam_id: Option<usize>) -> Result<()> {
    let sprint_data = db.get_sprint(project_id, sprint_number).await?;

    let sprint_data = match sprint_data {
        Some(s) => s,
        None => {
            println!("{} Sprint {} not found", style("✗").red(), sprint_number);
            return Ok(());
        }
    };

    let all_questions: Vec<kgate_core::ParsedQuestion> =
        serde_json::from_str(&sprint_data.questions_json)?;

    // Filter to MC-only (questions must have options)
    let questions: Vec<kgate_core::ParsedQuestion> = all_questions
        .into_iter()
        .filter(|q| !q.options.is_empty())
        .collect();

    if questions.is_empty() {
        println!("{} No multiple-choice questions in this sprint", style("✗").red());
        return Ok(());
    }

    let parsed_sprint = ParsedSprint {
        number: sprint_data.sprint_number,
        topic: sprint_data.topic.clone(),
        target_minutes: 3,
        pass_percent: 60,
        total_xp: sprint_data.xp_available,
        questions: questions.clone(),
    };

    let term = Term::stdout();
    term.clear_screen()?;

    // Load TTS if available (with automatic engine fallback)
    let tts: Option<Box<dyn TextToSpeech + Send>> = VoiceConfig::load()
        .ok()
        .filter(|c| c.general.enabled)
        .and_then(|c| create_tts_with_fallback(&c.tts));

    let id_label = exam_id.map(|id| format!(" [Exam {}]", id)).unwrap_or_default();
    println!(
        "{}",
        style(format!("Sprint {}: {}{}", sprint_data.sprint_number, sprint_data.topic, id_label))
            .cyan()
            .bold()
    );
    println!(
        "Pass: 60% | {} questions | {} XP\n",
        questions.len(),
        sprint_data.xp_available
    );

    // Announce sprint via TTS
    if let Some(ref tts) = tts {
        let intro = format!(
            "Sprint {}: {}. {} questions.",
            sprint_data.sprint_number,
            sprint_data.topic,
            questions.len()
        );
        let _ = tts.speak_blocking(&intro);
    }

    let mut user_answers: Vec<char> = Vec::new();
    let fast_mode = db.is_fast_answer_enabled().await.unwrap_or(true);
    let sound_enabled = db.is_sound_enabled().await.unwrap_or(true);

    // One question at a time
    for (i, q) in questions.iter().enumerate() {
        println!(
            "{} [{}] {} — {} XP",
            style(format!("Q{}.", q.number)).bold(),
            q.tier,
            q.difficulty,
            q.xp
        );
        println!("{}", q.text);

        // Show code snippet if present, with box formatting
        if let Some(ref code) = q.code_snippet {
            println!();
            println!("{}", style("┌─ Code ─────────────────────────────────").dim());
            for line in code.lines() {
                // Skip markdown code fence markers
                if line.starts_with("```") {
                    continue;
                }
                println!("{} {}", style("│").dim(), line);
            }
            println!("{}", style("└────────────────────────────────────────").dim());
        }
        println!();

        let options: Vec<&str> = q.options.iter().map(|s| s.as_str()).collect();

        // Read question aloud via TTS
        if let Some(ref tts) = tts {
            let speech = format_question_for_speech(&q.text, &q.options, q.code_snippet.as_deref());
            let _ = tts.speak_blocking(&speech);
        }

        let answer = if fast_mode && options.len() <= 4 {
            // Fast answer mode: show options with numbers, wait for single keypress
            for (idx, opt) in options.iter().enumerate() {
                println!("  {} {}", style(format!("{})", idx + 1)).cyan().bold(), opt);
            }
            println!();
            println!("{}", style("Press 1-4 to answer...").dim());

            let selection = read_number_key(options.len())?;
            (b'A' + selection as u8) as char
        } else {
            // Standard dialoguer mode
            let selection = Select::with_theme(&ColorfulTheme::default())
                .items(&options)
                .default(0)
                .interact()?;

            (b'A' + selection as u8) as char
        };

        user_answers.push(answer);

        // Play sound for immediate feedback and track combo
        let correct_answers: Vec<char> = serde_json::from_str(&sprint_data.answer_key_json)?;
        if i < correct_answers.len() {
            let is_correct = answer == correct_answers[i];
            // Track question stats for combo chain
            let _ = db.update_question_stats(is_correct).await;

            if sound_enabled {
                if is_correct {
                    sound::play_correct();
                } else {
                    sound::play_wrong();
                }
            }
        }

        // Clear for next question (except last)
        if i < questions.len() - 1 {
            println!();
        }
    }

    // Grade
    let attempt = sprint_data.attempts + 1;
    let result = grade_sprint(&parsed_sprint, &user_answers, attempt);

    // Update DB
    let xp = if result.passed { result.xp_earned } else { 0 };
    db.record_sprint_attempt(project_id, sprint_number, result.score_percent, result.passed, xp)
        .await?;

    if result.passed {
        db.update_profile_xp(result.xp_earned).await?;
        db.update_streak(true).await?;
        db.clear_debt(project_id, 3).await?;
    } else {
        db.update_streak(false).await?;
    }

    // Check for badges
    let new_badges = db.check_and_award_badges(project_id).await?;

    // Check for perfect score badge and track perfect sprints
    if result.passed && result.score_percent == 100 {
        let _ = db.unlock_badge("perfect", Some(project_id)).await;
        let _ = db.record_perfect_sprint().await;
    }

    // Show results
    println!("\n{}", style("─".repeat(40)).dim());

    let feedback = get_feedback(&result, &questions);

    if result.passed {
        println!(
            "\n{} {}",
            style("✓").green().bold(),
            style(&feedback.message).green()
        );
        let profile = db.get_profile().await?;
        if profile.current_streak > 1 {
            println!("  Streak: {} {}", profile.current_streak, style("🔥").red());
        }

        // Show new badges
        for badge_id in &new_badges {
            let badge_name = match badge_id.as_str() {
                "first_sprint" => "First Sprint",
                "streak_3" => "On Fire",
                "streak_5" => "Blazing",
                "streak_10" => "Unstoppable",
                "level_2" => "Config Wrangler",
                "level_3" => "System Operator",
                "level_5" => "Infra Architect",
                "perfect" => "Perfect Score",
                "project_clear" => "Gate Cleared",
                "xp_100" => "Century",
                "xp_500" => "Half K",
                "xp_1000" => "Grand Master",
                _ => badge_id,
            };
            println!(
                "  {} Badge unlocked: {}",
                style("🏅").yellow(),
                style(badge_name).cyan().bold()
            );
        }
    } else {
        println!(
            "\n{} {}",
            style("○").yellow(),
            style(&feedback.message).yellow()
        );

        if feedback.show_hints {
            println!("\n{}", style("Hints for wrong answers:").dim());
            for (qnum, hint) in &feedback.hints {
                println!("  Q{}: {}", qnum, hint);
            }
            println!("\nTry again? Run: kgate exam take {}", sprint_number);
        }

        if feedback.show_answers {
            println!("\n{}", style("Answers:").dim());
            for (qnum, answer, exp) in &feedback.explanations {
                println!("  Q{} → {}: {}", qnum, style(answer).green(), exp);
            }
        }
    }

    // Add wrong answers to spaced repetition review queue
    {
        let correct_answers: Vec<char> = serde_json::from_str(&sprint_data.answer_key_json)?;
        for (i, q) in questions.iter().enumerate() {
            if i < user_answers.len() && i < correct_answers.len() && user_answers[i] != correct_answers[i] {
                let domains = infer_domains(&q.text);
                let domain = domains.first().map(|s| s.as_str()).unwrap_or("architecture");
                let _ = db.add_review_item(
                    project_id,
                    sprint_number,
                    q.number,
                    &q.text,
                    &correct_answers[i].to_string(),
                    domain,
                ).await;
            }
        }
    }

    // Collect correct answers and track domains
    if result.passed {
        let correct_answers: Vec<char> = serde_json::from_str(&sprint_data.answer_key_json)?;

        for (i, q) in questions.iter().enumerate() {
            if i < user_answers.len() && user_answers[i] == correct_answers[i] {
                // Extract domains from question text
                let domains = infer_domains(&q.text);

                // Collect the question
                let _ = db
                    .collect_question(
                        project_id,
                        sprint_number,
                        q.number,
                        &q.text,
                        &correct_answers[i].to_string(),
                        &user_answers[i].to_string(),
                        &q.tier,
                        q.xp,
                        &domains,
                    )
                    .await;

                // Update domain progress
                for domain in &domains {
                    let _ = db.update_domain_progress(domain, q.xp, true).await;
                }

                // Record domain connections (if multiple domains in same question)
                if domains.len() > 1 {
                    for j in 0..domains.len() {
                        for k in (j + 1)..domains.len() {
                            let _ = db.record_domain_connection(&domains[j], &domains[k]).await;
                        }
                    }
                }
            }
        }

        // Check collection achievements
        let achievements = db.check_collection_achievements().await?;
        for ach in achievements {
            println!(
                "  {} Achievement unlocked: {}",
                style("🏆").yellow(),
                style(&ach).magenta().bold()
            );
            if sound_enabled {
                sound::play_badge();
            }
        }

        // Play sprint pass sound
        if sound_enabled {
            sound::play_sprint_pass();
        }
    }

    Ok(())
}

// Helper function to read a single number key (1-4)
fn read_number_key(max: usize) -> Result<usize> {
    enable_raw_mode()?;

    let result = loop {
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(KeyEvent { code, .. }) = event::read()? {
                match code {
                    KeyCode::Char('1') if max >= 1 => break Ok(0),
                    KeyCode::Char('2') if max >= 2 => break Ok(1),
                    KeyCode::Char('3') if max >= 3 => break Ok(2),
                    KeyCode::Char('4') if max >= 4 => break Ok(3),
                    KeyCode::Esc => break Err(anyhow::anyhow!("Cancelled")),
                    _ => {} // Ignore other keys
                }
            }
        }
    };

    disable_raw_mode()?;
    println!(); // Move to next line after keypress
    result
}

// Infer domains from question text using keyword matching
fn infer_domains(text: &str) -> Vec<String> {
    let text_lower = text.to_lowercase();
    let mut domains = Vec::new();

    for (domain_id, keywords) in DOMAIN_KEYWORDS {
        for keyword in *keywords {
            if text_lower.contains(keyword) {
                domains.push(domain_id.to_string());
                break;
            }
        }
    }

    // If no domains found, tag as "general"
    if domains.is_empty() {
        domains.push("architecture".to_string());
    }

    domains
}

// ============================================
// Spaced Repetition Review Session
// ============================================

/// Recover original MC question from sprint data for a review item
async fn recover_original_question(db: &Database, item: &ReviewItemRow) -> Option<ParsedQuestion> {
    let sprint = db.get_sprint(&item.project_id, item.sprint_number).await.ok()??;
    let questions: Vec<ParsedQuestion> = serde_json::from_str(&sprint.questions_json).ok()?;
    questions.into_iter().find(|q| q.number == item.question_number)
}

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

// ============================================
// Voice Mode Exam Taking
// ============================================

use crate::voice::stt::{parse_voice_answer, SpeechToText, WhisperStt};

/// Take an exam sprint with voice mode enabled
pub async fn cmd_exam_take_voice(db: &Database, project_id: &str, sprint_number: i32, exam_id: Option<usize>) -> Result<()> {
    // Load voice config
    let voice_config = VoiceConfig::load()?;

    if !voice_config.is_configured() {
        println!(
            "{} Voice mode not configured. Run: {}",
            style("✗").red(),
            style("kgate voice setup").yellow()
        );
        return Ok(());
    }

    // Create TTS with fallback and STT instances
    let tts: Box<dyn TextToSpeech + Send> = match create_tts_with_fallback(&voice_config.tts) {
        Some(tts) => tts,
        None => {
            println!(
                "{} No TTS engine available. Install espeak-ng, piper, or kokoro.",
                style("✗").red()
            );
            return Ok(());
        }
    };

    let stt = WhisperStt::new(&voice_config.stt, &voice_config.calibration);
    let stt_available = !VoiceConfig::detect_stt_engines().is_empty();

    // Load sprint data
    let sprint_data = db.get_sprint(project_id, sprint_number).await?;

    let sprint_data = match sprint_data {
        Some(s) => s,
        None => {
            println!("{} Sprint {} not found", style("✗").red(), sprint_number);
            return Ok(());
        }
    };

    let all_questions: Vec<kgate_core::ParsedQuestion> =
        serde_json::from_str(&sprint_data.questions_json)?;

    // Filter to MC-only (questions must have options)
    let questions: Vec<kgate_core::ParsedQuestion> = all_questions
        .into_iter()
        .filter(|q| !q.options.is_empty())
        .collect();

    if questions.is_empty() {
        println!("{} No multiple-choice questions in this sprint", style("✗").red());
        tts.speak_blocking("No multiple choice questions available in this sprint.")?;
        return Ok(());
    }

    let parsed_sprint = ParsedSprint {
        number: sprint_data.sprint_number,
        topic: sprint_data.topic.clone(),
        target_minutes: 3,
        pass_percent: 60,
        total_xp: sprint_data.xp_available,
        questions: questions.clone(),
    };

    let term = Term::stdout();
    term.clear_screen()?;

    // Announce sprint
    let id_label = exam_id.map(|id| format!(" [Exam {}]", id)).unwrap_or_default();
    println!(
        "{}",
        style(format!(
            "🎙️ Voice Mode - Sprint {}: {}{}",
            sprint_data.sprint_number, sprint_data.topic, id_label
        ))
        .cyan()
        .bold()
    );
    println!(
        "Pass: 60% | {} questions | {} XP\n",
        questions.len(),
        sprint_data.xp_available
    );

    let intro = format!(
        "Sprint {}: {}. {} questions, {} experience points available.",
        sprint_data.sprint_number,
        sprint_data.topic,
        questions.len(),
        sprint_data.xp_available
    );
    tts.speak_blocking(&intro)?;

    let mut user_answers: Vec<char> = Vec::new();
    let correct_answers: Vec<char> = serde_json::from_str(&sprint_data.answer_key_json)?;

    // Process each question
    for (i, q) in questions.iter().enumerate() {
        // Display question visually
        println!(
            "\n{} [{}] {} — {} XP",
            style(format!("Q{}.", q.number)).bold(),
            q.tier,
            q.difficulty,
            q.xp
        );
        println!("{}", q.text);

        // Show code snippet if present
        if let Some(ref code) = q.code_snippet {
            println!();
            println!("{}", style("┌─ Code ─────────────────────────────────").dim());
            for line in code.lines() {
                if line.starts_with("```") {
                    continue;
                }
                println!("{} {}", style("│").dim(), line);
            }
            println!("{}", style("└────────────────────────────────────────").dim());
        }

        // Show options (numbered)
        let options: Vec<&str> = q.options.iter().map(|s| s.as_str()).collect();
        println!();
        for (idx, opt) in options.iter().enumerate() {
            println!("  {} {}", style(format!("{})", idx + 1)).cyan().bold(), opt);
        }
        println!();

        // Speak question
        let speech_text = format_question_for_speech(
            &q.text,
            &q.options,
            q.code_snippet.as_deref(),
        );
        tts.speak_blocking(&speech_text)?;

        // Get answer (voice with keyboard fallback)
        let answer = get_voice_answer(
            &tts,
            &stt,
            stt_available,
            &voice_config,
            options.len(),
        )?;

        let answer_num = (answer as u8 - b'A' + 1) as usize;
        println!(
            "  {} Your answer: {}",
            style("→").dim(),
            style(answer_num).cyan().bold()
        );

        user_answers.push(answer);

        // Immediate feedback
        if i < correct_answers.len() {
            let is_correct = answer == correct_answers[i];
            let _ = db.update_question_stats(is_correct).await;

            if is_correct {
                println!("  {}", style("Correct!").green());
                sound::play_correct();

                // Mini sumup: why the answer is correct + bonus fact
                let mut sumup_parts: Vec<String> = vec!["Correct!".to_string()];
                if let Some(ref explanation) = q.explanation {
                    println!("  {}", style(explanation).dim());
                    sumup_parts.push(explanation.clone());
                }
                if let Some(ref extra) = q.extra {
                    println!("  {}", style(format!("💡 {}", extra)).dim());
                    sumup_parts.push(extra.clone());
                }
                tts.speak_blocking(&sumup_parts.join(". "))?;
            } else {
                println!("  {}", style("Not quite.").yellow());
                tts.speak("Not quite.")?;
                sound::play_wrong();
            }
        }

        // Brief pause between questions
        std::thread::sleep(Duration::from_millis(500));
    }

    // Grade sprint
    let attempt = sprint_data.attempts + 1;
    let result = grade_sprint(&parsed_sprint, &user_answers, attempt);

    // Update DB
    let xp = if result.passed { result.xp_earned } else { 0 };
    db.record_sprint_attempt(project_id, sprint_number, result.score_percent, result.passed, xp)
        .await?;

    if result.passed {
        db.update_profile_xp(result.xp_earned).await?;
        db.update_streak(true).await?;
        db.clear_debt(project_id, 3).await?;
    } else {
        db.update_streak(false).await?;
    }

    // Check for badges
    let new_badges = db.check_and_award_badges(project_id).await?;

    if result.passed && result.score_percent == 100 {
        let _ = db.unlock_badge("perfect", Some(project_id)).await;
        let _ = db.record_perfect_sprint().await;
    }

    // Show and speak results
    println!("\n{}", style("─".repeat(40)).dim());

    let feedback = get_feedback(&result, &questions);

    if result.passed {
        println!(
            "\n{} {}",
            style("✓").green().bold(),
            style(&feedback.message).green()
        );

        let result_speech = format!(
            "Sprint passed! You scored {}%. {} experience points earned.",
            result.score_percent, result.xp_earned
        );
        tts.speak_blocking(&result_speech)?;

        let profile = db.get_profile().await?;
        if profile.current_streak > 1 {
            println!("  Streak: {} {}", profile.current_streak, style("🔥").red());
            tts.speak(&format!("Streak: {}", profile.current_streak))?;
        }

        // Announce badges
        for badge_id in &new_badges {
            let badge_name = match badge_id.as_str() {
                "first_sprint" => "First Sprint",
                "streak_3" => "On Fire",
                "streak_5" => "Blazing",
                "streak_10" => "Unstoppable",
                "level_2" => "Config Wrangler",
                "level_3" => "System Operator",
                "level_5" => "Infra Architect",
                "perfect" => "Perfect Score",
                "project_clear" => "Gate Cleared",
                "xp_100" => "Century",
                "xp_500" => "Half K",
                "xp_1000" => "Grand Master",
                _ => badge_id,
            };
            println!(
                "  {} Badge unlocked: {}",
                style("🏅").yellow(),
                style(badge_name).cyan().bold()
            );
            tts.speak(&format!("Badge unlocked: {}", badge_name))?;
        }

        sound::play_sprint_pass();
    } else {
        println!(
            "\n{} {}",
            style("○").yellow(),
            style(&feedback.message).yellow()
        );

        let result_speech = format!(
            "Not quite. You scored {}%. Need 60% to pass. Try again!",
            result.score_percent
        );
        tts.speak_blocking(&result_speech)?;

        if feedback.show_hints {
            println!("\n{}", style("Hints for wrong answers:").dim());
            for (qnum, hint) in &feedback.hints {
                println!("  Q{}: {}", qnum, hint);
            }
        }

        if feedback.show_answers {
            println!("\n{}", style("Answers:").dim());
            for (qnum, answer, exp) in &feedback.explanations {
                println!("  Q{} → {}: {}", qnum, style(answer).green(), exp);
            }
        }
    }

    // Add wrong answers to spaced repetition review queue
    for (i, q) in questions.iter().enumerate() {
        if i < user_answers.len() && i < correct_answers.len() && user_answers[i] != correct_answers[i] {
            let domains = infer_domains(&q.text);
            let domain = domains.first().map(|s| s.as_str()).unwrap_or("architecture");
            let _ = db.add_review_item(
                project_id,
                sprint_number,
                q.number,
                &q.text,
                &correct_answers[i].to_string(),
                domain,
            ).await;
        }
    }

    // Collect questions and track domains (same as regular mode)
    if result.passed {
        for (i, q) in questions.iter().enumerate() {
            if i < user_answers.len() && user_answers[i] == correct_answers[i] {
                let domains = infer_domains(&q.text);

                let _ = db
                    .collect_question(
                        project_id,
                        sprint_number,
                        q.number,
                        &q.text,
                        &correct_answers[i].to_string(),
                        &user_answers[i].to_string(),
                        &q.tier,
                        q.xp,
                        &domains,
                    )
                    .await;

                for domain in &domains {
                    let _ = db.update_domain_progress(domain, q.xp, true).await;
                }

                if domains.len() > 1 {
                    for j in 0..domains.len() {
                        for k in (j + 1)..domains.len() {
                            let _ = db.record_domain_connection(&domains[j], &domains[k]).await;
                        }
                    }
                }
            }
        }

        let achievements = db.check_collection_achievements().await?;
        for ach in achievements {
            println!(
                "  {} Achievement unlocked: {}",
                style("🏆").yellow(),
                style(&ach).magenta().bold()
            );
            tts.speak(&format!("Achievement unlocked: {}", ach))?;
            sound::play_badge();
        }
    }

    Ok(())
}

/// Get answer via voice with keyboard fallback
fn get_voice_answer(
    tts: &Box<dyn TextToSpeech + Send>,
    stt: &WhisperStt,
    stt_available: bool,
    config: &VoiceConfig,
    num_options: usize,
) -> Result<char> {
    if !stt_available {
        // No STT available, use keyboard only
        println!("{}", style("Press 1-4 to answer...").dim());
        let selection = read_number_key(num_options)?;
        return Ok((b'A' + selection as u8) as char);
    }

    let max_attempts = 3;
    let mut attempts = 0;

    loop {
        attempts += 1;

        println!(
            "{}",
            style(format!(
                "🎤 Listening... (say 1, 2, 3, or 4, or press 1-4)"
            ))
            .dim()
        );

        // Try to listen for voice input with concurrent keyboard input
        let listen_result = listen_with_keyboard_fallback(
            stt,
            config.calibration.max_wait_time_ms,
            num_options,
        );

        match listen_result {
            Ok(InputResult::Voice(text)) => {
                if let Some(answer) = parse_voice_answer(&text) {
                    // Confirm if configured
                    if config.calibration.confirm_answers {
                        println!(
                            "  {} Heard: \"{}\" → {}",
                            style("?").yellow(),
                            text,
                            answer
                        );
                        let heard_num = (answer as u8 - b'A' + 1) as usize;
                        tts.speak_blocking(&format!("I heard {}. Is that correct?", heard_num))?;

                        // Wait for confirmation (voice or keyboard)
                        println!(
                            "{}",
                            style("Say yes/no or press Y/N...").dim()
                        );

                        if wait_for_confirmation(stt, config.calibration.max_wait_time_ms)? {
                            return Ok(answer);
                        } else {
                            tts.speak("Let's try again.")?;
                            continue;
                        }
                    } else {
                        return Ok(answer);
                    }
                } else {
                    println!(
                        "  {} Didn't understand: \"{}\"",
                        style("?").yellow(),
                        text
                    );
                    tts.speak("I didn't understand. Please say 1, 2, 3, or 4.")?;
                }
            }
            Ok(InputResult::Keyboard(selection)) => {
                return Ok((b'A' + selection as u8) as char);
            }
            Ok(InputResult::Timeout) => {
                if attempts >= max_attempts {
                    println!(
                        "{}",
                        style("Voice timeout. Using keyboard input.").yellow()
                    );
                    tts.speak("Using keyboard input.")?;
                    println!("{}", style("Press 1-4 to answer...").dim());
                    let selection = read_number_key(num_options)?;
                    return Ok((b'A' + selection as u8) as char);
                } else {
                    tts.speak("I didn't hear anything. Please try again.")?;
                }
            }
            Err(e) => {
                println!(
                    "{} Voice error: {}. Using keyboard.",
                    style("⚠").yellow(),
                    e
                );
                println!("{}", style("Press 1-4 to answer...").dim());
                let selection = read_number_key(num_options)?;
                return Ok((b'A' + selection as u8) as char);
            }
        }
    }
}

enum InputResult {
    Voice(String),
    Keyboard(usize),
    Timeout,
}

/// Listen for voice input with concurrent keyboard polling
fn listen_with_keyboard_fallback(
    stt: &WhisperStt,
    timeout_ms: u32,
    num_options: usize,
) -> Result<InputResult> {
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::mpsc;
    use std::sync::Arc;
    use std::thread;

    let (tx, rx) = mpsc::channel();
    let stop_flag = Arc::new(AtomicBool::new(false));
    let stop_flag_clone = Arc::clone(&stop_flag);

    // Spawn keyboard listener thread
    let keyboard_handle = thread::spawn(move || {
        enable_raw_mode().ok();
        loop {
            if stop_flag_clone.load(Ordering::Relaxed) {
                break;
            }
            if event::poll(Duration::from_millis(100)).unwrap_or(false) {
                if let Ok(Event::Key(KeyEvent { code, .. })) = event::read() {
                    let result = match code {
                        KeyCode::Char('1') if num_options >= 1 => Some(0),
                        KeyCode::Char('2') if num_options >= 2 => Some(1),
                        KeyCode::Char('3') if num_options >= 3 => Some(2),
                        KeyCode::Char('4') if num_options >= 4 => Some(3),
                        _ => None,
                    };
                    if let Some(selection) = result {
                        let _ = tx.send(InputResult::Keyboard(selection));
                        break;
                    }
                }
            }
        }
        disable_raw_mode().ok();
    });

    // Run STT in main thread (it blocks on recording)
    match stt.listen_with_timeout(timeout_ms) {
        Ok(Some(text)) => {
            // Signal keyboard thread to stop
            stop_flag.store(true, Ordering::Relaxed);
            keyboard_handle.join().ok();
            Ok(InputResult::Voice(text))
        }
        Ok(None) => {
            // Check if keyboard got input while waiting
            match rx.try_recv() {
                Ok(result) => {
                    stop_flag.store(true, Ordering::Relaxed);
                    keyboard_handle.join().ok();
                    Ok(result)
                }
                Err(_) => {
                    stop_flag.store(true, Ordering::Relaxed);
                    keyboard_handle.join().ok();
                    Ok(InputResult::Timeout)
                }
            }
        }
        Err(e) => {
            stop_flag.store(true, Ordering::Relaxed);
            keyboard_handle.join().ok();
            Err(e)
        }
    }
}

/// Wait for yes/no confirmation via voice or keyboard
fn wait_for_confirmation(_stt: &WhisperStt, timeout_ms: u32) -> Result<bool> {
    enable_raw_mode()?;

    let start = std::time::Instant::now();
    let timeout = Duration::from_millis(timeout_ms as u64);

    loop {
        // Check keyboard first (voice confirmation could be added later)
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(KeyEvent { code, .. }) = event::read()? {
                disable_raw_mode()?;
                return match code {
                    KeyCode::Char('y') | KeyCode::Char('Y') | KeyCode::Enter => Ok(true),
                    KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => Ok(false),
                    _ => Ok(true), // Default to yes for other keys
                };
            }
        }

        // Check timeout
        if start.elapsed() > timeout {
            disable_raw_mode()?;
            // Default to accepting the answer on timeout
            return Ok(true);
        }
    }
}
