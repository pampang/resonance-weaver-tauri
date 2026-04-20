use crate::services::vector_store::VectorStore;
use notify::{Watcher, RecursiveMode, Event};
use std::path::PathBuf;
use tauri::AppHandle;
use std::sync::Arc;
use crate::config;

pub struct Indexer {
    vector_store: Arc<VectorStore>,
    app_handle: AppHandle,
}

impl Indexer {
    pub fn new(vector_store: Arc<VectorStore>, app_handle: AppHandle) -> Self {
        Self { vector_store, app_handle }
    }

    pub async fn start_watching(&self) -> Result<(), String> {
        let config = config::load_config(&self.app_handle);
        let vector_store = self.vector_store.clone();
        
        let (tx, mut rx) = tokio::sync::mpsc::channel(100);

        let mut watcher = notify::recommended_watcher(move |res: notify::Result<Event>| {
            if let Ok(event) = res {
                if event.kind.is_modify() || event.kind.is_create() {
                    let _ = tx.blocking_send(event);
                }
            }
        }).map_err(|e| e.to_string())?;

        for source in config.kb_sources {
            let path = PathBuf::from(source);
            if path.exists() {
                watcher.watch(&path, RecursiveMode::Recursive).map_err(|e| e.to_string())?;
            }
        }

        tokio::spawn(async move {
            while let Some(event) = rx.recv().await {
                for path in event.paths {
                    if path.is_file() {
                        let _ = Self::index_file(&vector_store, path).await;
                    }
                }
            }
        });

        // We need to keep the watcher alive, so we'll store it or leak it for this demo.
        // In a real app, it should be part of the Indexer state.
        Box::leak(Box::new(watcher));

        Ok(())
    }

    async fn index_file(vector_store: &VectorStore, path: PathBuf) -> Result<(), String> {
        let content = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
        // Simple dehydration: just use the whole content if it's small, or chunk it.
        // For now, let's just use the first 2000 chars.
        let text = content.chars().take(2000).collect::<String>();
        
        let embedding = VectorStore::get_embedding(&text).await?;
        
        let id = chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0);
        vector_store.add_resonance(id, text, embedding, Some(path.to_string_lossy().to_string())).await?;
        
        Ok(())
    }
}
