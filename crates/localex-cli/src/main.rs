use std::{collections::HashMap, fs, path::PathBuf, sync::RwLock};

use anyhow::Result;
use clap::Parser;
use localex_cli::{app_router, AppState, backlinks, db, watcher, AppConfig};
use tokio::net::TcpListener;
use tracing_subscriber::EnvFilter;

#[derive(Debug, Parser)]
#[command(author, version, about = "Local-first reading shell for Markdown workspaces")]
struct Args {
    #[arg(default_value = ".")]
    directory: PathBuf,

    #[arg(long, default_value = "127.0.0.1")]
    host: String,

    #[arg(long, default_value_t = 3862)]
    port: u16,
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .with_target(false)
        .compact()
        .init();

    let args = Args::parse();
    let config = AppConfig::for_workspace(&args.directory)?.with_server(args.host, args.port);
    fs::create_dir_all(&config.data_dir)?;

    // Init database
    let database = db::init_db(&config.data_dir)?;

    // Ensure themes directory exists
    fs::create_dir_all(config.data_dir.join("themes"))?;

    // Build backlink index
    let backlink_index = backlinks::build_index(&config.workspace_root);
    eprintln!(
        "Indexed backlinks for {} files",
        config.workspace_root.display()
    );

    // Start file watcher
    let (watch_tx, _) = tokio::sync::broadcast::channel(16);
    watcher::start_watcher(config.workspace_root.clone(), watch_tx.clone())?;
    eprintln!("Watching {} for changes", config.workspace_root.display());

    let state = AppState {
        config,
        db: database,
        backlinks: std::sync::Arc::new(RwLock::new(backlink_index)),
        watch_tx: watch_tx.clone(),
        render_cache: std::sync::Arc::new(std::sync::RwLock::new(HashMap::new())),
    };

    // Background task: rebuild backlink index on file changes (debounced)
    {
        let bl_state = state.backlinks.clone();
        let ws_root = state.config.workspace_root.clone();
        let mut bl_rx = watch_tx.subscribe();
        tokio::spawn(async move {
            let mut debounce = tokio::time::Instant::now();
            loop {
                match bl_rx.recv().await {
                    Ok(_) => {
                        // Debounce: wait 2s after last event before rebuilding
                        debounce = tokio::time::Instant::now();
                        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
                        if debounce.elapsed() < std::time::Duration::from_millis(1900) {
                            continue; // another event arrived, keep waiting
                        }
                        let ws = ws_root.clone();
                        match tokio::task::spawn_blocking(move || backlinks::build_index(&ws)).await {
                            Ok(new_index) => {
                                *bl_state.write().unwrap() = new_index;
                                eprintln!("Backlink index rebuilt");
                            }
                            Err(e) => eprintln!("Backlink rebuild failed: {e}"),
                        }
                    }
                    Err(tokio::sync::broadcast::error::RecvError::Lagged(n)) => {
                        eprintln!("Backlink watcher lagged {n} events, rebuilding");
                        let ws = ws_root.clone();
                        if let Ok(new_index) = tokio::task::spawn_blocking(move || backlinks::build_index(&ws)).await {
                            *bl_state.write().unwrap() = new_index;
                        }
                    }
                    Err(_) => break,
                }
            }
        });
    }

    let address = format!("{}:{}", state.config.host, state.config.port);
    let listener = TcpListener::bind(&address).await?;

    println!("Localex dev shell on http://{address}");
    axum::serve(listener, app_router(state)).await?;

    Ok(())
}
