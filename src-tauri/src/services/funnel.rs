use std::process::Command;
use crate::services::vector_store::VectorStore;
use crate::services::db::Database;
use crate::config;
use tauri::{AppHandle, Emitter, Manager, WebviewWindowBuilder, WebviewUrl};
use std::sync::Arc;
use log::{info};
use serde::Serialize;
use tauri_plugin_notification::NotificationExt;

#[derive(Clone, Serialize)]
pub struct ResonancePayload {
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

        let search_text = format!("search_query: {}", content);
        let embedding = VectorStore::get_embedding(&search_text, &config.embedding_model).await?;
        let matches = self.vector_store.search(embedding, 3).await?;

        if !matches.is_empty() {
            let matched_text = matches[0].0.clone();
            let distance = matches[0].2;
            let score = (1.0 - distance).max(0.0).min(1.0);

            if score < config.threshold {
                log::info!("Match found but score {:.2} is below threshold {:.2}. Ignoring.", score, config.threshold);
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

            // 3. Show floating bubble window (best-effort, never blocks main flow)
            let bubble_app = self.app_handle.clone();
            let bubble_payload = payload.clone();
            tokio::spawn(async move {
                // Close any existing bubble first
                if let Some(w) = bubble_app.get_webview_window("resonance-bubble") {
                    let _ = w.close();
                    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                }

                // Calculate position at bottom-right of screen
                let (x, y) = if let Ok(Some(monitor)) = bubble_app.primary_monitor() {
                    let size = monitor.size();
                    let scale = monitor.scale_factor();
                    let lw = size.width as f64 / scale;
                    let lh = size.height as f64 / scale;
                    (lw - 440.0, lh - 300.0)
                } else {
                    (1000.0, 500.0)
                };

                let build_result = WebviewWindowBuilder::new(
                    &bubble_app,
                    "resonance-bubble",
                    WebviewUrl::App("bubble.html".into()),
                )
                .title("Resonance")
                .inner_size(420.0, 260.0)
                .decorations(false)
                .transparent(true)
                .always_on_top(true)
                .skip_taskbar(true)
                .resizable(false)
                .position(x, y)
                .build();

                if let Err(e) = build_result {
                    log::warn!("Failed to create bubble window: {}", e);
                    return;
                }

                // Wait for the webview to load before emitting data
                tokio::time::sleep(std::time::Duration::from_millis(500)).await;
                let _ = bubble_app.emit_to("resonance-bubble", "bubble-data", &bubble_payload);
            });
        } else {
            log::info!("Vector search returned empty matches.");
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
