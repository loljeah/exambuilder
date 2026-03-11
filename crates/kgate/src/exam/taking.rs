use anyhow::Result;
use console::style;
use console::Term;
use dialoguer::{theme::ColorfulTheme, Select};

use kgate_core::{grade_sprint, get_feedback, Database, ParsedSprint};
use crate::sound;
use crate::voice::{
    config::VoiceConfig,
    tts::{create_tts_with_fallback, format_question_for_speech, TextToSpeech},
};

use super::helpers::{infer_domains, read_number_key};

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
