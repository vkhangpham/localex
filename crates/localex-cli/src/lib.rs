pub mod backlinks;
pub mod config;
pub mod db;
pub mod highlights;
pub mod http;
pub mod markdown;
pub mod themes;
pub mod watcher;

use std::sync::{Arc, RwLock};

use crate::backlinks::BacklinkIndex;
use crate::watcher::FileChangeEvent;

pub use config::{AppConfig, LayoutMode, ReaderPreferences};
pub use db::Db;
pub use http::app_router;

pub struct AppState {
    pub config: AppConfig,
    pub db: Db,
    pub backlinks: Arc<RwLock<BacklinkIndex>>,
    pub watch_tx: tokio::sync::broadcast::Sender<FileChangeEvent>,
}
