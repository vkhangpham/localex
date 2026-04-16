use std::path::Path;

use anyhow::{Context, Result};
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct ThemeMeta {
    pub name: String,
    pub source: String,
}

const BUILT_IN_THEMES: &[&str] = &["light", "dark", "sepia"];

pub fn list_themes(data_dir: &Path) -> Vec<ThemeMeta> {
    let mut themes: Vec<ThemeMeta> = BUILT_IN_THEMES
        .iter()
        .map(|&name| ThemeMeta {
            name: name.to_string(),
            source: "built-in".to_string(),
        })
        .collect();

    let custom_dir = data_dir.join("themes");
    if let Ok(entries) = std::fs::read_dir(&custom_dir) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.ends_with(".css") {
                let theme_name = name.trim_end_matches(".css").to_string();
                themes.push(ThemeMeta {
                    name: theme_name,
                    source: "custom".to_string(),
                });
            }
        }
    }

    themes
}

pub fn load_theme_css(data_dir: &Path, name: &str) -> Result<String> {
    match name {
        "light" => Ok(String::new()),
        "dark" => Ok(dark_theme_css().to_string()),
        "sepia" => Ok(sepia_theme_css().to_string()),
        _ => {
            let path = data_dir.join("themes").join(format!("{name}.css"));
            std::fs::read_to_string(&path)
                .with_context(|| format!("custom theme '{name}' not found"))
        }
    }
}

fn dark_theme_css() -> &'static str {
    r#"
[data-theme="dark"] {
  color-scheme: dark;
  --page-bg: #1a1816;
  --surface: rgba(34, 30, 26, 0.92);
  --surface-strong: #2a2622;
  --border: rgba(200, 190, 175, 0.12);
  --muted: rgba(190, 180, 165, 0.72);
  --heading: rgba(240, 235, 225, 0.94);
  --accent: #6ba3d6;
  --shadow: 0 24px 60px rgba(0, 0, 0, 0.3);
}
"#
}

fn sepia_theme_css() -> &'static str {
    r#"
[data-theme="sepia"] {
  --page-bg: #f0e6d3;
  --surface: rgba(245, 238, 224, 0.9);
  --surface-strong: #f5eed9;
  --border: rgba(120, 90, 50, 0.15);
  --muted: rgba(100, 75, 45, 0.65);
  --heading: rgba(50, 35, 15, 0.94);
  --accent: #8b6914;
  --shadow: 0 24px 60px rgba(80, 60, 30, 0.1);
}
"#
}
