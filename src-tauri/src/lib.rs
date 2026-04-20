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
use tauri::tray::{TrayIconBuilder, TrayIconEvent};
use tauri::menu::{Menu, MenuItem};

struct AppState {
    vector_store: Arc<VectorStore>,
    _indexer: Arc<Indexer>,
    db: Arc<Database>,
}

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
        .plugin(tauri_plugin_notification::init())
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
                        window.show().unwrap();
                        window.set_focus().unwrap();
                    }
                    "quit" => {
                        app.exit(0);
                    }
                    _ => {}
                })
                .on_tray_icon_event(|_tray, event| {
                    if let TrayIconEvent::Click { .. } = event {
                        // Handle click
                    }
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
        .invoke_handler(tauri::generate_handler![greet, get_config, save_config, search_resonance, get_samples, open_deep_bridge])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
