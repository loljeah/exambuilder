use anyhow::Result;
use console::style;

use kgate_core::Database;

/// Pick a random unpassed sprint and start it
pub async fn cmd_random() -> Result<()> {
    use rand::seq::SliceRandom;

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
        println!(
            "{} No exams loaded. Run: {}",
            style("✗").red(),
            style("kgate scan ~/gitZ --import").yellow()
        );
        return Ok(());
    }

    // Collect all unpassed sprints across all projects
    let mut unpassed: Vec<(String, i32, String)> = Vec::new(); // (project_id, sprint_number, topic)

    for proj in &projects {
        let sprints = database.get_sprints(&proj.id).await?;
        for s in &sprints {
            if s.status != "passed" {
                unpassed.push((proj.id.clone(), s.sprint_number, s.topic.clone()));
            }
        }
    }

    if unpassed.is_empty() {
        println!();
        println!(
            "  {} {}",
            style("🎉").cyan(),
            style("All sprints passed! You've cleared every gate.").green().bold()
        );
        println!();
        println!("  Add more exams with: {}", style("kgate scan ~/gitZ --import").yellow());
        println!();
        return Ok(());
    }

    // Pick a random unpassed sprint
    let mut rng = rand::thread_rng();
    let (project_id, sprint_num, topic) = unpassed.choose(&mut rng).unwrap();

    // Find project name for display
    let project_name = projects
        .iter()
        .find(|p| p.id == *project_id)
        .map(|p| p.name.as_str())
        .unwrap_or("unknown");

    println!();
    println!(
        "  {} Random sprint: {} — Sprint {}: {}",
        style("🎲").cyan(),
        style(project_name).cyan(),
        sprint_num,
        style(topic).bold()
    );
    println!();

    // Check if voice mode is configured and any TTS engine binary exists on PATH
    let use_voice = crate::voice::config::VoiceConfig::load()
        .ok()
        .filter(|c| c.is_configured())
        .map(|_| !crate::voice::config::VoiceConfig::detect_tts_engines().is_empty())
        .unwrap_or(false);

    if use_voice {
        crate::exam::cmd_exam_take_voice(&database, project_id, *sprint_num, None).await?;
    } else {
        crate::exam::cmd_exam_take(&database, project_id, *sprint_num, None).await?;
    }

    Ok(())
}

