use crate::binaries;
use tauri::{AppHandle, Emitter};

#[derive(serde::Serialize, Clone)]
struct SetupProgress {
    message: String,
}

#[tauri::command]
pub fn check_binaries() -> bool {
    binaries::has_ytdlp()
}

#[tauri::command]
pub async fn setup_binaries(app: AppHandle) -> Result<(), String> {
    let app_clone = app.clone();
    binaries::download_binaries(move |msg| {
        let _ = app_clone.emit(
            "setup-progress",
            SetupProgress {
                message: msg.to_string(),
            },
        );
    })
    .await
}
