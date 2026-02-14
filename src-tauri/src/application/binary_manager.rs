//! Binary Manager - Runtime download and management of yt-dlp & ffmpeg
//!
//! Instead of bundling ~110MB of binaries as Tauri sidecars, this module
//! downloads them on first use to `app_data_dir/binaries/` and manages
//! version tracking and updates.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tauri::{AppHandle, Emitter, Manager};

// ============================================================================
// Types
// ============================================================================

/// Tracks installed binary versions (persisted as JSON)
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BinaryManifest {
    pub ytdlp_version: Option<String>,
    pub ffmpeg_installed: bool,
    #[serde(default)]
    pub deno_installed: bool,
}

/// Status check result sent to the frontend
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BinaryStatus {
    pub ytdlp_installed: bool,
    pub ffmpeg_installed: bool,
    pub deno_installed: bool,
    pub all_installed: bool,
}

/// Progress event payload for `binary-download-progress`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BinaryDownloadProgress {
    pub binary: String,
    pub downloaded: u64,
    pub total: Option<u64>,
    pub done: bool,
}

// ============================================================================
// Path Helpers
// ============================================================================

/// Root directory for downloaded binaries: `app_data_dir/binaries/`
pub fn binaries_dir(app: &AppHandle) -> Result<PathBuf, String> {
    let dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("Failed to get app data dir: {}", e))?
        .join("binaries");
    std::fs::create_dir_all(&dir).map_err(|e| format!("Failed to create binaries dir: {}", e))?;
    Ok(dir)
}

/// Path to the yt-dlp binary
pub fn ytdlp_path(app: &AppHandle) -> Result<PathBuf, String> {
    let name = if cfg!(windows) {
        "yt-dlp.exe"
    } else {
        "yt-dlp"
    };
    Ok(binaries_dir(app)?.join(name))
}

/// Path to the ffmpeg binary
pub fn ffmpeg_path(app: &AppHandle) -> Result<PathBuf, String> {
    let name = if cfg!(windows) {
        "ffmpeg.exe"
    } else {
        "ffmpeg"
    };
    Ok(binaries_dir(app)?.join(name))
}

/// Path to the deno binary (JS runtime for yt-dlp)
pub fn deno_path(app: &AppHandle) -> Result<PathBuf, String> {
    let name = if cfg!(windows) { "deno.exe" } else { "deno" };
    Ok(binaries_dir(app)?.join(name))
}

fn manifest_path(app: &AppHandle) -> Result<PathBuf, String> {
    Ok(binaries_dir(app)?.join("manifest.json"))
}

// ============================================================================
// Manifest I/O
// ============================================================================

pub fn load_manifest(app: &AppHandle) -> Result<BinaryManifest, String> {
    let path = manifest_path(app)?;
    if !path.exists() {
        return Ok(BinaryManifest::default());
    }
    let data =
        std::fs::read_to_string(&path).map_err(|e| format!("Failed to read manifest: {}", e))?;
    serde_json::from_str(&data).map_err(|e| format!("Failed to parse manifest: {}", e))
}

pub fn save_manifest(app: &AppHandle, manifest: &BinaryManifest) -> Result<(), String> {
    let path = manifest_path(app)?;
    let data = serde_json::to_string_pretty(manifest)
        .map_err(|e| format!("Failed to serialize manifest: {}", e))?;
    std::fs::write(&path, data).map_err(|e| format!("Failed to write manifest: {}", e))
}

// ============================================================================
// Status Check
// ============================================================================

pub fn check_binaries_installed(app: &AppHandle) -> Result<BinaryStatus, String> {
    let ytdlp = ytdlp_path(app)?.exists();
    let ffmpeg = ffmpeg_path(app)?.exists();
    let deno = deno_path(app)?.exists();
    Ok(BinaryStatus {
        ytdlp_installed: ytdlp,
        ffmpeg_installed: ffmpeg,
        deno_installed: deno,
        all_installed: ytdlp && ffmpeg && deno,
    })
}

// ============================================================================
// Download URLs
// ============================================================================

fn ytdlp_download_url() -> &'static str {
    #[cfg(target_os = "windows")]
    {
        "https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp.exe"
    }
    #[cfg(target_os = "macos")]
    {
        "https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp_macos"
    }
    #[cfg(target_os = "linux")]
    {
        "https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp_linux"
    }
}