pub async fn cmd_status() -> Result<()> {
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
    let profile = database.get_profile().await?;

    let xp_needed = profile.xp_for_next_level();
    let xp_progress = (profile.total_xp as f32 / xp_needed as f32 * 20.0) as usize;
    let xp_bar = format!(
        "{}{}",
        "█".repeat(xp_progress.min(20)),
        "░".repeat(20 - xp_progress.min(20))
    );

    println!();
    println!(
        "  {} {}",
        style("🦀").cyan(),
        style("KnowledgeGATEunlocker").cyan().bold()
    );
    println!();

    // List ALL projects with their sprint progress
    let projects = database.list_projects().await?;

    if projects.is_empty() {
        println!("  No exams loaded. Run: kgate scan ~/gitZ --import");
    } else {
        println!("  {}", style("Exams:").bold());
        println!();

        let mut total_passed = 0;
        let mut total_sprints = 0;
        let mut total_xp_available = 0;
        let mut total_xp_earned = 0;
        let mut exam_num = 0;

        for proj in &projects {
            let sprints = database.get_sprints(&proj.id).await?;
            let passed = sprints.iter().filter(|s| s.status == "passed").count();
            let total = sprints.len();
            let xp_avail: i32 = sprints.iter().map(|s| s.xp_available).sum();
            let xp_earned: i32 = sprints.iter().map(|s| s.xp_earned).sum();

            total_passed += passed;
            total_sprints += total;
            total_xp_available += xp_avail;
            total_xp_earned += xp_earned;

            if total == 0 {
                continue;
            }

            exam_num += 1;

            let status_icon = if passed == total {
                style("✓").green()
            } else if passed > 0 {
                style("◐").yellow()
            } else {
                style("○").dim()
            };

            // Sprint depth visualization
            let sprint_bar: String = sprints
                .iter()
                .map(|s| {
                    if s.status == "passed" {
                        "█"
                    } else {
                        "░"
                    }
                })
                .collect();

            // Generate creative exam name from project + topics
            let topics: Vec<String> = sprints.iter().map(|s| s.topic.clone()).collect();
            let exam_name = kgate_core::get_exam_display_name(&proj.name, &topics);
            let display_name = if exam_name.len() > 20 {
                format!("{}...", &exam_name[..17])
            } else {
                exam_name
            };

            // Get domain icons (from domains.toml)
            let domains = crate::domains::get_exam_domains(&proj.name, &topics);
            let domain_icons: String = domains.iter().map(|(icon, _)| icon.as_str()).collect();

            println!(
                "  {:2}. {} {:18} {:6} {} {}/{}  {:3} XP",
                style(exam_num).dim(),
                status_icon,
                display_name,
                domain_icons,
                sprint_bar,
                passed,
                total,
                xp_earned
            );
        }

        println!();
        println!(
            "  {} {}/{} sprints | {}/{} XP earned",
            style("Total:").bold(),
            total_passed,
            total_sprints,
            total_xp_earned,
            total_xp_available
        );
    }

    // Overall level and XP with enhanced stats
    println!();
    println!("  ╭──────────────────────────────────────────╮");
    println!(
        "  │  {} Level {}: {:20}      │",
        style("🎮").cyan(),
        style(profile.level).bold(),
        style(profile.level_title()).cyan()
    );
    println!("  │  XP: {:4}/{:4} {}    │", profile.total_xp, xp_needed, xp_bar);
    println!("  ├──────────────────────────────────────────┤");

    // Questions passed and accuracy
    let accuracy = if profile.questions_attempted > 0 {
        (profile.questions_passed as f32 / profile.questions_attempted as f32 * 100.0) as i32
    } else {
        0
    };
    let accuracy_color = if accuracy >= 80 {
        style(format!("{}%", accuracy)).green()
    } else if accuracy >= 60 {
        style(format!("{}%", accuracy)).yellow()
    } else {
        style(format!("{}%", accuracy)).red()
    };

    println!(
        "  │  {} Questions: {:4} passed ({:>4} acc)      │",
        style("📊").cyan(),
        profile.questions_passed,
        accuracy_color
    );

    // Combo chain (consecutive correct answers)
    let combo_display = if profile.current_combo >= 10 {
        format!("{} 🔥🔥🔥", profile.current_combo)
    } else if profile.current_combo >= 5 {
        format!("{} 🔥🔥", profile.current_combo)
    } else if profile.current_combo >= 3 {
        format!("{} 🔥", profile.current_combo)
    } else {
        format!("{}", profile.current_combo)
    };

    println!(
        "  │  {} Combo: {:8}  Best: {:4}           │",
        style("⚡").yellow(),
        combo_display,
        profile.best_combo
    );

    // Sprint streak
    let streak_display = if profile.current_streak >= 5 {
        format!("{} 🔥", profile.current_streak)
    } else {
        format!("{}", profile.current_streak)
    };

    println!(
        "  │  {} Streak: {:7}  Best: {:4}           │",
        style("📈").green(),
        streak_display,
        profile.best_streak
    );

    // Perfect sprints (100% score)
    if profile.perfect_sprints > 0 {
        println!(
            "  │  {} Perfect: {:4}                          │",
            style("💎").magenta(),
            profile.perfect_sprints
        );
    }

    // Study time (if tracked)
    if profile.total_study_seconds > 60 {
        let hours = profile.total_study_seconds / 3600;
        let mins = (profile.total_study_seconds % 3600) / 60;
        let time_str = if hours > 0 {
            format!("{}h {}m", hours, mins)
        } else {
            format!("{}m", mins)
        };
        println!(
            "  │  {} Study time: {:10}                 │",
            style("⏱️").dim(),
            time_str
        );
    }

    println!("  ╰──────────────────────────────────────────╯");
    println!();

    Ok(())
}

pub async fn cmd_profile() -> Result<()> {
    let db = crate::db_path();
    let database = Database::new(&db).await?;
    let profile = database.get_profile().await?;

    let xp_needed = profile.xp_for_next_level();
    let xp_progress = (profile.total_xp as f32 / xp_needed as f32 * 14.0) as usize;
    let xp_bar = format!(
        "{}{}",
        "█".repeat(xp_progress.min(14)),
        "░".repeat(14 - xp_progress.min(14))
    );

    println!("╭─────────────────────────────────────╮");
    println!(
        "│  Level {}: {:16}     │",
        profile.level,
        profile.level_title()
    );
    println!(
        "│  XP: {:3}/{:3} {}           │",
        profile.total_xp, xp_needed, xp_bar
    );
    println!("├─────────────────────────────────────┤");
    println!("│  Sprints passed: {:3}               │", profile.sprints_passed);
    println!(
        "│  Current streak: {:3} {}              │",
        profile.current_streak,
        if profile.current_streak > 0 {
            "🔥"
        } else {
            "  "
        }
    );
    println!("│  Best streak:    {:3}               │", profile.best_streak);
    println!("╰─────────────────────────────────────╯");

    Ok(())
}

pub async fn cmd_badges() -> Result<()> {
    let db = crate::db_path();
    let database = Database::new(&db).await?;
    let badges = database.get_badges().await?;

    if badges.is_empty() {
        println!("No badges unlocked yet.");
        println!("Pass sprints to earn badges!");
        return Ok(());
    }

    println!("{}", style("Badges:").bold());
    for b in badges {
        let icon = match b.rarity.as_str() {
            "rare" => "🏆",
            "uncommon" => "🥈",
            _ => "🏅",
        };
        println!("  {} {} — {}", icon, style(&b.name).cyan(), b.description);
    }

    Ok(())
}

