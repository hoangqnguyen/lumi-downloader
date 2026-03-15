use crate::ytdlp::runner::{run_download, DownloadRequest};
use crate::AppState;
use std::sync::Arc;
use tauri::{AppHandle, State};

#[tauri::command]
pub async fn start_download(
    app: AppHandle,
    state: State<'_, AppState>,
    job_id: String,
    url: String,
    output_dir: String,
    audio_only: bool,
    resolution: String,
    cookies_browser: String,
) -> Result<(), String> {
    let semaphore = Arc::clone(&state.semaphore);
    let abort_handles = state.abort_handles.clone();
    let children = Arc::clone(&state.children);
    let cancelled = Arc::clone(&state.cancelled);

    let req = DownloadRequest {
        job_id: job_id.clone(),
        url,
        output_dir,
        audio_only,
        resolution,
        cookies_browser,
    };

    let app_clone = app.clone();

    let handle = tokio::spawn(async move {
        let _permit = semaphore.acquire_owned().await.ok();
        run_download(app_clone, req, children, cancelled).await.ok();
    });

    abort_handles.insert(job_id, handle.abort_handle());
    Ok(())
}

#[tauri::command]
pub async fn cancel_download(
    state: State<'_, AppState>,
    job_id: String,
) -> Result<(), String> {
    // Mark as cancelled so the event loop won't emit Done
    state.cancelled.insert(job_id.clone());
    // Kill the yt-dlp child process
    if let Some((_, mut child)) = state.children.remove(&job_id) {
        let _ = child.kill().await;
    }
    // Abort the tokio task
    if let Some((_, handle)) = state.abort_handles.remove(&job_id) {
        handle.abort();
    }
    Ok(())
}

#[tauri::command]
pub async fn set_max_concurrent(
    _state: State<'_, AppState>,
    _max: usize,
) -> Result<(), String> {
    Ok(())
}
