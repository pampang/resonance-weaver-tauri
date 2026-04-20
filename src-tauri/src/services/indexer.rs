use crate::services::vector_store::VectorStore;
use notify::{Watcher, RecursiveMode, Event, RecommendedWatcher};
use std::path::PathBuf;
use tauri::AppHandle;
use std::sync::Arc;
use crate::config;
use log::{info, error};
use tokio::sync::Mutex;

pub struct Indexer {
    vector_store: Arc<VectorStore>,
    app_handle: AppHandle,
    watcher: Mutex<Option<RecommendedWatcher>>,
}

impl Indexer {
    pub fn new(vector_store: Arc<VectorStore>, app_handle: AppHandle) -> Self {
        Self { 
            vector_store, 
            app_handle,
            watcher: Mutex::new(None),
        }
    }

    pub async fn start_watching(&self) -> Result<(), String> {
        self.rebuild_watcher().await
    }

    pub async fn rebuild_watcher(&self) -> Result<(), String> {
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

        for source in &config.kb_sources {
            let path = PathBuf::from(source);
            if path.exists() {
                info!("Watching source: {:?}", path);
                watcher.watch(&path, RecursiveMode::Recursive).map_err(|e| e.to_string())?;
            } else {
                error!("Source path does not exist: {:?}", path);
            }
        }

        tokio::spawn(async move {
            while let Some(event) = rx.recv().await {
                for path in event.paths {
                    if path.is_file() {
                        info!("File change detected: {:?}", path);
                        let _ = Self::index_file(&vector_store, path).await;
                    }
                }
            }
        });

        let mut w = self.watcher.lock().await;
        *w = Some(watcher);

        Ok(())
    }

    pub async fn trigger_full_index(&self) -> Result<(), String> {
        let config = config::load_config(&self.app_handle);
        info!("Starting full indexing of all sources...");
        
        for source in config.kb_sources {
            let path = PathBuf::from(source);
            if path.exists() {
                self.index_directory(path).await?;
            }
        }
        info!("Full indexing complete.");
        Ok(())
    }

    async fn index_directory(&self, path: PathBuf) -> Result<(), String> {
        if path.is_dir() {
            for entry in std::fs::read_dir(path).map_err(|e| e.to_string())? {
                let entry = entry.map_err(|e| e.to_string())?;
                let path = entry.path();
                if path.is_dir() {
                    Box::pin(self.index_directory(path)).await?;
                } else {
                    let _ = Self::index_file(&self.vector_store, path).await;
                }
            }
        }
        Ok(())
    }

    async fn index_file(vector_store: &VectorStore, path: PathBuf) -> Result<(), String> {
        // Skip common binary files and hidden files
        if path.file_name().map(|n| n.to_string_lossy().starts_with('.')).unwrap_or(false) {
            return Ok(());
        }

        let extension = path.extension().map(|e| e.to_string_lossy().to_lowercase()).unwrap_or_default();
        let allowed_extensions = ["txt", "md", "markdown", "pdf", "docx", "rs", "ts", "js", "py", "c", "cpp", "h"];
        if !allowed_extensions.contains(&extension.as_str()) {
            return Ok(());
        }

        info!("Indexing file: {:?}", path);
        let content = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
        
        // Simple chunking logic for large files could be added here
        let text = content.chars().take(2000).collect::<String>();
        
        match VectorStore::get_embedding(&text).await {
            Ok(embedding) => {
                let id = chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0);
                vector_store.add_resonance(id, text, embedding, Some(path.to_string_lossy().to_string())).await?;
                info!("Successfully indexed: {:?}", path);
            }
            Err(e) => {
                error!("Failed to get embedding for {:?}: {}", path, e);
            }
        }
        
        Ok(())
    }
}
