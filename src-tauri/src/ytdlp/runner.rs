use crate::ytdlp::progress::{parse_line, ParsedLine};
use dashmap::{DashMap, DashSet};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::{AppHandle, Emitter};
use tauri_plugin_shell::{process::CommandChild, ShellExt};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DownloadRequest {
    pub job_id: String,
    pub url: String,
    pub output_dir: String,
    pub audio_only: bool,
    pub resolution: String,
    pub cookies_browser: String,
}

#[derive(Debug, Serialize, Clone)]
pub struct JobEvent {
    pub job_id: String,
    #[serde(flatten)]
    pub payload: JobEventPayload,
}

#[derive(Debug, Serialize, Clone)]
#[serde(tag = "kind", content = "data")]
pub enum JobEventPayload {
    Progress {
        percent: f32,
        size: String,
        speed: String,
        eta: String,
    },
    Merging,
    FilePath {
        path: String,
    },
    Title {
        title: String,
    },
    Done {
        success: bool,
        error: Option<String>,
    },
}

pub async fn run_download(
    app: AppHandle,
    req: DownloadRequest,
    children: Arc<DashMap<String, CommandChild>>,
    cancelled: Arc<DashSet<String>>,
) -> Result<(), String> {
    let args = build_args(&req);

    let sidecar = app
        .shell()
        .sidecar("yt-dlp")
        .map_err(|e| format!("Failed to find yt-dlp sidecar: {e}"))?
        .args(&args);

    let (mut rx, child) = sidecar
        .spawn()
        .map_err(|e| format!("Failed to spawn yt-dlp: {e}"))?;

    children.insert(req.job_id.clone(), child);

    let mut error_lines: Vec<String> = Vec::new();

    while let Some(event) = rx.recv().await {
        use tauri_plugin_shell::process::CommandEvent;
        match event {
            CommandEvent::Stdout(bytes) => {
                let text = String::from_utf8_lossy(&bytes).into_owned();
                for line in text.lines() {
                    let line = line.trim();
                    match parse_line(line) {
                        ParsedLine::Progress(p) => {
                            let _ = app.emit(
                                "job-event",
                                JobEvent {
                                    job_id: req.job_id.clone(),
                                    payload: JobEventPayload::Progress {
                                        percent: p.percent,
                                        size: p.size,
                                        speed: p.speed,
                                        eta: p.eta,
                                    },
                                },
                            );
                        }
                        ParsedLine::Merging => {
                            let _ = app.emit(
                                "job-event",
                                JobEvent {
                                    job_id: req.job_id.clone(),
                                    payload: JobEventPayload::Merging,
                                },
                            );
                        }
                        ParsedLine::Title(title) => {
                            let _ = app.emit(
                                "job-event",
                                JobEvent {
                                    job_id: req.job_id.clone(),
                                    payload: JobEventPayload::Title { title },
                                },
                            );
                        }
                        ParsedLine::FinalPath(path) => {
                            let _ = app.emit(
                                "job-event",
                                JobEvent {
                                    job_id: req.job_id.clone(),
                                    payload: JobEventPayload::FilePath { path },
                                },
                            );
                        }
                        ParsedLine::AlreadyDownloaded => {
                            let _ = app.emit(
                                "job-event",
                                JobEvent {
                                    job_id: req.job_id.clone(),
                                    payload: JobEventPayload::Done {
                                        success: true,
                                        error: None,
                                    },
                                },
                            );
                            return Ok(());
                        }
                        _ => {}
                    }
                }
            }
            CommandEvent::Stderr(bytes) => {
                let text = String::from_utf8_lossy(&bytes).into_owned();
                for line in text.lines() {
                    let line = line.trim();
                    // yt-dlp routes progress to stderr when not in a TTY
                    match parse_line(line) {
                        ParsedLine::Progress(p) => {
                            let _ = app.emit(
                                "job-event",
                                JobEvent {
                                    job_id: req.job_id.clone(),
                                    payload: JobEventPayload::Progress {
                                        percent: p.percent,
                                        size: p.size,
                                        speed: p.speed,
                                        eta: p.eta,
                                    },
                                },
                            );
                        }
                        ParsedLine::Merging => {
                            let _ = app.emit(
                                "job-event",
                                JobEvent {
                                    job_id: req.job_id.clone(),
                                    payload: JobEventPayload::Merging,
                                },
                            );
                        }
                        _ => {
                            if !line.is_empty() {
                                error_lines.push(line.to_string());
                            }
                        }
                    }
                }
            }
            CommandEvent::Terminated(status) => {
                children.remove(&req.job_id);
                // Don't emit Done for cancelled jobs
                if cancelled.remove(&req.job_id).is_some() {
                    break;
                }
                let success = status.code == Some(0);
                let error = if success {
                    None
                } else {
                    Some(error_lines.join("\n"))
                };
                let _ = app.emit(
                    "job-event",
                    JobEvent {
                        job_id: req.job_id.clone(),
                        payload: JobEventPayload::Done { success, error },
                    },
                );
                break;
            }
            _ => {}
        }
    }

    Ok(())
}

