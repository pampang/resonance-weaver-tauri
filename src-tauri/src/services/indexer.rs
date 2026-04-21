use crate::services::vector_store::VectorStore;
use notify::{Watcher, RecursiveMode, Event, RecommendedWatcher};
use std::path::PathBuf;
use tauri::{AppHandle, Emitter};
use std::sync::Arc;
use crate::config;
use log::{info, error};
use tokio::sync::Mutex;
use serde::Serialize;

#[derive(Clone, Serialize)]
struct IndexingProgress {
    total_files: usize,
    current_file: usize,
    file_name: String,
    percentage: f32,
    is_complete: bool,
}

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
                let _ = watcher.watch(&path, RecursiveMode::Recursive);
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

        let mut w = self.watcher.lock().await;
        *w = Some(watcher);
        Ok(())
    }

    pub async fn trigger_full_index(&self) -> Result<(), String> {
        let config = config::load_config(&self.app_handle);
        
        // 1. Collect all files first to calculate total
        let mut files_to_index = Vec::new();
        for source in config.kb_sources {
            let path = PathBuf::from(source);
            if path.exists() {
                self.collect_files(path, &mut files_to_index).await;
            }
        }

        let total = files_to_index.len();
        info!("Starting full indexing of {} files...", total);

        // 2. Index with progress reporting
        for (i, path) in files_to_index.into_iter().enumerate() {
            let file_name = path.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default();
            let percentage = (i as f32 / total as f32) * 100.0;
            
            let _ = self.app_handle.emit("indexing-progress", IndexingProgress {
                total_files: total,
                current_file: i + 1,
                file_name,
                percentage,
                is_complete: false,
            });

            let _ = Self::index_file(&self.vector_store, path).await;
        }

        let _ = self.app_handle.emit("indexing-progress", IndexingProgress {
            total_files: total,
            current_file: total,
            file_name: "Complete".to_string(),
            percentage: 100.0,
            is_complete: true,
        });

        info!("Full indexing complete.");
        Ok(())
    }

    async fn collect_files(&self, path: PathBuf, files: &mut Vec<PathBuf>) {
        if path.is_dir() {
            if let Ok(entries) = std::fs::read_dir(path) {
                for entry in entries.flatten() {
                    let child_path = entry.path();
                    if child_path.is_dir() {
                        Box::pin(self.collect_files(child_path, files)).await;
                    } else if self.is_allowed_file(&child_path) {
                        files.push(child_path);
                    }
                }
            }
        }
    }

    fn is_allowed_file(&self, path: &PathBuf) -> bool {
        if path.file_name().map(|n| n.to_string_lossy().starts_with('.')).unwrap_or(false) {
            return false;
        }
        let extension = path.extension().map(|e| e.to_string_lossy().to_lowercase()).unwrap_or_default();
        let allowed = ["txt", "md", "markdown", "rs", "ts", "js", "py", "c", "cpp", "h"];
        allowed.contains(&extension.as_str())
    }

    async fn index_file(vector_store: &VectorStore, path: PathBuf) -> Result<(), String> {
        let content = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
        
        // --- Smart Chunking ---
        // Split content into chunks of ~1000 characters with 200 overlap
        let chunk_size = 1000;
        let overlap = 200;
        let mut start = 0;
        let chars: Vec<char> = content.chars().collect();
        
        while start < chars.len() {
            let end = std::cmp::min(start + chunk_size, chars.len());
            let chunk: String = chars[start..end].iter().collect();
            
            if !chunk.trim().is_empty() {
                match VectorStore::get_embedding(&chunk).await {
                    Ok(embedding) => {
                        let id = chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0);
                        let metadata = format!("{}:{}", path.to_string_lossy(), start);
                        let _ = vector_store.add_resonance(id, chunk, embedding, Some(metadata)).await;
                    }
                    Err(e) => error!("Embedding error: {}", e),
                }
            }

            start += chunk_size - overlap;
            if start >= chars.len() { break; }
        }
        
        Ok(())
    }
}
