use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use http_body_util::BodyExt;
use localex_cli::{app_router, AppState, AppConfig, LayoutMode};
use localex_cli::backlinks;
use std::sync::RwLock;
use tower::ServiceExt;

async fn body_string(body: Body) -> String {
    let bytes = body.collect().await.unwrap().to_bytes();
    std::str::from_utf8(&bytes).unwrap().to_string()
}

fn make_state() -> AppState {
    let config = AppConfig::for_workspace("/tmp/localex-smoke-test").unwrap();
    let db = localex_cli::db::init_db(&config.data_dir).unwrap();
    let backlink_index = backlinks::build_index(&config.workspace_root);
    let (watch_tx, _) = tokio::sync::broadcast::channel(16);
    AppState {
        config,
        db,
        backlinks: std::sync::Arc::new(RwLock::new(backlink_index)),
        watch_tx,
        render_cache: std::sync::Arc::new(std::sync::RwLock::new(std::collections::HashMap::new())),
    }
}

fn make_state_for_dir(tmp: &std::path::Path) -> AppState {
    let config = AppConfig::for_workspace(tmp).unwrap();
    let db = localex_cli::db::init_db(&config.data_dir).unwrap();
    let backlink_index = backlinks::build_index(&config.workspace_root);
    let (watch_tx, _) = tokio::sync::broadcast::channel(16);
    AppState {
        config,
        db,
        backlinks: std::sync::Arc::new(RwLock::new(backlink_index)),
        watch_tx,
        render_cache: std::sync::Arc::new(std::sync::RwLock::new(std::collections::HashMap::new())),
    }
}

// ── Health ──

