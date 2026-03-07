use anyhow::Result;
use clap::{Parser, Subcommand};
use console::style;
use std::path::PathBuf;

use kgate_core::Database;

mod domains;
mod exam;
mod scan;
mod sound;
mod voice;

#[derive(Parser)]
#[command(name = "kgate")]
#[command(about = "Knowledge Gate - ADHD-optimized exam system", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize database and data directories
    Init,
    /// Show current status (debt, profile, active project)
    Status,
    /// Debt management
    Debt {
        #[command(subcommand)]
        action: Option<DebtCommands>,
    },
    /// Project management
    Project {
        #[command(subcommand)]
        action: ProjectCommands,
    },
    /// Take an exam sprint
    Take {
        #[command(subcommand)]
        action: TakeCommands,
    },
    /// Show exam details with all sprints
    Show {
        #[command(subcommand)]
        action: ShowCommands,
    },
    /// List all exams (alias for status)
    List,
    /// Legacy exam commands (use 'take' and 'show' instead)
    #[command(hide = true)]
    Exam {
        #[command(subcommand)]
        action: ExamCommands,
    },
    /// Scan directory for exam files
    Scan {
        #[arg(help = "Directory to scan (default: current)")]
        path: Option<PathBuf>,
        #[arg(short, long, help = "Import found exams")]
        import: bool,
    },
    /// Profile and stats
    Profile,
    /// View badges
    Badges,
    /// View attempt history
    History {
        #[arg(short, long, default_value = "10", help = "Number of entries")]
        limit: i32,
    },
    /// Knowledge domains and progress
    Domains,
    /// View collected questions
    Collection {
        #[arg(short, long, default_value = "20", help = "Number of entries")]
        limit: i32,
    },
    /// View achievements
    Achievements,
    /// Settings management
    Config {
        #[command(subcommand)]
        action: Option<ConfigCommands>,
    },
    /// Show your Knowledge ID
    Whoami,
    /// Export bookmarks from study resources
    ExportBookmarks {
        #[arg(help = "Output file (default: bookmarks.json)")]
        output: Option<PathBuf>,
    },
    /// Show domain icon legend
    Legend,
    /// Auto-generate exam from codebase analysis
    Generate {
        #[arg(help = "Path to project directory")]
        path: PathBuf,
        #[arg(short, long, help = "Output exam file (default: exam_<name>.md)")]
        output: Option<PathBuf>,
    },
    /// Spaced repetition review session
    Review {
        #[arg(short, long, default_value = "10", help = "Number of items to review")]
        limit: i32,
    },
    /// Domain catalog commands
    Catalog {
        #[command(subcommand)]
        action: CatalogCommands,
    },
    /// Grade an open-ended answer (testing LLM grader)
    Grade {
        #[arg(help = "Your answer text")]
        answer: String,
        #[arg(short, long, help = "Expected key concepts (comma-separated)")]
        concepts: String,
    },
    /// Harvest mode - build growing question catalog from codebases
    Harvest {
        #[command(subcommand)]
        action: HarvestCommands,
    },
    /// Voice mode setup and testing
    Voice {
        #[command(subcommand)]
        action: VoiceCommands,
    },
}

#[derive(Subcommand)]
enum DebtCommands {
    /// Add debt manually (for testing)
    Add {
        #[arg(help = "Action type (concept, architecture, bugfix, newfile, code)")]
        action: String,
        #[arg(help = "Optional description")]
        description: Option<String>,
    },
    /// Clear debt manually (for testing)
    Clear {
        #[arg(help = "Amount to clear")]
        amount: i32,
    },
}

#[derive(Subcommand)]
enum ProjectCommands {
    /// List all tracked projects
    List,
    /// Add a project
    Add {
        #[arg(help = "Path to project")]
        path: PathBuf,
    },
    /// Switch active project
    Set {
        #[arg(help = "Project ID (short hash)")]
        id: String,
    },
}

#[derive(Subcommand)]
enum ConfigCommands {
    /// Toggle sound on/off
    Sound {
        #[arg(help = "on or off")]
        value: String,
    },
    /// Toggle fast-answer mode
    FastAnswer {
        #[arg(help = "on or off")]
        value: String,
    },
    /// Set display name
    Name {
        #[arg(help = "Your display name")]
        name: String,
    },
    /// Show all settings
    Show,
}

// Take exam command: kgate take exam <number> sprint <sprint>
#[derive(Subcommand)]
enum TakeCommands {
    /// Take a specific exam sprint
    Exam {
        #[arg(help = "Exam number from list (e.g., 18)")]
        number: usize,
        #[command(subcommand)]
        action: TakeExamAction,
    },
}

#[derive(Subcommand)]
enum TakeExamAction {
    /// Take a specific sprint
    Sprint {
        #[arg(help = "Sprint number (1-3)")]
        number: i32,
        #[arg(short, long, help = "Enable voice mode")]
        voice: bool,
    },
}

