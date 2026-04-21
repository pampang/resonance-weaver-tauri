use rusqlite::{params, Connection};
use std::sync::Mutex;
use tauri::AppHandle;
use tauri::Manager;

pub struct Database {
    conn: Mutex<Connection>,
}

impl Database {
    pub fn new(app_handle: &AppHandle) -> Result<Self, String> {
        let app_dir = app_handle.path().app_data_dir().expect("failed to get app data dir");
        if !app_dir.exists() {
            std::fs::create_dir_all(&app_dir).map_err(|e| e.to_string())?;
        }
        let db_path = app_dir.join("buffer.db");
        let conn = Connection::open(db_path).map_err(|e| e.to_string())?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS samples (
                id INTEGER PRIMARY KEY,
                content TEXT NOT NULL,
                matched_content TEXT,
                source_app TEXT NOT NULL,
                distance REAL NOT NULL,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP
            )",
            [],
        ).map_err(|e| e.to_string())?;

        // Migration: check if matched_content exists
        let mut stmt = conn.prepare("PRAGMA table_info(samples)").map_err(|e| e.to_string())?;
        let rows = stmt.query_map([], |row| {
            let name: String = row.get(1)?;
            Ok(name)
        }).map_err(|e| e.to_string())?;

        let mut has_matched_content = false;
        for row in rows {
            if let Ok(name) = row {
                if name == "matched_content" {
                    has_matched_content = true;
                    break;
                }
            }
        }

        if !has_matched_content {
            let _ = conn.execute("ALTER TABLE samples ADD COLUMN matched_content TEXT", []);
        }

        Ok(Self { conn: Mutex::new(conn) })
    }

    pub async fn add_sample(&self, content: String, matched_content: String, app: String, distance: f32) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|_| "failed to lock db")?;
        conn.execute(
            "INSERT INTO samples (content, matched_content, source_app, distance) VALUES (?1, ?2, ?3, ?4)",
            params![content, matched_content, app, distance],
        ).map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn purge_old_records(&self) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|_| "failed to lock db")?;
        conn.execute(
            "DELETE FROM samples WHERE created_at < datetime('now', '-7 days')",
            [],
        ).map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn get_samples(&self) -> Result<Vec<Sample>, String> {
        let conn = self.conn.lock().map_err(|_| "failed to lock db")?;
        let mut stmt = conn.prepare("SELECT content, matched_content, source_app, distance, created_at FROM samples ORDER BY created_at DESC")
            .map_err(|e| e.to_string())?;
        
        let samples = stmt.query_map([], |row| {
            Ok(Sample {
                content: row.get(0)?,
                matched_content: row.get(1)?,
                source_app: row.get(2)?,
                distance: row.get(3)?,
                created_at: row.get(4)?,
            })
        }).map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

        Ok(samples)
    }
}

#[derive(serde::Serialize)]
pub struct Sample {
    pub content: String,
    pub matched_content: Option<String>,
    pub source_app: String,
    pub distance: f32,
    pub created_at: String,
}
