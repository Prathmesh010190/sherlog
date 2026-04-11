use crate::config::Config;
use rusqlite::{params, Connection};

/// Anonymous, opt-in telemetry for tracking command usage and error types.
pub struct Telemetry {
    conn: Option<Connection>,
}

impl Telemetry {
    pub fn new() -> Self {
        let conn = Self::try_open().ok();
        Self { conn }
    }

    fn try_open() -> anyhow::Result<Connection> {
        let dir = Config::config_dir()?;
        std::fs::create_dir_all(&dir)?;
        let path = dir.join("telemetry.db");
        let conn = Connection::open(path)?;
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS events (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                timestamp TEXT NOT NULL DEFAULT (datetime('now')),
                event_type TEXT NOT NULL,
                event_data TEXT NOT NULL DEFAULT ''
            );",
        )?;
        Ok(conn)
    }

    /// Record that a CLI command was invoked.
    pub fn record_command(&self, command: &str) {
        if let Some(conn) = &self.conn {
            let _ = conn.execute(
                "INSERT INTO events (event_type, event_data) VALUES ('command', ?1)",
                params![command],
            );
        }
    }

    /// Record an error type that was encountered.
    #[allow(dead_code)]
    pub fn record_error_type(&self, error_type: &str) {
        if let Some(conn) = &self.conn {
            let _ = conn.execute(
                "INSERT INTO events (event_type, event_data) VALUES ('error_type', ?1)",
                params![error_type],
            );
        }
    }
}