#[tokio::test]
async fn health_ok() {
    let app = app_router(make_state());
    let resp = app
        .oneshot(Request::builder().uri("/api/health").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = body_string(resp.into_body()).await;
    assert!(body.contains("\"status\":\"ok\""));
}

// ── Reader defaults ──

#[tokio::test]
async fn reader_defaults() {
    let app = app_router(make_state());
    let resp = app
        .oneshot(Request::builder().uri("/api/reader/defaults").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = body_string(resp.into_body()).await;
    assert!(body.contains("\"target_words_per_line\":12"));
    assert!(body.contains("\"font_family\":\"Inter\""));
    assert!(body.contains("\"layout_mode\":\"one_column\""));
}

// ── File tree ──

#[tokio::test]
async fn files_returns_tree() {
    let app = app_router(make_state());
    let resp = app
        .oneshot(Request::builder().uri("/api/files").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = body_string(resp.into_body()).await;
    let v: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert!(v.is_array());
}

// ── Markdown rendering ──

#[tokio::test]
async fn render_with_real_md_file() {
    let tmp = tempfile::tempdir().unwrap();
    let md_content = r#"# Hello World

Some **bold** text and a [link](other.md).

## Section Two

A paragraph with `inline code`.

```
fn main() {
    println!("hello");
}
```

This has a footnote[^1].

[^1]: Footnote text here.
"#;
    std::fs::write(tmp.path().join("test.md"), md_content).unwrap();

    let app = app_router(make_state_for_dir(tmp.path()));

    let resp = app
        .oneshot(
            Request::builder()
                .uri("/api/render?path=test.md")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
    let body = body_string(resp.into_body()).await;
    let doc: serde_json::Value = serde_json::from_str(&body).unwrap();

    let html = doc["html"].as_str().unwrap();
    assert!(html.contains("<h1"), "should have h1");
    assert!(html.contains("<h2"), "should have h2");
    assert!(html.contains("<strong>bold</strong>"), "should render bold");
    assert!(html.contains("<code>inline code</code>"), "should render inline code");
    assert!(html.contains("footnote"), "should contain footnote content");

    let headings = doc["headings"].as_array().unwrap();
    assert!(headings.len() >= 2, "should extract at least 2 headings");
    assert_eq!(headings[0]["text"], "Hello World");
    assert_eq!(headings[0]["level"], 1);
    assert_eq!(headings[1]["text"], "Section Two");
}

#[tokio::test]
async fn render_rejects_path_traversal() {
    let app = app_router(make_state());
    let resp = app
        .oneshot(
            Request::builder()
                .uri("/api/render?path=../../../etc/passwd")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_ne!(resp.status(), StatusCode::OK);
}

// ── Themes ──

#[tokio::test]
async fn themes_list_includes_builtins() {
    let app = app_router(make_state());
    let resp = app
        .oneshot(Request::builder().uri("/api/themes").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = body_string(resp.into_body()).await;
    let v: serde_json::Value = serde_json::from_str(&body).unwrap();
    let names: Vec<&str> = v["themes"].as_array().unwrap()
        .iter()
        .map(|t| t["name"].as_str().unwrap())
        .collect();
    assert!(names.contains(&"light"));
    assert!(names.contains(&"dark"));
    assert!(names.contains(&"sepia"));
}

#[tokio::test]
async fn theme_css_dark_returns_styles() {
    let app = app_router(make_state());
    let resp = app
        .oneshot(
            Request::builder()
                .uri("/api/themes/dark/css")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = body_string(resp.into_body()).await;
    assert!(body.contains("--page-bg"));
}

#[tokio::test]
async fn theme_css_unknown_404() {
    let app = app_router(make_state());
    let resp = app
        .oneshot(
            Request::builder()
                .uri("/api/themes/nonexistent/css")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

// ── Preferences ──

#[tokio::test]
async fn preferences_round_trip() {
    let state = make_state();
    let app = app_router(state);

    let set_body = r#"{"key":"theme","value":"dark"}"#;
    let resp = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/preferences")
                .header("content-type", "application/json")
                .body(Body::from(set_body))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = body_string(resp.into_body()).await;
    assert!(body.contains("\"ok\":true"));

    // Re-create app with same state to verify persistence
    let state2 = make_state();
    let app2 = app_router(state2);
    let resp = app2
        .oneshot(
            Request::builder()
                .uri("/api/preferences/theme")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let body = body_string(resp.into_body()).await;
    // The preference was stored in /tmp localex-smoke-test db
    assert!(body.contains("dark"), "preference should persist: {body}");
}

// ── Highlights CRUD ──

#[tokio::test]
async fn highlights_create_list_delete() {
    let state = make_state();
    let app = app_router(state);

    let create_body = r#"{"document_path":"notes/test.md","quote_text":"important passage","color":"yellow"}"#;
    let resp = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/highlights")
                .header("content-type", "application/json")
                .body(Body::from(create_body))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = body_string(resp.into_body()).await;
    let hl: serde_json::Value = serde_json::from_str(&body).unwrap();
    let id = hl["id"].as_i64().unwrap();
    assert_eq!(hl["quote_text"], "important passage");
    assert_eq!(hl["color"], "yellow");

    // List via same state
    let state = make_state();
    let app = app_router(state);
    let resp = app
        .oneshot(
            Request::builder()
                .uri("/api/highlights?path=notes/test.md")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = body_string(resp.into_body()).await;
    assert!(body.contains("important passage"));

    // Delete
    let state = make_state();
    let app = app_router(state);
    let resp = app
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(&format!("/api/highlights/{id}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
}

#[tokio::test]
async fn delete_nonexistent_highlight_404() {
    let app = app_router(make_state());
    let resp = app
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri("/api/highlights/999999")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

// ── Notes CRUD ──

#[tokio::test]
async fn notes_create_list_delete() {
    let state = make_state();
    let app = app_router(state);

    let create_body = r#"{"document_path":"notes/test.md","body":"My thought on this section"}"#;
    let resp = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/notes")
                .header("content-type", "application/json")
                .body(Body::from(create_body))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = body_string(resp.into_body()).await;
    let note: serde_json::Value = serde_json::from_str(&body).unwrap();
    let id = note["id"].as_i64().unwrap();
    assert_eq!(note["body"], "My thought on this section");

    // List
    let state = make_state();
    let app = app_router(state);
    let resp = app
        .oneshot(
            Request::builder()
                .uri("/api/notes?path=notes/test.md")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = body_string(resp.into_body()).await;
    assert!(body.contains("My thought on this section"));

    // Delete
    let state = make_state();
    let app = app_router(state);
    let resp = app
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(&format!("/api/notes/{id}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
}

#[tokio::test]
async fn delete_nonexistent_note_404() {
    let app = app_router(make_state());
    let resp = app
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri("/api/notes/999999")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

// ── Backlinks ──

#[tokio::test]
async fn backlinks_from_cross_linked_files() {
    let tmp = tempfile::tempdir().unwrap();
    std::fs::write(
        tmp.path().join("alpha.md"),
        "See [beta doc](beta.md) for details.\n",
    )
    .unwrap();
    std::fs::write(
        tmp.path().join("beta.md"),
        "Also check [alpha](alpha.md).\n",
    )
    .unwrap();

    let app = app_router(make_state_for_dir(tmp.path()));

    let resp = app
        .oneshot(
            Request::builder()
                .uri("/api/backlinks?path=beta.md")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = body_string(resp.into_body()).await;
    let v: serde_json::Value = serde_json::from_str(&body).unwrap();
    let links = v["backlinks"].as_array().unwrap();
    assert!(!links.is_empty(), "beta.md should have backlink from alpha.md");
    assert_eq!(links[0]["source_path"], "alpha.md");
    assert_eq!(links[0]["link_text"], "beta doc");
}

#[tokio::test]
async fn backlinks_empty_for_unlinked_file() {
    let tmp = tempfile::tempdir().unwrap();
    std::fs::write(tmp.path().join("solo.md"), "No links here.\n").unwrap();

    let app = app_router(make_state_for_dir(tmp.path()));

    let resp = app
        .oneshot(
            Request::builder()
                .uri("/api/backlinks?path=solo.md")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let body = body_string(resp.into_body()).await;
    let v: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert!(v["backlinks"].as_array().unwrap().is_empty());
}

// ── Config ──

#[test]
fn config_defaults_match_reading_prd() {
    let config = AppConfig::for_workspace("/tmp/localex-smoke-test").expect("config should build");
    assert_eq!(config.host, "127.0.0.1");
    assert_eq!(config.port, 3862);
    assert!(config.data_dir.ends_with(".localex"));
    assert_eq!(config.reader.target_words_per_line, 12);
    assert!((config.reader.line_height - 1.75).abs() < 0.001);
    assert_eq!(config.reader.font_size_px, 18);
    assert_eq!(config.reader.font_family, "Inter");
    assert_eq!(config.reader.layout_mode, LayoutMode::OneColumn);
}

// ── Markdown unit tests ──

#[test]
fn render_markdown_syntax_highlight_in_code_blocks() {
    let md = "```rust\nfn main() {}\n```\n";
    let doc = localex_cli::markdown::render_markdown(md);
    assert!(doc.html.contains("<pre"), "should wrap code in <pre>");
    assert!(!doc.html.contains("```"), "should not contain raw backticks");
}

#[test]
fn render_markdown_generates_heading_ids() {
    let md = "# First\n## Second Section\n";
    let doc = localex_cli::markdown::render_markdown(md);
    assert!(doc.html.contains("id=\"first\""));
    assert!(doc.html.contains("id=\"second-section\""));
    assert_eq!(doc.headings.len(), 2);
}

#[test]
fn render_markdown_tables() {
    let md = "| A | B |\n|---|---|\n| 1 | 2 |\n";
    let doc = localex_cli::markdown::render_markdown(md);
    assert!(doc.html.contains("<table>"));
    assert!(doc.html.contains("<td>"));
}

#[test]
fn render_markdown_duplicate_heading_slugs() {
    let md = "# Intro\n## Details\n# Intro\n";
    let doc = localex_cli::markdown::render_markdown(md);
    assert!(doc.html.contains("id=\"intro\""));
    assert!(doc.html.contains("id=\"intro-2\""));
}

#[test]
fn scan_workspace_filters_dotfiles_and_underscore_dirs() {
    let tmp = tempfile::tempdir().unwrap();
    std::fs::write(tmp.path().join("good.md"), "# Good").unwrap();
    std::fs::write(tmp.path().join(".hidden.md"), "# Hidden").unwrap();
    let underscore_dir = tmp.path().join("_drafts");
    std::fs::create_dir_all(&underscore_dir).unwrap();
    std::fs::write(underscore_dir.join("draft.md"), "# Draft").unwrap();

    let tree = localex_cli::markdown::scan_workspace(tmp.path());
    let names: Vec<&str> = tree.iter().map(|e| e.name.as_str()).collect();
    assert!(names.contains(&"good.md"));
    assert!(!names.contains(&".hidden.md"));
    assert!(!names.contains(&"_drafts"));
}

#[test]
fn safe_resolve_rejects_traversal() {
    let tmp = tempfile::tempdir().unwrap();
    std::fs::write(tmp.path().join("real.md"), "").unwrap();
    assert!(localex_cli::markdown::safe_resolve(tmp.path(), "real.md").is_ok());
    assert!(localex_cli::markdown::safe_resolve(tmp.path(), "../etc/passwd").is_err());
}
