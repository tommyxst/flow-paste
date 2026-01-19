//! Clipboard History Module
//!
//! Provides persistent storage and management of clipboard history.
//! Features:
//! - SQLite storage with 50-item limit
//! - Content deduplication via SHA-256 hash
//! - Debouncing to prevent rapid duplicate entries
//! - Preview generation for list display

use rusqlite::{params, Connection, OpenFlags};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::{AppHandle, Manager, Runtime};
use thiserror::Error;

use super::ClipboardKind;

pub const HISTORY_LIMIT: usize = 50;
pub const MAX_ITEM_BYTES: usize = 256 * 1024; // 256 KB
pub const PREVIEW_MAX_CHARS: usize = 200;
pub const DEBOUNCE_WINDOW_MS: i64 = 800;
pub const MIN_POLL_INTERVAL_MS: u64 = 300;
pub const MAX_POLL_INTERVAL_MS: u64 = 800;

#[derive(Debug, Error)]
pub enum HistoryError {
    #[error("database error: {0}")]
    Database(String),
    #[error("item not found: {0}")]
    NotFound(i64),
    #[error("content too large: {0} bytes (max: {1})")]
    TooLarge(usize, usize),
    #[error("config directory unavailable")]
    ConfigDirUnavailable,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClipboardHistoryEntry {
    pub id: i64,
    pub kind: ClipboardKind,
    pub preview: String,
    pub byte_length: usize,
    pub created_at_ms: i64,
    pub pii_detected: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClipboardHistoryItem {
    pub id: i64,
    pub kind: ClipboardKind,
    pub text: Option<String>,
    pub preview: String,
    pub byte_length: usize,
    pub created_at_ms: i64,
    pub pii_detected: bool,
}

#[derive(Debug, Clone)]
pub struct RecordResult {
    pub inserted: bool,
    pub id: Option<i64>,
    pub reason: Option<&'static str>,
}

struct LastSeen {
    text_hash: String,
    at_ms: i64,
}

pub struct ClipboardHistoryService {
    db: Mutex<Connection>,
    last_seen: Mutex<Option<LastSeen>>,
}

impl ClipboardHistoryService {
    pub fn init<R: Runtime>(app: &AppHandle<R>) -> Result<Self, HistoryError> {
        let config_dir = app
            .path()
            .app_config_dir()
            .map_err(|_| HistoryError::ConfigDirUnavailable)?;

        if !config_dir.exists() {
            std::fs::create_dir_all(&config_dir)
                .map_err(|e| HistoryError::Database(e.to_string()))?;
        }

        let db_path = config_dir.join("settings.db");
        let conn = Connection::open_with_flags(
            &db_path,
            OpenFlags::SQLITE_OPEN_READ_WRITE
                | OpenFlags::SQLITE_OPEN_CREATE
                | OpenFlags::SQLITE_OPEN_FULL_MUTEX,
        )
        .map_err(|e| HistoryError::Database(e.to_string()))?;

        conn.pragma_update(None, "journal_mode", "WAL")
            .map_err(|e| HistoryError::Database(e.to_string()))?;

        // Set busy timeout to handle concurrent access (5 seconds)
        conn.pragma_update(None, "busy_timeout", "5000")
            .map_err(|e| HistoryError::Database(e.to_string()))?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS clipboard_history (
                id            INTEGER PRIMARY KEY AUTOINCREMENT,
                kind          TEXT NOT NULL,
                text          TEXT,
                text_hash     TEXT,
                preview       TEXT NOT NULL,
                byte_length   INTEGER NOT NULL,
                created_at_ms INTEGER NOT NULL,
                source        TEXT NOT NULL DEFAULT 'poll',
                pii_detected  INTEGER NOT NULL DEFAULT 0
            )",
            [],
        )
        .map_err(|e| HistoryError::Database(e.to_string()))?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_clipboard_history_created_at
             ON clipboard_history(created_at_ms DESC)",
            [],
        )
        .map_err(|e| HistoryError::Database(e.to_string()))?;

        log::info!("ClipboardHistoryService initialized at {:?}", db_path);

        Ok(Self {
            db: Mutex::new(conn),
            last_seen: Mutex::new(None),
        })
    }

