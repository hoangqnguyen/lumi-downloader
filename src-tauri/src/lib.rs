mod commands;
mod ytdlp;

use commands::{download, playlist, settings};
use std::sync::Arc;
use tokio::sync::Semaphore;
use dashmap::DashMap;
use tokio::task::AbortHandle;
use tauri_plugin_shell::process::CommandChild;

pub struct AppState {
    pub semaphore: Arc<Semaphore>,
    pub abort_handles: DashMap<String, AbortHandle>,
    pub children: Arc<DashMap<String, CommandChild>>,
}

impl AppState {
    pub fn new(max_concurrent: usize) -> Self {
        Self {
            semaphore: Arc::new(Semaphore::new(max_concurrent)),
            abort_handles: DashMap::new(),
            children: Arc::new(DashMap::new()),
        }
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_store::Builder::default().build())
        .plugin(tauri_plugin_opener::init())
        .manage(AppState::new(3))
        .invoke_handler(tauri::generate_handler![
            download::start_download,
            download::cancel_download,
            download::set_max_concurrent,
            playlist::expand_playlist,
            settings::get_default_download_dir,
            settings::pick_folder,
            settings::open_folder,
            settings::open_file,
            settings::reveal_file,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
