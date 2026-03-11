//! Harvest Mode - Growing Question Catalog
//!
//! Continuously parses codebases to build a tree-like question catalog.
//! Questions are organized hierarchically:
//!   Domain → Subdomain → Topic → Questions
//!
//! Example tree:
//!   rust/
//!     ownership/
//!       borrowing/
//!         Q1: "What happens when..."
//!         Q2: "Why does..."
//!     async/
//!       futures/
//!         Q1: "What is a Future..."
//!   nix/
//!     services/
//!       systemd/
//!         Q1: "What does enable..."

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

use crate::analyzer::{CodebaseAnalyzer, GeneratedQuestion};

/// A node in the question tree
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuestionNode {
    pub id: String,
    pub name: String,
    pub node_type: NodeType,
    pub children: Vec<QuestionNode>,
    pub questions: Vec<HarvestedQuestion>,
    pub metadata: NodeMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum NodeType {
    Root,
    Domain,
    Subdomain,
    Topic,
    Source, // Specific file/project source
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct NodeMetadata {
    pub question_count: usize,
    pub total_xp: i32,
    pub avg_difficulty: f64,
    pub last_harvested: Option<DateTime<Utc>>,
    pub sources: Vec<String>,
}

/// A harvested question with full metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HarvestedQuestion {
    pub id: String,
    pub question_text: String,
    pub options: Vec<String>,
    pub correct_answer: char,
    pub hint: String,
    pub explanation: String,
    pub tier: String,
    pub difficulty: String,
    pub xp: i32,
    pub code_snippet: Option<String>,
    pub source_file: String,
    pub source_line: usize,
    pub source_project: String,
    pub domains: Vec<String>,
    pub tags: Vec<String>,
    pub harvested_at: DateTime<Utc>,
    pub times_used: i32,
    pub times_correct: i32,
}

impl HarvestedQuestion {
    pub fn from_generated(q: &GeneratedQuestion, project: &str) -> Self {
        let id = format!(
            "{}:{}:{}",
            project,
            q.source_file.split('/').next_back().unwrap_or("unknown"),
            q.source_line
        );

        Self {
            id,
            question_text: q.question_text.clone(),
            options: q.options.clone(),
            correct_answer: q.correct_answer,
            hint: q.hint.clone(),
            explanation: q.explanation.clone(),
            tier: q.tier.clone(),
            difficulty: q.difficulty.clone(),
            xp: q.xp,
            code_snippet: q.code_snippet.clone(),
            source_file: q.source_file.clone(),
            source_line: q.source_line,
            source_project: project.to_string(),
            domains: q.domains.clone(),
            tags: Vec::new(),
            harvested_at: Utc::now(),
            times_used: 0,
            times_correct: 0,
        }
    }

    pub fn accuracy(&self) -> f64 {
        if self.times_used == 0 {
            0.0
        } else {
            (self.times_correct as f64 / self.times_used as f64) * 100.0
        }
    }
}

/// The complete question catalog tree
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuestionCatalog {
    pub root: QuestionNode,
    pub total_questions: usize,
    pub total_domains: usize,
    pub last_harvest: Option<DateTime<Utc>>,
    pub version: u32,
}

impl Default for QuestionCatalog {
    fn default() -> Self {
        Self::new()
    }
}

impl QuestionCatalog {
    pub fn new() -> Self {
        Self {
            root: QuestionNode {
                id: "root".to_string(),
                name: "Question Catalog".to_string(),
                node_type: NodeType::Root,
                children: Vec::new(),
                questions: Vec::new(),
                metadata: NodeMetadata::default(),
            },
            total_questions: 0,
            total_domains: 0,
            last_harvest: None,
            version: 1,
        }
    }

    /// Add a question to the catalog tree
    pub fn add_question(&mut self, question: HarvestedQuestion) {
        let primary_domain = question.domains.first().cloned().unwrap_or("general".to_string());
        let subdomain = Self::infer_subdomain(&question);
        let topic = Self::infer_topic(&question);

        // Find or create domain index
        let domain_idx = self.ensure_domain_index(&primary_domain);

        // Find or create subdomain index
        let subdomain_idx = self.ensure_subdomain_index(domain_idx, &subdomain);

        // Find or create topic index
        let topic_idx = self.ensure_topic_index(domain_idx, subdomain_idx, &topic);

        // Add question to topic node
        self.root.children[domain_idx].children[subdomain_idx].children[topic_idx]
            .questions
            .push(question);

        self.total_questions += 1;
        self.update_metadata();
    }

    /// Ensure domain exists and return its index
    fn ensure_domain_index(&mut self, name: &str) -> usize {
        if let Some(idx) = self.root.children.iter().position(|n| n.name == name) {
            return idx;
        }

        self.root.children.push(QuestionNode {
            id: format!("domain:{}", name),
            name: name.to_string(),
            node_type: NodeType::Domain,
            children: Vec::new(),
            questions: Vec::new(),
            metadata: NodeMetadata::default(),
        });
        self.total_domains += 1;
        self.root.children.len() - 1
    }

    /// Ensure subdomain exists and return its index
    fn ensure_subdomain_index(&mut self, domain_idx: usize, name: &str) -> usize {
        let domain = &self.root.children[domain_idx];
        if let Some(idx) = domain.children.iter().position(|n| n.name == name) {
            return idx;
        }

        let parent_id = self.root.children[domain_idx].id.clone();
        self.root.children[domain_idx].children.push(QuestionNode {
            id: format!("{}:subdomain:{}", parent_id, name),
            name: name.to_string(),
            node_type: NodeType::Subdomain,
            children: Vec::new(),
            questions: Vec::new(),
            metadata: NodeMetadata::default(),
        });
        self.root.children[domain_idx].children.len() - 1
    }

    /// Ensure topic exists and return its index
    fn ensure_topic_index(&mut self, domain_idx: usize, subdomain_idx: usize, name: &str) -> usize {
        let subdomain = &self.root.children[domain_idx].children[subdomain_idx];
        if let Some(idx) = subdomain.children.iter().position(|n| n.name == name) {
            return idx;
        }

        let parent_id = self.root.children[domain_idx].children[subdomain_idx].id.clone();
        self.root.children[domain_idx].children[subdomain_idx]
            .children
            .push(QuestionNode {
                id: format!("{}:topic:{}", parent_id, name),
                name: name.to_string(),
                node_type: NodeType::Topic,
                children: Vec::new(),
                questions: Vec::new(),
                metadata: NodeMetadata::default(),
            });
        self.root.children[domain_idx].children[subdomain_idx].children.len() - 1
    }

    /// Infer subdomain from question content
    fn infer_subdomain(q: &HarvestedQuestion) -> String {
        let text = format!("{} {}", q.question_text, q.explanation).to_lowercase();

        // Rust subdomains
        if q.domains.contains(&"rust".to_string()) {
            if text.contains("async") || text.contains("future") || text.contains("await") {
                return "async".to_string();
            }
            if text.contains("ownership") || text.contains("borrow") || text.contains("lifetime") {
                return "ownership".to_string();
            }
            if text.contains("trait") || text.contains("impl") || text.contains("generic") {
                return "traits".to_string();
            }
            if text.contains("error") || text.contains("result") || text.contains("panic") {
                return "error_handling".to_string();
            }
            if text.contains("struct") || text.contains("enum") || text.contains("type") {
                return "types".to_string();
            }
            return "fundamentals".to_string();
        }

        // Nix subdomains
        if q.domains.contains(&"nix".to_string()) {
            if text.contains("service") || text.contains("systemd") {
                return "services".to_string();
            }
            if text.contains("network") || text.contains("firewall") {
                return "networking".to_string();
            }
            if text.contains("package") || text.contains("derivation") {
                return "packages".to_string();
            }
            if text.contains("flake") {
                return "flakes".to_string();
            }
            return "configuration".to_string();
        }

        // Python subdomains
        if q.domains.contains(&"python".to_string()) {
            if text.contains("async") || text.contains("await") || text.contains("coroutine") {
                return "async".to_string();
            }
            if text.contains("class") || text.contains("inheritance") || text.contains("method") {
                return "oop".to_string();
            }
            return "fundamentals".to_string();
        }

        // Docker subdomains
        if q.domains.contains(&"docker".to_string()) {
            if text.contains("compose") || text.contains("service") {
                return "compose".to_string();
            }
            if text.contains("image") || text.contains("layer") {
                return "images".to_string();
            }
            return "containers".to_string();
        }

        "general".to_string()
    }

    /// Infer topic from question content
    fn infer_topic(q: &HarvestedQuestion) -> String {
        // Use source file name as a topic hint
        let file_name = q.source_file.split('/').next_back().unwrap_or("unknown");
        let file_stem = file_name.trim_end_matches(".rs")
            .trim_end_matches(".nix")
            .trim_end_matches(".py")
            .trim_end_matches(".toml");

        file_stem.to_string()
    }

    /// Update all metadata
    fn update_metadata(&mut self) {
        self.last_harvest = Some(Utc::now());
        self.root.metadata.question_count = self.total_questions;

        // Recursively update child metadata
        for domain in &mut self.root.children {
            Self::update_node_metadata(domain);
        }
    }

    fn update_node_metadata(node: &mut QuestionNode) {
        let mut count = node.questions.len();
        let mut total_xp = node.questions.iter().map(|q| q.xp).sum::<i32>();

        for child in &mut node.children {
            Self::update_node_metadata(child);
            count += child.metadata.question_count;
            total_xp += child.metadata.total_xp;
        }

        node.metadata.question_count = count;
        node.metadata.total_xp = total_xp;
        node.metadata.last_harvested = Some(Utc::now());
    }

    /// Get all questions for a domain
    pub fn get_domain_questions(&self, domain: &str) -> Vec<&HarvestedQuestion> {
        let mut questions = Vec::new();

        for domain_node in &self.root.children {
            if domain_node.name == domain {
                Self::collect_questions(domain_node, &mut questions);
                break;
            }
        }

        questions
    }

    fn collect_questions<'a>(node: &'a QuestionNode, questions: &mut Vec<&'a HarvestedQuestion>) {
        questions.extend(node.questions.iter());
        for child in &node.children {
            Self::collect_questions(child, questions);
        }
    }

    /// Get tree structure as ASCII art
    pub fn tree_view(&self) -> String {
        let mut output = String::new();
        output.push_str(&format!("📚 {} ({} questions)\n", self.root.name, self.total_questions));

        for (i, domain) in self.root.children.iter().enumerate() {
            let is_last = i == self.root.children.len() - 1;
            let prefix = if is_last { "└── " } else { "├── " };
            let child_prefix = if is_last { "    " } else { "│   " };

            output.push_str(&format!(
                "{}📁 {} ({} Qs)\n",
                prefix, domain.name, domain.metadata.question_count
            ));

            for (j, subdomain) in domain.children.iter().enumerate() {
                let is_last_sub = j == domain.children.len() - 1;
                let sub_prefix = if is_last_sub { "└── " } else { "├── " };
                let sub_child_prefix = if is_last_sub { "    " } else { "│   " };

                output.push_str(&format!(
                    "{}{}📂 {} ({} Qs)\n",
                    child_prefix, sub_prefix, subdomain.name, subdomain.metadata.question_count
                ));

                for (k, topic) in subdomain.children.iter().enumerate() {
                    let is_last_topic = k == subdomain.children.len() - 1;
                    let topic_prefix = if is_last_topic { "└── " } else { "├── " };

                    output.push_str(&format!(
                        "{}{}{}📄 {} ({} Qs)\n",
                        child_prefix, sub_child_prefix, topic_prefix,
                        topic.name, topic.questions.len()
                    ));
                }
            }
        }

        output
    }

    /// Save catalog to JSON file
    pub fn save(&self, path: &Path) -> std::io::Result<()> {
        let json = serde_json::to_string_pretty(self)
            .map_err(std::io::Error::other)?;
        std::fs::write(path, json)
    }

    /// Load catalog from JSON file
    pub fn load(path: &Path) -> std::io::Result<Self> {
        let json = std::fs::read_to_string(path)?;
        serde_json::from_str(&json)
            .map_err(std::io::Error::other)
    }
}

