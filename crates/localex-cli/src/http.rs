use std::sync::Arc;
use std::time::Duration;

use axum::{
    extract::{Path as AxumPath, Query, State},
    response::IntoResponse,
    response::sse::{Event, KeepAlive, Sse},
    routing::{delete, get, post},
    Json, Router,
};
use serde::Deserialize;
use serde_json::{json, Value};
use tokio_stream::wrappers::BroadcastStream;
use tokio_stream::StreamExt;
use tower_http::services::{ServeDir, ServeFile};

use crate::db;
use crate::highlights::{self, CreateHighlight, CreateNote};
use crate::markdown;
use crate::themes;
use crate::{AppState, ReaderPreferences};

pub fn app_router(state: AppState) -> Router {
    let config = &state.config;
    // Resolve frontend assets: prefer exe-adjacent, then workspace-relative
    let exe_dist = std::env::current_exe()
        .ok()
        .and_then(|exe| exe.parent().map(|p| p.join("../lib/localex/web/dist")))
        .filter(|p| p.exists());
    let dist_dir = exe_dist.unwrap_or_else(|| config.workspace_root.join("web/dist"));
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
        // live reload
        .route("/api/events", get(sse_events))
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
    let root = state.config.workspace_root.clone();
    let tree = tokio::task::spawn_blocking(move || markdown::scan_workspace(&root)).await;
    match tree {
        Ok(t) => Json(t).into_response(),
        Err(e) => {
            let err = json!({ "error": format!("{e}") });
            (axum::http::StatusCode::INTERNAL_SERVER_ERROR, Json(err)).into_response()
        }
    }
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
            // Extension check — only render markdown files
            let ext = full_path.extension().and_then(|e| e.to_str()).unwrap_or("");
            if ext != "md" && ext != "markdown" {
                let err = json!({ "error": "not a markdown file" });
                return (axum::http::StatusCode::BAD_REQUEST, Json(err)).into_response();
            }

            // Size check — cap at 4MB
            match std::fs::metadata(&full_path) {
                Ok(meta) if meta.len() > 4 * 1024 * 1024 => {
                    let err = json!({ "error": "file too large to render" });
                    return (axum::http::StatusCode::PAYLOAD_TOO_LARGE, Json(err)).into_response();
                }
                Err(e) => {
                    let err = json!({ "error": format!("failed to stat file: {e}") });
                    return (axum::http::StatusCode::NOT_FOUND, Json(err)).into_response();
                }
                _ => {}
            }

            let mtime = std::fs::metadata(&full_path).ok().and_then(|m| m.modified().ok());

            // Check render cache
            if let Some(mt) = mtime {
                let cache = state.render_cache.read().unwrap();
                if let Some((cached_mtime, cached_doc)) = cache.get(&full_path) {
                    if *cached_mtime == mt {
                        return Json(cached_doc.clone()).into_response();
                    }
                }
            }

            match std::fs::read_to_string(&full_path) {
                Ok(content) => {
                    let doc = markdown::render_markdown(&content);
                    // Store in cache
                    if let Some(mt) = mtime {
                        state.render_cache.write().unwrap().insert(full_path, (mt, doc.clone()));
                    }
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
    let data_dir = state.config.data_dir.clone();
    let result = tokio::task::spawn_blocking(move || themes::list_themes(&data_dir)).await;
    match result {
        Ok(themes) => Json(json!({ "themes": themes })),
        Err(e) => Json(json!({ "error": format!("{e}") })),
    }
}

async fn theme_css(
    State(state): State<Arc<AppState>>,
    AxumPath(name): AxumPath<String>,
) -> impl IntoResponse {
    let data_dir = state.config.data_dir.clone();
    let result = tokio::task::spawn_blocking(move || themes::load_theme_css(&data_dir, &name)).await;
    match result {
        Ok(Ok(css)) => ([("content-type", "text/css")], css).into_response(),
        Ok(Err(e)) => {
            let err = json!({ "error": format!("{e}") });
            (axum::http::StatusCode::NOT_FOUND, Json(err)).into_response()
        }
        Err(e) => {
            let err = json!({ "error": format!("{e}") });
            (axum::http::StatusCode::INTERNAL_SERVER_ERROR, Json(err)).into_response()
        }
    }
}

// ── Preferences ──

async fn get_preference(
    State(state): State<Arc<AppState>>,
    AxumPath(key): AxumPath<String>,
) -> Json<Value> {
    let db = state.db.clone();
    let key_clone = key.clone();
    let result = tokio::task::spawn_blocking(move || {
        let conn = db.lock().unwrap();
        db::get_preference(&conn, &key_clone)
    }).await;
    match result {
        Ok(Some(v)) => Json(json!({ "key": key, "value": v })),
        _ => Json(json!({ "key": key, "value": null })),
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
    let db = state.db.clone();
    let result = tokio::task::spawn_blocking(move || {
        let conn = db.lock().unwrap();
        db::set_preference(&conn, &body.key, &body.value)
    }).await;
    match result {
        Ok(Ok(())) => Json(json!({ "ok": true })),
        _ => Json(json!({ "error": "failed to set preference" })),
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
    let db = state.db.clone();
    let result = tokio::task::spawn_blocking(move || {
        let conn = db.lock().unwrap();
        highlights::list_highlights(&conn, &query.path)
    }).await;
    match result {
        Ok(Ok(h)) => Json(json!({ "highlights": h })),
        _ => Json(json!({ "error": "failed to list highlights" })),
    }
}

async fn create_highlight(
    State(state): State<Arc<AppState>>,
    Json(body): Json<CreateHighlight>,
) -> impl IntoResponse {
    let db = state.db.clone();
    let result = tokio::task::spawn_blocking(move || {
        let conn = db.lock().unwrap();
        highlights::create_highlight(&conn, &body)
    }).await;
    match result {
        Ok(Ok(h)) => Json(h).into_response(),
        _ => {
            let err = json!({ "error": "failed to create highlight" });
            (axum::http::StatusCode::BAD_REQUEST, Json(err)).into_response()
        }
    }
}

async fn delete_highlight(
    State(state): State<Arc<AppState>>,
    AxumPath(id): AxumPath<i64>,
) -> impl IntoResponse {
    let db = state.db.clone();
    let result = tokio::task::spawn_blocking(move || {
        let conn = db.lock().unwrap();
        highlights::delete_highlight(&conn, id)
    }).await;
    match result {
        Ok(Ok(true)) => Json(json!({ "ok": true })).into_response(),
        Ok(Ok(false)) => {
            let err = json!({ "error": "not found" });
            (axum::http::StatusCode::NOT_FOUND, Json(err)).into_response()
        }
        _ => {
            let err = json!({ "error": "failed to delete highlight" });
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
    let db = state.db.clone();
    let result = tokio::task::spawn_blocking(move || {
        let conn = db.lock().unwrap();
        highlights::list_notes(&conn, &query.path)
    }).await;
    match result {
        Ok(Ok(n)) => Json(json!({ "notes": n })),
        _ => Json(json!({ "error": "failed to list notes" })),
    }
}

async fn create_note(
    State(state): State<Arc<AppState>>,
    Json(body): Json<CreateNote>,
) -> impl IntoResponse {
    let db = state.db.clone();
    let result = tokio::task::spawn_blocking(move || {
        let conn = db.lock().unwrap();
        highlights::create_note(&conn, &body)
    }).await;
    match result {
        Ok(Ok(n)) => Json(n).into_response(),
        _ => {
            let err = json!({ "error": "failed to create note" });
            (axum::http::StatusCode::BAD_REQUEST, Json(err)).into_response()
        }
    }
}

async fn delete_note(
    State(state): State<Arc<AppState>>,
    AxumPath(id): AxumPath<i64>,
) -> impl IntoResponse {
    let db = state.db.clone();
    let result = tokio::task::spawn_blocking(move || {
        let conn = db.lock().unwrap();
        highlights::delete_note(&conn, id)
    }).await;
    match result {
        Ok(Ok(true)) => Json(json!({ "ok": true })).into_response(),
        Ok(Ok(false)) => {
            let err = json!({ "error": "not found" });
            (axum::http::StatusCode::NOT_FOUND, Json(err)).into_response()
        }
        _ => {
            let err = json!({ "error": "failed to delete note" });
            (axum::http::StatusCode::INTERNAL_SERVER_ERROR, Json(err)).into_response()
        }
    }
}

// ── Live Reload (SSE) ──

async fn sse_events(
    State(state): State<Arc<AppState>>,
) -> Sse<impl tokio_stream::Stream<Item = Result<Event, std::convert::Infallible>>> {
    let rx = state.watch_tx.subscribe();
    let stream = BroadcastStream::new(rx).filter_map(|result| {
        let event = result.ok()?;
        let data = serde_json::to_string(&event).ok()?;
        Some(Ok(Event::default().data(data)))
    });

    Sse::new(stream).keep_alive(
        KeepAlive::new().interval(Duration::from_secs(15)),
    )
}
