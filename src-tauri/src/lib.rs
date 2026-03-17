pub mod binaries;
mod commands;
mod ytdlp;

use commands::{download, playlist, settings, setup};
use std::sync::Arc;
use tauri::Manager;
use tokio::sync::Semaphore;
use dashmap::DashSet;
use dashmap::DashMap;
use tokio::task::AbortHandle;
use tokio::process::Child;

pub struct AppState {
    pub semaphore: Arc<Semaphore>,
    pub abort_handles: DashMap<String, AbortHandle>,
    pub children: Arc<DashMap<String, Child>>,
    pub cancelled: Arc<DashSet<String>>,
}

impl AppState {
    pub fn new(max_concurrent: usize) -> Self {
        Self {
            semaphore: Arc::new(Semaphore::new(max_concurrent)),
            abort_handles: DashMap::new(),
            children: Arc::new(DashMap::new()),
            cancelled: Arc::new(DashSet::new()),
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
            setup::check_binaries,
            setup::setup_binaries,
            setup::check_ytdlp_update,
            setup::update_ytdlp,
        ])
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::Destroyed = event {
                let state = window.state::<AppState>();
                // Abort all running tokio tasks
                let abort_handles: &DashMap<String, AbortHandle> = &state.abort_handles;
                for entry in abort_handles.iter() {
                    entry.value().abort();
                }
                // Kill all child processes (yt-dlp / ffmpeg)
                let children: Arc<DashMap<String, Child>> = Arc::clone(&state.children);
                tauri::async_runtime::block_on(async move {
                    for mut entry in children.iter_mut() {
                        let child: &mut Child = entry.value_mut();
                        let _ = child.kill().await;
                    }
                });
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
