use crate::services::vector_store::VectorStore;
use notify::{Watcher, RecursiveMode, Event, RecommendedWatcher};
use std::path::PathBuf;
use tauri::{AppHandle, Emitter};
use std::sync::Arc;
use crate::config;
use log::{info, error};
use tokio::sync::Mutex;
use serde::Serialize;
use regex::Regex;

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
        // 1. CLEAR THE INDEX FIRST to avoid old metadata noise
        info!("Purging existing vector index for a clean slate...");
        self.vector_store.clear_table().await?;

        let config = config::load_config(&self.app_handle);
        let mut files_to_index = Vec::new();
        for source in config.kb_sources {
            let path = PathBuf::from(source);
            if path.exists() {
                self.collect_files(path, &mut files_to_index).await;
            }
        }

        let total = files_to_index.len();
        for (i, path) in files_to_index.into_iter().enumerate() {
            let file_name = path.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default();
            let _ = self.app_handle.emit("indexing-progress", IndexingProgress {
                total_files: total,
                current_file: i + 1,
                file_name,
                percentage: (i as f32 / total as f32) * 100.0,
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
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or_default().to_lowercase();
        ["txt", "md", "markdown", "rs", "ts", "py", "js"].contains(&ext.as_str())
    }

    async fn index_file(vector_store: &VectorStore, path: PathBuf) -> Result<(), String> {
        let content = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
        let file_name = path.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default();
        
        let chunks = Self::split_text_recursively(&content, 1000, 200);
        
        for (i, chunk_content) in chunks.into_iter().enumerate() {
            if chunk_content.trim().len() < 40 { continue; }

            // Context Injection for better relevance
            let augmented_content = format!("[Source: {}]\n{}", file_name, chunk_content);
            
            match VectorStore::get_embedding(&augmented_content).await {
                Ok(embedding) => {
                    let id = chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0) + (i as i64);
                    let metadata = format!("file:{}|part:{}", file_name, i);
                    let _ = vector_store.add_resonance(id, chunk_content, embedding, Some(metadata)).await;
                }
                Err(e) => error!("Embedding failed for {}: {}", file_name, e),
            }
        }
        Ok(())
    }

    fn split_text_recursively(text: &str, max_size: usize, overlap: usize) -> Vec<String> {
        // 0. AGGRESSIVE CLEANUP
        // re_props matches properties even with leading spaces, case-insensitive
        let re_frontmatter = Regex::new(r"(?s)^---[\r\n].*?---").unwrap();
        let re_props = Regex::new(r"(?mi)^\s*(type|status|tags|source|created|updated|summary|author|aliases|linter-yaml-title):.*$").unwrap();
        let re_callout_headers = Regex::new(r"(?m)^>\s*\[!\w+\][+-]?\s*").unwrap(); // Strips "[!abstract]"
        
        let cleaned = re_frontmatter.replace(text, "");
        let cleaned = re_props.replace_all(&cleaned, "");
        let cleaned = re_callout_headers.replace_all(&cleaned, "> "); // Keep the quote, strip the tag
        
        let mut final_chunks = Vec::new();
        let paragraphs: Vec<&str> = cleaned.split("\n\n").collect();
        let mut current_chunk = String::new();

        for paragraph in paragraphs {
            let p = paragraph.trim();
            if p.is_empty() { continue; }

            if p.chars().count() > max_size {
                if !current_chunk.is_empty() {
                    final_chunks.push(current_chunk.clone());
                    current_chunk.clear();
                }
                let sub_chunks = Self::split_by_fixed_window(p, max_size, overlap);
                final_chunks.extend(sub_chunks);
            } else {
                if current_chunk.chars().count() + p.chars().count() > max_size {
                    final_chunks.push(current_chunk.clone());
                    current_chunk = p.to_string();
                } else {
                    if !current_chunk.is_empty() { current_chunk.push_str("\n\n"); }
                    current_chunk.push_str(p);
                }
            }
        }
        if !current_chunk.is_empty() { final_chunks.push(current_chunk); }
        final_chunks
    }

    fn split_by_fixed_window(text: &str, size: usize, overlap: usize) -> Vec<String> {
        let chars: Vec<char> = text.chars().collect();
        let mut chunks = Vec::new();
        let mut start = 0;
        while start < chars.len() {
            let end = std::cmp::min(start + size, chars.len());
            let chunk: String = chars[start..end].iter().collect();
            chunks.push(chunk);
            if end == chars.len() { break; }
            start += size - overlap;
        }
        chunks
    }
}
