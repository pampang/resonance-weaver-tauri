use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use tauri::AppHandle;
use tauri::Manager;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AppConfig {
    pub kb_sources: Vec<String>,
    pub app_whitelist: Vec<String>,
    pub threshold: f32,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            kb_sources: Vec::new(),
            app_whitelist: vec!["Slack".to_string(), "Discord".to_string(), "Notes".to_string()],
            threshold: 0.7,
        }
    }
}

pub fn get_config_path(app: &AppHandle) -> PathBuf {
    app.path().app_data_dir().expect("failed to get app data dir")
        .join("config.json")
}

pub fn load_config(app: &AppHandle) -> AppConfig {
    let path = get_config_path(app);
    if path.exists() {
        let content = fs::read_to_string(path).unwrap_or_default();
        serde_json::from_str(&content).unwrap_or_default()
    } else {
        AppConfig::default()
    }
}

pub fn save_config(app: &AppHandle, config: &AppConfig) -> Result<(), String> {
    let path = get_config_path(app);
    let dir = path.parent().unwrap();
    if !dir.exists() {
        fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    }
    let content = serde_json::to_string_pretty(config).map_err(|e| e.to_string())?;
    fs::write(path, content).map_err(|e| e.to_string())?;
    Ok(())
}