pub async fn cmd_history(limit: i32) -> Result<()> {
    let db = crate::db_path();
    let database = Database::new(&db).await?;
    let history = database.get_history(limit).await?;

    if history.is_empty() {
        println!("No attempts yet.");
        return Ok(());
    }

    println!("{}", style("Recent Attempts:").bold());
    for h in history {
        let icon = if h.passed { "✓" } else { "○" };
        let color = if h.passed {
            style(format!("{}%", h.score_percent)).green()
        } else {
            style(format!("{}%", h.score_percent)).yellow()
        };

        // Get project name
        if let Some(proj) = database.get_project_by_id(&h.project_id).await? {
            println!(
                "  {} {} Sprint {} — {} (+{} XP)",
                icon, proj.name, h.sprint_number, color, h.xp_earned
            );
        }
    }

    Ok(())
}

pub async fn cmd_domains() -> Result<()> {
    let db = crate::db_path();
    let database = Database::new(&db).await?;
    let domains = database.get_domains().await?;

    if domains.is_empty() {
        println!("No domain progress yet.");
        println!("Pass sprints to build domain mastery!");
        return Ok(());
    }

    println!("{}", style("Knowledge Domains:").bold());
    println!();

    for d in domains {
        let icon = d.icon.as_deref().unwrap_or("📚");
        let mastery_bar = "█".repeat(d.mastery_level as usize) + &"░".repeat(5 - d.mastery_level as usize);
        let accuracy = if d.questions_seen > 0 {
            (d.questions_correct as f32 / d.questions_seen as f32 * 100.0) as i32
        } else {
            0
        };

        println!(
            "  {} {} {} — {} XP",
            icon,
            style(format!("{:12}", d.name)).cyan(),
            mastery_bar,
            d.total_xp
        );
        println!(
            "       Level {}: {} | {}% accuracy ({}/{})",
            d.mastery_level,
            d.mastery_title(),
            accuracy,
            d.questions_correct,
            d.questions_seen
        );
    }

    // Show connections
    let connections = database.get_domain_connections().await?;
    if !connections.is_empty() {
        println!();
        println!("{}", style("Domain Connections:").bold());
        for c in connections.iter().take(10) {
            println!(
                "  {} ↔ {} (strength: {})",
                style(&c.domain_a).cyan(),
                style(&c.domain_b).cyan(),
                c.strength
            );
        }
    }

    Ok(())
}

pub async fn cmd_collection(limit: i32) -> Result<()> {
    let db = crate::db_path();
    let database = Database::new(&db).await?;
    let questions = database.get_collected_questions().await?;
    let count = database.count_collected().await?;

    println!(
        "{} {} questions collected",
        style("📚").cyan(),
        style(count).bold()
    );
    println!();

    for q in questions.iter().take(limit as usize) {
        let domains: Vec<String> = q
            .domains_json
            .as_ref()
            .and_then(|j| serde_json::from_str(j).ok())
            .unwrap_or_default();

        println!(
            "  {} [{}] {} XP",
            style(format!("Q{}", q.question_number)).dim(),
            q.tier,
            q.xp_earned
        );
        // Truncate long questions
        let text = if q.question_text.len() > 60 {
            format!("{}...", &q.question_text[..60])
        } else {
            q.question_text.clone()
        };
        println!("     {}", text);
        if !domains.is_empty() {
            println!("     Tags: {}", domains.join(", "));
        }
        println!();
    }

    Ok(())
}

pub async fn cmd_achievements() -> Result<()> {
    let db = crate::db_path();
    let database = Database::new(&db).await?;
    let achievements = database.get_achievements().await?;

    if achievements.is_empty() {
        println!("No achievements unlocked yet.");
        println!("Keep learning to unlock achievements!");
        return Ok(());
    }

    println!("{}", style("Achievements:").bold());
    for a in achievements {
        let icon = a.icon.as_deref().unwrap_or("🏆");
        let rarity_color = match a.rarity.as_str() {
            "rare" => style(&a.name).magenta().bold(),
            "uncommon" => style(&a.name).cyan(),
            _ => style(&a.name).white(),
        };
        println!("  {} {} — {}", icon, rarity_color, a.description);
    }

    Ok(())
}

pub async fn cmd_whoami() -> Result<()> {
    let db = crate::db_path();
    let database = Database::new(&db).await?;
    let identity = database.get_knowledge_id().await?;
    let profile = database.get_profile().await?;
    let collected = database.count_collected().await?;

    println!("╭─────────────────────────────────────────╮");
    println!(
        "│  {}                │",
        style("Knowledge Identity").cyan().bold()
    );
    println!("├─────────────────────────────────────────┤");
    println!(
        "│  ID: {}               │",
        style(&identity.knowledge_id[..16]).yellow()
    );
    if let Some(name) = &identity.display_name {
        println!("│  Name: {:32} │", name);
    }
    println!("├─────────────────────────────────────────┤");
    println!(
        "│  Level {}: {:16}          │",
        profile.level,
        profile.level_title()
    );
    println!("│  Total XP: {:6}                       │", profile.total_xp);
    println!("│  Questions collected: {:5}            │", collected);
    println!("│  Sprints passed: {:5}                 │", profile.sprints_passed);
    println!("╰─────────────────────────────────────────╯");

    Ok(())
}
