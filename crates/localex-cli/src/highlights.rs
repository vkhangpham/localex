use anyhow::Result;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Highlight {
    pub id: i64,
    pub document_path: String,
    pub quote_text: String,
    pub prefix_context: String,
    pub suffix_context: String,
    pub heading_slug: String,
    pub color: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateHighlight {
    pub document_path: String,
    pub quote_text: String,
    pub prefix_context: Option<String>,
    pub suffix_context: Option<String>,
    pub heading_slug: Option<String>,
    pub color: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Note {
    pub id: i64,
    pub highlight_id: Option<i64>,
    pub document_path: String,
    pub anchor_text: String,
    pub body: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateNote {
    pub highlight_id: Option<i64>,
    pub document_path: String,
    pub anchor_text: Option<String>,
    pub body: String,
}

pub fn list_highlights(conn: &Connection, path: &str) -> Result<Vec<Highlight>> {
    let mut stmt = conn.prepare(
        "SELECT id, document_path, quote_text, prefix_context, suffix_context, heading_slug, color, created_at, updated_at FROM highlights WHERE document_path = ?1",
    )?;
    let rows = stmt.query_map([path], |row| {
        Ok(Highlight {
            id: row.get(0)?,
            document_path: row.get(1)?,
            quote_text: row.get(2)?,
            prefix_context: row.get(3)?,
            suffix_context: row.get(4)?,
            heading_slug: row.get(5)?,
            color: row.get(6)?,
            created_at: row.get(7)?,
            updated_at: row.get(8)?,
        })
    })?;
    let mut results = Vec::new();
    for row in rows {
        results.push(row?);
    }
    Ok(results)
}

pub fn create_highlight(conn: &Connection, input: &CreateHighlight) -> Result<Highlight> {
    conn.execute(
        "INSERT INTO highlights (document_path, quote_text, prefix_context, suffix_context, heading_slug, color) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        (
            &input.document_path,
            &input.quote_text,
            input.prefix_context.as_deref().unwrap_or(""),
            input.suffix_context.as_deref().unwrap_or(""),
            input.heading_slug.as_deref().unwrap_or(""),
            input.color.as_deref().unwrap_or("yellow"),
        ),
    )?;
    get_highlight_by_id(conn, conn.last_insert_rowid())
}

fn get_highlight_by_id(conn: &Connection, id: i64) -> Result<Highlight> {
    conn.query_row(
        "SELECT id, document_path, quote_text, prefix_context, suffix_context, heading_slug, color, created_at, updated_at FROM highlights WHERE id = ?1",
        [id],
        |row| {
            Ok(Highlight {
                id: row.get(0)?,
                document_path: row.get(1)?,
                quote_text: row.get(2)?,
                prefix_context: row.get(3)?,
                suffix_context: row.get(4)?,
                heading_slug: row.get(5)?,
                color: row.get(6)?,
                created_at: row.get(7)?,
                updated_at: row.get(8)?,
            })
        },
    ).map_err(|e| anyhow::anyhow!("failed to fetch highlight {id}: {e}"))
}

pub fn delete_highlight(conn: &Connection, id: i64) -> Result<bool> {
    let rows = conn.execute("DELETE FROM highlights WHERE id = ?1", [id])?;
    Ok(rows > 0)
}

pub fn list_notes(conn: &Connection, path: &str) -> Result<Vec<Note>> {
    let mut stmt = conn.prepare(
        "SELECT id, highlight_id, document_path, anchor_text, body, created_at, updated_at FROM notes WHERE document_path = ?1",
    )?;
    let rows = stmt.query_map([path], |row| {
        Ok(Note {
            id: row.get(0)?,
            highlight_id: row.get(1)?,
            document_path: row.get(2)?,
            anchor_text: row.get(3)?,
            body: row.get(4)?,
            created_at: row.get(5)?,
            updated_at: row.get(6)?,
        })
    })?;
    let mut results = Vec::new();
    for row in rows {
        results.push(row?);
    }
    Ok(results)
}

pub fn create_note(conn: &Connection, input: &CreateNote) -> Result<Note> {
    conn.execute(
        "INSERT INTO notes (highlight_id, document_path, anchor_text, body) VALUES (?1, ?2, ?3, ?4)",
        (
            &input.highlight_id,
            &input.document_path,
            input.anchor_text.as_deref().unwrap_or(""),
            &input.body,
        ),
    )?;
    get_note_by_id(conn, conn.last_insert_rowid())
}

fn get_note_by_id(conn: &Connection, id: i64) -> Result<Note> {
    conn.query_row(
        "SELECT id, highlight_id, document_path, anchor_text, body, created_at, updated_at FROM notes WHERE id = ?1",
        [id],
        |row| {
            Ok(Note {
                id: row.get(0)?,
                highlight_id: row.get(1)?,
                document_path: row.get(2)?,
                anchor_text: row.get(3)?,
                body: row.get(4)?,
                created_at: row.get(5)?,
                updated_at: row.get(6)?,
            })
        },
    ).map_err(|e| anyhow::anyhow!("failed to fetch note {id}: {e}"))
}

pub fn delete_note(conn: &Connection, id: i64) -> Result<bool> {
    let rows = conn.execute("DELETE FROM notes WHERE id = ?1", [id])?;
    Ok(rows > 0)
}
