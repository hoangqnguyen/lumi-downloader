use crate::binaries;
use crate::ytdlp::progress::{parse_line, ParsedLine};
use dashmap::{DashMap, DashSet};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::{AppHandle, Emitter};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::{Child, Command};

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
    children: Arc<DashMap<String, Child>>,
    cancelled: Arc<DashSet<String>>,
) -> Result<(), String> {
    let args = build_args(&req);
    let ytdlp_path = binaries::bin_path("yt-dlp");

    let mut child = Command::new(&ytdlp_path)
        .args(&args)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| format!("Failed to spawn yt-dlp: {e}"))?;

    let stdout = child.stdout.take().unwrap();
    let stderr = child.stderr.take().unwrap();

    children.insert(req.job_id.clone(), child);

    let mut error_lines: Vec<String> = Vec::new();

    let stdout_reader = BufReader::new(stdout);
    let stderr_reader = BufReader::new(stderr);

    let mut stdout_lines = stdout_reader.lines();
    let mut stderr_lines = stderr_reader.lines();

    loop {
        tokio::select! {
            line = stdout_lines.next_line() => {
                match line {
                    Ok(Some(text)) => {
                        for line in text.lines() {
                            let line = line.trim();
                            process_line(&app, &req.job_id, line, &mut error_lines, false);
                        }
                    }
                    Ok(None) => break,
                    Err(_) => break,
                }
            }
            line = stderr_lines.next_line() => {
                match line {
                    Ok(Some(text)) => {
                        for line in text.lines() {
                            let line = line.trim();
                            process_line(&app, &req.job_id, line, &mut error_lines, true);
                        }
                    }
                    Ok(None) => {}
                    Err(_) => {}
                }
            }
        }
    }

    // Wait for the process to finish
    let status = if let Some((_, mut child)) = children.remove(&req.job_id) {
        child.wait().await.ok()
    } else {
        None
    };

    // Don't emit Done for cancelled jobs
    if cancelled.remove(&req.job_id).is_some() {
        return Ok(());
    }

    let success = status.map(|s| s.success()).unwrap_or(false);
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

    Ok(())
}

fn process_line(app: &AppHandle, job_id: &str, line: &str, error_lines: &mut Vec<String>, is_stderr: bool) {
    match parse_line(line) {
        ParsedLine::Progress(p) => {
            let _ = app.emit(
                "job-event",
                JobEvent {
                    job_id: job_id.to_string(),
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
                    job_id: job_id.to_string(),
                    payload: JobEventPayload::Merging,
                },
            );
        }
        ParsedLine::Title(title) => {
            let _ = app.emit(
                "job-event",
                JobEvent {
                    job_id: job_id.to_string(),
                    payload: JobEventPayload::Title { title },
                },
            );
        }
        ParsedLine::FinalPath(path) => {
            let _ = app.emit(
                "job-event",
                JobEvent {
                    job_id: job_id.to_string(),
                    payload: JobEventPayload::FilePath { path },
                },
            );
        }
        ParsedLine::AlreadyDownloaded => {
            let _ = app.emit(
                "job-event",
                JobEvent {
                    job_id: job_id.to_string(),
                    payload: JobEventPayload::Done {
                        success: true,
                        error: None,
                    },
                },
            );
        }
        _ => {
            if is_stderr && !line.is_empty() {
                error_lines.push(line.to_string());
            }
        }
    }
}

/// Returns the directory containing ffmpeg (and ffprobe) so yt-dlp can find both.
fn find_ffmpeg_dir() -> Option<String> {
    binaries::ffmpeg_dir()
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
    if let Ok(nvm_bin) = std::env::var("NVM_BIN") {
        let candidate = std::path::Path::new(&nvm_bin).join("node");
        if candidate.is_file() {
            return Some(candidate.to_string_lossy().into_owned());
        }
    }
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
        "--replace-in-metadata".into(),
        "title".into(),
        "[/\\\\:*?\"<>|]".into(),
        "".into(),
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

    args.extend([
        "--embed-thumbnail".into(),
        "--add-metadata".into(),
    ]);

    args.extend([
        "--print".into(),
        "before_dl:YTDL_TITLE:%(title)s".into(),
    ]);

    args.extend([
        "--print".into(),
        "after_move:YTDL_FINAL:%(filepath)s".into(),
    ]);

    if let Some(dir) = find_ffmpeg_dir() {
        args.extend(["--ffmpeg-location".into(), dir]);
    }

    if let Some(node) = find_node() {
        args.extend(["--js-runtimes".into(), format!("node:{node}")]);
    }

    if !req.cookies_browser.is_empty() {
        args.extend(["--cookies-from-browser".into(), req.cookies_browser.clone()]);
    }

    args.push(req.url.clone());
    args
}
