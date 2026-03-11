use anyhow::Result;
use console::style;
use console::Term;
use crossterm::event::{self, Event, KeyCode, KeyEvent};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use std::time::Duration;

use kgate_core::{grade_sprint, get_feedback, Database, ParsedSprint};
use crate::sound;
use crate::voice::{
    config::VoiceConfig,
    stt::{parse_voice_answer, SpeechToText, WhisperStt},
    tts::{create_tts_with_fallback, format_question_for_speech, TextToSpeech},
};

use super::helpers::{infer_domains, read_number_key, InputResult};

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
            tts.as_ref(),
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
    tts: &(dyn TextToSpeech + Send),
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
            style("🎤 Listening... (say 1, 2, 3, or 4, or press 1-4)")
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