/// Returns the directory containing ffmpeg (and ffprobe) so yt-dlp can find both.
fn find_ffmpeg_dir() -> Option<String> {
    // Prefer the bundled sidecar directory (placed alongside the executable by Tauri)
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            for name in &["ffmpeg.exe", "ffmpeg"] {
                if dir.join(name).exists() {
                    return Some(dir.to_string_lossy().into_owned());
                }
            }
        }
    }
    // Search PATH so package-manager installs (apt, brew, winget) are found automatically
    if let Some(ffmpeg_path) = which_bin("ffmpeg") {
        if let Some(dir) = std::path::Path::new(&ffmpeg_path).parent() {
            return Some(dir.to_string_lossy().into_owned());
        }
    }
    // Fall back to well-known static locations
    let candidates = [
        "/opt/homebrew/bin/ffmpeg", // Homebrew Apple Silicon
        "/usr/local/bin/ffmpeg",    // Homebrew Intel
        "/usr/bin/ffmpeg",          // Linux / system
    ];
    candidates
        .iter()
        .find(|&&p| std::path::Path::new(p).exists())
        .and_then(|&p| std::path::Path::new(p).parent())
        .map(|dir| dir.to_string_lossy().into_owned())
}

/// Returns a path to a Node.js binary if one exists on the system.
fn find_node() -> Option<String> {
    // Search PATH first (works when nvm is active in the current shell)
    if let Some(node) = which_bin("node").or_else(|| which_bin("nodejs")) {
        return Some(node);
    }
    // Fall back to version-manager directories (handles GUI app launches where shell init doesn't run)
    if let Some(node) = find_node_from_version_managers() {
        return Some(node);
    }
    // Fall back to well-known static locations
    let candidates = [
        "/opt/homebrew/bin/node", // Homebrew Apple Silicon
        "/usr/local/bin/node",    // Homebrew Intel / nvm default
        "/usr/bin/node",          // Linux system
        "/usr/bin/nodejs",        // Debian/Ubuntu alias
    ];
    candidates
        .iter()
        .find(|&&p| std::path::Path::new(p).exists())
        .map(|&s| s.to_string())
}

/// Searches PATH for a binary, returning its full path if found.
fn which_bin(name: &str) -> Option<String> {
    let path_var = std::env::var("PATH").ok()?;
    #[cfg(target_os = "windows")]
    let sep = ';';
    #[cfg(not(target_os = "windows"))]
    let sep = ':';
    for dir in path_var.split(sep) {
        let candidate = std::path::Path::new(dir).join(name);
        if candidate.is_file() {
            return Some(candidate.to_string_lossy().into_owned());
        }
    }
    None
}