fn deno_download_url() -> &'static str {
    #[cfg(target_os = "windows")]
    {
        "https://github.com/denoland/deno/releases/latest/download/deno-x86_64-pc-windows-msvc.zip"
    }
    #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
    {
        "https://github.com/denoland/deno/releases/latest/download/deno-aarch64-apple-darwin.zip"
    }
    #[cfg(all(target_os = "macos", target_arch = "x86_64"))]
    {
        "https://github.com/denoland/deno/releases/latest/download/deno-x86_64-apple-darwin.zip"
    }
    #[cfg(target_os = "linux")]
    {
        "https://github.com/denoland/deno/releases/latest/download/deno-x86_64-unknown-linux-gnu.zip"
    }
}

fn ffmpeg_download_url() -> &'static str {
    #[cfg(target_os = "windows")]
    {
        "https://github.com/BtbN/FFmpeg-Builds/releases/download/latest/ffmpeg-master-latest-win64-gpl.zip"
    }
    #[cfg(target_os = "macos")]
    {
        "https://evermeet.cx/ffmpeg/getrelease/ffmpeg/zip"
    }
    #[cfg(target_os = "linux")]
    {
        "https://johnvansickle.com/ffmpeg/releases/ffmpeg-release-amd64-static.tar.xz"
    }
}

// ============================================================================
// Download Helpers
// ============================================================================

/// Download a file with streaming progress events.
/// Returns the bytes of the downloaded file.
async fn download_with_progress(
    app: &AppHandle,
    url: &str,
    binary_name: &str,
) -> Result<Vec<u8>, String> {
    use futures::StreamExt;

    let client = reqwest::Client::new();
    let response = client
        .get(url)
        .send()
        .await
        .map_err(|e| format!("Failed to download {}: {}", binary_name, e))?;

    if !response.status().is_success() {
        return Err(format!(
            "Download {} failed with status: {}",
            binary_name,
            response.status()
        ));
    }

    let total = response.content_length();
    let mut stream = response.bytes_stream();
    let mut downloaded: u64 = 0;
    let mut buffer = Vec::new();

    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|e| format!("Stream error for {}: {}", binary_name, e))?;
        downloaded += chunk.len() as u64;
        buffer.extend_from_slice(&chunk);

        let _ = app.emit(
            "binary-download-progress",
            BinaryDownloadProgress {
                binary: binary_name.to_string(),
                downloaded,
                total,
                done: false,
            },
        );
    }

    let _ = app.emit(
        "binary-download-progress",
        BinaryDownloadProgress {
            binary: binary_name.to_string(),
            downloaded,
            total,
            done: true,
        },
    );

    Ok(buffer)
}

/// Atomically write bytes to a file (write to .tmp, then rename)
fn atomic_write(path: &PathBuf, data: &[u8]) -> Result<(), String> {
    let tmp_path = path.with_extension("tmp");
    std::fs::write(&tmp_path, data).map_err(|e| format!("Failed to write temp file: {}", e))?;
    std::fs::rename(&tmp_path, path).map_err(|e| format!("Failed to rename temp file: {}", e))?;
    Ok(())
}

/// Set executable permission on Unix
#[cfg(unix)]
fn set_executable(path: &PathBuf) -> Result<(), String> {
    use std::os::unix::fs::PermissionsExt;
    let perms = std::fs::Permissions::from_mode(0o755);
    std::fs::set_permissions(path, perms)
        .map_err(|e| format!("Failed to set executable permission: {}", e))
}

#[cfg(not(unix))]
fn set_executable(_path: &PathBuf) -> Result<(), String> {
    Ok(())
}

// ============================================================================
// Download Functions
// ============================================================================

/// Download yt-dlp (direct binary, no extraction needed)
pub async fn download_ytdlp(app: &AppHandle) -> Result<(), String> {
    let url = ytdlp_download_url();
    let dest = ytdlp_path(app)?;

    tracing::info!(url = url, dest = ?dest, "Downloading yt-dlp");

    let data = download_with_progress(app, url, "yt-dlp").await?;
    atomic_write(&dest, &data)?;
    set_executable(&dest)?;

    // Try to extract version from the GitHub redirect URL tag
    // We'll store "latest" for now and update on version check
    let mut manifest = load_manifest(app)?;
    manifest.ytdlp_version = Some("latest".to_string());
    save_manifest(app, &manifest)?;

    tracing::info!("yt-dlp downloaded successfully");
    Ok(())
}

