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

fn find_ffmpeg() -> Option<String> {
    // Prefer the bundled sidecar (placed alongside the executable by Tauri)
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            for name in &["ffmpeg.exe", "ffmpeg"] {
                let bundled = dir.join(name);
                if bundled.exists() {
                    return Some(bundled.to_string_lossy().into_owned());
                }
            }
        }
    }
    // Fall back to system installations
    let candidates = [
        "/opt/homebrew/bin/ffmpeg", // Homebrew Apple Silicon
        "/usr/local/bin/ffmpeg",    // Homebrew Intel
        "/usr/bin/ffmpeg",          // Linux / system
    ];
    candidates
        .iter()
        .find(|&&p| std::path::Path::new(p).exists())
        .map(|&s| s.to_string())
}

fn build_args(req: &DownloadRequest) -> Vec<String> {
    let mut args: Vec<String> = vec![
        "--newline".into(),
        "--no-colors".into(),
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

    // Pass explicit ffmpeg path so it's found when running inside the app bundle
    if let Some(ffmpeg) = find_ffmpeg() {
        args.extend(["--ffmpeg-location".into(), ffmpeg]);
    }

    // Use browser cookies to bypass bot detection / age gates
    if !req.cookies_browser.is_empty() {
        args.extend(["--cookies-from-browser".into(), req.cookies_browser.clone()]);
    }

    args.push(req.url.clone());
    args
}