// Show exam command: kgate show exam <number>
#[derive(Subcommand)]
enum ShowCommands {
    /// Show details of a specific exam
    Exam {
        #[arg(help = "Exam number from list")]
        number: usize,
    },
}

// Keep old Exam subcommand for backwards compatibility, but mark as hidden
#[derive(Subcommand)]
enum ExamCommands {
    /// List sprints for current project
    List,
    /// Load exam from markdown file
    Load {
        #[arg(help = "Path to exam_*.md file")]
        file: PathBuf,
    },
}

// Domain catalog commands
#[derive(Subcommand)]
enum CatalogCommands {
    /// List all domains in catalog
    List,
    /// Show questions for a specific domain
    Show {
        #[arg(help = "Domain name (e.g., rust, nix, networking)")]
        domain: String,
    },
    /// Export catalog to JSON
    Export {
        #[arg(help = "Output file")]
        output: Option<PathBuf>,
    },
    /// Show domain statistics
    Stats,
}

// Harvest mode commands - grow the question catalog
#[derive(Subcommand)]
enum HarvestCommands {
    /// Harvest questions from a directory
    Add {
        #[arg(help = "Directory to harvest")]
        path: PathBuf,
    },
    /// Harvest from all known projects
    All,
    /// Show the question tree
    Tree,
    /// Show harvest statistics
    Stats,
    /// Export the full catalog
    Export {
        #[arg(help = "Output file (default: ~/.kgate/catalog.json)")]
        output: Option<PathBuf>,
    },
}

// Voice mode commands
#[derive(Subcommand)]
enum VoiceCommands {
    /// Run voice setup wizard
    Setup,
    /// Test text-to-speech
    TestSpeak {
        #[arg(help = "Text to speak")]
        text: String,
    },
    /// Test speech recognition
    TestListen,
    /// Show voice configuration
    Config,
}

fn kgate_dir() -> PathBuf {
    dirs::home_dir()
        .expect("No home directory")
        .join(".kgate")
}

fn data_dir() -> PathBuf {
    kgate_dir().join("data")
}

fn db_path() -> PathBuf {
    data_dir().join("db").join("knowledge-gate.db")
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        None => cmd_random().await?,
        Some(Commands::Init) => cmd_init().await?,
        Some(Commands::Status) => cmd_status().await?,
        Some(Commands::Debt { action }) => cmd_debt(action).await?,
        Some(Commands::Project { action }) => cmd_project(action).await?,
        Some(Commands::Take { action }) => cmd_take(action).await?,
        Some(Commands::Show { action }) => cmd_show(action).await?,
        Some(Commands::List) => cmd_status().await?,
        Some(Commands::Exam { action }) => cmd_exam_legacy(action).await?,
        Some(Commands::Scan { path, import }) => cmd_scan(path, import).await?,
        Some(Commands::Profile) => cmd_profile().await?,
        Some(Commands::Badges) => cmd_badges().await?,
        Some(Commands::History { limit }) => cmd_history(limit).await?,
        Some(Commands::Domains) => cmd_domains().await?,
        Some(Commands::Collection { limit }) => cmd_collection(limit).await?,
        Some(Commands::Achievements) => cmd_achievements().await?,
        Some(Commands::Config { action }) => cmd_config(action).await?,
        Some(Commands::Whoami) => cmd_whoami().await?,
        Some(Commands::ExportBookmarks { output }) => cmd_export_bookmarks(output).await?,
        Some(Commands::Legend) => cmd_legend().await?,
        Some(Commands::Generate { path, output }) => cmd_generate(&path, output).await?,
        Some(Commands::Review { limit }) => cmd_review(limit).await?,
        Some(Commands::Catalog { action }) => cmd_catalog(action).await?,
        Some(Commands::Grade { answer, concepts }) => cmd_grade(&answer, &concepts).await?,
        Some(Commands::Harvest { action }) => cmd_harvest(action).await?,
        Some(Commands::Voice { action }) => cmd_voice(action).await?,
    }

    Ok(())
}

async fn cmd_init() -> Result<()> {
    let kgate = kgate_dir();
    let data = data_dir();
    let db = db_path();

    std::fs::create_dir_all(&kgate)?;
    std::fs::create_dir_all(data.join("db"))?;
    std::fs::create_dir_all(kgate.join("export"))?;
    std::fs::create_dir_all(kgate.join("sounds"))?;
    std::fs::create_dir_all(kgate.join("bookmarks"))?;

    // Copy domains.toml if it doesn't exist
    let domains_dest = kgate.join("domains.toml");
    if !domains_dest.exists() {
        // Try to copy from project directory
        let project_domains = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(|p| p.parent())
            .map(|p| p.join("domains.toml"));

        if let Some(src) = project_domains {
            if src.exists() {
                std::fs::copy(&src, &domains_dest)?;
                println!(
                    "{} Copied domains.toml to {}",
                    style("✓").green(),
                    domains_dest.display()
                );
            }
        }
    }

    println!(
        "{} Creating profile at {}",
        style("✓").green(),
        kgate.display()
    );

    let database = Database::new(&db).await?;
    database.init().await?;

    // Show Knowledge ID
    let identity = database.get_knowledge_id().await?;
    println!("{} Database initialized", style("✓").green());
    println!();
    println!(
        "{}  KnowledgeGATEunlocker ready!",
        style("🦀").cyan()
    );
    println!(
        "   Knowledge ID: {}",
        style(&identity.knowledge_id[..16]).yellow().bold()
    );

    Ok(())
}

