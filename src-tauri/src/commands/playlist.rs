use serde::{Deserialize, Serialize};
use tauri::AppHandle;
use tauri_plugin_shell::ShellExt;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PlaylistEntry {
    pub url: String,
    pub title: String,
    pub id: String,
    pub duration: Option<f64>,
    pub thumbnail: Option<String>,
}

#[tauri::command]
pub async fn expand_playlist(
    app: AppHandle,
    url: String,
    cookies_browser: String,
) -> Result<Vec<PlaylistEntry>, String> {
    let mut args = vec![
        "--flat-playlist".to_string(),
        "--dump-json".to_string(),
        "--no-warnings".to_string(),
    ];
    if !cookies_browser.is_empty() {
        args.push("--cookies-from-browser".to_string());
        args.push(cookies_browser);
    }
    args.push(url);

    let sidecar = app
        .shell()
        .sidecar("yt-dlp")
        .map_err(|e| format!("Failed to find yt-dlp: {e}"))?
        .args(&args);

    let output = sidecar
        .output()
        .await
        .map_err(|e| format!("Failed to run yt-dlp: {e}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("yt-dlp error: {stderr}"));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut entries = Vec::new();

    for line in stdout.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        if let Ok(val) = serde_json::from_str::<serde_json::Value>(line) {
            let id = val["id"].as_str().unwrap_or("").to_string();
            let title = val["title"]
                .as_str()
                .unwrap_or("Unknown Title")
                .to_string();
            let webpage_url = val["webpage_url"]
                .as_str()
                .map(|s| s.to_string())
                .unwrap_or_else(|| format!("https://www.youtube.com/watch?v={}", id));
            let duration = val["duration"].as_f64();
            let thumbnail = val["thumbnail"].as_str().map(|s| s.to_string());

            if !id.is_empty() {
                entries.push(PlaylistEntry {
                    url: webpage_url,
                    title,
                    id,
                    duration,
                    thumbnail,
                });
            }
        }
    }

    Ok(entries)
}
