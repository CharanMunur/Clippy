use std::fs;
use std::path::{Path, PathBuf};
use rusqlite::{params, Connection, OptionalExtension, Result};
use chrono::{DateTime, Utc};
use directories::ProjectDirs;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum EntryKind {
    Text,
    Image,
}

impl EntryKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            EntryKind::Text => "text",
            EntryKind::Image => "image",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "text" => Some(EntryKind::Text),
            "image" => Some(EntryKind::Image),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ClipboardEntry {
    pub id: i64,
    pub kind: EntryKind,
    pub text_content: Option<String>,
    pub image_path: Option<String>,
    pub pinned: bool,
    pub created_at: DateTime<Utc>,
    pub content_hash: String,
}

/// Helper function to return the application's XDG data directory path.
pub fn get_data_dir() -> PathBuf {
    if let Some(proj_dirs) = ProjectDirs::from("io", "github", "CharanMunur.Clippy") {
        proj_dirs.data_dir().to_path_buf()
    } else {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        Path::new(&home).join(".local").join("share").join("clippy")
    }
}

/// Helper function to return the directory path where clipboard images are saved.
pub fn get_images_dir() -> PathBuf {
    get_data_dir().join("images")
}

/// Creates the database tables and indexes on the given connection.
fn create_schema(conn: &Connection) -> Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS clippy_history (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            kind TEXT NOT NULL,
            text_content TEXT,
            image_path TEXT,
            pinned INTEGER NOT NULL DEFAULT 0,
            created_at TEXT NOT NULL,
            content_hash TEXT NOT NULL UNIQUE
        )",
        [],
    )?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_content_hash ON clippy_history (content_hash)",
        [],
    )?;
    
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_sorting ON clippy_history (pinned DESC, created_at DESC)",
        [],
    )?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS clippy_config (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL
        )",
        [],
    )?;

    conn.execute(
        "INSERT OR IGNORE INTO clippy_config (key, value) VALUES ('autostart', 'true')",
        [],
    )?;

    conn.execute(
        "INSERT OR IGNORE INTO clippy_config (key, value) VALUES ('shortcut', '<Super>j')",
        [],
    )?;

    conn.execute(
        "INSERT OR IGNORE INTO clippy_config (key, value) VALUES ('history_limit', '200')",
        [],
    )?;

    Ok(())
}

/// Initializes the SQLite database file and directory structure on disk.
pub fn init_db() -> Result<Connection> {
    let data_dir = get_data_dir();
    fs::create_dir_all(&data_dir).expect("Failed to create data directory");
    
    let images_dir = get_images_dir();
    fs::create_dir_all(&images_dir).expect("Failed to create images directory");

    let db_path = data_dir.join("clippy.db");
    let conn = Connection::open(db_path)?;
    create_schema(&conn)?;

    Ok(conn)
}

