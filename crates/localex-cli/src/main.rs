use std::{fs, path::PathBuf, sync::RwLock};

use anyhow::Result;
use clap::Parser;
use localex_cli::{app_router, AppState, backlinks, db, AppConfig};
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

    let state = AppState {
        config,
        db: database,
        backlinks: std::sync::Arc::new(RwLock::new(backlink_index)),
    };

    let address = format!("{}:{}", state.config.host, state.config.port);
    let listener = TcpListener::bind(&address).await?;

    println!("Localex dev shell on http://{address}");
    axum::serve(listener, app_router(state)).await?;

    Ok(())
}
