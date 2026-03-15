use std::path::PathBuf;

/// Returns the directory where downloaded binaries are stored.
/// macOS:   ~/Library/Application Support/io.github.lumidownloader.app/bin
/// Linux:   ~/.local/share/io.github.lumidownloader.app/bin
/// Windows: %APPDATA%/io.github.lumidownloader.app/bin
pub fn bin_dir() -> PathBuf {
    let base = dirs::data_dir().unwrap_or_else(|| PathBuf::from("."));
    base.join("io.github.lumidownloader.app").join("bin")
}

/// Returns the expected path for a given binary name.
pub fn bin_path(name: &str) -> PathBuf {
    let p = bin_dir().join(name);
    #[cfg(target_os = "windows")]
    {
        return p.with_extension("exe");
    }
    #[cfg(not(target_os = "windows"))]
    p
}

/// Check if yt-dlp is available.
pub fn has_ytdlp() -> bool {
    bin_path("yt-dlp").is_file()
}

/// Check if ffmpeg is available (downloaded or on system).
pub fn has_ffmpeg() -> bool {
    bin_path("ffmpeg").is_file() || find_system_ffmpeg().is_some()
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

/// Find ffmpeg on the system PATH or well-known locations.
pub fn find_system_ffmpeg() -> Option<String> {
    if let Some(p) = which_bin("ffmpeg") {
        return Some(p);
    }
    let candidates = [
        "/opt/homebrew/bin/ffmpeg",
        "/usr/local/bin/ffmpeg",
        "/usr/bin/ffmpeg",
    ];
    candidates
        .iter()
        .find(|&&p| std::path::Path::new(p).exists())
        .map(|&s| s.to_string())
}

/// Returns the directory containing ffmpeg for --ffmpeg-location.
pub fn ffmpeg_dir() -> Option<String> {
    // Prefer our downloaded copy
    let downloaded = bin_path("ffmpeg");
    if downloaded.is_file() {
        return downloaded.parent().map(|d| d.to_string_lossy().into_owned());
    }
    // Fall back to system
    find_system_ffmpeg()
        .and_then(|p| std::path::Path::new(&p).parent().map(|d| d.to_string_lossy().into_owned()))
}

/// Download yt-dlp and ffmpeg binaries. Returns progress via callback.
pub async fn download_binaries(
    on_progress: impl Fn(&str) + Send + 'static,
) -> Result<(), String> {
    let dir = bin_dir();
    std::fs::create_dir_all(&dir).map_err(|e| format!("Failed to create bin dir: {e}"))?;

    // Download yt-dlp
    on_progress("Downloading yt-dlp...");
    download_ytdlp(&dir).await?;

    // Download ffmpeg + ffprobe (skip if system ffmpeg exists)
    if find_system_ffmpeg().is_none() {
        on_progress("Downloading ffmpeg...");
        download_ffmpeg(&dir).await?;
    }

    on_progress("Ready!");
    Ok(())
}

async fn download_ytdlp(dir: &PathBuf) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    let url = "https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp_macos";
    #[cfg(target_os = "linux")]
    let url = "https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp_linux";
    #[cfg(target_os = "windows")]
    let url = "https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp.exe";

    let dest = dir.join(ytdlp_filename());
    download_file(url, &dest).await?;
    set_executable(&dest)?;
    Ok(())
}

