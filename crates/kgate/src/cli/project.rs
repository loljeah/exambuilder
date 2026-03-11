use anyhow::Result;
use console::style;

use kgate_core::Database;

use crate::ProjectCommands;

pub async fn cmd_project(action: ProjectCommands) -> Result<()> {
    let db = crate::db_path();
    let database = Database::new(&db).await?;

    match action {
        ProjectCommands::List => {
            let projects = database.list_projects().await?;
            if projects.is_empty() {
                println!(
                    "No projects. Add: {}",
                    style("kgate project add <path>").yellow()
                );
            } else {
                println!("{}", style("Projects:").bold());
                for (i, p) in projects.iter().enumerate() {
                    let debt = database.get_debt(&p.id).await?;
                    let icon = if debt >= 10 {
                        "🔒"
                    } else if i == 0 {
                        "→"
                    } else {
                        " "
                    };
                    println!("  {} {} ({}) — debt: {}/10", icon, p.name, p.id, debt);
                }
            }
        }
        ProjectCommands::Add { path } => {
            let abs_path = std::fs::canonicalize(&path)?;
            let name = abs_path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unnamed")
                .to_string();

            let project = database
                .get_or_create_project(abs_path.to_str().unwrap(), &name)
                .await?;

            // Auto-load exam if exists
            crate::scan::cmd_auto_load(&database, &project.id, &abs_path).await?;

            println!(
                "{} Added: {} ({})",
                style("✓").green(),
                project.name,
                project.id
            );
        }
        ProjectCommands::Set { id } => {
            if let Some(project) = database.get_project_by_id(&id).await? {
                database.set_active_project(&id).await?;
                println!(
                    "{} Switched to: {} ({})",
                    style("✓").green(),
                    project.name,
                    project.id
                );
            } else {
                println!("{} Project '{}' not found", style("✗").red(), id);
            }
        }
    }

    Ok(())
}
