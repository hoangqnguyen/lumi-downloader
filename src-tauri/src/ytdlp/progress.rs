use once_cell::sync::Lazy;
use regex::Regex;
use serde::{Deserialize, Serialize};

static MERGER_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"\[Merger\]|Merging formats").unwrap()
});

static ALREADY_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"\[download\] .+ has already been downloaded").unwrap()
});

static FINAL_PATH_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^YTDL_FINAL:(.+)$").unwrap()
});

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProgressEvent {
    pub percent: f32,
    pub size: String,
    pub speed: String,
    pub eta: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum ParsedLine {
    Progress(ProgressEvent),
    FinalPath(String),
    Merging,
    AlreadyDownloaded,
    Other(String),
}

pub fn parse_line(line: &str) -> ParsedLine {
    // Custom tagged progress line: YTDLP_PROG:45.3%:47.23MiB:2.34MiB/s:00:19
    if let Some(rest) = line.strip_prefix("YTDLP_PROG:") {
        let parts: Vec<&str> = rest.splitn(4, ':').collect();
        if parts.len() == 4 {
            let percent = parts[0]
                .trim()
                .trim_end_matches('%')
                .parse::<f32>()
                .unwrap_or(0.0);
            return ParsedLine::Progress(ProgressEvent {
                percent,
                size: parts[1].trim().to_string(),
                speed: parts[2].trim().to_string(),
                eta: parts[3].trim().to_string(),
            });
        }
    }

    if let Some(caps) = FINAL_PATH_RE.captures(line) {
        return ParsedLine::FinalPath(caps[1].trim().to_string());
    }

    if MERGER_RE.is_match(line) {
        return ParsedLine::Merging;
    }

    if ALREADY_RE.is_match(line) {
        return ParsedLine::AlreadyDownloaded;
    }

    ParsedLine::Other(line.to_string())
}
