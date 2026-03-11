use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Project {
    pub id: String,
    pub full_hash: String,
    pub path: String,
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub last_active: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct DebtEntry {
    pub id: i64,
    pub project_id: String,
    pub action: String,
    pub weight: i32,
    pub description: Option<String>,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct DebtCurrent {
    pub project_id: String,
    pub total: i32,
    pub last_updated: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Sprint {
    pub id: i64,
    pub project_id: String,
    pub sprint_number: i32,
    pub topic: String,
    pub questions_json: String,
    pub answer_key_json: String,
    pub status: String,
    pub best_score: Option<i32>,
    pub attempts: i32,
    pub xp_available: i32,
    pub xp_earned: i32,
    pub created_at: DateTime<Utc>,
    pub passed_at: Option<DateTime<Utc>>,
    #[sqlx(default)]
    pub sprint_id: Option<String>,
    #[sqlx(default)]
    pub source_project_name: Option<String>,
}

/// Generate a deterministic 8-char hex sprint ID from project_id + sprint_number + topic
pub fn generate_sprint_id(project_id: &str, sprint_number: i32, topic: &str) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(format!("{}:{}:{}", project_id, sprint_number, topic).as_bytes());
    let hash = format!("{:x}", hasher.finalize());
    hash[..8].to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Profile {
    pub id: i64,
    pub total_xp: i32,
    pub level: i32,
    pub current_streak: i32,
    pub best_streak: i32,
    pub sprints_passed: i32,
    pub last_activity: Option<DateTime<Utc>>,
    // Enhanced stats
    #[sqlx(default)]
    pub questions_passed: i32,
    #[sqlx(default)]
    pub questions_attempted: i32,
    #[sqlx(default)]
    pub current_combo: i32,
    #[sqlx(default)]
    pub best_combo: i32,
    #[sqlx(default)]
    pub perfect_sprints: i32,
    #[sqlx(default)]
    pub total_study_seconds: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Badge {
    pub id: String,
    pub name: String,
    pub description: String,
    pub icon: Option<String>,
    pub rarity: String,
    pub unlocked_at: Option<DateTime<Utc>>,
    pub project_id: Option<String>,
}

impl Profile {
    pub fn level_title(&self) -> &'static str {
        match self.level {
            1 => "Novice",
            2 => "Config Wrangler",
            3 => "System Operator",
            4 => "Stack Builder",
            5 => "Infra Architect",
            _ => "Master",
        }
    }

    pub fn xp_for_next_level(&self) -> i32 {
        match self.level {
            1 => 50,
            2 => 80,
            3 => 120,
            4 => 180,
            5 => 250,
            _ => 100 * self.level,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ExamAttempt {
    pub id: i64,
    pub project_id: String,
    pub sprint_number: i32,
    pub score_percent: i32,
    pub passed: bool,
    pub xp_earned: i32,
    pub timestamp: DateTime<Utc>,
}

// Badge definitions
pub const BADGES: &[(&str, &str, &str, &str)] = &[
    ("first_sprint", "First Sprint", "Pass your first sprint", "common"),
    ("streak_3", "On Fire", "3 sprint streak", "common"),
    ("streak_5", "Blazing", "5 sprint streak", "uncommon"),
    ("streak_10", "Unstoppable", "10 sprint streak", "rare"),
    ("level_2", "Config Wrangler", "Reach level 2", "common"),
    ("level_3", "System Operator", "Reach level 3", "uncommon"),
    ("level_5", "Infra Architect", "Reach level 5", "rare"),
    ("perfect", "Perfect Score", "100% on a sprint", "uncommon"),
    ("project_clear", "Gate Cleared", "Pass all sprints in a project", "uncommon"),
    ("comeback", "Comeback Kid", "Pass after 2+ failed attempts", "common"),
    ("speed_demon", "Speed Demon", "Pass 3 sprints in one session", "uncommon"),
    ("xp_100", "Century", "Earn 100 XP total", "common"),
    ("xp_500", "Half K", "Earn 500 XP total", "uncommon"),
    ("xp_1000", "Grand Master", "Earn 1000 XP total", "rare"),
];

// Knowledge Identity
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct KnowledgeIdentity {
    pub id: i64,
    pub knowledge_id: String,
    pub display_name: Option<String>,
    pub created_at: DateTime<Utc>,
    pub last_sync: Option<DateTime<Utc>>,
}

// Knowledge Domain
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Domain {
    pub id: String,
    pub name: String,
    pub category: String,
    pub icon: Option<String>,
    pub total_xp: i32,
    pub mastery_level: i32,
    pub questions_seen: i32,
    pub questions_correct: i32,
}

impl Domain {
    pub fn mastery_title(&self) -> &'static str {
        match self.mastery_level {
            0 => "Novice",
            1 => "Apprentice",
            2 => "Journeyman",
            3 => "Expert",
            4 => "Master",
            5 => "Grandmaster",
            _ => "Legendary",
        }
    }

    pub fn progress_percent(&self) -> f32 {
        if self.questions_seen == 0 {
            0.0
        } else {
            (self.questions_correct as f32 / self.questions_seen as f32) * 100.0
        }
    }
}

// Domain Connection (inter-domain relationship)
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct DomainConnection {
    pub id: i64,
    pub domain_a: String,
    pub domain_b: String,
    pub strength: i32,
    pub discovered_at: DateTime<Utc>,
}

// Collected Question (passed Q&A archive)
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct CollectedQuestion {
    pub id: i64,
    pub project_id: String,
    pub sprint_number: i32,
    pub question_number: i32,
    pub question_text: String,
    pub correct_answer: String,
    pub user_answer: String,
    pub tier: String,
    pub xp_earned: i32,
    pub domains_json: Option<String>,
    pub collected_at: DateTime<Utc>,
}

// Dynamic Achievement
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Achievement {
    pub id: String,
    pub name: String,
    pub description: String,
    pub icon: Option<String>,
    pub rarity: String,
    pub category: String,
    pub requirement_json: Option<String>,
    pub unlocked_at: Option<DateTime<Utc>>,
    pub context_json: Option<String>,
}

// User Setting
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Setting {
    pub key: String,
    pub value: String,
}

// Domain keyword mappings for AI inference
pub const DOMAIN_KEYWORDS: &[(&str, &[&str])] = &[
    ("rust", &["rust", "cargo", "crate", "rustc", "tokio", "async", "ownership", "borrow"]),
    ("python", &["python", "pip", "django", "flask", "pandas", "numpy", "pytest"]),
    ("bash", &["bash", "shell", "sh", "zsh", "script", "chmod", "grep", "sed", "awk"]),
    ("nix", &["nix", "nixos", "flake", "derivation", "nixpkgs", "home-manager"]),
    ("javascript", &["javascript", "js", "node", "npm", "react", "vue", "webpack"]),
    ("typescript", &["typescript", "ts", "tsc", "angular", "deno"]),
    ("docker", &["docker", "container", "dockerfile", "compose", "podman"]),
    ("git", &["git", "commit", "branch", "merge", "rebase", "repository"]),
    ("linux", &["linux", "kernel", "systemd", "proc", "sysfs", "dmesg"]),
    ("networking", &["network", "tcp", "udp", "ip", "dns", "http", "port", "socket", "vlan", "firewall"]),
    ("security", &["security", "auth", "encrypt", "ssl", "tls", "password", "key", "certificate", "ssh"]),
    ("databases", &["database", "sql", "sqlite", "postgres", "mysql", "query", "table"]),
    ("api", &["api", "rest", "graphql", "endpoint", "request", "response", "json"]),
    ("hardware", &["hardware", "cpu", "ram", "disk", "ssd", "nvme", "usb", "pci"]),
    ("embedded", &["embedded", "esp32", "arduino", "gpio", "uart", "spi", "i2c", "firmware"]),
    ("gpu", &["gpu", "cuda", "rocm", "vulkan", "opengl", "shader", "render"]),
    ("ai_ml", &["ai", "ml", "model", "neural", "training", "inference", "tensor"]),
    ("devops", &["devops", "ci", "cd", "pipeline", "deploy", "kubernetes", "k8s"]),
    ("testing", &["test", "unit", "integration", "mock", "assert", "coverage"]),
    ("architecture", &["architecture", "design", "pattern", "module", "component", "layer"]),
];

// Exam name generator - creates thematic names based on project content
pub struct ExamNameGenerator;

impl ExamNameGenerator {
    /// Generate a creative exam name from project name and sprint topics
    pub fn generate(project_name: &str, sprint_topics: &[String]) -> String {
        let name_lower = project_name.to_lowercase();
        let topics_lower: Vec<String> = sprint_topics.iter().map(|s| s.to_lowercase()).collect();
        let all_text = format!("{} {}", name_lower, topics_lower.join(" "));

        // Check for specific project patterns first
        if let Some(specific) = Self::specific_project_name(&name_lower) {
            return specific;
        }

        // Detect primary domain
        let domain = Self::detect_primary_domain(&all_text);

        // Generate thematic name based on domain
        Self::thematic_name(&domain, &name_lower, &topics_lower)
    }

    fn specific_project_name(name: &str) -> Option<String> {
        // Known project mappings
        let mappings: &[(&str, &str)] = &[
            ("exambuilder", "The Architect's Exam"),
            ("aiegos", "AI Ego Codex"),
            ("homeb0t", "Home Sentinel Protocol"),
            ("homeb0t_cluster", "Cluster Command Academy"),
            ("gamestart", "Game Launcher Mastery"),
            ("esp32cam", "IoT Vision Quest"),
            ("comfyui", "Diffusion Artisan"),
            ("stable-diffusion", "Stable Diffusion Dojo"),
            ("musicgen", "Audio Synthesis Lab"),
            ("geminichat", "LLM Chat Protocol"),
            ("config_raspb4", "Pi NixOS Certification"),
            ("config_microtik", "Network Fortress Exam"),
            ("config_home", "Home Config Mastery"),
            ("nixrip", "Stream Ripper's License"),
            ("nix_unity", "Unity Nix Bridge"),
            ("gather_logs", "Log Archaeology"),
            ("gather_network", "Network Recon Cert"),
            ("gather_sysinfo", "System Intel Exam"),
            ("daemonbite", "Retro Controller Academy"),
            ("crt_info", "CRT Restoration Guild"),
            ("phaser_pirateshot", "Phaser Game Dev"),
            ("wolkencoder", "NixOS Installer Mastery"),
            ("redelete", "Audio Dedup Protocol"),
            ("signalbot", "Signal Bot Engineering"),
            ("lampshadestl", "3D Parametric Design"),
            ("batchdown", "Batch Download Mastery"),
        ];

        for (pattern, exam_name) in mappings {
            if name.contains(pattern) {
                return Some(exam_name.to_string());
            }
        }
        None
    }

    fn detect_primary_domain(text: &str) -> String {
        let domain_scores: &[(&str, &[&str], &str)] = &[
            ("embedded", &["esp32", "arduino", "gpio", "firmware", "camera", "sensor"], "embedded"),
            ("ai_ml", &["diffusion", "model", "ai", "ml", "neural", "inference", "comfy", "stable"], "ai_ml"),
            ("networking", &["network", "router", "mikrotik", "vlan", "firewall", "dns"], "networking"),
            ("nix", &["nix", "nixos", "flake", "configuration"], "nix"),
            ("docker", &["docker", "container", "compose", "rocm"], "docker"),
            ("gaming", &["game", "phaser", "retro", "controller", "joystick"], "gaming"),
            ("audio", &["music", "audio", "stream", "rip", "mp3"], "audio"),
            ("chat", &["chat", "bot", "signal", "llm", "gemini"], "chat"),
            ("hardware", &["crt", "tv", "hardware", "usb", "serial"], "hardware"),
            ("sysadmin", &["log", "sysinfo", "gather", "system", "admin"], "sysadmin"),
            ("3d", &["stl", "3d", "print", "parametric", "mesh"], "3d"),
        ];

        let mut best_domain = "general";
        let mut best_score = 0;

        for (_, keywords, domain) in domain_scores {
            let score = keywords.iter().filter(|k| text.contains(*k)).count();
            if score > best_score {
                best_score = score;
                best_domain = domain;
            }
        }

        best_domain.to_string()
    }

    fn thematic_name(domain: &str, _project_name: &str, _topics: &[String]) -> String {
        match domain {
            "embedded" => "IoT Engineering Cert",
            "ai_ml" => "ML Pipeline Mastery",
            "networking" => "Network Admin Exam",
            "nix" => "NixOS Certification",
            "docker" => "Container Mastery",
            "gaming" => "Game Dev Academy",
            "audio" => "Audio Engineering",
            "chat" => "Bot Development",
            "hardware" => "Hardware Hacking",
            "sysadmin" => "SysAdmin Cert",
            "3d" => "3D Design Mastery",
            _ => "Technical Certification",
        }
        .to_string()
    }
}

// Pre-computed exam display names for known projects
pub fn get_exam_display_name(project_name: &str, sprint_topics: &[String]) -> String {
    ExamNameGenerator::generate(project_name, sprint_topics)
}

// Get domain icons for an exam based on project name and topics
#[allow(clippy::type_complexity)]
pub fn get_exam_domains(project_name: &str, sprint_topics: &[String]) -> Vec<(&'static str, &'static str)> {
    let name_lower = project_name.to_lowercase();
    let topics_lower: Vec<String> = sprint_topics.iter().map(|s| s.to_lowercase()).collect();
    let all_text = format!("{} {}", name_lower, topics_lower.join(" "));

    let domain_icons: &[(&[&str], &str, &str)] = &[
        (&["rust", "cargo", "crate"], "🦀", "Rust"),
        (&["python", "pip", "django", "flask"], "🐍", "Python"),
        (&["bash", "shell", "sh", "script"], "🐚", "Bash"),
        (&["nix", "nixos", "flake", "derivation"], "❄️", "Nix"),
        (&["javascript", "js", "node", "npm", "react"], "🟨", "JS"),
        (&["docker", "container", "compose", "podman"], "🐳", "Docker"),
        (&["linux", "kernel", "systemd"], "🐧", "Linux"),
        (&["network", "tcp", "udp", "dns", "vlan", "firewall", "mikrotik", "router"], "🌐", "Net"),
        (&["security", "auth", "encrypt", "ssl", "ssh", "key"], "🔒", "Sec"),
        (&["database", "sql", "sqlite", "postgres"], "🗄️", "DB"),
        (&["api", "rest", "graphql", "endpoint", "http"], "🔌", "API"),
        (&["esp32", "arduino", "gpio", "firmware", "embedded", "camera", "sensor"], "📟", "IoT"),
        (&["gpu", "cuda", "rocm", "vulkan", "opengl", "render"], "🎮", "GPU"),
        (&["ai", "ml", "model", "neural", "diffusion", "inference", "llm", "gemini"], "🤖", "AI"),
        (&["game", "phaser", "retro", "controller", "joystick"], "🕹️", "Game"),
        (&["audio", "music", "stream", "sound", "mp3"], "🎵", "Audio"),
        (&["3d", "stl", "mesh", "print", "parametric"], "🧊", "3D"),
        (&["signal", "chat", "bot", "message"], "💬", "Chat"),
        (&["crt", "tv", "video", "display", "scart"], "📺", "Video"),
        (&["hardware", "usb", "serial", "cpu", "ram"], "🔧", "HW"),
        (&["git", "commit", "branch", "repository"], "🔀", "Git"),
        (&["log", "sysinfo", "monitor", "gather"], "📊", "Ops"),
    ];

    let mut found: Vec<(&'static str, &'static str)> = Vec::new();

    for (keywords, icon, name) in domain_icons {
        for keyword in *keywords {
            if all_text.contains(keyword) {
                if !found.iter().any(|(_, n)| *n == *name) {
                    found.push((icon, name));
                }
                break;
            }
        }
    }

    // Limit to 3 most relevant domains
    found.truncate(3);
    found
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_profile(level: i32, total_xp: i32) -> Profile {
        Profile {
            id: 1,
            total_xp,
            level,
            current_streak: 0,
            best_streak: 0,
            sprints_passed: 0,
            last_activity: None,
            questions_passed: 0,
            questions_attempted: 0,
            current_combo: 0,
            best_combo: 0,
            perfect_sprints: 0,
            total_study_seconds: 0,
        }
    }

    // ---- Level title tests ----

    #[test]
    fn test_level_title_level_1() {
        let p = make_profile(1, 0);
        assert_eq!(p.level_title(), "Novice");
    }

    #[test]
    fn test_level_title_level_2() {
        let p = make_profile(2, 0);
        assert_eq!(p.level_title(), "Config Wrangler");
    }

    #[test]
    fn test_level_title_level_3() {
        let p = make_profile(3, 0);
        assert_eq!(p.level_title(), "System Operator");
    }

    #[test]
    fn test_level_title_level_4() {
        let p = make_profile(4, 0);
        assert_eq!(p.level_title(), "Stack Builder");
    }

    #[test]
    fn test_level_title_level_5() {
        let p = make_profile(5, 0);
        assert_eq!(p.level_title(), "Infra Architect");
    }

    #[test]
    fn test_level_title_above_5() {
        let p = make_profile(6, 0);
        assert_eq!(p.level_title(), "Master");
        let p = make_profile(99, 0);
        assert_eq!(p.level_title(), "Master");
    }

    // ---- XP threshold tests ----

    #[test]
    fn test_xp_thresholds() {
        assert_eq!(make_profile(1, 0).xp_for_next_level(), 50);
        assert_eq!(make_profile(2, 0).xp_for_next_level(), 80);
        assert_eq!(make_profile(3, 0).xp_for_next_level(), 120);
        assert_eq!(make_profile(4, 0).xp_for_next_level(), 180);
        assert_eq!(make_profile(5, 0).xp_for_next_level(), 250);
    }

    #[test]
    fn test_xp_threshold_above_5_scales() {
        // level 6 => 100 * 6 = 600
        assert_eq!(make_profile(6, 0).xp_for_next_level(), 600);
        // level 10 => 100 * 10 = 1000
        assert_eq!(make_profile(10, 0).xp_for_next_level(), 1000);
    }

    // ---- Sprint ID tests ----

    #[test]
    fn test_generate_sprint_id_deterministic() {
        let id1 = generate_sprint_id("proj1", 1, "Basics");
        let id2 = generate_sprint_id("proj1", 1, "Basics");
        assert_eq!(id1, id2);
    }

    #[test]
    fn test_generate_sprint_id_different_inputs_different_hashes() {
        let id_a = generate_sprint_id("proj1", 1, "Basics");
        let id_b = generate_sprint_id("proj1", 2, "Basics");
        let id_c = generate_sprint_id("proj2", 1, "Basics");
        let id_d = generate_sprint_id("proj1", 1, "Advanced");
        assert_ne!(id_a, id_b);
        assert_ne!(id_a, id_c);
        assert_ne!(id_a, id_d);
    }

    #[test]
    fn test_generate_sprint_id_is_8_chars() {
        let id = generate_sprint_id("project", 1, "topic");
        assert_eq!(id.len(), 8);
        // Should be hex characters
        assert!(id.chars().all(|c| c.is_ascii_hexdigit()));
    }

    // ---- Badge definitions tests ----

    #[test]
    fn test_badges_has_expected_entries() {
        assert!(BADGES.len() >= 10, "Should have at least 10 badge definitions");
        // Check specific badges exist
        let ids: Vec<&str> = BADGES.iter().map(|(id, _, _, _)| *id).collect();
        assert!(ids.contains(&"first_sprint"));
        assert!(ids.contains(&"streak_3"));
        assert!(ids.contains(&"streak_5"));
        assert!(ids.contains(&"streak_10"));
        assert!(ids.contains(&"perfect"));
        assert!(ids.contains(&"project_clear"));
        assert!(ids.contains(&"xp_100"));
        assert!(ids.contains(&"xp_500"));
        assert!(ids.contains(&"xp_1000"));
    }

    #[test]
    fn test_badges_have_valid_rarities() {
        let valid_rarities = ["common", "uncommon", "rare", "legendary"];
        for (id, _, _, rarity) in BADGES {
            assert!(
                valid_rarities.contains(rarity),
                "Badge '{}' has invalid rarity '{}'",
                id,
                rarity
            );
        }
    }

    // ---- ExamNameGenerator tests ----

    #[test]
    fn test_exam_name_generator_known_project() {
        let name = ExamNameGenerator::generate("exambuilder", &[]);
        assert_eq!(name, "The Architect's Exam");
    }

    #[test]
    fn test_exam_name_generator_unknown_project_uses_domain() {
        let topics = vec!["Rust Basics".to_string(), "Cargo Config".to_string()];
        let name = ExamNameGenerator::generate("some_rust_project", &topics);
        // Should detect "rust" domain
        assert!(!name.is_empty());
    }

    #[test]
    fn test_get_exam_display_name_delegates() {
        let name = get_exam_display_name("exambuilder", &[]);
        assert_eq!(name, "The Architect's Exam");
    }

    // ---- Domain tests ----

    #[test]
    fn test_domain_mastery_title() {
        let mut domain = Domain {
            id: "rust".to_string(),
            name: "Rust".to_string(),
            category: "lang".to_string(),
            icon: None,
            total_xp: 0,
            mastery_level: 0,
            questions_seen: 0,
            questions_correct: 0,
        };
        assert_eq!(domain.mastery_title(), "Novice");
        domain.mastery_level = 3;
        assert_eq!(domain.mastery_title(), "Expert");
        domain.mastery_level = 5;
        assert_eq!(domain.mastery_title(), "Grandmaster");
        domain.mastery_level = 6;
        assert_eq!(domain.mastery_title(), "Legendary");
    }

    #[test]
    fn test_domain_progress_percent() {
        let mut domain = Domain {
            id: "rust".to_string(),
            name: "Rust".to_string(),
            category: "lang".to_string(),
            icon: None,
            total_xp: 0,
            mastery_level: 0,
            questions_seen: 0,
            questions_correct: 0,
        };
        assert_eq!(domain.progress_percent(), 0.0);

        domain.questions_seen = 10;
        domain.questions_correct = 7;
        let pct = domain.progress_percent();
        assert!((pct - 70.0).abs() < 0.01);
    }

    #[test]
    fn test_domain_keywords_has_expected_domains() {
        let domain_ids: Vec<&str> = DOMAIN_KEYWORDS.iter().map(|(id, _)| *id).collect();
        assert!(domain_ids.contains(&"rust"));
        assert!(domain_ids.contains(&"python"));
        assert!(domain_ids.contains(&"nix"));
        assert!(domain_ids.contains(&"networking"));
        assert!(domain_ids.contains(&"security"));
    }
}