/// Download ffmpeg (requires extraction from archive)
pub async fn download_ffmpeg(app: &AppHandle) -> Result<(), String> {
    let url = ffmpeg_download_url();
    let dest = ffmpeg_path(app)?;

    tracing::info!(url = url, dest = ?dest, "Downloading ffmpeg");

    let data = download_with_progress(app, url, "ffmpeg").await?;

    // Extract based on platform
    extract_ffmpeg(&data, &dest)?;
    set_executable(&dest)?;

    let mut manifest = load_manifest(app)?;
    manifest.ffmpeg_installed = true;
    save_manifest(app, &manifest)?;

    tracing::info!("ffmpeg downloaded successfully");
    Ok(())
}

/// Extract ffmpeg binary from the downloaded archive
fn extract_ffmpeg(archive_data: &[u8], dest: &PathBuf) -> Result<(), String> {
    #[cfg(target_os = "linux")]
    {
        extract_ffmpeg_tar_xz(archive_data, dest)
    }
    #[cfg(not(target_os = "linux"))]
    {
        extract_ffmpeg_zip(archive_data, dest)
    }
}

/// Extract ffmpeg from a ZIP archive (Windows/macOS)
#[cfg(not(target_os = "linux"))]
fn extract_ffmpeg_zip(archive_data: &[u8], dest: &PathBuf) -> Result<(), String> {
    use std::io::{Cursor, Read};

    let cursor = Cursor::new(archive_data);
    let mut archive =
        zip::ZipArchive::new(cursor).map_err(|e| format!("Failed to open ffmpeg ZIP: {}", e))?;

    // Find the ffmpeg binary inside the archive
    let ffmpeg_name = if cfg!(windows) {
        "ffmpeg.exe"
    } else {
        "ffmpeg"
    };

    for i in 0..archive.len() {
        let mut file = archive
            .by_index(i)
            .map_err(|e| format!("ZIP entry error: {}", e))?;

        let name = file.name().to_string();
        // Match "ffmpeg" or "ffmpeg.exe" at end of path (could be nested in dirs)
        if name.ends_with(ffmpeg_name)
            && !name.ends_with("ffprobe.exe")
            && !name.ends_with("ffprobe")
            && !name.ends_with("ffplay.exe")
            && !name.ends_with("ffplay")
        {
            let mut data = Vec::new();
            file.read_to_end(&mut data)
                .map_err(|e| format!("Failed to read ffmpeg from ZIP: {}", e))?;
            atomic_write(dest, &data)?;
            return Ok(());
        }
    }

    Err("ffmpeg binary not found in ZIP archive".to_string())
}

/// Extract ffmpeg from a tar.xz archive (Linux)
#[cfg(target_os = "linux")]
fn extract_ffmpeg_tar_xz(archive_data: &[u8], dest: &PathBuf) -> Result<(), String> {
    use std::io::{Cursor, Read};

    let cursor = Cursor::new(archive_data);
    let xz_decoder = xz2::read::XzDecoder::new(cursor);
    let mut archive = tar::Archive::new(xz_decoder);

    let entries = archive
        .entries()
        .map_err(|e| format!("Failed to read tar entries: {}", e))?;

    for entry in entries {
        let mut entry = entry.map_err(|e| format!("Tar entry error: {}", e))?;
        let path = entry
            .path()
            .map_err(|e| format!("Tar path error: {}", e))?
            .to_path_buf();

        if let Some(file_name) = path.file_name() {
            if file_name == "ffmpeg" {
                let mut data = Vec::new();
                entry
                    .read_to_end(&mut data)
                    .map_err(|e| format!("Failed to read ffmpeg from tar: {}", e))?;
                atomic_write(dest, &data)?;
                return Ok(());
            }
        }
    }

    Err("ffmpeg binary not found in tar.xz archive".to_string())
}

/// Download deno JS runtime (required by yt-dlp for YouTube extraction)
pub async fn download_deno(app: &AppHandle) -> Result<(), String> {
    let url = deno_download_url();
    let dest = deno_path(app)?;

    tracing::info!(url = url, dest = ?dest, "Downloading deno");

    let data = download_with_progress(app, url, "deno").await?;
    extract_deno(&data, &dest)?;
    set_executable(&dest)?;

    let mut manifest = load_manifest(app)?;
    manifest.deno_installed = true;
    save_manifest(app, &manifest)?;

    tracing::info!("deno downloaded successfully");
    Ok(())
}