/// Harvester - scans directories and builds the catalog
pub struct Harvester {
    analyzer: CodebaseAnalyzer,
    catalog: QuestionCatalog,
}

impl Default for Harvester {
    fn default() -> Self {
        Self::new()
    }
}

impl Harvester {
    pub fn new() -> Self {
        Self {
            analyzer: CodebaseAnalyzer::new(),
            catalog: QuestionCatalog::new(),
        }
    }

    pub fn with_catalog(catalog: QuestionCatalog) -> Self {
        Self {
            analyzer: CodebaseAnalyzer::new(),
            catalog,
        }
    }

    /// Harvest questions from a directory
    pub fn harvest(&mut self, path: &Path) -> HarvestResult {
        let mut result = HarvestResult::default();

        let project_name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string();

        // Analyze the codebase
        match self.analyzer.analyze(path) {
            Ok(analysis) => {
                result.files_scanned = analysis.elements.len();
                result.languages = analysis.detected_languages.clone();

                // Generate questions from sprints
                for sprint in &analysis.suggested_sprints {
                    for q in &sprint.questions {
                        let harvested = HarvestedQuestion::from_generated(q, &project_name);
                        self.catalog.add_question(harvested);
                        result.questions_generated += 1;
                    }
                }

                result.success = true;
            }
            Err(e) => {
                result.error = Some(e.to_string());
            }
        }

        result
    }

