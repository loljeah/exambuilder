use std::path::PathBuf;

use anyhow::Result;
use console::style;

pub async fn cmd_export_bookmarks(output: Option<PathBuf>) -> Result<()> {
    let output_path = output.unwrap_or_else(|| crate::kgate_dir().join("bookmarks").join("bookmarks.json"));

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
