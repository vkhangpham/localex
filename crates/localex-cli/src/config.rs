use std::path::PathBuf;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum LayoutMode {
    OneColumn,
    TwoColumn,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct ReaderPreferences {
    pub target_words_per_line: u8,
    pub line_height: f32,
    pub font_size_px: u8,
    pub font_family: String,
    pub layout_mode: LayoutMode,
}

impl Default for ReaderPreferences {
    fn default() -> Self {
        Self {
            target_words_per_line: 12,
            line_height: 1.75,
            font_size_px: 18,
            font_family: "Inter".to_string(),
            layout_mode: LayoutMode::OneColumn,
        }
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct AppConfig {
    pub workspace_root: PathBuf,
    pub host: String,
    pub port: u16,
    pub data_dir: PathBuf,
    pub reader: ReaderPreferences,
}

impl AppConfig {
    pub fn for_workspace(workspace_root: impl Into<PathBuf>) -> Result<Self> {
        let data_dir = dirs::home_dir()
            .map(|path| path.join(".localex"))
            .context("failed to resolve home directory for ~/.localex")?;

        Ok(Self {
            workspace_root: workspace_root.into(),
            host: "127.0.0.1".to_string(),
            port: 3862,
            data_dir,
            reader: ReaderPreferences::default(),
        })
    }

    pub fn with_server(mut self, host: impl Into<String>, port: u16) -> Self {
        self.host = host.into();
        self.port = port;
        self
    }
}
