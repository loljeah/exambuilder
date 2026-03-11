use anyhow::Result;
use clap::{Parser, Subcommand};
use console::style;
use std::path::PathBuf;

use kgate_core::Database;

mod cli;
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
        #[arg(long, help = "Force LLM generation (requires ANTHROPIC_API_KEY)")]
        llm: bool,
        #[arg(long, help = "Force template-based generation (no API needed)")]
        templates: bool,
        #[arg(long, help = "Show what would be generated without writing files")]
        dry_run: bool,
        #[arg(long, help = "Override LLM model (default: claude-opus-4-20250514)")]
        model: Option<String>,
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
    /// Run built-in self-test diagnostics
    Selftest {
        #[arg(short, long, help = "Show detailed output")]
        verbose: bool,
        #[arg(short, long, help = "Auto-fix issues where possible")]
        fix: bool,
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

pub(crate) fn kgate_dir() -> PathBuf {
    dirs::home_dir()
        .expect("No home directory")
        .join(".kgate")
}

pub(crate) fn data_dir() -> PathBuf {
    kgate_dir().join("data")
}

pub(crate) fn db_path() -> PathBuf {
    data_dir().join("db").join("knowledge-gate.db")
}

// Helper to find project by exam number (1-indexed, matching status display)
pub(crate) async fn find_project_by_number(database: &Database, exam_number: usize) -> Result<Option<kgate_core::Project>> {
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

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        None => cli::progress::cmd_random().await?,
        Some(Commands::Init) => cmd_init().await?,
        Some(Commands::Status) => cli::progress::cmd_status().await?,
        Some(Commands::Debt { action }) => cli::debt::cmd_debt(action).await?,
        Some(Commands::Project { action }) => cli::project::cmd_project(action).await?,
        Some(Commands::Take { action }) => cmd_take(action).await?,
        Some(Commands::Show { action }) => cmd_show(action).await?,
        Some(Commands::List) => cli::progress::cmd_status().await?,
        Some(Commands::Exam { action }) => cmd_exam_legacy(action).await?,
        Some(Commands::Scan { path, import }) => cmd_scan(path, import).await?,
        Some(Commands::Profile) => cli::progress::cmd_profile().await?,
        Some(Commands::Badges) => cli::progress::cmd_badges().await?,
        Some(Commands::History { limit }) => cli::progress::cmd_history(limit).await?,
        Some(Commands::Domains) => cli::progress::cmd_domains().await?,
        Some(Commands::Collection { limit }) => cli::progress::cmd_collection(limit).await?,
        Some(Commands::Achievements) => cli::progress::cmd_achievements().await?,
        Some(Commands::Config { action }) => cmd_config(action).await?,
        Some(Commands::Whoami) => cli::progress::cmd_whoami().await?,
        Some(Commands::ExportBookmarks { output }) => cli::bookmarks::cmd_export_bookmarks(output).await?,
        Some(Commands::Legend) => cmd_legend().await?,
        Some(Commands::Generate { path, output, llm, templates, dry_run, model }) => cli::generation::cmd_generate(&path, output, llm, templates, dry_run, model).await?,
        Some(Commands::Review { limit }) => cmd_review(limit).await?,
        Some(Commands::Catalog { action }) => cli::catalog::cmd_catalog(action).await?,
        Some(Commands::Grade { answer, concepts }) => cli::catalog::cmd_grade(&answer, &concepts).await?,
        Some(Commands::Harvest { action }) => cli::catalog::cmd_harvest(action).await?,
        Some(Commands::Voice { action }) => cmd_voice(action).await?,
        Some(Commands::Selftest { verbose, fix }) => cli::selftest::cmd_selftest(verbose, fix).await?,
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
                exam::cmd_exam_take_voice(&database, &project.id, sprint_num, Some(number)).await?;
            } else {
                exam::cmd_exam_take(&database, &project.id, sprint_num, Some(number)).await?;
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

            // Show source project name if available
            if let Some(ref src_name) = sprints.first().and_then(|s| s.source_project_name.as_ref()) {
                println!("  Source: {}", style(src_name).dim());
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
                let id_label = s.sprint_id.as_deref().map(|id| format!(" [{}]", id)).unwrap_or_default();

                println!(
                    "    {} Sprint {}: {} — {} XP (best: {}){}",
                    status,
                    s.sprint_number,
                    s.topic,
                    s.xp_available,
                    score_str,
                    style(id_label).dim()
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

async fn cmd_legend() -> Result<()> {
    println!("{}", style("Domain Icon Legend").cyan().bold());
    println!();
    println!("Source: ~/.kgate/domains.toml (or built-in defaults)");
    println!();

    domains::print_legend();

    Ok(())
}

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
    exam::cmd_review_session(&database, limit).await?;

    Ok(())
}

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
