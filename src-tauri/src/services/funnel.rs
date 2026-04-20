use std::process::Command;
use crate::services::vector_store::VectorStore;
use crate::services::db::Database;
use crate::config;
use tauri::AppHandle;
use std::sync::Arc;
use log::{info};

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
        // Tier 1: Metadata (Length)
        if content.len() < 10 || content.len() > 10000 {
            return Ok(());
        }

        // Tier 2: Whitelist
        let app_name = self.get_frontmost_app()?;
        let config = config::load_config(&self.app_handle);
        if !config.app_whitelist.iter().any(|a| app_name.to_lowercase().contains(&a.to_lowercase())) {
            info!("Skipping clipboard from non-whitelisted app: {}", app_name);
            return Ok(());
        }

        info!("Processing resonance for app: {}", app_name);

        // Tier 3: Search Resonance
        let embedding = VectorStore::get_embedding(&content).await?;
        let matches = self.vector_store.search(embedding, 3).await?;

        if !matches.is_empty() {
            // Found resonance! Store in triage buffer (SQLite)
            info!("Resonance found! Saving to triage buffer.");
            self.db.add_sample(content, app_name.clone(), matches[0].1).await?;

            // Send notification
            use tauri_plugin_notification::NotificationExt;
            self.app_handle.notification()
                .builder()
                .title("New Resonance Found")
                .body(format!("Resonance detected from {}", app_name))
                .show()
                .unwrap();
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
