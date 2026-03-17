use crate::binaries;
use serde::Serialize;
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

#[derive(Serialize)]
pub struct YtdlpVersionInfo {
    pub current: String,
    pub latest: String,
    pub update_available: bool,
}

#[tauri::command]
pub async fn check_ytdlp_update() -> Result<YtdlpVersionInfo, String> {
    let ytdlp_path = binaries::bin_path("yt-dlp");

    // Get current version
    let output = tokio::process::Command::new(&ytdlp_path)
        .arg("--version")
        .output()
        .await
        .map_err(|e| format!("Failed to run yt-dlp: {e}"))?;

    let current = String::from_utf8_lossy(&output.stdout).trim().to_string();

    // Get latest version from GitHub API
    let client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::limited(5))
        .build()
        .map_err(|e| format!("HTTP client error: {e}"))?;

    let resp = client
        .get("https://api.github.com/repos/yt-dlp/yt-dlp/releases/latest")
        .header("User-Agent", "LumiDownloader")
        .send()
        .await
        .map_err(|e| format!("Failed to check for updates: {e}"))?;

    let json: serde_json::Value = resp
        .json()
        .await
        .map_err(|e| format!("Failed to parse GitHub response: {e}"))?;

    let latest = json["tag_name"]
        .as_str()
        .unwrap_or("")
        .to_string();

    let update_available = !latest.is_empty() && latest != current;

    Ok(YtdlpVersionInfo {
        current,
        latest,
        update_available,
    })
}

#[tauri::command]
pub async fn update_ytdlp() -> Result<String, String> {
    let dir = binaries::bin_dir();
    std::fs::create_dir_all(&dir).map_err(|e| format!("Failed to create bin dir: {e}"))?;
    binaries::download_ytdlp_public(&dir).await?;

    // Return new version
    let ytdlp_path = binaries::bin_path("yt-dlp");
    let output = tokio::process::Command::new(&ytdlp_path)
        .arg("--version")
        .output()
        .await
        .map_err(|e| format!("Failed to verify update: {e}"))?;

    let version = String::from_utf8_lossy(&output.stdout).trim().to_string();
    Ok(version)
}
