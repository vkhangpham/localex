use std::collections::HashMap;
use std::path::Path;

use comrak::nodes::NodeValue;
use comrak::{Arena, Options};
use serde::Serialize;
use walkdir::WalkDir;

#[derive(Debug, Clone, Serialize)]
pub struct Backlink {
    pub source_path: String,
    pub link_text: String,
    pub excerpt: String,
}

pub struct BacklinkIndex {
    incoming: HashMap<String, Vec<Backlink>>,
}

impl BacklinkIndex {
    pub fn get(&self, path: &str) -> &[Backlink] {
        self.incoming.get(path).map(|v| v.as_slice()).unwrap_or(&[])
    }
}

pub fn build_index(workspace_root: &Path) -> BacklinkIndex {
    let mut incoming: HashMap<String, Vec<Backlink>> = HashMap::new();

    for entry in WalkDir::new(workspace_root)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        if !path
            .extension()
            .is_some_and(|ext| ext == "md" || ext == "markdown")
        {
            continue;
        }

        let rel = match path.strip_prefix(workspace_root) {
            Ok(r) => r.to_string_lossy().to_string(),
            Err(_) => continue,
        };

        let file_name = path.file_name().unwrap_or_default().to_string_lossy().to_string();
        if file_name.starts_with('.') || file_name.starts_with('_') {
            continue;
        }

        let Ok(content) = std::fs::read_to_string(path) else {
            continue;
        };

        let links = extract_links(&content);
        let dir = path.parent().unwrap_or(workspace_root);

        for (link_text, href) in links {
            if !href.ends_with(".md") && !href.ends_with(".markdown") {
                continue;
            }

            let target_resolved = dir.join(&href);
            let target_rel = match target_resolved.strip_prefix(workspace_root) {
                Ok(r) => r.to_string_lossy().to_string(),
                Err(_) => continue,
            };

            let excerpt = extract_excerpt(&content, &href);

            incoming
                .entry(target_rel)
                .or_default()
                .push(Backlink {
                    source_path: rel.clone(),
                    link_text,
                    excerpt,
                });
        }
    }

    BacklinkIndex { incoming }
}

fn extract_links(content: &str) -> Vec<(String, String)> {
    let arena = Arena::new();
    let options = Options::default();
    let root = comrak::parse_document(&arena, content, &options);

    let mut links = Vec::new();
    for node in root.descendants() {
        let url;
        {
            let data = node.data.borrow();
            if let NodeValue::Link(link) = &data.value {
                url = link.url.clone();
            } else {
                continue;
            }
        }

        let text: String = node
            .descendants()
            .filter_map(|n| {
                let data = n.data.borrow();
                if let NodeValue::Text(t) = &data.value {
                    Some(t.clone())
                } else {
                    None
                }
            })
            .collect();
        links.push((text, url));
    }
    links
}

fn extract_excerpt(content: &str, href: &str) -> String {
    for line in content.lines() {
        if line.contains(href) {
            let trimmed = line.trim();
            if trimmed.len() > 120 {
                let start = trimmed.find(href).unwrap_or(0).saturating_sub(40);
                let end = (start + 120).min(trimmed.len());
                let mut excerpt = trimmed[start..end].to_string();
                if start > 0 {
                    excerpt = format!("...{excerpt}");
                }
                if end < trimmed.len() {
                    excerpt = format!("{excerpt}...");
                }
                return excerpt;
            }
            return trimmed.to_string();
        }
    }
    String::new()
}
