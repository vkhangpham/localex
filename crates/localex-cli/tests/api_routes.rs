use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use http_body_util::BodyExt;
use localex_cli::{app_router, AppState, AppConfig, LayoutMode};
use localex_cli::backlinks;
use std::sync::RwLock;
use tower::ServiceExt;

fn test_state() -> AppState {
    let config = AppConfig::for_workspace("/tmp/localex-test").unwrap();
    let db = localex_cli::db::init_db(&config.data_dir).unwrap();
    let backlink_index = backlinks::build_index(&config.workspace_root);
    AppState {
        config,
        db,
        backlinks: std::sync::Arc::new(RwLock::new(backlink_index)),
    }
}

#[test]
fn config_defaults_match_reading_prd() {
    let config = AppConfig::for_workspace("/tmp/localex-test").expect("config should build");

    assert_eq!(config.host, "127.0.0.1");
    assert_eq!(config.port, 3862);
    assert!(config.data_dir.ends_with(".localex"));
    assert_eq!(config.reader.target_words_per_line, 12);
    assert!((config.reader.line_height - 1.75).abs() < 0.001);
    assert_eq!(config.reader.font_size_px, 18);
    assert_eq!(config.reader.font_family, "Inter");
    assert_eq!(config.reader.layout_mode, LayoutMode::OneColumn);
}

#[tokio::test]
async fn health_route_returns_ok_json() {
    let app = app_router(test_state());

    let response = app
        .oneshot(Request::builder().uri("/api/health").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let body = std::str::from_utf8(&body).unwrap();
    assert!(body.contains("\"status\":\"ok\""));
    assert!(body.contains("\"app\":\"localex\""));
}

#[tokio::test]
async fn reader_defaults_route_returns_expected_preferences() {
    let app = app_router(test_state());

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/reader/defaults")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let body = std::str::from_utf8(&body).unwrap();
    assert!(body.contains("\"target_words_per_line\":12"));
    assert!(body.contains("\"layout_mode\":\"one_column\""));
}
