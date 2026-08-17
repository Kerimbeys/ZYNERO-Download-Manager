//! SQLite connection, migrations and repositories.

use rusqlite::{params, Connection, OptionalExtension};
use serde::Serialize;
use std::{fs, path::PathBuf, sync::Mutex};

#[derive(Debug)]
pub struct DatabaseState {
    connection: Mutex<Connection>,
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
    pub created_at: String,
    pub updated_at: String,
}

impl DatabaseState {
    pub fn open(app_data_dir: PathBuf) -> Result<Self, String> {
        fs::create_dir_all(&app_data_dir)
            .map_err(|error| format!("Could not create app data directory: {error}"))?;
        let path = app_data_dir.join("zynero.sqlite3");
        let connection = Connection::open(path)
            .map_err(|error| format!("Could not open SQLite database: {error}"))?;
        connection
            .pragma_update(None, "foreign_keys", "ON")
            .map_err(|error| format!("Could not enable SQLite foreign keys: {error}"))?;
        connection
            .execute_batch(include_str!("../../migrations/0001_initial.sql"))
            .map_err(|error| format!("Could not run database migrations: {error}"))?;
        Ok(Self {
            connection: Mutex::new(connection),
        })
    }

    pub fn insert_download(&self, download: &StoredDownload) -> Result<(), String> {
        let connection = self
            .connection
            .lock()
            .map_err(|_| "Database lock poisoned".to_string())?;
        connection.execute(
            "INSERT INTO downloads (id, url, filename, destination, status, total_bytes, downloaded_bytes, content_type, supports_range, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)",
            params![download.id, download.url, download.filename, download.destination, download.status, download.total_bytes, download.downloaded_bytes, download.content_type, download.supports_range as i64],
        ).map_err(|error| format!("Could not insert download: {error}"))?;
        Ok(())
    }

    pub fn list_downloads(&self) -> Result<Vec<StoredDownload>, String> {
        let connection = self
            .connection
            .lock()
            .map_err(|_| "Database lock poisoned".to_string())?;
        let mut statement = connection.prepare("SELECT id, url, filename, destination, status, total_bytes, downloaded_bytes, content_type, supports_range, created_at, updated_at FROM downloads ORDER BY created_at DESC").map_err(|error| format!("Could not prepare download query: {error}"))?;
        let rows = statement
            .query_map([], |row| {
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
                    created_at: row.get(9)?,
                    updated_at: row.get(10)?,
                })
            })
            .map_err(|error| format!("Could not read downloads: {error}"))?;
        rows.map(|row| row.map_err(|error| format!("Could not decode download: {error}")))
            .collect()
    }

    pub fn find_download(&self, id: &str) -> Result<Option<StoredDownload>, String> {
        let connection = self
            .connection
            .lock()
            .map_err(|_| "Database lock poisoned".to_string())?;
        connection.query_row("SELECT id, url, filename, destination, status, total_bytes, downloaded_bytes, content_type, supports_range, created_at, updated_at FROM downloads WHERE id = ?1", [id], |row| {
            Ok(StoredDownload {
                id: row.get(0)?, url: row.get(1)?, filename: row.get(2)?, destination: row.get(3)?, status: row.get(4)?, total_bytes: row.get(5)?, downloaded_bytes: row.get(6)?, content_type: row.get(7)?, supports_range: row.get::<_, i64>(8)? != 0, created_at: row.get(9)?, updated_at: row.get(10)?,
            })
        }).optional().map_err(|error| format!("Could not find download: {error}"))
    }
}
