use std::process::Command;
use crate::services::vector_store::VectorStore;
use crate::services::db::Database;
use crate::config;
use tauri::{AppHandle, Manager, Emitter, LogicalPosition, Position};
use std::sync::Arc;
use log::{info};
use serde::Serialize;

#[derive(Clone, Serialize)]
struct ResonancePayload {
    id: i64,
    app_name: String,
    score: f32,
    content: String,
    matched_content: String,
}

pub struct Funnel {
    app_handle: AppHandle,
    vector_store: Arc<VectorStore>,
    db: Arc<Database>,
}

impl Funnel {
    pub fn new(app_handle: AppHandle, vector_store: Arc<VectorStore>, db: Arc<Database>) -> Self {
        Self { app_handle, vector_store, db }
    }

    pub async fn process_clipboard(&self, content: String) -> Result<(), String> {
        if content.len() < 10 || content.len() > 10000 {
            return Ok(());
        }

        let app_name = self.get_frontmost_app()?;
        let config = config::load_config(&self.app_handle);
        if !config.app_whitelist.iter().any(|a| app_name.to_lowercase().contains(&a.to_lowercase())) {
            return Ok(());
        }

        let embedding = VectorStore::get_embedding(&content).await?;
        let matches = self.vector_store.search(embedding, 3).await?;

        if !matches.is_empty() {
            let matched_text = matches[0].0.clone();
            let score = (1.0 - matches[0].1).max(0.0).min(1.0);

            if score < config.threshold {
                return Ok(());
            }

            info!("Resonance found! Score: {:.2}", score);
            let id = chrono::Utc::now().timestamp_millis();
            self.db.add_sample(content.clone(), matched_text.clone(), app_name.clone(), matches[0].1).await?;

            // Trigger the Bubble Window with correct positioning
            if let Some(window) = self.app_handle.get_webview_window("resonance-bubble") {
                // Position at bottom right
                if let Some(monitor) = window.current_monitor().map_err(|e| e.to_string())? {
                    let size = monitor.size();
                    let scale_factor = monitor.scale_factor();
                    
                    // Logical size of the bubble is 300x120 (from tauri.conf.json)
                    let x = (size.width as f64 / scale_factor) - 320.0;
                    let y = (size.height as f64 / scale_factor) - 180.0;
                    
                    let _ = window.set_position(Position::Logical(LogicalPosition { x, y }));
                }

                let _ = window.emit("new-resonance", ResonancePayload {
                    id,
                    app_name,
                    score,
                    content,
                    matched_content: matched_text,
                });
                let _ = window.show();
            }
        }

        Ok(())
    }

    fn get_frontmost_app(&self) -> Result<String, String> {
        let output = Command::new("osascript")
            .arg("-e")
            .arg("tell application \"System Events\" to get name of first process whose frontmost is true")
            .output()
            .map_err(|e| e.to_string())?;

        let name = String::from_utf8_lossy(&output.stdout).trim().to_string();
        Ok(name)
    }
}