    pub fn list(&self, limit: usize, offset: usize) -> Result<Vec<ClipboardHistoryEntry>, HistoryError> {
        let conn = self.db.lock().map_err(|e| HistoryError::Database(e.to_string()))?;

        let mut stmt = conn
            .prepare(
                "SELECT id, kind, preview, byte_length, created_at_ms, pii_detected
                 FROM clipboard_history
                 ORDER BY id DESC
                 LIMIT ?1 OFFSET ?2",
            )
            .map_err(|e| HistoryError::Database(e.to_string()))?;

        let rows = stmt
            .query_map(params![limit as i64, offset as i64], |row| {
                Ok(ClipboardHistoryEntry {
                    id: row.get(0)?,
                    kind: parse_kind(&row.get::<_, String>(1)?),
                    preview: row.get(2)?,
                    byte_length: row.get::<_, i64>(3)? as usize,
                    created_at_ms: row.get(4)?,
                    pii_detected: row.get::<_, i64>(5)? != 0,
                })
            })
            .map_err(|e| HistoryError::Database(e.to_string()))?;

        let mut entries = Vec::new();
        for row in rows {
            entries.push(row.map_err(|e| HistoryError::Database(e.to_string()))?);
        }

        Ok(entries)
    }

    pub fn get(&self, id: i64) -> Result<ClipboardHistoryItem, HistoryError> {
        let conn = self.db.lock().map_err(|e| HistoryError::Database(e.to_string()))?;

        let item = conn
            .query_row(
                "SELECT id, kind, text, preview, byte_length, created_at_ms, pii_detected
                 FROM clipboard_history
                 WHERE id = ?1",
                params![id],
                |row| {
                    Ok(ClipboardHistoryItem {
                        id: row.get(0)?,
                        kind: parse_kind(&row.get::<_, String>(1)?),
                        text: row.get(2)?,
                        preview: row.get(3)?,
                        byte_length: row.get::<_, i64>(4)? as usize,
                        created_at_ms: row.get(5)?,
                        pii_detected: row.get::<_, i64>(6)? != 0,
                    })
                },
            )
            .map_err(|e| match e {
                rusqlite::Error::QueryReturnedNoRows => HistoryError::NotFound(id),
                _ => HistoryError::Database(e.to_string()),
            })?;

        Ok(item)
    }

    pub fn delete(&self, id: i64) -> Result<(), HistoryError> {
        let conn = self.db.lock().map_err(|e| HistoryError::Database(e.to_string()))?;

        let affected = conn
            .execute("DELETE FROM clipboard_history WHERE id = ?1", params![id])
            .map_err(|e| HistoryError::Database(e.to_string()))?;

        if affected == 0 {
            return Err(HistoryError::NotFound(id));
        }

        Ok(())
    }

    pub fn clear(&self) -> Result<(), HistoryError> {
        let conn = self.db.lock().map_err(|e| HistoryError::Database(e.to_string()))?;

        conn.execute("DELETE FROM clipboard_history", [])
            .map_err(|e| HistoryError::Database(e.to_string()))?;

        // Reset last_seen
        if let Ok(mut last_seen) = self.last_seen.lock() {
            *last_seen = None;
        }

        Ok(())
    }

