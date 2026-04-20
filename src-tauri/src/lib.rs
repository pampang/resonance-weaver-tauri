mod config;
mod services;

use std::sync::Arc;
use tauri::Manager;
use crate::services::vector_store::VectorStore;
use crate::services::indexer::Indexer;
use crate::services::db::Database;
use crate::services::funnel::Funnel;
use crate::services::clipboard::ClipboardListener;

struct AppState {
    vector_store: Arc<VectorStore>,
    indexer: Arc<Indexer>,
    db: Arc<Database>,
}

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
fn get_config(app: tauri::AppHandle) -> config::AppConfig {
    config::load_config(&app)
}

#[tauri::command]
fn save_config(app: tauri::AppHandle, config: config::AppConfig) -> Result<(), String> {
    config::save_config(&app, &config)
}

#[tauri::command]
async fn search_resonance(state: tauri::State<'_, AppState>, text: String) -> Result<Vec<(String, f32)>, String> {
    let embedding = VectorStore::get_embedding(&text).await?;
    state.vector_store.search(embedding, 3).await
}

#[tauri::command]
fn get_samples(state: tauri::State<'_, AppState>) -> Result<Vec<services::db::Sample>, String> {
    state.db.get_samples()
}

#[tauri::command]
fn open_deep_bridge(content: String) -> Result<(), String> {
    let url = format!("https://gemini.google.com/app?prompt={}", urlencoding::encode(&content));
    open::that(url).map_err(|e| e.to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_log::Builder::new().build())
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let handle = app.handle().clone();
            tauri::async_runtime::block_on(async move {
                let db = Arc::new(Database::new(&handle).expect("failed to init db"));
                db.purge_old_records().expect("failed to purge records");
                
                let vector_store = Arc::new(VectorStore::new(&handle).await.expect("failed to init vector store"));
                let indexer = Arc::new(Indexer::new(vector_store.clone(), handle.clone()));
                let _ = indexer.start_watching().await;
                
                let funnel = Arc::new(Funnel::new(handle.clone(), vector_store.clone(), db.clone()));
                let clipboard_listener = ClipboardListener::new(handle.clone(), funnel);
                
                tokio::spawn(async move {
                    clipboard_listener.start().await;
                });
                
                app.manage(AppState { vector_store, indexer, db });
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![greet, get_config, save_config, search_resonance, get_samples, open_deep_bridge])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
