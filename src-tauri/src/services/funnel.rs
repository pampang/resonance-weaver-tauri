use std::process::Command;
use crate::services::vector_store::VectorStore;
use crate::services::db::Database;
use crate::config;
use tauri::{AppHandle, Manager, Emitter};
use std::sync::Arc;
use log::{info};
use serde::Serialize;
use tauri_plugin_notification::NotificationExt;

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
            let distance = matches[0].2;
            let score = (1.0 - distance).max(0.0).min(1.0);

            if score < config.threshold {
                return Ok(());
            }

            info!("Resonance found! Saving to database...");
            let id = chrono::Utc::now().timestamp_millis();
            self.db.add_sample(content.clone(), matched_text.clone(), app_name.clone(), distance).await?;

            let payload = ResonancePayload {
                id,
                app_name: app_name.clone(),
                score,
                content: content.clone(),
                matched_content: matched_text,
            };

            // 1. Emit to main window for live UI updates
            let _ = self.app_handle.emit_to("main", "new-resonance", &payload);

            // 2. Native OS Notification (Elegant and non-intrusive)
            let snippet = if content.chars().count() > 40 {
                let s: String = content.chars().take(40).collect();
                format!("{}...", s)
            } else {
                content
            };
            
            let _ = self.app_handle.notification()
                .builder()
                .title(format!("Resonance with {}", app_name))
                .body(format!("{}% Match: {}", (score * 100.0) as i32, snippet))
                .show();
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