/// Extract deno binary from ZIP archive
fn extract_deno(archive_data: &[u8], dest: &PathBuf) -> Result<(), String> {
    use std::io::{Cursor, Read};

    let cursor = Cursor::new(archive_data);
    let mut archive =
        zip::ZipArchive::new(cursor).map_err(|e| format!("Failed to open deno ZIP: {}", e))?;

    let deno_name = if cfg!(windows) { "deno.exe" } else { "deno" };

    for i in 0..archive.len() {
        let mut file = archive
            .by_index(i)
            .map_err(|e| format!("ZIP entry error: {}", e))?;

        if file.name().ends_with(deno_name) {
            let mut data = Vec::new();
            file.read_to_end(&mut data)
                .map_err(|e| format!("Failed to read deno from ZIP: {}", e))?;
            atomic_write(dest, &data)?;
            return Ok(());
        }
    }

    Err("deno binary not found in ZIP archive".to_string())
}

/// Download yt-dlp, ffmpeg, and deno in parallel
pub async fn download_all_binaries(app: &AppHandle) -> Result<(), String> {
    let (ytdlp_result, ffmpeg_result, deno_result) = tokio::join!(
        download_ytdlp(app),
        download_ffmpeg(app),
        download_deno(app)
    );

    ytdlp_result?;
    ffmpeg_result?;
    deno_result?;

    Ok(())
}

// ============================================================================
// Update Check
// ============================================================================

/// GitHub API response for latest release
#[derive(Debug, Deserialize)]
struct GitHubRelease {
    tag_name: String,
}

/// Check if a yt-dlp update is available.
/// Returns `Some(new_version)` if an update exists, `None` otherwise.
pub async fn check_ytdlp_update(app: &AppHandle) -> Result<Option<String>, String> {
    let manifest = load_manifest(app)?;
    let current_version = match &manifest.ytdlp_version {
        Some(v) => v.clone(),
        None => return Ok(None), // Not installed
    };

    let client = reqwest::Client::builder()
        .user_agent("voiceboard")
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

    let release: GitHubRelease = client
        .get("https://api.github.com/repos/yt-dlp/yt-dlp/releases/latest")
        .send()
        .await
        .map_err(|e| format!("Failed to check yt-dlp update: {}", e))?
        .json()
        .await
        .map_err(|e| format!("Failed to parse GitHub release: {}", e))?;

    if release.tag_name != current_version {
        Ok(Some(release.tag_name))
    } else {
        Ok(None)
    }
}

/// Re-download yt-dlp to update it
pub async fn update_ytdlp(app: &AppHandle) -> Result<(), String> {
    download_ytdlp(app).await?;

    // Fetch and store the actual version
    let client = reqwest::Client::builder()
        .user_agent("voiceboard")
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

    if let Ok(response) = client
        .get("https://api.github.com/repos/yt-dlp/yt-dlp/releases/latest")
        .send()
        .await
    {
        if let Ok(release) = response.json::<GitHubRelease>().await {
            let mut manifest = load_manifest(app)?;
            manifest.ytdlp_version = Some(release.tag_name);
            save_manifest(app, &manifest)?;
        }
    }

    Ok(())
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_binary_manifest_default() {
        let manifest = BinaryManifest::default();
        assert!(manifest.ytdlp_version.is_none());
        assert!(!manifest.ffmpeg_installed);
    }

    #[test]
    fn test_binary_manifest_serialization() {
        let manifest = BinaryManifest {
            ytdlp_version: Some("2024.01.01".to_string()),
            ffmpeg_installed: true,
            deno_installed: true,
        };

        let json = serde_json::to_string(&manifest).unwrap();
        let parsed: BinaryManifest = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.ytdlp_version, Some("2024.01.01".to_string()));
        assert!(parsed.ffmpeg_installed);
    }

    #[test]
    fn test_download_urls_are_https() {
        let ytdlp_url = ytdlp_download_url();
        assert!(
            ytdlp_url.starts_with("https://"),
            "yt-dlp URL must be HTTPS"
        );
        assert!(!ytdlp_url.is_empty());

        let ffmpeg_url = ffmpeg_download_url();
        assert!(
            ffmpeg_url.starts_with("https://"),
            "ffmpeg URL must be HTTPS"
        );
        assert!(!ffmpeg_url.is_empty());
    }

    #[test]
    fn test_binary_extension() {
        if cfg!(windows) {
            assert!(
                ytdlp_download_url().ends_with(".exe"),
                "Windows yt-dlp should have .exe extension"
            );
        } else {
            assert!(
                !ytdlp_download_url().ends_with(".exe"),
                "Non-Windows yt-dlp should not have .exe extension"
            );
        }
    }
}