/// Inserts a new clipboard entry or updates the timestamp of an existing one.
///
/// If an entry with the same content hash already exists, its timestamp is
/// updated to the current time, bringing it to the top of the history.
pub fn insert_entry(
    conn: &Connection,
    kind: EntryKind,
    text_content: Option<&str>,
    image_path: Option<&str>,
    content_hash: &str,
) -> Result<i64> {
    let now = Utc::now().to_rfc3339();

    // Check if an entry with this hash already exists
    let existing: Option<(i64, String)> = conn
        .query_row(
            "SELECT id, created_at FROM clippy_history WHERE content_hash = ?1",
            params![content_hash],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .optional()?;

    if let Some((id, _created_at)) = existing {
        // Update timestamp so it rises to the top of the history list
        conn.execute(
            "UPDATE clippy_history SET created_at = ?1 WHERE id = ?2",
            params![now, id],
        )?;
        Ok(id)
    } else {
        // Insert new entry
        conn.execute(
            "INSERT INTO clippy_history (kind, text_content, image_path, pinned, created_at, content_hash)
             VALUES (?1, ?2, ?3, 0, ?4, ?5)",
            params![kind.as_str(), text_content, image_path, now, content_hash],
        )?;
        Ok(conn.last_insert_rowid())
    }
}

/// Retrieves clipboard entries, optionally filtered by a text search query.
///
/// Results are ordered with pinned items first, then by creation date descending.
pub fn get_entries(conn: &Connection, search_query: Option<&str>) -> Result<Vec<ClipboardEntry>> {
    let mut sql = "SELECT id, kind, text_content, image_path, pinned, created_at, content_hash \
                   FROM clippy_history".to_string();

    let mut params_vec: Vec<String> = Vec::new();
    if let Some(query) = search_query
        && !query.trim().is_empty() {
            sql.push_str(" WHERE text_content LIKE ?1");
            params_vec.push(format!("%{}%", query));
        }

    sql.push_str(" ORDER BY pinned DESC, created_at DESC");

    let mut stmt = conn.prepare(&sql)?;
    
    let mapper = |row: &rusqlite::Row| {
        let kind_str: String = row.get(1)?;
        let created_at_str: String = row.get(5)?;
        Ok(ClipboardEntry {
            id: row.get(0)?,
            kind: EntryKind::from_str(&kind_str).unwrap_or(EntryKind::Text),
            text_content: row.get(2)?,
            image_path: row.get(3)?,
            pinned: row.get::<_, i32>(4)? != 0,
            created_at: DateTime::parse_from_rfc3339(&created_at_str)
                .unwrap_or_else(|_| Utc::now().into())
                .with_timezone(&Utc),
            content_hash: row.get(6)?,
        })
    };

    let entries = if params_vec.is_empty() {
        let rows = stmt.query_map([], mapper)?;
        let mut list = Vec::new();
        for r in rows {
            list.push(r?);
        }
        list
    } else {
        let rows = stmt.query_map(params![params_vec[0]], mapper)?;
        let mut list = Vec::new();
        for r in rows {
            list.push(r?);
        }
        list
    };

    Ok(entries)
}

/// Toggles the pinned status of a clipboard entry.
pub fn toggle_pin_entry(conn: &Connection, id: i64) -> Result<()> {
    conn.execute(
        "UPDATE clippy_history SET pinned = 1 - pinned WHERE id = ?1",
        params![id],
    )?;
    Ok(())
}

/// Deletes a clipboard entry from the database, removing any associated image files on disk.
pub fn delete_entry(conn: &Connection, id: i64) -> Result<()> {
    let image_path: Option<String> = conn
        .query_row(
            "SELECT image_path FROM clippy_history WHERE id = ?1",
            params![id],
            |row| row.get(0),
        )
        .optional()?
        .flatten();

    if let Some(path) = image_path {
        let path = Path::new(&path);
        if path.exists() {
            let _ = fs::remove_file(path);
        }
    }

    conn.execute("DELETE FROM clippy_history WHERE id = ?1", params![id])?;
    Ok(())
}

/// Automatically prunes the oldest unpinned entries exceeding the maximum limit.
pub fn prune_entries(conn: &Connection, max_unpinned: usize) -> Result<()> {
    let mut stmt = conn.prepare(
        "SELECT id, image_path FROM clippy_history \
         WHERE pinned = 0 \
         ORDER BY created_at DESC \
         LIMIT -1 OFFSET ?1",
    )?;
    
    let rows = stmt.query_map(params![max_unpinned as i64], |row| {
        Ok((row.get::<_, i64>(0)?, row.get::<_, Option<String>>(1)?))
    })?;

    let mut to_delete = Vec::new();
    for row in rows {
        to_delete.push(row?);
    }

    for (id, image_path) in to_delete {
        if let Some(path) = image_path {
            let path = Path::new(&path);
            if path.exists() {
                let _ = fs::remove_file(path);
            }
        }
        conn.execute("DELETE FROM clippy_history WHERE id = ?1", params![id])?;
    }

    Ok(())
}

/// Clears all unpinned clipboard entries from the database and deletes their images.
pub fn clear_unpinned_entries(conn: &Connection) -> Result<()> {
    let mut stmt = conn.prepare("SELECT image_path FROM clippy_history WHERE pinned = 0")?;
    let rows = stmt.query_map([], |row| row.get::<_, Option<String>>(0))?;
    
    for path_opt in rows {
        if let Ok(Some(path)) = path_opt {
            let path = Path::new(&path);
            if path.exists() {
                let _ = fs::remove_file(path);
            }
        }
    }

    conn.execute("DELETE FROM clippy_history WHERE pinned = 0", [])?;
    Ok(())
}

/// Gets a configuration value by key.
pub fn get_config_val(conn: &Connection, key: &str) -> Result<Option<String>> {
    let val: Option<String> = conn
        .query_row(
            "SELECT value FROM clippy_config WHERE key = ?1",
            params![key],
            |row| row.get(0),
        )
        .optional()?;
    Ok(val)
}

/// Sets a configuration value by key, updating it if it already exists.
pub fn set_config_val(conn: &Connection, key: &str, value: &str) -> Result<()> {
    conn.execute(
        "INSERT INTO clippy_config (key, value)
         VALUES (?1, ?2)
         ON CONFLICT(key) DO UPDATE SET value = excluded.value",
        params![key, value],
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_in_memory_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        create_schema(&conn).unwrap();
        conn
    }

    #[test]
    fn test_insert_and_get_entries() {
        let conn = setup_in_memory_db();
        
        let id1 = insert_entry(&conn, EntryKind::Text, Some("first copy"), None, "hash1").unwrap();
        let id2 = insert_entry(&conn, EntryKind::Text, Some("second copy"), None, "hash2").unwrap();

        assert_eq!(id1, 1);
        assert_eq!(id2, 2);

        let entries = get_entries(&conn, None).unwrap();
        assert_eq!(entries.len(), 2);
        // Order is descending created_at, so second copy first
        assert_eq!(entries[0].text_content.as_deref(), Some("second copy"));
        assert_eq!(entries[1].text_content.as_deref(), Some("first copy"));
    }

    #[test]
    fn test_deduplication_moves_to_top() {
        let conn = setup_in_memory_db();
        
        insert_entry(&conn, EntryKind::Text, Some("item 1"), None, "hash1").unwrap();
        insert_entry(&conn, EntryKind::Text, Some("item 2"), None, "hash2").unwrap();
        
        // Re-inserting item 1 should update its timestamp and bring it to top
        let id = insert_entry(&conn, EntryKind::Text, Some("item 1"), None, "hash1").unwrap();
        assert_eq!(id, 1); // ID should remain 1

        let entries = get_entries(&conn, None).unwrap();
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].text_content.as_deref(), Some("item 1"));
        assert_eq!(entries[1].text_content.as_deref(), Some("item 2"));
    }

    #[test]
    fn test_search_entries() {
        let conn = setup_in_memory_db();
        
        insert_entry(&conn, EntryKind::Text, Some("apple pie"), None, "hash1").unwrap();
        insert_entry(&conn, EntryKind::Text, Some("banana split"), None, "hash2").unwrap();
        insert_entry(&conn, EntryKind::Text, Some("pineapple juice"), None, "hash3").unwrap();

        let entries = get_entries(&conn, Some("apple")).unwrap();
        assert_eq!(entries.len(), 2); // apple pie, pineapple juice
        
        let no_match = get_entries(&conn, Some("grape")).unwrap();
        assert_eq!(no_match.len(), 0);
    }

    #[test]
    fn test_pin_and_sorting() {
        let conn = setup_in_memory_db();
        
        let id1 = insert_entry(&conn, EntryKind::Text, Some("item 1"), None, "hash1").unwrap();
        let _id2 = insert_entry(&conn, EntryKind::Text, Some("item 2"), None, "hash2").unwrap();
        
        // Pin item 1
        toggle_pin_entry(&conn, id1).unwrap();

        let entries = get_entries(&conn, None).unwrap();
        assert_eq!(entries[0].text_content.as_deref(), Some("item 1")); // pinned should be first
        assert!(entries[0].pinned);
        assert!(!entries[1].pinned);
    }

    #[test]
    fn test_prune_entries() {
        let conn = setup_in_memory_db();
        
        let id1 = insert_entry(&conn, EntryKind::Text, Some("item 1"), None, "hash1").unwrap();
        let _id2 = insert_entry(&conn, EntryKind::Text, Some("item 2"), None, "hash2").unwrap();
        let id3 = insert_entry(&conn, EntryKind::Text, Some("item 3"), None, "hash3").unwrap();

        // Pin item 1 (so it won't be pruned)
        toggle_pin_entry(&conn, id1).unwrap();

        // Prune keeping max 1 unpinned
        prune_entries(&conn, 1).unwrap();

        let entries = get_entries(&conn, None).unwrap();
        assert_eq!(entries.len(), 2); // pinned item 1, plus newest unpinned (item 3)
        assert_eq!(entries[0].id, id1);
        assert_eq!(entries[1].id, id3);
    }

    #[test]
    fn test_clear_unpinned_entries() {
        let conn = setup_in_memory_db();
        
        let id1 = insert_entry(&conn, EntryKind::Text, Some("item 1"), None, "hash1").unwrap();
        let _id2 = insert_entry(&conn, EntryKind::Text, Some("item 2"), None, "hash2").unwrap();
        
        // Pin item 1
        toggle_pin_entry(&conn, id1).unwrap();

        // Clear all unpinned
        clear_unpinned_entries(&conn).unwrap();

        let entries = get_entries(&conn, None).unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].id, id1);
        assert_eq!(entries[0].text_content.as_deref(), Some("item 1"));
    }
}