    /// Harvest from multiple directories
    pub fn harvest_all(&mut self, paths: &[PathBuf]) -> Vec<HarvestResult> {
        paths.iter().map(|p| self.harvest(p)).collect()
    }

    /// Get the catalog
    pub fn catalog(&self) -> &QuestionCatalog {
        &self.catalog
    }

    /// Take ownership of the catalog
    pub fn into_catalog(self) -> QuestionCatalog {
        self.catalog
    }
}

#[derive(Debug, Clone, Default)]
pub struct HarvestResult {
    pub success: bool,
    pub files_scanned: usize,
    pub questions_generated: usize,
    pub languages: Vec<String>,
    pub error: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_catalog_creation() {
        let catalog = QuestionCatalog::new();
        assert_eq!(catalog.total_questions, 0);
        assert_eq!(catalog.total_domains, 0);
    }

    #[test]
    fn test_add_question() {
        let mut catalog = QuestionCatalog::new();

        let q = HarvestedQuestion {
            id: "test:1".to_string(),
            question_text: "What is async?".to_string(),
            options: vec!["A".to_string(), "B".to_string()],
            correct_answer: 'A',
            hint: "Think about futures".to_string(),
            explanation: "Async returns a Future".to_string(),
            tier: "RECALL".to_string(),
            difficulty: "Easy".to_string(),
            xp: 10,
            code_snippet: None,
            source_file: "test.rs".to_string(),
            source_line: 1,
            source_project: "test".to_string(),
            domains: vec!["rust".to_string()],
            tags: vec![],
            harvested_at: Utc::now(),
            times_used: 0,
            times_correct: 0,
        };

        catalog.add_question(q);

        assert_eq!(catalog.total_questions, 1);
        assert_eq!(catalog.total_domains, 1);
    }