/// Searches nvm/fnm/volta version manager directories for a Node.js binary.
fn find_node_from_version_managers() -> Option<String> {
    let home = std::env::var("HOME").ok()?;
    // nvm: $NVM_BIN or glob $HOME/.nvm/versions/node/*/bin/node (pick newest by sorting)
    if let Ok(nvm_bin) = std::env::var("NVM_BIN") {
        let candidate = std::path::Path::new(&nvm_bin).join("node");
        if candidate.is_file() {
            return Some(candidate.to_string_lossy().into_owned());
        }
    }
    // Fallback: find newest nvm node by listing version dirs
    let nvm_versions = std::path::Path::new(&home).join(".nvm/versions/node");
    if nvm_versions.is_dir() {
        if let Ok(entries) = std::fs::read_dir(&nvm_versions) {
            let mut versions: Vec<std::path::PathBuf> = entries
                .filter_map(|e| e.ok().map(|e| e.path()))
                .filter(|p| p.join("bin/node").is_file())
                .collect();
            versions.sort();
            if let Some(newest) = versions.last() {
                return Some(newest.join("bin/node").to_string_lossy().into_owned());
            }
        }
    }
    // volta: $VOLTA_HOME/bin/node
    if let Ok(volta_home) = std::env::var("VOLTA_HOME") {
        let candidate = std::path::Path::new(&volta_home).join("bin/node");
        if candidate.is_file() {
            return Some(candidate.to_string_lossy().into_owned());
        }
    }
    None
}

fn build_args(req: &DownloadRequest) -> Vec<String> {
    let mut args: Vec<String> = vec![
        "--newline".into(),
        "--no-colors".into(),
        "--encoding".into(),
        "utf-8".into(),
        "--no-playlist".into(),
        "--progress".into(),
        "--progress-template".into(),
        "download:YTDLP_PROG:%(progress._percent_str)s:%(progress._total_bytes_str)s:%(progress._speed_str)s:%(progress._eta_str)s".into(),
        // Strip characters illegal on Windows/macOS/Linux instead of replacing with fullwidth unicode
        "--replace-in-metadata".into(),
        "title".into(),
        "[/\\\\:*?\"<>|]".into(),
        "".into(),
        // Trim leading/trailing dots and spaces (Windows rejects filenames ending with them)
        "--replace-in-metadata".into(),
        "title".into(),
        "^[\\s.]+|[\\s.]+$".into(),
        "".into(),
        "-o".into(),
        format!("{}/%(title)s.%(ext)s", req.output_dir),
    ];

    if req.audio_only {
        args.extend([
            "--extract-audio".into(),
            "--audio-format".into(),
            "mp3".into(),
            "--audio-quality".into(),
            "0".into(),
        ]);
    } else {
        let format = match req.resolution.as_str() {
            "best" | "" => "bestvideo+bestaudio/best".into(),
            res => format!(
                "bestvideo[height<={}]+bestaudio/best[height<={}]",
                res, res
            ),
        };
        args.extend(["--format".into(), format, "--merge-output-format".into(), "mp4".into()]);
    }

    // Embed thumbnail and metadata
    args.extend([
        "--embed-thumbnail".into(),
        "--add-metadata".into(),
    ]);

    // Print video title before download starts
    args.extend([
        "--print".into(),
        "before_dl:YTDL_TITLE:%(title)s".into(),
    ]);

    // Print final file path after all post-processing (merge, conversion, etc.)
    args.extend([
        "--print".into(),
        "after_move:YTDL_FINAL:%(filepath)s".into(),
    ]);

    // Pass ffmpeg directory so yt-dlp can find both ffmpeg and ffprobe
    if let Some(dir) = find_ffmpeg_dir() {
        args.extend(["--ffmpeg-location".into(), dir]);
    }

    // Pass a JS runtime so yt-dlp can extract all YouTube formats
    if let Some(node) = find_node() {
        args.extend(["--js-runtimes".into(), format!("node:{node}")]);
    }

    // Use browser cookies to bypass bot detection / age gates
    if !req.cookies_browser.is_empty() {
        args.extend(["--cookies-from-browser".into(), req.cookies_browser.clone()]);
    }

    args.push(req.url.clone());
    args
}
