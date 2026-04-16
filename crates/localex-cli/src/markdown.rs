use std::path::Path;

use anyhow::{Context, Result};
use comrak::nodes::{AstNode, NodeHeading, NodeValue};
use comrak::{Arena, Options};
use serde::Serialize;

// ── File tree ──

#[derive(Debug, Serialize)]
pub struct FileEntry {
    pub path: String,
    pub name: String,
    pub is_dir: bool,
    pub children: Vec<FileEntry>,
}

pub fn scan_workspace(root: &Path) -> Vec<FileEntry> {
    build_tree(root, root)
}

fn build_tree(root: &Path, prefix: &Path) -> Vec<FileEntry> {
    let mut entries: Vec<FileEntry> = Vec::new();

    let read_dir = match std::fs::read_dir(prefix) {
        Ok(rd) => rd,
        Err(_) => return entries,
    };

    let mut raw: Vec<std::fs::DirEntry> = read_dir.filter_map(|e| e.ok()).collect();
    raw.sort_by(|a, b| {
        let a_dir = a.file_type().map(|t| t.is_dir()).unwrap_or(false);
        let b_dir = b.file_type().map(|t| t.is_dir()).unwrap_or(false);
        b_dir.cmp(&a_dir).then(a.file_name().cmp(&b.file_name()))
    });

    for entry in raw {
        let name = entry.file_name().to_string_lossy().to_string();
        if name.starts_with('.') || name.starts_with('_') || name == "node_modules" || name == "target" {
            continue;
        }
        let full_path = entry.path();
        let rel = full_path.strip_prefix(root).unwrap_or(&full_path);
        let rel_str = rel.to_string_lossy().to_string();

        if entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
            let children = build_tree(root, &full_path);
            if !children.is_empty() {
                entries.push(FileEntry {
                    path: rel_str,
                    name,
                    is_dir: true,
                    children,
                });
            }
        } else if name.ends_with(".md") || name.ends_with(".markdown") {
            entries.push(FileEntry {
                path: rel_str,
                name,
                is_dir: false,
                children: Vec::new(),
            });
        }
    }
    entries
}

// ── Markdown rendering ──

#[derive(Debug, Serialize)]
pub struct RenderedDocument {
    pub html: String,
    pub headings: Vec<Heading>,
}

#[derive(Debug, Serialize, Clone)]
pub struct Heading {
    pub level: u8,
    pub id: String,
    pub text: String,
}

pub fn render_markdown(input: &str) -> RenderedDocument {
    let arena = Arena::new();
    let mut options = Options::default();
    options.extension.table = true;
    options.extension.tasklist = true;
    options.extension.strikethrough = true;
    options.extension.footnotes = true;
    options.extension.autolink = true;
    options.extension.description_lists = true;
    options.extension.front_matter_delimiter = Some("---".into());

    let root = comrak::parse_document(&arena, input, &options);

    let mut headings = Vec::new();
    let mut heading_counter: std::collections::HashMap<String, usize> =
        std::collections::HashMap::new();

    for node in root.descendants() {
        if let NodeValue::Heading(NodeHeading { level, .. }) = &node.data.borrow().value {
            let text = collect_text(node);
            let id = slugify_heading(&text, &mut heading_counter);
            headings.push(Heading {
                level: *level as u8,
                id,
                text,
            });
        }
    }

    let mut html_buf = Vec::new();
    comrak::format_html(root, &options, &mut html_buf).unwrap();
    let html = String::from_utf8(html_buf).unwrap();

    // Inject ids into heading tags
    let html = inject_heading_ids(html, &headings);

    RenderedDocument { html, headings }
}

fn inject_heading_ids(mut html: String, headings: &[Heading]) -> String {
    for heading in headings {
        let old = format!("<h{}>", heading.level);
        let new = format!("<h{} id=\"{}\">", heading.level, heading.id);
        if let Some(pos) = html.find(&old) {
            html = format!("{}{}{}", &html[..pos], new, &html[pos + old.len()..]);
        }
    }
    html
}

fn collect_text<'a>(node: &'a AstNode<'a>) -> String {
    let mut text = String::new();
    for child in node.descendants() {
        if let NodeValue::Text(t) | NodeValue::Code(comrak::nodes::NodeCode { literal: t, .. }) =
            &child.data.borrow().value
        {
            text.push_str(t);
        }
    }
    text
}

fn slugify_heading(text: &str, counter: &mut std::collections::HashMap<String, usize>) -> String {
    let base: String = text
        .to_lowercase()
        .chars()
        .map(|c| {
            if c.is_alphanumeric() {
                c
            } else if c == ' ' || c == '-' || c == '_' {
                '-'
            } else {
                '\0'
            }
        })
        .filter(|c| *c != '\0')
        .collect();
    let base = base.trim_matches('-').to_string();
    let count = counter.entry(base.clone()).or_insert(0);
    *count += 1;
    if *count == 1 {
        base
    } else {
        format!("{}-{}", base, count)
    }
}

// ── Path safety ──

/// Resolve and validate a relative path is within workspace root.
pub fn safe_resolve(workspace_root: &Path, rel: &str) -> Result<std::path::PathBuf> {
    let full = workspace_root.join(rel);
    let canonical_root = workspace_root
        .canonicalize()
        .context("workspace root does not exist")?;
    let canonical_target = full
        .canonicalize()
        .context("requested file does not exist")?;
    if !canonical_target.starts_with(&canonical_root) {
        anyhow::bail!("path escapes workspace root");
    }
    Ok(canonical_target)
}
