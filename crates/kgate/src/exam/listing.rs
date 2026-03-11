use anyhow::Result;
use console::style;
use std::path::Path;

use kgate_core::{parse_exam_file, generate_sprint_id, Database};

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
            "  {} Sprint {}: {} — {} XP{}",
            status,
            s.sprint_number,
            s.topic,
            s.xp_available,
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
