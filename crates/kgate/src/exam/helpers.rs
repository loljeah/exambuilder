use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEvent};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use std::time::Duration;

use kgate_core::{Database, ParsedQuestion, DOMAIN_KEYWORDS};
use kgate_core::db::ReviewItemRow;

/// Result of waiting for user input (voice or keyboard)
pub(crate) enum InputResult {
    Voice(String),
    Keyboard(usize),
    Timeout,
}

/// Read a single number key (1-4) in raw terminal mode
pub(crate) fn read_number_key(max: usize) -> Result<usize> {
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

/// Infer domains from question text using keyword matching
pub(crate) fn infer_domains(text: &str) -> Vec<String> {
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

/// Recover original MC question from sprint data for a review item
pub(crate) async fn recover_original_question(db: &Database, item: &ReviewItemRow) -> Option<ParsedQuestion> {
    let sprint = db.get_sprint(&item.project_id, item.sprint_number).await.ok()??;
    let questions: Vec<ParsedQuestion> = serde_json::from_str(&sprint.questions_json).ok()?;
    questions.into_iter().find(|q| q.number == item.question_number)
}
