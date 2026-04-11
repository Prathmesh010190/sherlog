use anyhow::{Context, Result};
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HistoryEntry {
    pub id: i64,
    pub timestamp: String,
    pub command: String,
    pub error_text: String,
    pub root_cause: String,
    pub fixes_json: String,
    pub project_context: String,
    pub resolved: bool,
}

pub struct Storage {
    conn: Connection,
}

impl Storage {
    pub fn db_path() -> Result<PathBuf> {
        let dir = dirs::home_dir()
            .context("Could not determine home directory")?
            .join(".sherlog");
        std::fs::create_dir_all(&dir)
            .with_context(|| format!("Failed to create directory: {}", dir.display()))?;
        Ok(dir.join("history.db"))
    }

    pub fn open() -> Result<Self> {
        let path = Self::db_path()?;
        let conn = Connection::open(&path)
            .with_context(|| format!("Failed to open database: {}", path.display()))?;
        let storage = Self { conn };
        storage.init_schema()?;
        Ok(storage)
    }

    fn init_schema(&self) -> Result<()> {
        self.conn
            .execute_batch(
                "CREATE TABLE IF NOT EXISTS history (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    timestamp TEXT NOT NULL DEFAULT (datetime('now')),
                    command TEXT NOT NULL DEFAULT '',
                    error_text TEXT NOT NULL,
                    root_cause TEXT NOT NULL DEFAULT '',
                    fixes_json TEXT NOT NULL DEFAULT '[]',
                    project_context TEXT NOT NULL DEFAULT '',
                    resolved INTEGER NOT NULL DEFAULT 0
                );",
            )
            .context("Failed to initialize database schema")?;
        Ok(())
    }

    pub fn insert(
        &self,
        command: &str,
        error_text: &str,
        root_cause: &str,
        fixes_json: &str,
        project_context: &str,
    ) -> Result<i64> {
        self.conn
            .execute(
                "INSERT INTO history (command, error_text, root_cause, fixes_json, project_context) VALUES (?1, ?2, ?3, ?4, ?5)",
                params![command, error_text, root_cause, fixes_json, project_context],
            )
            .context("Failed to insert history entry")?;
        Ok(self.conn.last_insert_rowid())
    }

    pub fn get_recent(&self, limit: usize) -> Result<Vec<HistoryEntry>> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT id, timestamp, command, error_text, root_cause, fixes_json, project_context, resolved FROM history ORDER BY id DESC LIMIT ?1",
            )
            .context("Failed to prepare query")?;

        let entries = stmt
            .query_map(params![limit], |row| {
                Ok(HistoryEntry {
                    id: row.get(0)?,
                    timestamp: row.get(1)?,
                    command: row.get(2)?,
                    error_text: row.get(3)?,
                    root_cause: row.get(4)?,
                    fixes_json: row.get(5)?,
                    project_context: row.get(6)?,
                    resolved: row.get(7)?,
                })
            })
            .context("Failed to query history")?
            .collect::<Result<Vec<_>, _>>()
            .context("Failed to read history entries")?;

        Ok(entries)
    }

    /// Get entries since a given datetime string (SQLite datetime format).
    pub fn get_since(&self, since_datetime: &str) -> Result<Vec<HistoryEntry>> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT id, timestamp, command, error_text, root_cause, fixes_json, project_context, resolved FROM history WHERE timestamp >= ?1 ORDER BY timestamp ASC",
            )
            .context("Failed to prepare query")?;

        let entries = stmt
            .query_map(params![since_datetime], |row| {
                Ok(HistoryEntry {
                    id: row.get(0)?,
                    timestamp: row.get(1)?,
                    command: row.get(2)?,
                    error_text: row.get(3)?,
                    root_cause: row.get(4)?,
                    fixes_json: row.get(5)?,
                    project_context: row.get(6)?,
                    resolved: row.get(7)?,
                })
            })
            .context("Failed to query history")?
            .collect::<Result<Vec<_>, _>>()
            .context("Failed to read history entries")?;

        Ok(entries)
    }

    /// Get aggregate counts of root causes since a given datetime.
    pub fn get_root_cause_counts(&self, since_datetime: &str) -> Result<Vec<(String, i64)>> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT root_cause, COUNT(*) as cnt FROM history WHERE timestamp >= ?1 AND root_cause != '' GROUP BY root_cause ORDER BY cnt DESC",
            )
            .context("Failed to prepare query")?;

        let rows = stmt
            .query_map(params![since_datetime], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
            })
            .context("Failed to query patterns")?
            .collect::<Result<Vec<_>, _>>()
            .context("Failed to read pattern data")?;

        Ok(rows)
    }

    /// Get error counts grouped by day since a given datetime.
    pub fn get_daily_counts(&self, since_datetime: &str) -> Result<Vec<(String, i64)>> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT date(timestamp) as day, COUNT(*) as cnt FROM history WHERE timestamp >= ?1 GROUP BY day ORDER BY day ASC",
            )
            .context("Failed to prepare query")?;

        let rows = stmt
            .query_map(params![since_datetime], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
            })
            .context("Failed to query daily counts")?
            .collect::<Result<Vec<_>, _>>()
            .context("Failed to read daily counts")?;

        Ok(rows)
    }

    /// Get the total number of entries since a given datetime.
    pub fn get_count_since(&self, since_datetime: &str) -> Result<i64> {
        let count: i64 = self
            .conn
            .query_row(
                "SELECT COUNT(*) FROM history WHERE timestamp >= ?1",
                params![since_datetime],
                |row| row.get(0),
            )
            .context("Failed to count entries")?;
        Ok(count)
    }

    /// Get most common commands that produce errors.
    pub fn get_command_counts(&self, since_datetime: &str) -> Result<Vec<(String, i64)>> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT command, COUNT(*) as cnt FROM history WHERE timestamp >= ?1 AND command != '' GROUP BY command ORDER BY cnt DESC LIMIT 10",
            )
            .context("Failed to prepare query")?;

        let rows = stmt
            .query_map(params![since_datetime], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
            })
            .context("Failed to query command counts")?
            .collect::<Result<Vec<_>, _>>()
            .context("Failed to read command counts")?;

        Ok(rows)
    }

    #[allow(dead_code)]
    pub fn mark_resolved(&self, id: i64) -> Result<()> {
        self.conn
            .execute("UPDATE history SET resolved = 1 WHERE id = ?1", params![id])
            .context("Failed to mark entry as resolved")?;
        Ok(())
    }
}
