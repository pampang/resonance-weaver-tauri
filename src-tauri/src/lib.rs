#![recursion_limit = "1024"]
mod config;
mod services;

use std::sync::Arc;
use tauri::Manager;
use crate::services::vector_store::VectorStore;
use crate::services::indexer::Indexer;
use crate::services::db::Database;
use crate::services::funnel::Funnel;
use crate::services::clipboard::ClipboardListener;
use tauri::tray::TrayIconBuilder;
use tauri::menu::{Menu, MenuItem};
use log::info;
use arboard::Clipboard;

struct AppState {
    vector_store: Arc<VectorStore>,
    _indexer: Arc<Indexer>,
    db: Arc<Database>,
}

#[tauri::command]
fn get_config(app: tauri::AppHandle) -> config::AppConfig {
    config::load_config(&app)
}

#[tauri::command]
async fn save_config(state: tauri::State<'_, AppState>, app: tauri::AppHandle, config: config::AppConfig) -> Result<(), String> {
    info!("Saving configuration...");
    config::save_config(&app, &config)?;
    state._indexer.rebuild_watcher().await?;
    let _ = state._indexer.trigger_full_index().await;
    Ok(())
}

#[tauri::command]
async fn reindex(state: tauri::State<'_, AppState>) -> Result<(), String> {
    state._indexer.trigger_full_index().await
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
fn delete_sample(state: tauri::State<'_, AppState>, id: i64) -> Result<(), String> {
    state.db.delete_sample(id)
}

#[tauri::command]
fn open_deep_bridge(content: String, matched_content: Option<String>) -> Result<String, String> {
    let prompt = if let Some(matched) = matched_content {
        format!(
            "Please synthesize and analyze the relationship between the following two pieces of information:\n\n### CAPTURED CONTENT (from clipboard):\n{}\n\n### ASSOCIATED KNOWLEDGE (from my local library):\n{}\n\nProvide a deep synthesis that integrates these concepts.",
            content, matched
        )
    } else {
        content
    };
    
    // 1. Copy to clipboard
    let mut cb = Clipboard::new().map_err(|e| e.to_string())?;
    cb.set_text(prompt.clone()).map_err(|e| e.to_string())?;
    
    // 2. Open Gemini
    let url = "https://gemini.google.com/app";
    let _ = open::that(url);

    Ok("Synthesized prompt copied to clipboard! Paste it into Gemini.".to_string())
}

#[tauri::command]
fn get_running_apps() -> Result<Vec<String>, String> {
    let output = std::process::Command::new("osascript")
        .arg("-e")
        .arg("tell application \"System Events\" to get name of every process whose background only is false")
        .output()
        .map_err(|e| e.to_string())?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut apps: Vec<String> = stdout
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();
    apps.sort();
    apps.dedup();
    Ok(apps)
}

#[tauri::command]
fn show_main_window(app: tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.set_focus();
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_log::Builder::new().build())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let handle = app.handle().clone();
            
            let show = MenuItem::with_id(app, "show", "Show Hub", true, None::<&str>).unwrap();
            let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>).unwrap();
            let menu = Menu::with_items(app, &[&show, &quit]).unwrap();

            let _tray = TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .menu(&menu)
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "show" => {
                        let window = app.get_webview_window("main").unwrap();
                        let _ = window.show();
                        let _ = window.set_focus();
                    }
                    "quit" => {
                        app.exit(0);
                    }
                    _ => {}
                })
                .build(app)
                .unwrap();

            tauri::async_runtime::block_on(async move {
                let db = Arc::new(Database::new(&handle).expect("failed to init db"));
                let _ = db.purge_old_records();
                
                let vector_store = Arc::new(VectorStore::new(&handle).await.expect("failed to init vector store"));
                let indexer = Arc::new(Indexer::new(vector_store.clone(), handle.clone()));
                let _ = indexer.start_watching().await;
                
                let funnel = Arc::new(Funnel::new(handle.clone(), vector_store.clone(), db.clone()));
                let clipboard_listener = ClipboardListener::new(handle.clone(), funnel);
                
                tokio::spawn(async move {
                    clipboard_listener.start().await;
                });
                
                app.manage(AppState { vector_store, _indexer: indexer, db });
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_config, save_config, search_resonance, 
            get_samples, delete_sample, open_deep_bridge, 
            reindex, get_running_apps, show_main_window
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
