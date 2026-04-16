use std::sync::Arc;

use axum::{
    extract::State,
    response::Html,
    routing::get,
    Json, Router,
};
use serde_json::{json, Value};

use crate::{AppConfig, ReaderPreferences};

pub fn app_router(config: AppConfig) -> Router {
    let state = Arc::new(config);

    Router::new()
        .route("/", get(index))
        .route("/api/health", get(health))
        .route("/api/reader/defaults", get(reader_defaults))
        .with_state(state)
}

async fn index() -> Html<&'static str> {
    Html(INDEX_HTML)
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

const INDEX_HTML: &str = r#"<!doctype html>
<html lang="en">
  <head>
    <meta charset="utf-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1" />
    <title>Localex</title>
    <style>
      :root {
        color-scheme: light;
        font-family: Inter, system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif;
        background: #f7f4ee;
        color: rgba(20, 18, 14, 0.92);
      }
      body {
        margin: 0;
        min-height: 100vh;
        display: grid;
        place-items: center;
        background: linear-gradient(180deg, #faf8f3 0%, #f2ede4 100%);
      }
      main {
        max-width: 720px;
        padding: 3rem;
        border: 1px solid rgba(20, 18, 14, 0.08);
        border-radius: 24px;
        background: rgba(255, 255, 255, 0.82);
        box-shadow: 0 24px 60px rgba(20, 18, 14, 0.08);
      }
      h1 {
        margin: 0 0 0.75rem;
        font-size: 2.75rem;
        line-height: 1;
        letter-spacing: -0.06em;
      }
      p {
        margin: 0;
        font-size: 1.05rem;
        line-height: 1.7;
      }
      code {
        padding: 0.2rem 0.45rem;
        border-radius: 999px;
        background: rgba(20, 18, 14, 0.08);
      }
    </style>
  </head>
  <body>
    <main>
      <h1>Localex backend ready.</h1>
      <p>
        Rust shell live. Frontend scaffold lives in <code>web/</code>. Next step: wire compiled reader UI into this server.
      </p>
    </main>
  </body>
</html>
"#;