    pub fn record_text(
        &self,
        text: &str,
        source: &str,
        pii_detected: bool,
    ) -> Result<RecordResult, HistoryError> {
        let byte_length = text.len();

        // Check size limit
        if byte_length > MAX_ITEM_BYTES {
            return Ok(RecordResult {
                inserted: false,
                id: None,
                reason: Some("too_large"),
            });
        }

        let text_hash = compute_text_hash(text);
        let now_ms = current_time_ms();

        // Check debounce
        {
            let last_seen = self.last_seen.lock().map_err(|e| HistoryError::Database(e.to_string()))?;
            if let Some(ref last) = *last_seen {
                if last.text_hash == text_hash && (now_ms - last.at_ms) < DEBOUNCE_WINDOW_MS {
                    return Ok(RecordResult {
                        inserted: false,
                        id: None,
                        reason: Some("debounced"),
                    });
                }
            }
        }

        let preview = make_preview(text, PREVIEW_MAX_CHARS);
        let kind_str = "text";

        let conn = self.db.lock().map_err(|e| HistoryError::Database(e.to_string()))?;

        // Insert new record
        conn.execute(
            "INSERT INTO clipboard_history (kind, text, text_hash, preview, byte_length, created_at_ms, source, pii_detected)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![kind_str, text, text_hash, preview, byte_length as i64, now_ms, source, pii_detected as i64],
        )
        .map_err(|e| HistoryError::Database(e.to_string()))?;

        let id = conn.last_insert_rowid();

        // Truncate to limit
        conn.execute(
            "DELETE FROM clipboard_history WHERE id NOT IN
             (SELECT id FROM clipboard_history ORDER BY id DESC LIMIT ?1)",
            params![HISTORY_LIMIT as i64],
        )
        .map_err(|e| HistoryError::Database(e.to_string()))?;

        // Update last_seen
        drop(conn);
        {
            let mut last_seen = self.last_seen.lock().map_err(|e| HistoryError::Database(e.to_string()))?;
            *last_seen = Some(LastSeen {
                text_hash,
                at_ms: now_ms,
            });
        }

        Ok(RecordResult {
            inserted: true,
            id: Some(id),
            reason: None,
        })
    }

    pub fn get_last_hash(&self) -> Option<String> {
        self.last_seen
            .lock()
            .ok()
            .and_then(|guard| guard.as_ref().map(|ls| ls.text_hash.clone()))
    }
}

fn parse_kind(s: &str) -> ClipboardKind {
    match s {
        "text" => ClipboardKind::Text,
        "image" => ClipboardKind::Image,
        _ => ClipboardKind::Unknown,
    }
}

pub fn make_preview(text: &str, max_chars: usize) -> String {
    let normalized: String = text
        .chars()
        .map(|c| if c.is_whitespace() { ' ' } else { c })
        .collect();

    let trimmed = normalized.trim();

    if trimmed.chars().count() <= max_chars {
        trimmed.to_string()
    } else {
        let preview: String = trimmed.chars().take(max_chars).collect();
        format!("{}...", preview.trim_end())
    }
}

pub fn compute_text_hash(text: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(text.as_bytes());
    let result = hasher.finalize();
    hex::encode(result)
}

pub fn current_time_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

pub fn next_poll_interval_ms(current: u64, inserted: bool) -> u64 {
    if inserted {
        MIN_POLL_INTERVAL_MS
    } else {
        // Gradually increase interval when no changes
        let next = current + 50;
        next.min(MAX_POLL_INTERVAL_MS)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_make_preview_short() {
        let text = "Hello World";
        let preview = make_preview(text, 200);
        assert_eq!(preview, "Hello World");
    }

    #[test]
    fn test_make_preview_long() {
        let text = "a".repeat(300);
        let preview = make_preview(&text, 200);
        assert!(preview.ends_with("..."));
        assert!(preview.len() <= 203); // 200 + "..."
    }

    #[test]
    fn test_make_preview_whitespace() {
        let text = "Hello\n\nWorld\t\tTest";
        let preview = make_preview(text, 200);
        assert_eq!(preview, "Hello  World  Test");
    }

    #[test]
    fn test_compute_hash_consistent() {
        let text = "test content";
        let hash1 = compute_text_hash(text);
        let hash2 = compute_text_hash(text);
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_compute_hash_different() {
        let hash1 = compute_text_hash("text1");
        let hash2 = compute_text_hash("text2");
        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_next_poll_interval_inserted() {
        assert_eq!(next_poll_interval_ms(800, true), MIN_POLL_INTERVAL_MS);
    }

    #[test]
    fn test_next_poll_interval_no_change() {
        assert_eq!(next_poll_interval_ms(300, false), 350);
        assert_eq!(next_poll_interval_ms(750, false), 800);
        assert_eq!(next_poll_interval_ms(800, false), 800);
    }
}
