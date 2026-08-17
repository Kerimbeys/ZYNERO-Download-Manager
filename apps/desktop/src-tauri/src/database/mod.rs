//! SQLite connection, migrations and repositories.

use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use std::{
    fs,
    path::PathBuf,
    sync::{Arc, Mutex},
};

#[derive(Debug, Clone)]
pub struct DatabaseState {
    connection: Arc<Mutex<Connection>>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StoredDownload {
    pub id: String,
    pub url: String,
    pub filename: String,
    pub destination: String,
    pub status: String,
    pub total_bytes: Option<i64>,
    pub downloaded_bytes: i64,
    pub content_type: Option<String>,
    pub supports_range: bool,
    pub temp_path: Option<String>,
    pub final_path: Option<String>,
    pub error_message: Option<String>,
    pub speed_bps: i64,
    pub eta_seconds: i64,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct QueueRecord {
    pub id: String,
    pub name: String,
    pub priority: i64,
    pub max_concurrent: i64,
    pub auto_start: bool,
    pub start_at: Option<String>,
    pub stop_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

impl DatabaseState {
    pub fn open(app_data_dir: PathBuf) -> Result<Self, String> {
        fs::create_dir_all(&app_data_dir)
            .map_err(|error| format!("Could not create app data directory: {error}"))?;
        let connection = Connection::open(app_data_dir.join("zynero.sqlite3"))
            .map_err(|error| format!("Could not open SQLite database: {error}"))?;
        connection
            .pragma_update(None, "foreign_keys", "ON")
            .map_err(|error| format!("Could not enable SQLite foreign keys: {error}"))?;
        connection
            .execute_batch(include_str!("../../migrations/0001_initial.sql"))
            .map_err(|error| format!("Could not run initial migration: {error}"))?;
        if let Err(error) =
            connection.execute_batch(include_str!("../../migrations/0002_download_runtime.sql"))
        {
            let message = error.to_string();
            if !message.contains("duplicate column name") {
                return Err(format!("Could not run runtime migration: {message}"));
            }
        }
        connection
            .execute_batch(include_str!(
                "../../migrations/0003_queues_segments_settings.sql"
            ))
            .map_err(|error| format!("Could not run queue/settings migration: {error}"))?;
        if let Err(error) =
            connection.execute_batch(include_str!("../../migrations/0004_queue_schedule.sql"))
        {
            let message = error.to_string();
            if !message.contains("duplicate column name") {
                return Err(format!("Could not run queue schedule migration: {message}"));
            }
        }
        Ok(Self {
            connection: Arc::new(Mutex::new(connection)),
        })
    }

    pub fn insert_download(&self, download: &StoredDownload) -> Result<(), String> {
        let connection = self.lock()?;
        connection.execute(
            "INSERT INTO downloads (id, url, filename, destination, status, total_bytes, downloaded_bytes, content_type, supports_range, temp_path, final_path, error_message, speed_bps, eta_seconds, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)",
            params![download.id, download.url, download.filename, download.destination, download.status, download.total_bytes, download.downloaded_bytes, download.content_type, download.supports_range as i64, download.temp_path, download.final_path, download.error_message, download.speed_bps, download.eta_seconds],
        ).map_err(|error| format!("Could not insert download: {error}"))?;
        Ok(())
    }

    pub fn list_queued_downloads(&self, limit: i64) -> Result<Vec<StoredDownload>, String> {
        let connection = self.lock()?;
        let mut statement = connection.prepare("SELECT id, url, filename, destination, status, total_bytes, downloaded_bytes, content_type, supports_range, temp_path, final_path, error_message, speed_bps, eta_seconds, created_at, updated_at FROM downloads WHERE status = 'queued' ORDER BY created_at ASC LIMIT ?1").map_err(|error| format!("Could not prepare queued download query: {error}"))?;
        let rows = statement
            .query_map([limit.max(0)], row_to_download)
            .map_err(|error| format!("Could not read queued downloads: {error}"))?;
        rows.map(|row| row.map_err(|error| format!("Could not decode queued download: {error}")))
            .collect()
    }

    pub fn count_active_downloads(&self) -> Result<i64, String> {
        let connection = self.lock()?;
        connection
            .query_row(
                "SELECT COUNT(*) FROM downloads WHERE status = 'active'",
                [],
                |row| row.get(0),
            )
            .map_err(|error| format!("Could not count active downloads: {error}"))
    }

    pub fn list_downloads(&self) -> Result<Vec<StoredDownload>, String> {
        let connection = self.lock()?;
        let mut statement = connection.prepare("SELECT id, url, filename, destination, status, total_bytes, downloaded_bytes, content_type, supports_range, temp_path, final_path, error_message, speed_bps, eta_seconds, created_at, updated_at FROM downloads ORDER BY created_at DESC").map_err(|error| format!("Could not prepare download query: {error}"))?;
        let rows = statement
            .query_map([], row_to_download)
            .map_err(|error| format!("Could not read downloads: {error}"))?;
        rows.map(|row| row.map_err(|error| format!("Could not decode download: {error}")))
            .collect()
    }

    pub fn find_download(&self, id: &str) -> Result<Option<StoredDownload>, String> {
        let connection = self.lock()?;
        connection.query_row("SELECT id, url, filename, destination, status, total_bytes, downloaded_bytes, content_type, supports_range, temp_path, final_path, error_message, speed_bps, eta_seconds, created_at, updated_at FROM downloads WHERE id = ?1", [id], row_to_download).optional().map_err(|error| format!("Could not find download: {error}"))
    }

    pub fn recover_incomplete(&self) -> Result<(), String> {
        let connection = self.lock()?;
        connection.execute("UPDATE downloads SET status = 'paused', speed_bps = 0, eta_seconds = 0, updated_at = CURRENT_TIMESTAMP WHERE status = 'active'", [])
            .map_err(|error| format!("Could not recover incomplete downloads: {error}"))?;
        Ok(())
    }

    pub fn transition_status(&self, id: &str, next_status: &str) -> Result<(), String> {
        let allowed = [
            "queued",
            "active",
            "paused",
            "completed",
            "failed",
            "cancelled",
        ];
        if !allowed.contains(&next_status) {
            return Err("Invalid download status".to_string());
        }
        let current = self
            .find_download(id)?
            .ok_or_else(|| "Download not found".to_string())?;
        let valid = match (current.status.as_str(), next_status) {
            ("queued", "active" | "cancelled")
            | ("active", "paused" | "completed" | "failed" | "cancelled")
            | ("paused", "active" | "cancelled")
            | ("failed", "queued" | "cancelled")
            | ("completed", "cancelled") => true,
            (from, to) if from == to => true,
            _ => false,
        };
        if !valid {
            return Err(format!(
                "Invalid status transition: {} -> {next_status}",
                current.status
            ));
        }
        let connection = self.lock()?;
        connection
            .execute(
                "UPDATE downloads SET status = ?2, updated_at = CURRENT_TIMESTAMP WHERE id = ?1",
                params![id, next_status],
            )
            .map_err(|error| format!("Could not transition download: {error}"))?;
        Ok(())
    }

    pub fn update_runtime(
        &self,
        id: &str,
        status: &str,
        downloaded_bytes: i64,
        speed_bps: i64,
        eta_seconds: i64,
        error_message: Option<&str>,
    ) -> Result<(), String> {
        let connection = self.lock()?;
        connection.execute("UPDATE downloads SET status = ?2, downloaded_bytes = ?3, speed_bps = ?4, eta_seconds = ?5, error_message = ?6, updated_at = CURRENT_TIMESTAMP WHERE id = ?1", params![id, status, downloaded_bytes, speed_bps, eta_seconds, error_message]).map_err(|error| format!("Could not update download runtime: {error}"))?;
        Ok(())
    }

    pub fn set_paths(&self, id: &str, temp_path: &str, final_path: &str) -> Result<(), String> {
        let connection = self.lock()?;
        connection.execute("UPDATE downloads SET temp_path = ?2, final_path = ?3, updated_at = CURRENT_TIMESTAMP WHERE id = ?1", params![id, temp_path, final_path]).map_err(|error| format!("Could not persist download paths: {error}"))?;
        Ok(())
    }

    pub fn delete_download(&self, id: &str) -> Result<Option<StoredDownload>, String> {
        let download = self.find_download(id)?;
        let connection = self.lock()?;
        connection
            .execute("DELETE FROM downloads WHERE id = ?1", [id])
            .map_err(|error| format!("Could not delete download: {error}"))?;
        Ok(download)
    }

    pub fn list_queues(&self) -> Result<Vec<QueueRecord>, String> {
        let connection = self.lock()?;
        let mut statement = connection.prepare("SELECT id, name, priority, max_concurrent, auto_start, start_at, stop_at, created_at, updated_at FROM queues ORDER BY priority DESC, created_at ASC").map_err(|error| format!("Could not prepare queue query: {error}"))?;
        let rows = statement
            .query_map([], |row| {
                Ok(QueueRecord {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    priority: row.get(2)?,
                    max_concurrent: row.get(3)?,
                    auto_start: row.get::<_, i64>(4)? != 0,
                    start_at: row.get(5)?,
                    stop_at: row.get(6)?,
                    created_at: row.get(7)?,
                    updated_at: row.get(8)?,
                })
            })
            .map_err(|error| format!("Could not read queues: {error}"))?;
        rows.map(|row| row.map_err(|error| format!("Could not decode queue: {error}")))
            .collect()
    }

    pub fn upsert_queue(&self, queue: &QueueRecord) -> Result<(), String> {
        if queue.name.trim().is_empty() || !(1..=32).contains(&queue.max_concurrent) {
            return Err("Queue name and max concurrent downloads are invalid".to_string());
        }
        let connection = self.lock()?;
        connection.execute("INSERT INTO queues (id, name, priority, max_concurrent, auto_start, start_at, stop_at, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP) ON CONFLICT(id) DO UPDATE SET name = excluded.name, priority = excluded.priority, max_concurrent = excluded.max_concurrent, auto_start = excluded.auto_start, start_at = excluded.start_at, stop_at = excluded.stop_at, updated_at = CURRENT_TIMESTAMP", params![queue.id, queue.name.trim(), queue.priority, queue.max_concurrent, queue.auto_start as i64, queue.start_at, queue.stop_at]).map_err(|error| format!("Could not save queue: {error}"))?;
        Ok(())
    }

    pub fn get_setting(&self, key: &str) -> Result<Option<String>, String> {
        let connection = self.lock()?;
        connection
            .query_row("SELECT value FROM settings WHERE key = ?1", [key], |row| {
                row.get(0)
            })
            .optional()
            .map_err(|error| format!("Could not read setting: {error}"))
    }

    pub fn set_setting(&self, key: &str, value: &str) -> Result<(), String> {
        if key.trim().is_empty() || key.len() > 80 || value.len() > 4000 {
            return Err("Setting key or value is invalid".to_string());
        }
        let connection = self.lock()?;
        connection.execute("INSERT INTO settings (key, value, updated_at) VALUES (?1, ?2, CURRENT_TIMESTAMP) ON CONFLICT(key) DO UPDATE SET value = excluded.value, updated_at = CURRENT_TIMESTAMP", params![key.trim(), value]).map_err(|error| format!("Could not save setting: {error}"))?;
        Ok(())
    }

    fn lock(&self) -> Result<std::sync::MutexGuard<'_, Connection>, String> {
        self.connection
            .lock()
            .map_err(|_| "Database lock poisoned".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample(status: &str) -> StoredDownload {
        StoredDownload {
            id: "test-download".into(),
            url: "https://example.com/file.bin".into(),
            filename: "file.bin".into(),
            destination: "Downloads".into(),
            status: status.into(),
            total_bytes: Some(100),
            downloaded_bytes: 10,
            content_type: Some("application/octet-stream".into()),
            supports_range: true,
            temp_path: None,
            final_path: None,
            error_message: None,
            speed_bps: 0,
            eta_seconds: 0,
            created_at: String::new(),
            updated_at: String::new(),
        }
    }

    #[test]
    fn transition_rules_accept_and_reject_expected_states() {
        let path = std::env::temp_dir().join(format!("zynero-test-{}", uuid::Uuid::new_v4()));
        let database = DatabaseState::open(path.clone()).expect("database opens");
        database
            .insert_download(&sample("queued"))
            .expect("insert succeeds");
        database
            .transition_status("test-download", "active")
            .expect("queued to active");
        database
            .transition_status("test-download", "paused")
            .expect("active to paused");
        assert!(database
            .transition_status("test-download", "completed")
            .is_err());
        let _ = std::fs::remove_dir_all(path);
    }

    #[test]
    fn recovery_pauses_active_downloads() {
        let path = std::env::temp_dir().join(format!("zynero-recovery-{}", uuid::Uuid::new_v4()));
        let database = DatabaseState::open(path.clone()).expect("database opens");
        database
            .insert_download(&sample("active"))
            .expect("insert succeeds");
        database.recover_incomplete().expect("recovery succeeds");
        assert_eq!(
            database
                .find_download("test-download")
                .expect("query succeeds")
                .expect("row exists")
                .status,
            "paused"
        );
        let _ = std::fs::remove_dir_all(path);
    }
}

fn row_to_download(row: &rusqlite::Row<'_>) -> rusqlite::Result<StoredDownload> {
    Ok(StoredDownload {
        id: row.get(0)?,
        url: row.get(1)?,
        filename: row.get(2)?,
        destination: row.get(3)?,
        status: row.get(4)?,
        total_bytes: row.get(5)?,
        downloaded_bytes: row.get(6)?,
        content_type: row.get(7)?,
        supports_range: row.get::<_, i64>(8)? != 0,
        temp_path: row.get(9)?,
        final_path: row.get(10)?,
        error_message: row.get(11)?,
        speed_bps: row.get(12)?,
        eta_seconds: row.get(13)?,
        created_at: row.get(14)?,
        updated_at: row.get(15)?,
    })
}
