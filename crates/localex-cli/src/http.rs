use std::sync::Arc;

use axum::{
    extract::{Query, State},
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use serde::Deserialize;
use serde_json::{json, Value};
use tower_http::services::{ServeDir, ServeFile};

use crate::markdown;
use crate::{AppConfig, ReaderPreferences};

pub fn app_router(config: AppConfig) -> Router {
    let dist_dir = config.workspace_root.join("web/dist");
    let index_html = dist_dir.join("index.html");
    let state = Arc::new(config);

    let static_files = ServeDir::new(&dist_dir)
        .fallback(ServeFile::new(&index_html));

    Router::new()
        .route("/api/health", get(health))
        .route("/api/reader/defaults", get(reader_defaults))
        .route("/api/files", get(files))
        .route("/api/render", get(render))
        .with_state(state)
        .fallback_service(static_files)
}

async fn health() -> Json<Value> {
    Json(json!({
        "status": "ok",
        "app": "localex"
    }))
}

async fn reader_defaults(State(config): State<Arc<AppConfig>>) -> Json<ReaderPreferences> {
    Json(config.reader.clone())
}

async fn files(State(config): State<Arc<AppConfig>>) -> impl IntoResponse {
    let tree = markdown::scan_workspace(&config.workspace_root);
    Json(tree)
}

#[derive(Deserialize)]
struct RenderQuery {
    path: String,
}

async fn render(
    State(config): State<Arc<AppConfig>>,
    Query(query): Query<RenderQuery>,
) -> impl IntoResponse {
    match markdown::safe_resolve(&config.workspace_root, &query.path) {
        Ok(full_path) => {
            match std::fs::read_to_string(&full_path) {
                Ok(content) => {
                    let doc = markdown::render_markdown(&content);
                    Json(doc).into_response()
                }
                Err(e) => {
                    let err = json!({ "error": format!("failed to read file: {e}") });
                    (axum::http::StatusCode::NOT_FOUND, Json(err)).into_response()
                }
            }
        }
        Err(e) => {
            let err = json!({ "error": format!("invalid path: {e}") });
            (axum::http::StatusCode::BAD_REQUEST, Json(err)).into_response()
        }
    }
}
