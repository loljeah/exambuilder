use anyhow::Result;
use console::style;
use std::path::Path;

use kgate_core::{scan_directory, find_exam_file, parse_exam_file, generate_sprint_id, Database};

pub async fn cmd_scan(db: &Database, path: &Path, import: bool) -> Result<()> {
    println!("Scanning {}...\n", style(path.display()).cyan());

    let result = scan_directory(path, 3)?;

    if result.projects.is_empty() {
        println!("No projects found.");
        return Ok(());
    }

    let mut imported = 0;

    for proj in &result.projects {
        let has_exam = proj.exam_file.is_some();
        let has_qa = !proj.qa_files.is_empty();
        let has_knowledge = !proj.knowledge_files.is_empty();

        if !has_exam && !has_qa && !has_knowledge && !proj.is_git_repo {
            continue;
        }

        let icon = if has_exam { "📝" } else if proj.is_git_repo { "📁" } else { "📄" };

        println!("{} {}", icon, style(&proj.name).bold());

        if let Some(ref exam) = proj.exam_file {
            let fname = exam.file_name().and_then(|n| n.to_str()).unwrap_or("exam.md");
            println!("   exam: {}", style(fname).green());

            if import {
                match import_exam(db, &proj.path, exam).await {
                    Ok((count, is_new)) => {
                        if is_new {
                            println!("   {} imported {} sprints", style("✓").green(), count);
                        } else {
                            println!("   {} updated {} sprints", style("↻").cyan(), count);
                        }
                        imported += 1;
                    }
                    Err(e) => {
                        println!("   {} import failed: {}", style("✗").red(), e);
                    }
                }
            }
        }

        for qa in &proj.qa_files {
            let fname = qa.file_name().and_then(|n| n.to_str()).unwrap_or("qa.md");
            println!("   qa: {}", style(fname).dim());
        }

        for k in &proj.knowledge_files {
            let fname = k.file_name().and_then(|n| n.to_str()).unwrap_or("knowledge.md");
            println!("   knowledge: {}", style(fname).dim());
        }

        println!();
    }

    println!("{}", style("─".repeat(40)).dim());
    println!(
        "Found: {} projects, {} exams, {} QA, {} knowledge",
        result.projects.len(),
        result.total_exams,
        result.total_qa,
        result.total_knowledge
    );

    if import {
        println!("Imported: {} projects", imported);
    }

    Ok(())
}

async fn import_exam(db: &Database, project_path: &Path, exam_file: &Path) -> Result<(usize, bool)> {
    let name = project_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unnamed")
        .to_string();

    let project = db
        .get_or_create_project(project_path.to_str().unwrap(), &name)
        .await?;

    // Check if project already has sprints
    let existing = db.get_sprints(&project.id).await?;
    let is_new = existing.is_empty();

    let content = std::fs::read_to_string(exam_file)?;
    let exam = parse_exam_file(&content)?;

    for sprint in &exam.sprints {
        let questions_json = serde_json::to_string(&sprint.questions)?;
        let answers: Vec<char> = sprint.questions.iter().map(|q| q.answer).collect();
        let answer_key_json = serde_json::to_string(&answers)?;

        let s = kgate_core::Sprint {
            id: 0,
            project_id: project.id.clone(),
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
            sprint_id: Some(generate_sprint_id(&project.id, sprint.number, &sprint.topic)),
            source_project_name: Some(exam.project_name.clone()),
        };

        db.upsert_sprint(&s).await?;
    }

    Ok((exam.sprints.len(), is_new))
}

pub async fn cmd_auto_load(db: &Database, project_id: &str, project_path: &Path) -> Result<bool> {
    if let Some(exam_file) = find_exam_file(project_path) {
        let content = std::fs::read_to_string(&exam_file)?;
        let exam = parse_exam_file(&content)?;

        for sprint in &exam.sprints {
            let questions_json = serde_json::to_string(&sprint.questions)?;
            let answers: Vec<char> = sprint.questions.iter().map(|q| q.answer).collect();
            let answer_key_json = serde_json::to_string(&answers)?;

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
                sprint_id: Some(generate_sprint_id(project_id, sprint.number, &sprint.topic)),
                source_project_name: Some(exam.project_name.clone()),
            };

            db.upsert_sprint(&s).await?;
        }

        println!(
            "{} Auto-loaded {} sprints from {}",
            style("✓").green(),
            exam.sprints.len(),
            exam_file.file_name().and_then(|n| n.to_str()).unwrap_or("exam.md")
        );

        return Ok(true);
    }

    Ok(false)
}
