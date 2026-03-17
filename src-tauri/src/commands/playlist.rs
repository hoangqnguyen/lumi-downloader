use crate::binaries;
use regex::Regex;
use serde::{Deserialize, Serialize};
use tokio::process::Command;

#[cfg(target_os = "windows")]
const CREATE_NO_WINDOW: u32 = 0x08000000;

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
    url: String,
    cookie_browser: String,
) -> Result<Vec<PlaylistEntry>, String> {
    let ytdlp_path = binaries::bin_path("yt-dlp");

    let mut args = vec![
        "--flat-playlist".to_string(),
        "--dump-json".to_string(),
        "--no-warnings".to_string(),
    ];
    if !cookie_browser.is_empty() && cookie_browser != "none" {
        args.extend(["--cookies-from-browser".to_string(), cookie_browser.clone()]);
    }
    args.push(url.clone());

    let mut cmd = Command::new(&ytdlp_path);
    cmd.args(&args);
    #[cfg(target_os = "windows")]
    cmd.creation_flags(CREATE_NO_WINDOW);
    let output = cmd
        .output()
        .await
        .map_err(|e| format!("Failed to run yt-dlp: {e}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);

        // If yt-dlp can't extract TikTok user ID, try resolving via the API
        if stderr.contains("Unable to extract secondary user ID") {
            let tiktok_user_re =
                Regex::new(r"(?i)^https?://(?:www\.)?tiktok\.com/@([\w.-]+)").unwrap();
            if let Some(caps) = tiktok_user_re.captures(&url) {
                let username = &caps[1];
                if let Ok(resolved) = resolve_tiktok_user(&ytdlp_path, username, &cookie_browser).await {
                    return expand_with_url(&ytdlp_path, &resolved, &cookie_browser).await;
                }
            }
            return Err(
                "TikTok user profiles are not supported yet. Try pasting individual video URLs instead, or enable Firefox cookies."
                    .to_string(),
            );
        }

        return Err(format!("yt-dlp error: {stderr}"));
    }

    parse_playlist_output(&output.stdout)
}

/// Try to resolve a TikTok username to a tiktokuser:secUid URL.
/// Uses yt-dlp to grab one video from the user's page and extract the channel_id.
async fn resolve_tiktok_user(
    ytdlp_path: &std::path::Path,
    username: &str,
    cookie_browser: &str,
) -> Result<String, String> {
    // Use yt-dlp to extract channel_id from the user page via --print
    let mut args = vec![
        "--print".to_string(),
        "channel_id".to_string(),
        "--playlist-items".to_string(),
        "1".to_string(),
        "--no-warnings".to_string(),
    ];
    if !cookie_browser.is_empty() && cookie_browser != "none" {
        args.extend(["--cookies-from-browser".to_string(), cookie_browser.to_string()]);
    }
    args.push(format!("https://www.tiktok.com/@{username}"));

    let mut cmd = Command::new(ytdlp_path);
    cmd.args(&args);
    #[cfg(target_os = "windows")]
    cmd.creation_flags(CREATE_NO_WINDOW);
    let output = cmd
        .output()
        .await
        .map_err(|e| format!("Failed to resolve TikTok user: {e}"))?;

    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if output.status.success() && !stdout.is_empty() && stdout != "NA" {
        return Ok(format!("tiktokuser:{stdout}"));
    }

    Err("Could not resolve TikTok user".to_string())
}

/// Run yt-dlp playlist expansion on a resolved URL.
async fn expand_with_url(
    ytdlp_path: &std::path::Path,
    url: &str,
    cookie_browser: &str,
) -> Result<Vec<PlaylistEntry>, String> {
    let mut args = vec![
        "--flat-playlist".to_string(),
        "--dump-json".to_string(),
        "--no-warnings".to_string(),
    ];
    if !cookie_browser.is_empty() && cookie_browser != "none" {
        args.extend(["--cookies-from-browser".to_string(), cookie_browser.to_string()]);
    }
    args.push(url.to_string());

    let mut cmd = Command::new(ytdlp_path);
    cmd.args(&args);
    #[cfg(target_os = "windows")]
    cmd.creation_flags(CREATE_NO_WINDOW);
    let output = cmd
        .output()
        .await
        .map_err(|e| format!("Failed to run yt-dlp: {e}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("yt-dlp error: {stderr}"));
    }

    parse_playlist_output(&output.stdout)
}

fn parse_playlist_output(stdout: &[u8]) -> Result<Vec<PlaylistEntry>, String> {
    let stdout = String::from_utf8_lossy(stdout);
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
                .or_else(|| val["url"].as_str())
                .map(|s| s.to_string())
                .unwrap_or_else(|| format!("https://www.tiktok.com/@/video/{}", id));
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
