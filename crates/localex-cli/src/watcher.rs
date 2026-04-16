use std::path::PathBuf;

use anyhow::Result;
use notify::{Config, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use serde::Serialize;
use tokio::sync::broadcast;

#[derive(Clone, Debug, Serialize)]
pub struct FileChangeEvent {
    pub paths: Vec<String>,
    pub kind: String,
}

pub fn start_watcher(
    workspace_root: PathBuf,
    tx: broadcast::Sender<FileChangeEvent>,
) -> Result<()> {
    let root = workspace_root.clone();
    let tx = tx.clone();

    let mut watcher = RecommendedWatcher::new(
        move |res: Result<notify::Event, notify::Error>| {
            let event = match res {
                Ok(e) => e,
                Err(_) => return,
            };

            let kind = match event.kind {
                EventKind::Create(_) => "create",
                EventKind::Modify(_) => "modify",
                EventKind::Remove(_) => "remove",
                _ => return,
            };

            let paths: Vec<String> = event
                .paths
                .iter()
                .filter_map(|p| p.strip_prefix(&root).ok())
                .filter(|p| {
                    let s = p.to_string_lossy();
                    s.ends_with(".md") || s.ends_with(".markdown")
                })
                .map(|p| p.to_string_lossy().to_string())
                .collect();

            if paths.is_empty() {
                return;
            }

            let _ = tx.send(FileChangeEvent {
                paths,
                kind: kind.to_string(),
            });
        },
        Config::default(),
    )?;

    watcher.watch(&workspace_root, RecursiveMode::Recursive)?;

    // Leak watcher to keep it alive for process lifetime.
    // The OS file-watch handles stay registered; callbacks continue firing.
    std::mem::forget(watcher);

    Ok(())
}
