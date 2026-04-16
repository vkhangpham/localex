pub mod backlinks;
pub mod config;
pub mod db;
pub mod highlights;
pub mod http;
pub mod markdown;
pub mod themes;
pub mod watcher;

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};
use std::time::SystemTime;

use crate::backlinks::BacklinkIndex;
use crate::markdown::RenderedDocument;
use crate::watcher::FileChangeEvent;

pub use config::{AppConfig, LayoutMode, ReaderPreferences};
pub use db::Db;
pub use http::app_router;

pub struct AppState {
    pub config: AppConfig,
    pub db: Db,
    pub backlinks: Arc<RwLock<BacklinkIndex>>,
    pub watch_tx: tokio::sync::broadcast::Sender<FileChangeEvent>,
    pub render_cache: Arc<RwLock<HashMap<PathBuf, (SystemTime, RenderedDocument)>>>,
}
