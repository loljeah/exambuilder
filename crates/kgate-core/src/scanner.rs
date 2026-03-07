use anyhow::Result;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct ScannedProject {
    pub path: PathBuf,
    pub name: String,
    pub exam_file: Option<PathBuf>,
    pub qa_files: Vec<PathBuf>,
    pub knowledge_files: Vec<PathBuf>,
    pub is_git_repo: bool,
}

#[derive(Debug, Clone)]
pub struct ScanResult {
    pub root: PathBuf,
    pub projects: Vec<ScannedProject>,
    pub total_exams: usize,
    pub total_qa: usize,
    pub total_knowledge: usize,
}

/// Scan a directory for projects containing exam/qa/knowledge files
pub fn scan_directory(root: &Path, max_depth: usize) -> Result<ScanResult> {
    let mut projects: Vec<ScannedProject> = Vec::new();

    scan_recursive(root, root, &mut projects, 0, max_depth)?;

    let total_exams = projects.iter().filter(|p| p.exam_file.is_some()).count();
    let total_qa: usize = projects.iter().map(|p| p.qa_files.len()).sum();
    let total_knowledge: usize = projects.iter().map(|p| p.knowledge_files.len()).sum();

    Ok(ScanResult {
        root: root.to_path_buf(),
        projects,
        total_exams,
        total_qa,
        total_knowledge,
    })
}

fn scan_recursive(
    root: &Path,
    current: &Path,
    projects: &mut Vec<ScannedProject>,
    depth: usize,
    max_depth: usize,
) -> Result<()> {
    if depth > max_depth {
        return Ok(());
    }

    // Skip hidden dirs, node_modules, target, venv, etc.
    let dir_name = current.file_name().and_then(|n| n.to_str()).unwrap_or("");
    if dir_name.starts_with('.') && depth > 0 {
        return Ok(());
    }
    if matches!(dir_name, "node_modules" | "target" | "venv" | ".venv" | "__pycache__" | "dist" | "build") {
        return Ok(());
    }

    let entries: Vec<_> = std::fs::read_dir(current)?
        .filter_map(|e| e.ok())
        .collect();

    // Check if this is a project directory (has .git or relevant files)
    let is_git_repo = entries.iter().any(|e| e.file_name() == ".git");

    let mut exam_file: Option<PathBuf> = None;
    let mut qa_files: Vec<PathBuf> = Vec::new();
    let mut knowledge_files: Vec<PathBuf> = Vec::new();
    let mut subdirs: Vec<PathBuf> = Vec::new();

    for entry in entries {
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().to_lowercase();

        if path.is_file() && name.ends_with(".md") {
            if name.starts_with("exam_") || name == "exam.md" {
                exam_file = Some(path);
            } else if name.starts_with("qa_") || name == "qa.md" {
                qa_files.push(path);
            } else if name.starts_with("knowledge_") || name == "knowledge.md" {
                knowledge_files.push(path);
            }
        } else if path.is_dir() {
            subdirs.push(path);
        }
    }

    // If we found relevant files, register as project
    let has_content = exam_file.is_some() || !qa_files.is_empty() || !knowledge_files.is_empty();

    if has_content || is_git_repo {
        let name = current
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unnamed")
            .to_string();

        projects.push(ScannedProject {
            path: current.to_path_buf(),
            name,
            exam_file,
            qa_files,
            knowledge_files,
            is_git_repo,
        });
    }

    // Recurse into subdirs
    for subdir in subdirs {
        scan_recursive(root, &subdir, projects, depth + 1, max_depth)?;
    }

    Ok(())
}

/// Find exam file in a project directory
pub fn find_exam_file(project_path: &Path) -> Option<PathBuf> {
    if !project_path.is_dir() {
        return None;
    }

    let entries = std::fs::read_dir(project_path).ok()?;

    for entry in entries.filter_map(|e| e.ok()) {
        let name = entry.file_name().to_string_lossy().to_lowercase();
        if name.starts_with("exam_") && name.ends_with(".md") {
            return Some(entry.path());
        }
        if name == "exam.md" {
            return Some(entry.path());
        }
    }

    // Check .claude/exams/ subdirectory
    let claude_exams = project_path.join(".claude").join("exams");
    if claude_exams.is_dir() {
        if let Ok(entries) = std::fs::read_dir(&claude_exams) {
            for entry in entries.filter_map(|e| e.ok()) {
                let name = entry.file_name().to_string_lossy().to_lowercase();
                if name.ends_with(".md") {
                    return Some(entry.path());
                }
            }
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scan_skips_hidden() {
        // Would need temp dir setup for proper test
    }
}
