use std::sync::Arc;

use axum::{
    extract::{Path as AxumPath, Query, State},
    response::IntoResponse,
    routing::{delete, get, post},
    Json, Router,
};
use serde::Deserialize;
use serde_json::{json, Value};
use tower_http::services::{ServeDir, ServeFile};

use crate::db;
use crate::highlights::{self, CreateHighlight, CreateNote};
use crate::markdown;
use crate::themes;
use crate::{AppState, ReaderPreferences};

pub fn app_router(state: AppState) -> Router {
    let config = &state.config;
    let dist_dir = config.workspace_root.join("web/dist");
    let index_html = dist_dir.join("index.html");
    let shared = Arc::new(state);

    let static_files = ServeDir::new(&dist_dir)
        .fallback(ServeFile::new(&index_html));

    Router::new()
        // existing
        .route("/api/health", get(health))
        .route("/api/reader/defaults", get(reader_defaults))
        .route("/api/files", get(files))
        .route("/api/render", get(render))
        // themes
        .route("/api/themes", get(list_themes))
        .route("/api/themes/{name}/css", get(theme_css))
        // preferences
        .route("/api/preferences/{key}", get(get_preference))
        .route("/api/preferences", post(set_preference))
        // backlinks
        .route("/api/backlinks", get(backlinks))
        // highlights
        .route("/api/highlights", get(list_highlights))
        .route("/api/highlights", post(create_highlight))
        .route("/api/highlights/{id}", delete(delete_highlight))
        // notes
        .route("/api/notes", get(list_notes))
        .route("/api/notes", post(create_note))
        .route("/api/notes/{id}", delete(delete_note))
        .with_state(shared)
        .fallback_service(static_files)
}

// ── Existing handlers ──

async fn health() -> Json<Value> {
    Json(json!({ "status": "ok", "app": "localex" }))
}

async fn reader_defaults(State(state): State<Arc<AppState>>) -> Json<ReaderPreferences> {
    Json(state.config.reader.clone())
}

async fn files(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let tree = markdown::scan_workspace(&state.config.workspace_root);
    Json(tree)
}

#[derive(Deserialize)]
struct RenderQuery {
    path: String,
}

async fn render(
    State(state): State<Arc<AppState>>,
    Query(query): Query<RenderQuery>,
) -> impl IntoResponse {
    match markdown::safe_resolve(&state.config.workspace_root, &query.path) {
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

// ── Themes ──

async fn list_themes(State(state): State<Arc<AppState>>) -> Json<Value> {
    let themes = themes::list_themes(&state.config.data_dir);
    Json(json!({ "themes": themes }))
}

async fn theme_css(
    State(state): State<Arc<AppState>>,
    AxumPath(name): AxumPath<String>,
) -> impl IntoResponse {
    match themes::load_theme_css(&state.config.data_dir, &name) {
        Ok(css) => {
            ([("content-type", "text/css")], css).into_response()
        }
        Err(e) => {
            let err = json!({ "error": format!("{e}") });
            (axum::http::StatusCode::NOT_FOUND, Json(err)).into_response()
        }
    }
}

// ── Preferences ──

async fn get_preference(
    State(state): State<Arc<AppState>>,
    AxumPath(key): AxumPath<String>,
) -> Json<Value> {
    let conn = state.db.lock().unwrap();
    let value = db::get_preference(&conn, &key);
    match value {
        Some(v) => Json(json!({ "key": key, "value": v })),
        None => Json(json!({ "key": key, "value": null })),
    }
}

#[derive(Deserialize)]
struct SetPrefRequest {
    key: String,
    value: String,
}

async fn set_preference(
    State(state): State<Arc<AppState>>,
    Json(body): Json<SetPrefRequest>,
) -> Json<Value> {
    let conn = state.db.lock().unwrap();
    match db::set_preference(&conn, &body.key, &body.value) {
        Ok(()) => Json(json!({ "ok": true })),
        Err(e) => Json(json!({ "error": format!("{e}") })),
    }
}

// ── Backlinks ──

#[derive(Deserialize)]
struct BacklinksQuery {
    path: String,
}

async fn backlinks(
    State(state): State<Arc<AppState>>,
    Query(query): Query<BacklinksQuery>,
) -> Json<Value> {
    let index = state.backlinks.read().unwrap();
    let links = index.get(&query.path).to_vec();
    Json(json!({ "backlinks": links }))
}

// ── Highlights ──

#[derive(Deserialize)]
struct HighlightsQuery {
    path: String,
}

async fn list_highlights(
    State(state): State<Arc<AppState>>,
    Query(query): Query<HighlightsQuery>,
) -> Json<Value> {
    let conn = state.db.lock().unwrap();
    match highlights::list_highlights(&conn, &query.path) {
        Ok(h) => Json(json!({ "highlights": h })),
        Err(e) => Json(json!({ "error": format!("{e}") })),
    }
}

async fn create_highlight(
    State(state): State<Arc<AppState>>,
    Json(body): Json<CreateHighlight>,
) -> impl IntoResponse {
    let conn = state.db.lock().unwrap();
    match highlights::create_highlight(&conn, &body) {
        Ok(h) => Json(h).into_response(),
        Err(e) => {
            let err = json!({ "error": format!("{e}") });
            (axum::http::StatusCode::BAD_REQUEST, Json(err)).into_response()
        }
    }
}

async fn delete_highlight(
    State(state): State<Arc<AppState>>,
    AxumPath(id): AxumPath<i64>,
) -> impl IntoResponse {
    let conn = state.db.lock().unwrap();
    match highlights::delete_highlight(&conn, id) {
        Ok(true) => Json(json!({ "ok": true })).into_response(),
        Ok(false) => {
            let err = json!({ "error": "not found" });
            (axum::http::StatusCode::NOT_FOUND, Json(err)).into_response()
        }
        Err(e) => {
            let err = json!({ "error": format!("{e}") });
            (axum::http::StatusCode::INTERNAL_SERVER_ERROR, Json(err)).into_response()
        }
    }
}

// ── Notes ──

#[derive(Deserialize)]
struct NotesQuery {
    path: String,
}

async fn list_notes(
    State(state): State<Arc<AppState>>,
    Query(query): Query<NotesQuery>,
) -> Json<Value> {
    let conn = state.db.lock().unwrap();
    match highlights::list_notes(&conn, &query.path) {
        Ok(n) => Json(json!({ "notes": n })),
        Err(e) => Json(json!({ "error": format!("{e}") })),
    }
}

async fn create_note(
    State(state): State<Arc<AppState>>,
    Json(body): Json<CreateNote>,
) -> impl IntoResponse {
    let conn = state.db.lock().unwrap();
    match highlights::create_note(&conn, &body) {
        Ok(n) => Json(n).into_response(),
        Err(e) => {
            let err = json!({ "error": format!("{e}") });
            (axum::http::StatusCode::BAD_REQUEST, Json(err)).into_response()
        }
    }
}

async fn delete_note(
    State(state): State<Arc<AppState>>,
    AxumPath(id): AxumPath<i64>,
) -> impl IntoResponse {
    let conn = state.db.lock().unwrap();
    match highlights::delete_note(&conn, id) {
        Ok(true) => Json(json!({ "ok": true })).into_response(),
        Ok(false) => {
            let err = json!({ "error": "not found" });
            (axum::http::StatusCode::NOT_FOUND, Json(err)).into_response()
        }
        Err(e) => {
            let err = json!({ "error": format!("{e}") });
            (axum::http::StatusCode::INTERNAL_SERVER_ERROR, Json(err)).into_response()
        }
    }
}
