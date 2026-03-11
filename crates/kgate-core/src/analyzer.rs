//! Codebase Analyzer - Auto-generates exam questions from project files
//!
//! Phase 7: Scans project files and generates exam content based on:
//! - Code patterns (functions, modules, structs, traits)
//! - Config files (Nix, Docker, TOML, YAML)
//! - Dependencies and their usage
//! - Architecture patterns

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Represents an analyzed code element that can generate questions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeElement {
    pub element_type: ElementType,
    pub name: String,
    pub file_path: String,
    pub line_number: usize,
    pub context: String,           // Surrounding code/config
    pub complexity: Complexity,
    pub domains: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ElementType {
    Function,
    Struct,
    Trait,
    Module,
    NixOption,
    NixService,
    DockerService,
    Dependency,
    ConfigValue,
    SecurityPattern,
    ErrorHandler,
    ApiEndpoint,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum Complexity {
    Simple,    // Tier 1-2: RECALL, COMPREHENSION
    Medium,    // Tier 3: APPLICATION
    Complex,   // Tier 4: ANALYSIS
}

/// Generated question from code analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratedQuestion {
    pub tier: String,
    pub difficulty: String,
    pub xp: i32,
    pub question_text: String,
    pub code_snippet: Option<String>,
    pub options: Vec<String>,
    pub correct_answer: char,
    pub hint: String,
    pub explanation: String,
    pub source_file: String,
    pub source_line: usize,
    pub domains: Vec<String>,
}

/// Analysis result for a project
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectAnalysis {
    pub project_path: PathBuf,
    pub project_name: String,
    pub elements: Vec<CodeElement>,
    pub detected_languages: Vec<String>,
    pub detected_frameworks: Vec<String>,
    pub suggested_sprints: Vec<SprintSuggestion>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SprintSuggestion {
    pub topic: String,
    pub questions: Vec<GeneratedQuestion>,
    pub total_xp: i32,
}

/// Main analyzer struct
#[derive(Default)]
pub struct CodebaseAnalyzer;

impl CodebaseAnalyzer {
    pub fn new() -> Self {
        Self
    }

    /// Analyze a project directory and return findings
    pub fn analyze(&self, project_path: &Path) -> Result<ProjectAnalysis> {
        let project_name = project_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string();

        let mut elements = Vec::new();
        let mut detected_languages = Vec::new();
        let mut detected_frameworks = Vec::new();

        // Scan for different file types
        self.scan_directory(project_path, &mut elements, &mut detected_languages, &mut detected_frameworks)?;

        // Group elements into sprint suggestions
        let suggested_sprints = self.generate_sprint_suggestions(&elements, &detected_languages);

        Ok(ProjectAnalysis {
            project_path: project_path.to_path_buf(),
            project_name,
            elements,
            detected_languages,
            detected_frameworks,
            suggested_sprints,
        })
    }

    fn scan_directory(
        &self,
        path: &Path,
        elements: &mut Vec<CodeElement>,
        languages: &mut Vec<String>,
        frameworks: &mut Vec<String>,
    ) -> Result<()> {
        if !path.is_dir() {
            return Ok(());
        }

        for entry in std::fs::read_dir(path)? {
            let entry = entry?;
            let file_path = entry.path();

            // Skip hidden dirs and common excludes
            let file_name = file_path.file_name().and_then(|n| n.to_str()).unwrap_or("");
            if file_name.starts_with('.') || file_name == "target" || file_name == "node_modules" {
                continue;
            }

            if file_path.is_dir() {
                self.scan_directory(&file_path, elements, languages, frameworks)?;
            } else if let Some(ext) = file_path.extension().and_then(|e| e.to_str()) {
                match ext {
                    "rs" => {
                        if !languages.contains(&"rust".to_string()) {
                            languages.push("rust".to_string());
                        }
                        self.analyze_rust_file(&file_path, elements)?;
                    }
                    "nix" => {
                        if !languages.contains(&"nix".to_string()) {
                            languages.push("nix".to_string());
                        }
                        self.analyze_nix_file(&file_path, elements)?;
                    }
                    "py" => {
                        if !languages.contains(&"python".to_string()) {
                            languages.push("python".to_string());
                        }
                        self.analyze_python_file(&file_path, elements)?;
                    }
                    "toml" => {
                        self.analyze_toml_file(&file_path, elements, frameworks)?;
                    }
                    "yaml" | "yml" => {
                        self.analyze_yaml_file(&file_path, elements, frameworks)?;
                    }
                    _ => {}
                }

                // Check for Dockerfile
                if file_name == "Dockerfile" || file_name.starts_with("Dockerfile.") {
                    if !frameworks.contains(&"docker".to_string()) {
                        frameworks.push("docker".to_string());
                    }
                    self.analyze_dockerfile(&file_path, elements)?;
                }
            }
        }

        Ok(())
    }

    fn analyze_rust_file(&self, path: &Path, elements: &mut Vec<CodeElement>) -> Result<()> {
        let content = std::fs::read_to_string(path)?;
        let path_str = path.to_string_lossy().to_string();

        // Function definitions
        let fn_regex = regex::Regex::new(r"(?m)^(\s*)(pub\s+)?(async\s+)?fn\s+(\w+)")?;
        for cap in fn_regex.captures_iter(&content) {
            let name = cap.get(4).map(|m| m.as_str()).unwrap_or("");
            let line_num = content[..cap.get(0).unwrap().start()].lines().count() + 1;
            let context = self.extract_context(&content, line_num, 5);

            let complexity = if context.contains("async") || context.contains("Result<") {
                Complexity::Medium
            } else if context.contains("unsafe") || context.len() > 200 {
                Complexity::Complex
            } else {
                Complexity::Simple
            };

            elements.push(CodeElement {
                element_type: ElementType::Function,
                name: name.to_string(),
                file_path: path_str.clone(),
                line_number: line_num,
                context,
                complexity,
                domains: vec!["rust".to_string()],
            });
        }

        // Struct definitions
        let struct_regex = regex::Regex::new(r"(?m)^(\s*)(pub\s+)?struct\s+(\w+)")?;
        for cap in struct_regex.captures_iter(&content) {
            let name = cap.get(3).map(|m| m.as_str()).unwrap_or("");
            let line_num = content[..cap.get(0).unwrap().start()].lines().count() + 1;
            let context = self.extract_context(&content, line_num, 8);

            elements.push(CodeElement {
                element_type: ElementType::Struct,
                name: name.to_string(),
                file_path: path_str.clone(),
                line_number: line_num,
                context,
                complexity: Complexity::Medium,
                domains: vec!["rust".to_string(), "architecture".to_string()],
            });
        }

        // Trait implementations
        let impl_regex = regex::Regex::new(r"(?m)^impl\s+(\w+)\s+for\s+(\w+)")?;
        for cap in impl_regex.captures_iter(&content) {
            let trait_name = cap.get(1).map(|m| m.as_str()).unwrap_or("");
            let struct_name = cap.get(2).map(|m| m.as_str()).unwrap_or("");
            let line_num = content[..cap.get(0).unwrap().start()].lines().count() + 1;
            let context = self.extract_context(&content, line_num, 5);

            elements.push(CodeElement {
                element_type: ElementType::Trait,
                name: format!("{} for {}", trait_name, struct_name),
                file_path: path_str.clone(),
                line_number: line_num,
                context,
                complexity: Complexity::Complex,
                domains: vec!["rust".to_string(), "architecture".to_string()],
            });
        }

        // Error handling patterns
        let error_regex = regex::Regex::new(r"(?m)\?\s*;|\.unwrap\(\)|\.expect\(|anyhow!|thiserror")?;
        for mat in error_regex.find_iter(&content) {
            let line_num = content[..mat.start()].lines().count() + 1;
            let context = self.extract_context(&content, line_num, 3);

            elements.push(CodeElement {
                element_type: ElementType::ErrorHandler,
                name: "error_handling".to_string(),
                file_path: path_str.clone(),
                line_number: line_num,
                context,
                complexity: Complexity::Medium,
                domains: vec!["rust".to_string(), "testing".to_string()],
            });
        }

        Ok(())
    }

    fn analyze_nix_file(&self, path: &Path, elements: &mut Vec<CodeElement>) -> Result<()> {
        let content = std::fs::read_to_string(path)?;
        let path_str = path.to_string_lossy().to_string();

        // Service enables
        let service_regex = regex::Regex::new(r"services\.(\w+)\.enable\s*=\s*(true|false)")?;
        for cap in service_regex.captures_iter(&content) {
            let service = cap.get(1).map(|m| m.as_str()).unwrap_or("");
            let enabled = cap.get(2).map(|m| m.as_str()).unwrap_or("") == "true";
            let line_num = content[..cap.get(0).unwrap().start()].lines().count() + 1;
            let context = self.extract_context(&content, line_num, 5);

            elements.push(CodeElement {
                element_type: ElementType::NixService,
                name: format!("services.{} = {}", service, enabled),
                file_path: path_str.clone(),
                line_number: line_num,
                context,
                complexity: Complexity::Simple,
                domains: vec!["nix".to_string(), "linux".to_string()],
            });
        }

        // Nix options
        let option_regex = regex::Regex::new(r"(\w+(?:\.\w+)+)\s*=\s*(.+?);")?;
        for cap in option_regex.captures_iter(&content) {
            let option = cap.get(1).map(|m| m.as_str()).unwrap_or("");
            let line_num = content[..cap.get(0).unwrap().start()].lines().count() + 1;
            let context = self.extract_context(&content, line_num, 3);

            // Skip if already captured as service
            if option.starts_with("services.") && option.ends_with(".enable") {
                continue;
            }

            let complexity = if option.contains("security") || option.contains("firewall") {
                Complexity::Complex
            } else if option.contains("networking") || option.contains("boot") {
                Complexity::Medium
            } else {
                Complexity::Simple
            };

            let mut domains = vec!["nix".to_string()];
            if option.contains("network") {
                domains.push("networking".to_string());
            }
            if option.contains("security") || option.contains("firewall") {
                domains.push("security".to_string());
            }

            elements.push(CodeElement {
                element_type: ElementType::NixOption,
                name: option.to_string(),
                file_path: path_str.clone(),
                line_number: line_num,
                context,
                complexity,
                domains,
            });
        }

        Ok(())
    }

    fn analyze_python_file(&self, path: &Path, elements: &mut Vec<CodeElement>) -> Result<()> {
        let content = std::fs::read_to_string(path)?;
        let path_str = path.to_string_lossy().to_string();

        // Function definitions
        let fn_regex = regex::Regex::new(r"(?m)^(\s*)(async\s+)?def\s+(\w+)")?;
        for cap in fn_regex.captures_iter(&content) {
            let name = cap.get(3).map(|m| m.as_str()).unwrap_or("");
            let is_async = cap.get(2).is_some();
            let line_num = content[..cap.get(0).unwrap().start()].lines().count() + 1;
            let context = self.extract_context(&content, line_num, 5);

            let complexity = if is_async || context.contains("yield") {
                Complexity::Medium
            } else {
                Complexity::Simple
            };

            elements.push(CodeElement {
                element_type: ElementType::Function,
                name: name.to_string(),
                file_path: path_str.clone(),
                line_number: line_num,
                context,
                complexity,
                domains: vec!["python".to_string()],
            });
        }

        // Class definitions
        let class_regex = regex::Regex::new(r"(?m)^class\s+(\w+)")?;
        for cap in class_regex.captures_iter(&content) {
            let name = cap.get(1).map(|m| m.as_str()).unwrap_or("");
            let line_num = content[..cap.get(0).unwrap().start()].lines().count() + 1;
            let context = self.extract_context(&content, line_num, 8);

            elements.push(CodeElement {
                element_type: ElementType::Struct, // Reuse for classes
                name: name.to_string(),
                file_path: path_str.clone(),
                line_number: line_num,
                context,
                complexity: Complexity::Medium,
                domains: vec!["python".to_string(), "architecture".to_string()],
            });
        }

        Ok(())
    }

    fn analyze_toml_file(
        &self,
        path: &Path,
        elements: &mut Vec<CodeElement>,
        frameworks: &mut Vec<String>,
    ) -> Result<()> {
        let content = std::fs::read_to_string(path)?;
        let path_str = path.to_string_lossy().to_string();
        let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");

        // Cargo.toml dependencies
        if file_name == "Cargo.toml" {
            let dep_regex = regex::Regex::new(r#"(?m)^(\w[\w-]*)\s*=\s*(?:"([^"]+)"|\{[^}]*version\s*=\s*"([^"]+)")"#)?;
            let in_deps = content.contains("[dependencies]") || content.contains("[dev-dependencies]");

            if in_deps {
                for cap in dep_regex.captures_iter(&content) {
                    let dep_name = cap.get(1).map(|m| m.as_str()).unwrap_or("");
                    let line_num = content[..cap.get(0).unwrap().start()].lines().count() + 1;

                    // Detect frameworks
                    match dep_name {
                        "tokio" | "async-std" => {
                            if !frameworks.contains(&"async-rust".to_string()) {
                                frameworks.push("async-rust".to_string());
                            }
                        }
                        "axum" | "actix-web" | "rocket" => {
                            if !frameworks.contains(&"web-rust".to_string()) {
                                frameworks.push("web-rust".to_string());
                            }
                        }
                        "sqlx" | "diesel" | "rusqlite" => {
                            if !frameworks.contains(&"database".to_string()) {
                                frameworks.push("database".to_string());
                            }
                        }
                        _ => {}
                    }

                    elements.push(CodeElement {
                        element_type: ElementType::Dependency,
                        name: dep_name.to_string(),
                        file_path: path_str.clone(),
                        line_number: line_num,
                        context: format!("Dependency: {}", dep_name),
                        complexity: Complexity::Simple,
                        domains: vec!["rust".to_string()],
                    });
                }
            }
        }

        Ok(())
    }

    fn analyze_yaml_file(
        &self,
        path: &Path,
        elements: &mut Vec<CodeElement>,
        frameworks: &mut Vec<String>,
    ) -> Result<()> {
        let content = std::fs::read_to_string(path)?;
        let path_str = path.to_string_lossy().to_string();
        let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");

        // Docker Compose
        if file_name == "docker-compose.yml" || file_name == "docker-compose.yaml" || file_name == "compose.yml" {
            if !frameworks.contains(&"docker-compose".to_string()) {
                frameworks.push("docker-compose".to_string());
            }

            let service_regex = regex::Regex::new(r"(?m)^  (\w[\w-]*):\s*$")?;
            for cap in service_regex.captures_iter(&content) {
                let service = cap.get(1).map(|m| m.as_str()).unwrap_or("");
                let line_num = content[..cap.get(0).unwrap().start()].lines().count() + 1;
                let context = self.extract_context(&content, line_num, 10);

                elements.push(CodeElement {
                    element_type: ElementType::DockerService,
                    name: service.to_string(),
                    file_path: path_str.clone(),
                    line_number: line_num,
                    context,
                    complexity: Complexity::Medium,
                    domains: vec!["docker".to_string(), "devops".to_string()],
                });
            }
        }

        Ok(())
    }

    fn analyze_dockerfile(&self, path: &Path, elements: &mut Vec<CodeElement>) -> Result<()> {
        let content = std::fs::read_to_string(path)?;
        let path_str = path.to_string_lossy().to_string();

        // FROM statements
        let from_regex = regex::Regex::new(r"(?m)^FROM\s+(\S+)")?;
        for cap in from_regex.captures_iter(&content) {
            let image = cap.get(1).map(|m| m.as_str()).unwrap_or("");
            let line_num = content[..cap.get(0).unwrap().start()].lines().count() + 1;

            elements.push(CodeElement {
                element_type: ElementType::DockerService,
                name: format!("base: {}", image),
                file_path: path_str.clone(),
                line_number: line_num,
                context: format!("FROM {}", image),
                complexity: Complexity::Simple,
                domains: vec!["docker".to_string()],
            });
        }

        // EXPOSE statements (port mappings)
        let expose_regex = regex::Regex::new(r"(?m)^EXPOSE\s+(\d+)")?;
        for cap in expose_regex.captures_iter(&content) {
            let port = cap.get(1).map(|m| m.as_str()).unwrap_or("");
            let line_num = content[..cap.get(0).unwrap().start()].lines().count() + 1;

            elements.push(CodeElement {
                element_type: ElementType::ConfigValue,
                name: format!("port: {}", port),
                file_path: path_str.clone(),
                line_number: line_num,
                context: format!("EXPOSE {}", port),
                complexity: Complexity::Simple,
                domains: vec!["docker".to_string(), "networking".to_string()],
            });
        }

        Ok(())
    }

    fn extract_context(&self, content: &str, line_num: usize, num_lines: usize) -> String {
        let lines: Vec<&str> = content.lines().collect();
        let start = line_num.saturating_sub(1);
        let end = (start + num_lines).min(lines.len());

        lines[start..end].join("\n")
    }

    fn generate_sprint_suggestions(&self, elements: &[CodeElement], languages: &[String]) -> Vec<SprintSuggestion> {
        let mut sprints = Vec::new();
        let mut domain_elements: HashMap<String, Vec<&CodeElement>> = HashMap::new();

        // Group elements by primary domain
        for elem in elements {
            if let Some(domain) = elem.domains.first() {
                domain_elements.entry(domain.clone()).or_default().push(elem);
            }
        }

        // Generate sprint for each domain with enough elements
        for (domain, elems) in domain_elements {
            if elems.len() < 2 {
                continue;
            }

            let questions = self.generate_questions_for_domain(&domain, &elems);
            if questions.len() >= 2 {
                let total_xp: i32 = questions.iter().map(|q| q.xp).sum();
                sprints.push(SprintSuggestion {
                    topic: self.domain_to_topic(&domain, languages),
                    questions,
                    total_xp,
                });
            }
        }

        // Ensure we have at least one sprint
        if sprints.is_empty() && !elements.is_empty() {
            let questions = self.generate_generic_questions(elements);
            if !questions.is_empty() {
                let total_xp: i32 = questions.iter().map(|q| q.xp).sum();
                sprints.push(SprintSuggestion {
                    topic: "Project Fundamentals".to_string(),
                    questions,
                    total_xp,
                });
            }
        }

        // Limit to 3 questions per sprint, max 3 sprints
        for sprint in &mut sprints {
            sprint.questions.truncate(3);
            sprint.total_xp = sprint.questions.iter().map(|q| q.xp).sum();
        }
        sprints.truncate(3);

        sprints
    }

    fn generate_questions_for_domain(&self, _domain: &str, elements: &[&CodeElement]) -> Vec<GeneratedQuestion> {
        let mut questions = Vec::new();

        for (i, elem) in elements.iter().enumerate().take(3) {
            let question = self.element_to_question(elem, i + 1);
            questions.push(question);
        }

        questions
    }

    fn element_to_question(&self, elem: &CodeElement, _q_num: usize) -> GeneratedQuestion {
        let (tier, difficulty, xp) = match elem.complexity {
            Complexity::Simple => ("RECALL", "Easy", 10),
            Complexity::Medium => ("COMPREHENSION", "Medium", 10),
            Complexity::Complex => ("APPLICATION", "Challenge", 10),
        };

        let (question_text, options, correct, hint, explanation) = match elem.element_type {
            ElementType::Function => self.generate_function_question(elem),
            ElementType::Struct => self.generate_struct_question(elem),
            ElementType::NixService => self.generate_nix_service_question(elem),
            ElementType::NixOption => self.generate_nix_option_question(elem),
            ElementType::DockerService => self.generate_docker_question(elem),
            ElementType::Dependency => self.generate_dependency_question(elem),
            _ => self.generate_generic_element_question(elem),
        };

        GeneratedQuestion {
            tier: tier.to_string(),
            difficulty: difficulty.to_string(),
            xp,
            question_text,
            code_snippet: Some(elem.context.clone()),
            options,
            correct_answer: correct,
            hint,
            explanation,
            source_file: elem.file_path.clone(),
            source_line: elem.line_number,
            domains: elem.domains.clone(),
        }
    }

    fn generate_function_question(&self, elem: &CodeElement) -> (String, Vec<String>, char, String, String) {
        let is_async = elem.context.contains("async");
        let is_pub = elem.context.contains("pub ");

        if is_async {
            (
                format!("What does the `{}` function do when called?", elem.name),
                vec![
                    format!("A) Executes synchronously and blocks"),
                    format!("B) Returns a Future that must be awaited"),
                    format!("C) Spawns a new thread automatically"),
                    format!("D) Panics if not in async context"),
                ],
                'B',
                "Think about what 'async' means in Rust".to_string(),
                "Async functions return a Future that must be .await'd to execute".to_string(),
            )
        } else if is_pub {
            (
                format!("Why is `{}` marked as `pub`?", elem.name),
                vec![
                    format!("A) It can be called from other modules"),
                    format!("B) It runs in a separate thread"),
                    format!("C) It's automatically tested"),
                    format!("D) It cannot panic"),
                ],
                'A',
                "Think about visibility in Rust".to_string(),
                "pub makes the function accessible from other modules".to_string(),
            )
        } else {
            (
                format!("What is the scope of the `{}` function?", elem.name),
                vec![
                    format!("A) Global - accessible everywhere"),
                    format!("B) Module-private - only this module"),
                    format!("C) Crate-public - anywhere in crate"),
                    format!("D) File-private - only this file"),
                ],
                'B',
                "Functions without pub are private".to_string(),
                "Without pub, functions are private to their module".to_string(),
            )
        }
    }

    fn generate_struct_question(&self, elem: &CodeElement) -> (String, Vec<String>, char, String, String) {
        let has_derive = elem.context.contains("#[derive");

        if has_derive {
            (
                format!("What does #[derive(...)] do for `{}`?", elem.name),
                vec![
                    format!("A) Automatically implements traits"),
                    format!("B) Creates a new instance"),
                    format!("C) Makes the struct mutable"),
                    format!("D) Adds runtime checks"),
                ],
                'A',
                "derive is a procedural macro".to_string(),
                "#[derive] automatically implements specified traits".to_string(),
            )
        } else {
            (
                format!("What is `{}` in this code?", elem.name),
                vec![
                    format!("A) A function"),
                    format!("B) A data structure (struct)"),
                    format!("C) A module"),
                    format!("D) A constant"),
                ],
                'B',
                "Look at the keyword before the name".to_string(),
                "struct defines a custom data type in Rust".to_string(),
            )
        }
    }

    fn generate_nix_service_question(&self, elem: &CodeElement) -> (String, Vec<String>, char, String, String) {
        let service_name = elem.name.split('.').nth(1).unwrap_or(&elem.name);
        let _is_enabled = elem.name.contains("true");

        (
            format!("What happens when `services.{}.enable = true`?", service_name),
            vec![
                format!("A) The service is installed but not started"),
                format!("B) The service is configured and started on boot"),
                format!("C) Only the service package is downloaded"),
                format!("D) The service runs once then stops"),
            ],
            'B',
            "enable = true does more than just install".to_string(),
            "Setting enable = true configures systemd to start the service on boot".to_string(),
        )
    }

    fn generate_nix_option_question(&self, elem: &CodeElement) -> (String, Vec<String>, char, String, String) {
        (
            format!("What type of configuration is `{}`?", elem.name),
            vec![
                format!("A) Runtime environment variable"),
                format!("B) NixOS module option"),
                format!("C) Shell alias"),
                format!("D) Temporary setting"),
            ],
            'B',
            "NixOS uses a declarative configuration model".to_string(),
            "This is a NixOS option that gets evaluated during system build".to_string(),
        )
    }

    fn generate_docker_question(&self, _elem: &CodeElement) -> (String, Vec<String>, char, String, String) {
        (
            "What does this Docker configuration define?".to_string(),
            vec![
                format!("A) A container service"),
                format!("B) A network bridge"),
                format!("C) A volume mount"),
                format!("D) An environment file"),
            ],
            'A',
            "Look at the structure of the YAML".to_string(),
            "This defines a Docker container service configuration".to_string(),
        )
    }

    fn generate_dependency_question(&self, elem: &CodeElement) -> (String, Vec<String>, char, String, String) {
        (
            format!("What is `{}` in Cargo.toml?", elem.name),
            vec![
                format!("A) A build script"),
                format!("B) A crate dependency"),
                format!("C) A binary target"),
                format!("D) A feature flag"),
            ],
            'B',
            "Dependencies are external crates".to_string(),
            format!("{} is an external crate that this project depends on", elem.name),
        )
    }

    fn generate_generic_element_question(&self, elem: &CodeElement) -> (String, Vec<String>, char, String, String) {
        (
            format!("What is the purpose of `{}`?", elem.name),
            vec![
                format!("A) Configuration setting"),
                format!("B) Code definition"),
                format!("C) Documentation comment"),
                format!("D) Test fixture"),
            ],
            'A',
            "Think about the context where this appears".to_string(),
            format!("This is a configuration or code element: {}", elem.name),
        )
    }

    fn generate_generic_questions(&self, elements: &[CodeElement]) -> Vec<GeneratedQuestion> {
        elements
            .iter()
            .take(3)
            .enumerate()
            .map(|(i, e)| self.element_to_question(e, i + 1))
            .collect()
    }

    fn domain_to_topic(&self, domain: &str, languages: &[String]) -> String {
        match domain {
            "rust" => "Rust Fundamentals".to_string(),
            "nix" => "NixOS Configuration".to_string(),
            "python" => "Python Basics".to_string(),
            "docker" => "Docker & Containers".to_string(),
            "networking" => "Networking Concepts".to_string(),
            "security" => "Security Practices".to_string(),
            "architecture" => "Code Architecture".to_string(),
            _ => {
                if let Some(lang) = languages.first() {
                    format!("{} Fundamentals", lang.to_uppercase())
                } else {
                    "Project Basics".to_string()
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_analyze_rust_file() {
        let temp_dir = TempDir::new().unwrap();
        let rust_file = temp_dir.path().join("test.rs");

        fs::write(&rust_file, r#"
pub fn hello_world() {
    println!("Hello!");
}

pub async fn fetch_data() -> Result<String, Error> {
    Ok("data".to_string())
}

struct MyStruct {
    field: i32,
}
"#).unwrap();

        let analyzer = CodebaseAnalyzer::new();
        let mut elements = Vec::new();
        analyzer.analyze_rust_file(&rust_file, &mut elements).unwrap();

        assert!(elements.len() >= 2);
        assert!(elements.iter().any(|e| e.name == "hello_world"));
        assert!(elements.iter().any(|e| e.name == "fetch_data"));
    }

    #[test]
    fn test_analyze_nix_file() {
        let temp_dir = TempDir::new().unwrap();
        let nix_file = temp_dir.path().join("configuration.nix");

        fs::write(&nix_file, r#"
{ config, pkgs, ... }:
{
  services.openssh.enable = true;
  services.nginx.enable = false;
  networking.firewall.enable = true;
}
"#).unwrap();

        let analyzer = CodebaseAnalyzer::new();
        let mut elements = Vec::new();
        analyzer.analyze_nix_file(&nix_file, &mut elements).unwrap();

        assert!(elements.len() >= 2);
        assert!(elements.iter().any(|e| e.name.contains("openssh")));
    }

    #[test]
    fn test_generate_sprint_suggestions() {
        let elements = vec![
            CodeElement {
                element_type: ElementType::Function,
                name: "test_fn".to_string(),
                file_path: "/test.rs".to_string(),
                line_number: 1,
                context: "pub fn test_fn() {}".to_string(),
                complexity: Complexity::Simple,
                domains: vec!["rust".to_string()],
            },
            CodeElement {
                element_type: ElementType::Function,
                name: "another_fn".to_string(),
                file_path: "/test.rs".to_string(),
                line_number: 5,
                context: "pub async fn another_fn() {}".to_string(),
                complexity: Complexity::Medium,
                domains: vec!["rust".to_string()],
            },
        ];

        let analyzer = CodebaseAnalyzer::new();
        let sprints = analyzer.generate_sprint_suggestions(&elements, &["rust".to_string()]);

        assert!(!sprints.is_empty());
        assert!(sprints[0].questions.len() >= 2);
    }

    #[test]
    fn test_element_to_question() {
        let elem = CodeElement {
            element_type: ElementType::Function,
            name: "my_async_fn".to_string(),
            file_path: "/test.rs".to_string(),
            line_number: 1,
            context: "pub async fn my_async_fn() -> Result<()> {}".to_string(),
            complexity: Complexity::Medium,
            domains: vec!["rust".to_string()],
        };

        let analyzer = CodebaseAnalyzer::new();
        let question = analyzer.element_to_question(&elem, 1);

        assert!(!question.question_text.is_empty());
        assert_eq!(question.options.len(), 4);
        assert!(question.correct_answer >= 'A' && question.correct_answer <= 'D');
    }

    #[test]
    fn test_complexity_mapping() {
        let simple = CodeElement {
            element_type: ElementType::Function,
            name: "simple".to_string(),
            file_path: "/test.rs".to_string(),
            line_number: 1,
            context: "fn simple() {}".to_string(),
            complexity: Complexity::Simple,
            domains: vec!["rust".to_string()],
        };

        let analyzer = CodebaseAnalyzer::new();
        let q = analyzer.element_to_question(&simple, 1);

        assert_eq!(q.tier, "RECALL");
        assert_eq!(q.difficulty, "Easy");
        assert_eq!(q.xp, 10);
    }
}
