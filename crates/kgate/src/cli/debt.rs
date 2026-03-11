use anyhow::Result;
use console::style;

use kgate_core::Database;

use crate::DebtCommands;

pub async fn cmd_debt(action: Option<DebtCommands>) -> Result<()> {
    let db = crate::db_path();
    let database = Database::new(&db).await?;
    let projects = database.list_projects().await?;

    let project = match projects.first() {
        Some(p) => p,
        None => {
            println!(
                "{} No projects. Run {} first.",
                style("✗").red(),
                style("kgate project add <path>").yellow()
            );
            return Ok(());
        }
    };

    match action {
        Some(DebtCommands::Add { action, description }) => {
            let weight = match action.as_str() {
                "concept" => 1,
                "architecture" => 2,
                "bugfix" => 1,
                "newfile" => 1,
                "code" => 2,
                _ => 1,
            };
            let new_debt = database
                .add_debt(&project.id, &action, weight, description.as_deref())
                .await?;
            println!(
                "{} +{} debt for '{}'. Total: {}/10",
                style("✓").green(),
                weight,
                action,
                new_debt
            );
        }
        Some(DebtCommands::Clear { amount }) => {
            let new_debt = database.clear_debt(&project.id, amount).await?;
            println!(
                "{} -{} debt. Remaining: {}/10",
                style("✓").green(),
                amount,
                new_debt
            );
        }
        None => {
            let debt = database.get_debt(&project.id).await?;
            println!("Debt for {}: {}/10", project.name, debt);
        }
    }

    Ok(())
}