/// Pick a random unpassed sprint and start it
async fn cmd_random() -> Result<()> {
    use rand::seq::SliceRandom;

    let db = db_path();
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
    let use_voice = voice::config::VoiceConfig::load()
        .ok()
        .filter(|c| c.is_configured())
        .map(|_| !voice::config::VoiceConfig::detect_tts_engines().is_empty())
        .unwrap_or(false);

    if use_voice {
        exam::cmd_exam_take_voice(&database, project_id, *sprint_num).await?;
    } else {
        exam::cmd_exam_take(&database, project_id, *sprint_num).await?;
    }

    Ok(())
}

async fn cmd_status() -> Result<()> {
    let db = db_path();
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

async fn cmd_debt(action: Option<DebtCommands>) -> Result<()> {
    let db = db_path();
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

async fn cmd_project(action: ProjectCommands) -> Result<()> {
    let db = db_path();
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
            scan::cmd_auto_load(&database, &project.id, &abs_path).await?;

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

// kgate take exam <number> sprint <sprint> [--voice]
async fn cmd_take(action: TakeCommands) -> Result<()> {
    let db = db_path();
    let database = Database::new(&db).await?;

    match action {
        TakeCommands::Exam { number, action: TakeExamAction::Sprint { number: sprint_num, voice } } => {
            // Find project by exam number
            let project = find_project_by_number(&database, number).await?;

            let project = match project {
                Some(p) => p,
                None => {
                    println!(
                        "{} Exam {} not found. Run {} to see available exams.",
                        style("✗").red(),
                        number,
                        style("kgate status").yellow()
                    );
                    return Ok(());
                }
            };

            // Get sprints
            let sprints = database.get_sprints(&project.id).await?;

            if sprints.is_empty() {
                println!(
                    "{} No sprints in exam '{}'. Try reloading with {} --import",
                    style("✗").red(),
                    project.name,
                    style("kgate scan").yellow()
                );
                return Ok(());
            }

            // Validate sprint number
            if sprint_num < 1 || sprint_num > sprints.len() as i32 {
                println!(
                    "{} Sprint {} not found. This exam has {} sprints (1-{}).",
                    style("✗").red(),
                    sprint_num,
                    sprints.len(),
                    sprints.len()
                );
                return Ok(());
            }

            // Take the sprint (with or without voice mode)
            if voice {
                exam::cmd_exam_take_voice(&database, &project.id, sprint_num).await?;
            } else {
                exam::cmd_exam_take(&database, &project.id, sprint_num).await?;
            }
        }
    }

    Ok(())
}

// kgate show exam <number>
async fn cmd_show(action: ShowCommands) -> Result<()> {
    let db = db_path();
    let database = Database::new(&db).await?;

    match action {
        ShowCommands::Exam { number } => {
            let project = find_project_by_number(&database, number).await?;

            let project = match project {
                Some(p) => p,
                None => {
                    println!(
                        "{} Exam {} not found.",
                        style("✗").red(),
                        number
                    );
                    return Ok(());
                }
            };

            let sprints = database.get_sprints(&project.id).await?;
            let topics: Vec<String> = sprints.iter().map(|s| s.topic.clone()).collect();
            let exam_name = kgate_core::get_exam_display_name(&project.name, &topics);
            let domains = crate::domains::get_exam_domains(&project.name, &topics);

            println!();
            println!("  {} (Exam #{})", style(&exam_name).cyan().bold(), number);
            println!("  {}", style(&project.name).dim());
            println!();

            // Show domains
            if !domains.is_empty() {
                let domain_str: String = domains.iter().map(|(i, n)| format!("{} {}", i, n)).collect::<Vec<_>>().join("  ");
                println!("  Domains: {}", domain_str);
                println!();
            }

            // Show sprints
            println!("  {}", style("Sprints:").bold());
            for s in &sprints {
                let status = match s.status.as_str() {
                    "passed" => style("✓").green(),
                    _ => style("○").dim(),
                };
                let score_str = if let Some(score) = s.best_score {
                    format!("{}%", score)
                } else {
                    "-".to_string()
                };

                println!(
                    "    {} Sprint {}: {} — {} XP (best: {})",
                    status,
                    s.sprint_number,
                    s.topic,
                    s.xp_available,
                    score_str
                );
            }

            // Summary
            let passed = sprints.iter().filter(|s| s.status == "passed").count();
            let total_xp: i32 = sprints.iter().map(|s| s.xp_available).sum();
            let earned_xp: i32 = sprints.iter().map(|s| s.xp_earned).sum();

            println!();
            println!(
                "  Progress: {}/{} sprints | {}/{} XP",
                passed,
                sprints.len(),
                earned_xp,
                total_xp
            );
            println!();
            println!(
                "  To take sprint 1: {}",
                style(format!("kgate take exam {} sprint 1", number)).yellow()
            );
            println!();
        }
    }

    Ok(())
}

// Helper to find project by exam number (1-indexed, matching status display)
async fn find_project_by_number(database: &Database, exam_number: usize) -> Result<Option<kgate_core::Project>> {
    let projects = database.list_projects().await?;

    // Filter projects with sprints to match status numbering
    let mut exam_num = 0;
    for p in &projects {
        let sprints = database.get_sprints(&p.id).await?;
        if !sprints.is_empty() {
            exam_num += 1;
            if exam_num == exam_number {
                return Ok(Some(p.clone()));
            }
        }
    }

    Ok(None)
}

// Legacy exam command (for backwards compatibility)
async fn cmd_exam_legacy(action: ExamCommands) -> Result<()> {
    let db = db_path();
    let database = Database::new(&db).await?;
    let projects = database.list_projects().await?;

    let project = match projects.first() {
        Some(p) => p,
        None => {
            println!(
                "{} No projects. Run {} first.",
                style("✗").red(),
                style("kgate scan ~/gitZ --import").yellow()
            );
            return Ok(());
        }
    };

    match action {
        ExamCommands::List => {
            exam::cmd_exam_list(&database, &project.id).await?;
        }
        ExamCommands::Load { file } => {
            exam::cmd_exam_load(&database, &project.id, &file).await?;
        }
    }

    Ok(())
}

async fn cmd_scan(path: Option<PathBuf>, import: bool) -> Result<()> {
    let db = db_path();
    let database = Database::new(&db).await?;

    let scan_path = path.unwrap_or_else(|| std::env::current_dir().unwrap());
    scan::cmd_scan(&database, &scan_path, import).await?;

    Ok(())
}

async fn cmd_profile() -> Result<()> {
    let db = db_path();
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

async fn cmd_badges() -> Result<()> {
    let db = db_path();
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

async fn cmd_history(limit: i32) -> Result<()> {
    let db = db_path();
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

async fn cmd_domains() -> Result<()> {
    let db = db_path();
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

async fn cmd_collection(limit: i32) -> Result<()> {
    let db = db_path();
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

async fn cmd_achievements() -> Result<()> {
    let db = db_path();
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

async fn cmd_config(action: Option<ConfigCommands>) -> Result<()> {
    let db = db_path();
    let database = Database::new(&db).await?;

    match action {
        Some(ConfigCommands::Sound { value }) => {
            let enabled = value == "on" || value == "true" || value == "1";
            database
                .set_setting("sound_enabled", if enabled { "true" } else { "false" })
                .await?;
            println!(
                "{} Sound {}",
                style("✓").green(),
                if enabled { "enabled" } else { "disabled" }
            );
        }
        Some(ConfigCommands::FastAnswer { value }) => {
            let enabled = value == "on" || value == "true" || value == "1";
            database
                .set_setting("fast_answer_mode", if enabled { "true" } else { "false" })
                .await?;
            println!(
                "{} Fast answer mode {}",
                style("✓").green(),
                if enabled { "enabled" } else { "disabled" }
            );
        }
        Some(ConfigCommands::Name { name }) => {
            database.set_display_name(&name).await?;
            println!("{} Display name set to: {}", style("✓").green(), name);
        }
        Some(ConfigCommands::Show) | None => {
            let sound = database
                .get_setting("sound_enabled")
                .await?
                .unwrap_or("true".to_string());
            let fast = database
                .get_setting("fast_answer_mode")
                .await?
                .unwrap_or("true".to_string());
            let identity = database.get_knowledge_id().await?;

            println!("{}", style("Settings:").bold());
            println!(
                "  Display name:    {}",
                identity.display_name.unwrap_or("(not set)".to_string())
            );
            println!("  Sound:           {}", sound);
            println!("  Fast answer:     {}", fast);
        }
    }

    Ok(())
}

async fn cmd_whoami() -> Result<()> {
    let db = db_path();
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

async fn cmd_export_bookmarks(output: Option<PathBuf>) -> Result<()> {
    let output_path = output.unwrap_or_else(|| kgate_dir().join("bookmarks").join("bookmarks.json"));

    // Ensure parent directory exists
    if let Some(parent) = output_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    // Scan all exam files for study resources
    let mut bookmarks: Vec<serde_json::Value> = Vec::new();

    let projects_dir = dirs::home_dir()
        .expect("No home directory")
        .join("gitZ");

    if projects_dir.exists() {
        for entry in std::fs::read_dir(&projects_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                // Look for exam_*.md files
                for file in std::fs::read_dir(&path)? {
                    let file = file?;
                    let filename = file.file_name();
                    let filename_str = filename.to_string_lossy();
                    if filename_str.starts_with("exam_") && filename_str.ends_with(".md") {
                        if let Ok(content) = std::fs::read_to_string(file.path()) {
                            // Extract URLs from study resources section
                            let urls = extract_urls(&content);
                            for (title, url) in urls {
                                bookmarks.push(serde_json::json!({
                                    "title": title,
                                    "url": url,
                                    "folder": path.file_name().unwrap().to_string_lossy().to_string()
                                }));
                            }
                        }
                    }
                }
            }
        }
    }

    // Create browser-importable bookmark format (Netscape Bookmark File)
    let bookmark_json = serde_json::json!({
        "version": 1,
        "generator": "kgate",
        "bookmarks": bookmarks
    });

    std::fs::write(&output_path, serde_json::to_string_pretty(&bookmark_json)?)?;

    println!(
        "{} Exported {} bookmarks to {}",
        style("✓").green(),
        bookmarks.len(),
        output_path.display()
    );

    Ok(())
}

fn extract_urls(content: &str) -> Vec<(String, String)> {
    let mut urls = Vec::new();
    let url_regex = regex::Regex::new(r"\[([^\]]+)\]\((https?://[^\)]+)\)").unwrap();

    // Find study resources section
    let in_resources = content.contains("Study Resources");

    if in_resources {
        for cap in url_regex.captures_iter(content) {
            let title = cap.get(1).map(|m| m.as_str().to_string()).unwrap_or_default();
            let url = cap.get(2).map(|m| m.as_str().to_string()).unwrap_or_default();
            if !url.is_empty() {
                urls.push((title, url));
            }
        }
    }

    urls
}

async fn cmd_legend() -> Result<()> {
    println!("{}", style("Domain Icon Legend").cyan().bold());
    println!();
    println!("Source: ~/.kgate/domains.toml (or built-in defaults)");
    println!();

    domains::print_legend();

    Ok(())
}

// ============================================
// Phase 7: Auto-generate exam from codebase
// ============================================

async fn cmd_generate(path: &PathBuf, output: Option<PathBuf>) -> Result<()> {
    use kgate_core::CodebaseAnalyzer;

    let abs_path = std::fs::canonicalize(path)?;
    println!(
        "{} Analyzing codebase: {}",
        style("🔍").cyan(),
        abs_path.display()
    );

    let analyzer = CodebaseAnalyzer::new();
    let analysis = analyzer.analyze(&abs_path)?;

    println!(
        "  Found {} code elements in {} files",
        analysis.elements.len(),
        analysis.detected_languages.len()
    );
    println!(
        "  Languages: {}",
        analysis.detected_languages.join(", ")
    );
    if !analysis.detected_frameworks.is_empty() {
        println!(
            "  Frameworks: {}",
            analysis.detected_frameworks.join(", ")
        );
    }

    if analysis.suggested_sprints.is_empty() {
        println!(
            "{} Not enough code elements to generate exam. Add more code!",
            style("⚠").yellow()
        );
        return Ok(());
    }

    // Generate exam markdown
    let exam_content = generate_exam_markdown(&analysis);

    // Determine output path
    let output_path = output.unwrap_or_else(|| {
        abs_path.join(format!("exam_{}.md", analysis.project_name))
    });

    std::fs::write(&output_path, &exam_content)?;

    println!();
    println!(
        "{} Generated exam: {}",
        style("✓").green(),
        output_path.display()
    );
    println!(
        "  {} sprints, {} questions total",
        analysis.suggested_sprints.len(),
        analysis.suggested_sprints.iter().map(|s| s.questions.len()).sum::<usize>()
    );
    println!();
    println!(
        "  Import with: {}",
        style(format!("kgate scan {} --import", abs_path.display())).yellow()
    );

    Ok(())
}

fn generate_exam_markdown(analysis: &kgate_core::ProjectAnalysis) -> String {
    let mut md = String::new();

    // Header
    md.push_str(&format!("# Exam: {}\n", analysis.project_name));
    md.push_str(&format!("# Generated: {}\n", chrono::Utc::now().format("%Y-%m-%d")));
    md.push_str(&format!("# Languages: {}\n", analysis.detected_languages.join(", ")));
    md.push_str("# Pass: 60% per sprint | Retakes: unlimited\n");
    md.push_str("# Voice-Ready: yes\n\n");
    md.push_str("---\n\n");

    // Sprints
    for (i, sprint) in analysis.suggested_sprints.iter().enumerate() {
        md.push_str(&format!("## Sprint {}: {}\n", i + 1, sprint.topic));
        md.push_str(&format!(
            "⏱️ Target: 3 min | 🎯 Pass: 60% | ⚡ {} XP\n",
            sprint.total_xp
        ));
        md.push_str("🎙️ Voice-compatible: yes\n\n");

        for (j, q) in sprint.questions.iter().enumerate() {
            md.push_str(&format!(
                "### Q{}. [{}] {} — {} XP\n",
                j + 1,
                q.tier,
                q.difficulty,
                q.xp
            ));
            md.push_str(&format!("{}\n\n", q.question_text));

            if let Some(ref code) = q.code_snippet {
                md.push_str("```\n");
                md.push_str(code);
                if !code.ends_with('\n') {
                    md.push('\n');
                }
                md.push_str("```\n\n");
            }

            for opt in &q.options {
                md.push_str(&format!("- {}\n", opt));
            }
            md.push('\n');
        }

        md.push_str("---\n\n");
    }

    // Answer key
    md.push_str("## 🔑 Answer Key\n\n");

    for (i, sprint) in analysis.suggested_sprints.iter().enumerate() {
        md.push_str(&format!("### Sprint {}\n\n", i + 1));

        for (j, q) in sprint.questions.iter().enumerate() {
            md.push_str(&format!(
                "**Q{}. Answer: {}** — {} XP\n",
                j + 1,
                q.correct_answer,
                q.xp
            ));
            md.push_str(&format!("Hint: {}\n", q.hint));
            md.push_str(&format!("Full: {}\n", q.explanation));
            md.push_str(&format!("📁 `{}:{}`\n\n", q.source_file, q.source_line));
        }
    }

    md
}

// ============================================
// Phase 3: Spaced Repetition Review
// ============================================

async fn cmd_review(limit: i32) -> Result<()> {
    let db = db_path();
    if !db.exists() {
        println!(
            "{} Not initialized. Run {} first.",
            style("✗").red(),
            style("kgate init").yellow()
        );
        return Ok(());
    }

    let database = Database::new(&db).await?;

    // Get due reviews
    let due_items = database.get_due_reviews(limit).await?;

    if due_items.is_empty() {
        let stats = database.get_review_stats().await?;
        println!(
            "{} No reviews due right now!",
            style("✓").green()
        );
        println!(
            "  {} items in review queue, next review scheduled later",
            stats.total_items
        );
        return Ok(());
    }

    println!(
        "{} {} items due for review",
        style("📚").cyan(),
        due_items.len()
    );
    println!();

    // Simple review session (for now, just show the items)
    for (i, item) in due_items.iter().enumerate() {
        println!(
            "  {}. [{}] Q{}: {}",
            i + 1,
            style(&item.domain).cyan(),
            item.question_number,
            if item.question_text.len() > 50 {
                format!("{}...", &item.question_text[..50])
            } else {
                item.question_text.clone()
            }
        );
        println!(
            "     Streak: {} | EF: {:.2} | Next: {} days",
            item.streak,
            item.easiness_factor,
            item.interval_days
        );
    }

    println!();
    println!(
        "Full review mode coming soon. For now, take sprints to build your review queue."
    );

    Ok(())
}

// ============================================
// Domain Catalog Commands
// ============================================

async fn cmd_catalog(action: CatalogCommands) -> Result<()> {
    let db = db_path();
    if !db.exists() {
        println!(
            "{} Not initialized. Run {} first.",
            style("✗").red(),
            style("kgate init").yellow()
        );
        return Ok(());
    }

    let database = Database::new(&db).await?;

    match action {
        CatalogCommands::List => {
            let stats = database.get_domain_catalog_stats().await?;

            if stats.is_empty() {
                println!("No questions in catalog yet.");
                println!("Pass exam sprints to collect questions!");
                return Ok(());
            }

            println!("{}", style("Domain Catalog").cyan().bold());
            println!();

            for stat in stats {
                let accuracy_color = if stat.accuracy >= 80.0 {
                    style(format!("{:.0}%", stat.accuracy)).green()
                } else if stat.accuracy >= 60.0 {
                    style(format!("{:.0}%", stat.accuracy)).yellow()
                } else {
                    style(format!("{:.0}%", stat.accuracy)).red()
                };

                println!(
                    "  {:12} {:4} questions | {} accuracy",
                    style(&stat.domain).cyan(),
                    stat.question_count,
                    accuracy_color
                );
            }
        }

        CatalogCommands::Show { domain } => {
            let entries = database.get_domain_catalog(&domain).await?;

            if entries.is_empty() {
                println!("No questions found for domain: {}", domain);
                return Ok(());
            }

            println!(
                "{} {} questions",
                style(format!("Domain: {}", domain)).cyan().bold(),
                entries.len()
            );
            println!();

            for entry in entries.iter().take(20) {
                let accuracy = if entry.times_seen > 0 {
                    (entry.times_correct as f64 / entry.times_seen as f64) * 100.0
                } else {
                    0.0
                };

                println!(
                    "  [{}] {} — {:.0}% ({}/{})",
                    entry.tier,
                    if entry.question_text.len() > 50 {
                        format!("{}...", &entry.question_text[..50])
                    } else {
                        entry.question_text.clone()
                    },
                    accuracy,
                    entry.times_correct,
                    entry.times_seen
                );
            }

            if entries.len() > 20 {
                println!("  ... and {} more", entries.len() - 20);
            }
        }

        CatalogCommands::Export { output } => {
            let output_path = output.unwrap_or_else(|| {
                kgate_dir().join("catalog.json")
            });

            let json = database.export_domain_catalog().await?;
            std::fs::write(&output_path, &json)?;

            println!(
                "{} Exported catalog to {}",
                style("✓").green(),
                output_path.display()
            );
        }

        CatalogCommands::Stats => {
            let stats = database.get_domain_catalog_stats().await?;
            let review_stats = database.get_review_stats().await?;

            println!("{}", style("Catalog Statistics").cyan().bold());
            println!();

            let total_questions: i64 = stats.iter().map(|s| s.question_count).sum();
            let total_attempts: i64 = stats.iter().map(|s| s.total_attempts).sum();
            let total_correct: i64 = stats.iter().map(|s| s.total_correct).sum();

            println!("  Total questions: {}", total_questions);
            println!("  Total attempts:  {}", total_attempts);
            println!(
                "  Overall accuracy: {:.1}%",
                if total_attempts > 0 {
                    (total_correct as f64 / total_attempts as f64) * 100.0
                } else {
                    0.0
                }
            );
            println!("  Domains covered: {}", stats.len());
            println!();

            println!("{}", style("Review Queue").cyan().bold());
            println!("  Items in queue: {}", review_stats.total_items);
            println!("  Due now: {}", review_stats.due_now);
            println!(
                "  Average EF: {:.2}",
                review_stats.avg_easiness.unwrap_or(2.5)
            );
            println!(
                "  Longest streak: {}",
                review_stats.max_streak.unwrap_or(0)
            );
        }
    }

    Ok(())
}

// ============================================
// Phase 6: LLM Grading for Open-Ended Answers
// ============================================

async fn cmd_grade(answer: &str, concepts: &str) -> Result<()> {
    use kgate_core::{AnswerKey, LocalGrader};

    // Parse concepts from comma-separated string
    let key_concepts: Vec<&str> = concepts
        .split(',')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .collect();

    if key_concepts.is_empty() {
        println!(
            "{} Please provide at least one concept to grade against",
            style("✗").red()
        );
        return Ok(());
    }

    let answer_key = AnswerKey::new(key_concepts);
    let grader = LocalGrader::new();
    let result = grader.grade(answer, &answer_key);

    // Display results
    println!();
    println!("{}", style("LLM Grading Result").cyan().bold());
    println!();

    // Score visualization
    let score_bar = match result.score {
        3 => style("███").green(),
        2 => style("██░").yellow(),
        1 => style("█░░").red(),
        _ => style("░░░").dim(),
    };

    let score_label = match result.score {
        3 => style("COMPLETE").green().bold(),
        2 => style("PARTIAL").yellow().bold(),
        1 => style("SURFACE").red().bold(),
        _ => style("INCORRECT").red().bold(),
    };

    println!(
        "  Score: {} {}/3 — {}",
        score_bar, result.score, score_label
    );
    println!("  XP Multiplier: {}%", (result.xp_multiplier() * 100.0) as i32);
    println!("  Confidence: {:.0}%", result.confidence * 100.0);
    println!();

    // Feedback
    println!("  {}", style("Feedback:").bold());
    println!("    {}", result.feedback);
    println!();

    // Matched concepts
    if !result.matched_concepts.is_empty() {
        println!("  {} Matched:", style("✓").green());
        for concept in &result.matched_concepts {
            println!("    • {}", style(concept).green());
        }
    }

    // Missing concepts
    if !result.missing_concepts.is_empty() {
        println!("  {} Missing:", style("○").yellow());
        for concept in &result.missing_concepts {
            println!("    • {}", style(concept).yellow());
        }
    }

    println!();

    // Pass/fail indicator
    if result.passed() {
        println!(
            "  {} This answer would {} the question",
            style("✓").green(),
            style("PASS").green().bold()
        );
    } else {
        println!(
            "  {} This answer would {} the question",
            style("✗").red(),
            style("FAIL").red().bold()
        );
    }

    Ok(())
}

// ============================================
// Harvest Mode - Growing Question Catalog
// ============================================

fn catalog_path() -> PathBuf {
    kgate_dir().join("catalog.json")
}

async fn cmd_harvest(action: HarvestCommands) -> Result<()> {
    use kgate_core::{Harvester, QuestionCatalog};

    // Load existing catalog or create new
    let catalog = if catalog_path().exists() {
        QuestionCatalog::load(&catalog_path()).unwrap_or_default()
    } else {
        QuestionCatalog::new()
    };

    let mut harvester = Harvester::with_catalog(catalog);

    match action {
        HarvestCommands::Add { path } => {
            let abs_path = std::fs::canonicalize(&path)?;
            println!(
                "{} Harvesting questions from: {}",
                style("🌱").green(),
                abs_path.display()
            );

            let result = harvester.harvest(&abs_path);

            if result.success {
                println!(
                    "  {} Scanned {} elements",
                    style("✓").green(),
                    result.files_scanned
                );
                println!(
                    "  {} Generated {} questions",
                    style("✓").green(),
                    result.questions_generated
                );
                println!(
                    "  Languages: {}",
                    result.languages.join(", ")
                );

                // Save catalog
                let catalog = harvester.into_catalog();
                catalog.save(&catalog_path())?;

                println!();
                println!(
                    "{} Catalog saved: {} total questions",
                    style("📚").cyan(),
                    catalog.total_questions
                );
            } else {
                println!(
                    "{} Harvest failed: {}",
                    style("✗").red(),
                    result.error.unwrap_or("Unknown error".to_string())
                );
            }
        }

        HarvestCommands::All => {
            let db = db_path();
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
                println!("No projects found. Add projects with: kgate scan ~/gitZ --import");
                return Ok(());
            }

            println!(
                "{} Harvesting from {} projects...",
                style("🌱").green(),
                projects.len()
            );
            println!();

            let mut total_questions = 0;

            for proj in &projects {
                let path = PathBuf::from(&proj.path);
                if path.exists() {
                    print!("  {} {}... ", style("→").dim(), proj.name);
                    let result = harvester.harvest(&path);

                    if result.success {
                        println!(
                            "{} {} Qs",
                            style("✓").green(),
                            result.questions_generated
                        );
                        total_questions += result.questions_generated;
                    } else {
                        println!("{} skip", style("○").dim());
                    }
                }
            }

            // Save catalog
            let catalog = harvester.into_catalog();
            catalog.save(&catalog_path())?;

            println!();
            println!(
                "{} Harvested {} new questions. Catalog total: {}",
                style("🌳").green(),
                total_questions,
                catalog.total_questions
            );
        }

        HarvestCommands::Tree => {
            if !catalog_path().exists() {
                println!("No catalog found. Run: kgate harvest add <path>");
                return Ok(());
            }

            let catalog = QuestionCatalog::load(&catalog_path())?;
            println!();
            println!("{}", catalog.tree_view());
        }

        HarvestCommands::Stats => {
            if !catalog_path().exists() {
                println!("No catalog found. Run: kgate harvest add <path>");
                return Ok(());
            }

            let catalog = QuestionCatalog::load(&catalog_path())?;

            println!();
            println!("{}", style("Harvest Catalog Statistics").cyan().bold());
            println!();
            println!("  Total questions: {}", catalog.total_questions);
            println!("  Total domains:   {}", catalog.total_domains);
            println!(
                "  Last harvest:    {}",
                catalog.last_harvest
                    .map(|d| d.format("%Y-%m-%d %H:%M").to_string())
                    .unwrap_or("Never".to_string())
            );
            println!("  Catalog version: {}", catalog.version);
            println!();

            // Domain breakdown
            println!("  {}", style("By Domain:").bold());
            for domain in &catalog.root.children {
                println!(
                    "    {:12} {:4} questions ({} XP)",
                    style(&domain.name).cyan(),
                    domain.metadata.question_count,
                    domain.metadata.total_xp
                );
            }
        }

        HarvestCommands::Export { output } => {
            if !catalog_path().exists() {
                println!("No catalog found. Run: kgate harvest add <path>");
                return Ok(());
            }

            let catalog = QuestionCatalog::load(&catalog_path())?;
            let output_path = output.unwrap_or_else(|| kgate_dir().join("catalog_export.json"));

            catalog.save(&output_path)?;

            println!(
                "{} Exported catalog ({} questions) to: {}",
                style("✓").green(),
                catalog.total_questions,
                output_path.display()
            );
        }
    }

    Ok(())
}

// ============================================
// Voice Mode Commands
// ============================================

async fn cmd_voice(action: VoiceCommands) -> Result<()> {
    match action {
        VoiceCommands::Setup => {
            voice::run_setup_wizard()?;
        }
        VoiceCommands::TestSpeak { text } => {
            voice::setup::test_speak(&text)?;
        }
        VoiceCommands::TestListen => {
            voice::setup::test_listen()?;
        }
        VoiceCommands::Config => {
            voice::setup::show_config()?;
        }
    }

    Ok(())
}
