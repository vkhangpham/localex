pub mod backlinks;
pub mod config;
pub mod db;
pub mod highlights;
pub mod http;
pub mod markdown;
pub mod themes;

use std::sync::{Arc, RwLock};

use crate::backlinks::BacklinkIndex;

pub use config::{AppConfig, LayoutMode, ReaderPreferences};
pub use db::Db;
pub use http::app_router;

pub struct AppState {
    pub config: AppConfig,
    pub db: Db,
    pub backlinks: Arc<RwLock<BacklinkIndex>>,
}
