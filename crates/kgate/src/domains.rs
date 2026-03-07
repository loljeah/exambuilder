use once_cell::sync::Lazy;
use serde::Deserialize;
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Clone, Deserialize)]
pub struct DomainDef {
    pub name: String,
    pub icon: String,
    pub category: String,
    pub keywords: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct DomainsFile {
    domains: HashMap<String, DomainDef>,
}

/// Global domain definitions loaded from domains.toml
pub static DOMAINS: Lazy<HashMap<String, DomainDef>> = Lazy::new(|| load_domains());

fn domains_file_path() -> PathBuf {
    // First check ~/.kgate/domains.toml (user override)
    let user_path = dirs::home_dir()
        .expect("No home directory")
        .join(".kgate")
        .join("domains.toml");

    if user_path.exists() {
        return user_path;
    }

    // Then check project directory (for development)
    let project_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .map(|p| p.join("domains.toml"))
        .unwrap_or_default();

    if project_path.exists() {
        return project_path;
    }

    // Fallback: embedded default path
    user_path
}

fn load_domains() -> HashMap<String, DomainDef> {
    let path = domains_file_path();

    if let Ok(content) = std::fs::read_to_string(&path) {
        if let Ok(parsed) = toml::from_str::<DomainsFile>(&content) {
            return parsed.domains;
        }
    }

    // Fallback to hardcoded defaults if file not found
    default_domains()
}

fn default_domains() -> HashMap<String, DomainDef> {
    let mut domains = HashMap::new();

    let defaults = vec![
        ("rust", "Rust", "🦀", "lang", vec!["rust", "cargo", "crate"]),
        ("python", "Python", "🐍", "lang", vec!["python", "pip", "django"]),
        ("bash", "Bash", "🐚", "lang", vec!["bash", "shell", "sh", "script"]),
        ("nix", "Nix", "❄️", "lang", vec!["nix", "nixos", "flake"]),
        ("javascript", "JavaScript", "🟨", "lang", vec!["javascript", "js", "node"]),
        ("docker", "Docker", "🐳", "tool", vec!["docker", "container", "compose"]),
        ("linux", "Linux", "🐧", "tech", vec!["linux", "kernel", "systemd"]),
        ("networking", "Networking", "🌐", "concept", vec!["network", "tcp", "dns", "vlan"]),
        ("security", "Security", "🔒", "concept", vec!["security", "auth", "encrypt", "ssh"]),
        ("databases", "Databases", "🗄️", "concept", vec!["database", "sql", "sqlite"]),
        ("api", "APIs", "🔌", "concept", vec!["api", "rest", "endpoint", "http"]),
        ("embedded", "Embedded/IoT", "📟", "tech", vec!["esp32", "arduino", "gpio", "firmware"]),
        ("gpu", "GPU/Graphics", "🎮", "tech", vec!["gpu", "cuda", "rocm", "vulkan"]),
        ("ai_ml", "AI/ML", "🤖", "concept", vec!["ai", "ml", "model", "diffusion", "llm"]),
        ("gaming", "Gaming", "🕹️", "tech", vec!["game", "phaser", "retro", "controller"]),
        ("audio", "Audio", "🎵", "tech", vec!["audio", "music", "stream", "sound"]),
        ("video", "Video", "📺", "tech", vec!["video", "crt", "tv", "rtsp"]),
        ("printing3d", "3D Printing", "🧊", "tech", vec!["3d", "stl", "mesh", "print"]),
        ("chat", "Chat/Messaging", "💬", "concept", vec!["chat", "bot", "signal", "message"]),
        ("ops", "Operations", "📊", "concept", vec!["log", "sysinfo", "monitor", "gather"]),
        ("hardware", "Hardware", "🔧", "tech", vec!["hardware", "cpu", "ram", "usb"]),
        ("git", "Git", "🔀", "tool", vec!["git", "commit", "branch"]),
    ];

    for (id, name, icon, category, keywords) in defaults {
        domains.insert(
            id.to_string(),
            DomainDef {
                name: name.to_string(),
                icon: icon.to_string(),
                category: category.to_string(),
                keywords: keywords.into_iter().map(String::from).collect(),
            },
        );
    }

    domains
}

/// Get domain icons for an exam based on project name and topics
pub fn get_exam_domains(project_name: &str, sprint_topics: &[String]) -> Vec<(String, String)> {
    let name_lower = project_name.to_lowercase();
    let topics_lower: Vec<String> = sprint_topics.iter().map(|s| s.to_lowercase()).collect();
    let all_text = format!("{} {}", name_lower, topics_lower.join(" "));

    let mut found: Vec<(String, String)> = Vec::new();

    for (_, domain) in DOMAINS.iter() {
        for keyword in &domain.keywords {
            if all_text.contains(keyword) {
                if !found.iter().any(|(_, n)| *n == domain.name) {
                    found.push((domain.icon.clone(), domain.name.clone()));
                }
                break;
            }
        }
    }

    // Limit to 3 most relevant domains
    found.truncate(3);
    found
}

/// Print domain legend
pub fn print_legend() {
    println!("Domain Icons:");
    println!();

    let mut by_category: HashMap<&str, Vec<&DomainDef>> = HashMap::new();
    for domain in DOMAINS.values() {
        by_category
            .entry(&domain.category)
            .or_default()
            .push(domain);
    }

    let categories = ["lang", "tool", "tech", "concept"];
    let category_names = ["Languages", "Tools", "Technologies", "Concepts"];

    for (cat, cat_name) in categories.iter().zip(category_names.iter()) {
        if let Some(domains) = by_category.get(cat) {
            println!("  {}:", cat_name);
            for d in domains {
                println!("    {} {}", d.icon, d.name);
            }
            println!();
        }
    }
}