async fn download_ffmpeg(dir: &PathBuf) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        // Download ffmpeg
        let ffmpeg_url = "https://evermeet.cx/ffmpeg/getrelease/zip";
        let zip_path = dir.join("ffmpeg.zip");
        download_file(ffmpeg_url, &zip_path).await?;
        extract_zip_single(&zip_path, dir, "ffmpeg")?;
        std::fs::remove_file(&zip_path).ok();

        // Download ffprobe
        let ffprobe_url = "https://evermeet.cx/ffmpeg/getrelease/ffprobe/zip";
        let zip_path = dir.join("ffprobe.zip");
        download_file(ffprobe_url, &zip_path).await?;
        extract_zip_single(&zip_path, dir, "ffprobe")?;
        std::fs::remove_file(&zip_path).ok();
    }

    #[cfg(target_os = "windows")]
    {
        let url = "https://www.gyan.dev/ffmpeg/builds/ffmpeg-release-essentials.zip";
        let zip_path = dir.join("ffmpeg.zip");
        download_file(url, &zip_path).await?;
        extract_ffmpeg_windows(&zip_path, dir)?;
        std::fs::remove_file(&zip_path).ok();
    }

    #[cfg(target_os = "linux")]
    {
        // On Linux, we expect system ffmpeg. This shouldn't be called.
        return Err("Please install ffmpeg via your package manager (e.g. sudo apt install ffmpeg)".into());
    }

    Ok(())
}

fn ytdlp_filename() -> &'static str {
    #[cfg(target_os = "windows")]
    return "yt-dlp.exe";
    #[cfg(not(target_os = "windows"))]
    return "yt-dlp";
}

async fn download_file(url: &str, dest: &PathBuf) -> Result<(), String> {
    let response = reqwest::get(url)
        .await
        .map_err(|e| format!("Download failed: {e}"))?;

    if !response.status().is_success() {
        return Err(format!("Download failed with status: {}", response.status()));
    }

    let bytes = response
        .bytes()
        .await
        .map_err(|e| format!("Failed to read response: {e}"))?;

    std::fs::write(dest, &bytes).map_err(|e| format!("Failed to write file: {e}"))?;
    Ok(())
}

#[cfg(unix)]
fn set_executable(path: &PathBuf) -> Result<(), String> {
    use std::os::unix::fs::PermissionsExt;
    let perms = std::fs::Permissions::from_mode(0o755);
    std::fs::set_permissions(path, perms).map_err(|e| format!("Failed to set permissions: {e}"))
}

#[cfg(windows)]
fn set_executable(_path: &PathBuf) -> Result<(), String> {
    Ok(())
}

#[cfg(target_os = "macos")]
fn extract_zip_single(zip_path: &PathBuf, dest_dir: &PathBuf, binary_name: &str) -> Result<(), String> {
    let file = std::fs::File::open(zip_path).map_err(|e| format!("Failed to open zip: {e}"))?;
    let mut archive = zip::ZipArchive::new(file).map_err(|e| format!("Failed to read zip: {e}"))?;

    for i in 0..archive.len() {
        let mut entry = archive.by_index(i).map_err(|e| format!("Zip entry error: {e}"))?;
        let name = entry.name().to_string();
        if name == binary_name || name.ends_with(&format!("/{binary_name}")) {
            let dest = dest_dir.join(binary_name);
            let mut outfile = std::fs::File::create(&dest).map_err(|e| format!("Failed to create file: {e}"))?;
            std::io::copy(&mut entry, &mut outfile).map_err(|e| format!("Failed to extract: {e}"))?;
            set_executable(&dest)?;
            return Ok(());
        }
    }
    Err(format!("{binary_name} not found in zip"))
}

#[cfg(target_os = "windows")]
fn extract_ffmpeg_windows(zip_path: &PathBuf, dest_dir: &PathBuf) -> Result<(), String> {
    let file = std::fs::File::open(zip_path).map_err(|e| format!("Failed to open zip: {e}"))?;
    let mut archive = zip::ZipArchive::new(file).map_err(|e| format!("Failed to read zip: {e}"))?;

    for i in 0..archive.len() {
        let mut entry = archive.by_index(i).map_err(|e| format!("Zip entry error: {e}"))?;
        let name = entry.name().to_string();
        for bin in &["ffmpeg.exe", "ffprobe.exe"] {
            if name.ends_with(bin) {
                let dest = dest_dir.join(bin);
                let mut outfile = std::fs::File::create(&dest).map_err(|e| format!("Failed to create file: {e}"))?;
                std::io::copy(&mut entry, &mut outfile).map_err(|e| format!("Failed to extract: {e}"))?;
            }
        }
    }
    Ok(())
}
