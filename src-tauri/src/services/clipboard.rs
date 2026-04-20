use std::sync::Arc;
use tokio::time::{sleep, Duration};
use arboard::Clipboard;
use tauri::AppHandle;
use crate::services::funnel::Funnel;

pub struct ClipboardListener {
    _app_handle: AppHandle,
    funnel: Arc<Funnel>,
}

impl ClipboardListener {
    pub fn new(app_handle: AppHandle, funnel: Arc<Funnel>) -> Self {
        Self { _app_handle: app_handle, funnel }
    }

    pub async fn start(&self) {
        let mut clipboard = Clipboard::new().expect("failed to init clipboard");
        let mut last_content = String::new();

        loop {
            if let Ok(content) = clipboard.get_text() {
                if !content.is_empty() && content != last_content {
                    last_content = content.clone();
                    let funnel = self.funnel.clone();
                    tokio::spawn(async move {
                        let _ = funnel.process_clipboard(content).await;
                    });
                }
            }
            sleep(Duration::from_millis(500)).await;
        }
    }
}