    #[test]
    fn test_domain_inference() {
        let rust_async_q = HarvestedQuestion {
            id: "test:1".to_string(),
            question_text: "What is async await?".to_string(),
            options: vec![],
            correct_answer: 'A',
            hint: String::new(),
            explanation: "Async functions use await".to_string(),
            tier: "RECALL".to_string(),
            difficulty: "Easy".to_string(),
            xp: 10,
            code_snippet: None,
            source_file: "async.rs".to_string(),
            source_line: 1,
            source_project: "test".to_string(),
            domains: vec!["rust".to_string()],
            tags: vec![],
            harvested_at: Utc::now(),
            times_used: 0,
            times_correct: 0,
        };

        let subdomain = QuestionCatalog::infer_subdomain(&rust_async_q);
        assert_eq!(subdomain, "async");
    }

    #[test]
    fn test_tree_view() {
        let mut catalog = QuestionCatalog::new();

        for i in 0..3 {
            let q = HarvestedQuestion {
                id: format!("test:{}", i),
                question_text: format!("Question {}?", i),
                options: vec![],
                correct_answer: 'A',
                hint: String::new(),
                explanation: "Async test".to_string(),
                tier: "RECALL".to_string(),
                difficulty: "Easy".to_string(),
                xp: 10,
                code_snippet: None,
                source_file: "test.rs".to_string(),
                source_line: i,
                source_project: "test".to_string(),
                domains: vec!["rust".to_string()],
                tags: vec![],
                harvested_at: Utc::now(),
                times_used: 0,
                times_correct: 0,
            };
            catalog.add_question(q);
        }

        let tree = catalog.tree_view();
        assert!(tree.contains("rust"));
        assert!(tree.contains("3 Qs"));
    }

    #[test]
    fn test_catalog_save_load() {
        let mut catalog = QuestionCatalog::new();
        catalog.add_question(HarvestedQuestion {
            id: "test:1".to_string(),
            question_text: "Test?".to_string(),
            options: vec![],
            correct_answer: 'A',
            hint: String::new(),
            explanation: String::new(),
            tier: "RECALL".to_string(),
            difficulty: "Easy".to_string(),
            xp: 10,
            code_snippet: None,
            source_file: "test.rs".to_string(),
            source_line: 1,
            source_project: "test".to_string(),
            domains: vec!["rust".to_string()],
            tags: vec![],
            harvested_at: Utc::now(),
            times_used: 0,
            times_correct: 0,
        });

        let temp_dir = std::env::temp_dir();
        let path = temp_dir.join("test_catalog.json");

        catalog.save(&path).unwrap();
        let loaded = QuestionCatalog::load(&path).unwrap();

        assert_eq!(loaded.total_questions, 1);

        std::fs::remove_file(&path).ok();
    }
}
