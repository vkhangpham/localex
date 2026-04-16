use std::path::Path;
use std::sync::{Arc, Mutex};

use anyhow::{Context, Result};
use rusqlite::Connection;

pub type Db = Arc<Mutex<Connection>>;

pub fn init_db(data_dir: &Path) -> Result<Db> {
    std::fs::create_dir_all(data_dir).context("failed to create data directory")?;
    let db_path = data_dir.join("localex.db");
    let conn = Connection::open(&db_path).context("failed to open database")?;
    conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;")
        .context("failed to set pragmas")?;
    migrate(&conn)?;
    Ok(Arc::new(Mutex::new(conn)))
}

fn migrate(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS preferences (
            key   TEXT PRIMARY KEY,
            value TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS highlights (
            id              INTEGER PRIMARY KEY AUTOINCREMENT,
            document_path   TEXT NOT NULL,
            quote_text      TEXT NOT NULL,
            prefix_context  TEXT NOT NULL DEFAULT '',
            suffix_context  TEXT NOT NULL DEFAULT '',
            heading_slug    TEXT NOT NULL DEFAULT '',
            color           TEXT NOT NULL DEFAULT 'yellow',
            created_at      TEXT NOT NULL DEFAULT (datetime('now')),
            updated_at      TEXT NOT NULL DEFAULT (datetime('now'))
        );
        CREATE INDEX IF NOT EXISTS idx_highlights_doc ON highlights(document_path);

        CREATE TABLE IF NOT EXISTS notes (
            id              INTEGER PRIMARY KEY AUTOINCREMENT,
            highlight_id    INTEGER,
            document_path   TEXT NOT NULL,
            anchor_text     TEXT NOT NULL DEFAULT '',
            body            TEXT NOT NULL,
            created_at      TEXT NOT NULL DEFAULT (datetime('now')),
            updated_at      TEXT NOT NULL DEFAULT (datetime('now')),
            FOREIGN KEY (highlight_id) REFERENCES highlights(id) ON DELETE CASCADE
        );
        CREATE INDEX IF NOT EXISTS idx_notes_doc ON notes(document_path);
        CREATE INDEX IF NOT EXISTS idx_notes_highlight ON notes(highlight_id);
        ",
    )
    .context("failed to run migrations")?;
    Ok(())
}

pub fn get_preference(conn: &Connection, key: &str) -> Option<String> {
    let mut stmt = conn.prepare("SELECT value FROM preferences WHERE key = ?1").ok()?;
    stmt.query_row([key], |row| row.get(0)).ok()
}

pub fn set_preference(conn: &Connection, key: &str, value: &str) -> Result<()> {
    conn.execute(
        "INSERT INTO preferences (key, value) VALUES (?1, ?2) ON CONFLICT(key) DO UPDATE SET value = ?2",
        [key, value],
    )
    .context("failed to set preference")?;
    Ok(())
}
