use crate::{
    database::{DatabaseState, QueueRecord, StoredDownload},
    download::DownloadManager,
    scheduler::{evaluate_window, ScheduleDecision},
};
use reqwest::header::{ACCEPT_RANGES, CONTENT_LENGTH, CONTENT_RANGE, CONTENT_TYPE, RANGE};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;
use tauri::{command, State};
use url::Url;
use uuid::Uuid;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AddDownloadRequest {
    pub url: String,
    pub destination: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RemoteMetadata {
    pub url: String,
    pub total_bytes: Option<i64>,
    pub content_type: Option<String>,
    pub supports_range: bool,
    pub status_code: u16,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DownloadInfo {
    pub id: String,
    pub url: String,
    pub filename: String,
    pub destination: String,
    pub status: String,
    pub total_bytes: Option<i64>,
    pub content_type: Option<String>,
    pub supports_range: bool,
    pub category: String,
}

#[command]
pub async fn inspect_url(url: String) -> Result<RemoteMetadata, String> {
    let url = validate_url(&url)?;
    inspect_remote_url(url).await
}

#[command]
pub fn get_file_category(filename: String) -> Result<String, String> {
    let filename = filename.trim();
    if filename.is_empty() {
        return Err("Filename is required".to_string());
    }
    Ok(category_for_filename(filename))
}

#[command]
pub async fn add_download(
    request: AddDownloadRequest,
    database: State<'_, DatabaseState>,
    manager: State<'_, DownloadManager>,
) -> Result<DownloadInfo, String> {
    let url = validate_url(&request.url)?;
    let destination = validate_destination(&request.destination)?;
    let filename = filename_from_url(&url);
    let metadata = inspect_remote_url(url.clone()).await?;
    let download = DownloadInfo {
        id: Uuid::new_v4().to_string(),
        url: metadata.url.clone(),
        filename: filename.clone(),
        destination,
        status: "queued".to_string(),
        total_bytes: metadata.total_bytes,
        content_type: metadata.content_type.clone(),
        supports_range: metadata.supports_range,
        category: category_for_filename(&filename),
    };

    let stored = StoredDownload {
        id: download.id.clone(),
        url: download.url.clone(),
        filename: download.filename.clone(),
        destination: download.destination.clone(),
        status: download.status.clone(),
        total_bytes: download.total_bytes,
        downloaded_bytes: 0,
        content_type: download.content_type.clone(),
        supports_range: download.supports_range,
        temp_path: None,
        final_path: None,
        error_message: None,
        speed_bps: 0,
        eta_seconds: 0,
        created_at: String::new(),
        updated_at: String::new(),
    };
    database.insert_download(&stored)?;
    manager.start(stored)?;
    Ok(download)
}

#[command]
pub fn get_downloads(database: State<'_, DatabaseState>) -> Result<Vec<StoredDownload>, String> {
    database.list_downloads()
}

#[command]
pub fn evaluate_queue_schedule(
    start_at: Option<String>,
    stop_at: Option<String>,
) -> Result<String, String> {
    let decision = evaluate_window(
        chrono::Local::now(),
        start_at.as_deref(),
        stop_at.as_deref(),
    )?;
    Ok(match decision {
        ScheduleDecision::Waiting => "waiting",
        ScheduleDecision::Ready => "ready",
        ScheduleDecision::Expired => "expired",
    }
    .to_string())
}

#[command]
pub fn start_queued_downloads(
    database: State<'_, DatabaseState>,
    manager: State<'_, DownloadManager>,
) -> Result<usize, String> {
    let max_concurrent = database
        .get_setting("max_concurrent_downloads")?
        .and_then(|value| value.parse::<i64>().ok())
        .unwrap_or(3)
        .clamp(1, 32);
    manager.start_queued(max_concurrent)
}

#[command]
pub fn get_queues(database: State<'_, DatabaseState>) -> Result<Vec<QueueRecord>, String> {
    database.list_queues()
}

#[command]
pub fn save_queue(queue: QueueRecord, database: State<'_, DatabaseState>) -> Result<(), String> {
    database.upsert_queue(&queue)
}

#[command]
pub fn get_setting(
    key: String,
    database: State<'_, DatabaseState>,
) -> Result<Option<String>, String> {
    database.get_setting(&key)
}

#[command]
pub fn set_setting(
    key: String,
    value: String,
    database: State<'_, DatabaseState>,
) -> Result<(), String> {
    database.set_setting(&key, &value)
}

#[command]
pub fn pause_download(id: String, manager: State<'_, DownloadManager>) -> Result<(), String> {
    manager.pause(&id)
}

#[command]
pub fn resume_download(
    id: String,
    database: State<'_, DatabaseState>,
    manager: State<'_, DownloadManager>,
) -> Result<(), String> {
    let download = database
        .find_download(&id)?
        .ok_or_else(|| "Download not found".to_string())?;
    if !matches!(download.status.as_str(), "paused" | "queued" | "failed") {
        return Err("Download cannot be resumed in its current state".to_string());
    }
    manager.start(download)
}

#[command]
pub fn cancel_download(id: String, manager: State<'_, DownloadManager>) -> Result<(), String> {
    manager.cancel(&id)
}

#[command]
pub fn verify_download_hash(
    id: String,
    expected_sha256: String,
    database: State<'_, DatabaseState>,
) -> Result<bool, String> {
    let expected = expected_sha256.trim().to_ascii_lowercase();
    if expected.len() != 64
        || !expected
            .chars()
            .all(|character| character.is_ascii_hexdigit())
    {
        return Err("Expected SHA-256 must be a 64-character hexadecimal string".to_string());
    }
    let download = database
        .find_download(&id)?
        .ok_or_else(|| "Download not found".to_string())?;
    if download.status != "completed" {
        return Err("Only completed downloads can be verified".to_string());
    }
    let path = download
        .final_path
        .ok_or_else(|| "Download has no finalized file".to_string())?;
    let file =
        File::open(&path).map_err(|error| format!("Could not open finalized file: {error}"))?;
    let mut reader = BufReader::new(file);
    let mut hasher = Sha256::new();
    let mut buffer = [0_u8; 1024 * 1024];
    loop {
        let read = reader
            .read(&mut buffer)
            .map_err(|error| format!("Could not read finalized file: {error}"))?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }
    let actual = format!("{:x}", hasher.finalize());
    Ok(actual == expected)
}

#[command]
pub fn open_download_file(id: String, database: State<'_, DatabaseState>) -> Result<(), String> {
    let download = database
        .find_download(&id)?
        .ok_or_else(|| "Download not found".to_string())?;
    let path = download
        .final_path
        .ok_or_else(|| "Download has no finalized file".to_string())?;
    if !Path::new(&path).is_file() {
        return Err("Finalized file does not exist".to_string());
    }
    #[cfg(target_os = "windows")]
    std::process::Command::new("explorer.exe")
        .args(["/select,", &path])
        .spawn()
        .map_err(|error| format!("Could not open file: {error}"))?;
    #[cfg(not(target_os = "windows"))]
    return Err("Opening files is currently supported on Windows only".to_string());
    Ok(())
}

#[command]
pub fn open_download_folder(id: String, database: State<'_, DatabaseState>) -> Result<(), String> {
    let download = database
        .find_download(&id)?
        .ok_or_else(|| "Download not found".to_string())?;
    let path = download
        .final_path
        .or(download.temp_path)
        .ok_or_else(|| "Download has no file path".to_string())?;
    let folder = Path::new(&path)
        .parent()
        .ok_or_else(|| "Could not resolve download folder".to_string())?;
    #[cfg(target_os = "windows")]
    std::process::Command::new("explorer.exe")
        .arg(folder)
        .spawn()
        .map_err(|error| format!("Could not open folder: {error}"))?;
    #[cfg(not(target_os = "windows"))]
    return Err("Opening folders is currently supported on Windows only".to_string());
    Ok(())
}

#[command]
pub fn delete_download(
    id: String,
    database: State<'_, DatabaseState>,
    manager: State<'_, DownloadManager>,
) -> Result<(), String> {
    let _ = manager.cancel(&id);
    let download = database
        .delete_download(&id)?
        .ok_or_else(|| "Download not found".to_string())?;
    if let Some(path) = download.temp_path.or(download.final_path) {
        let _ = std::fs::remove_file(path);
    }
    Ok(())
}

async fn inspect_remote_url(url: Url) -> Result<RemoteMetadata, String> {
    let client = reqwest::Client::builder()
        .user_agent("ZYNERO/0.1.0")
        .build()
        .map_err(|error| format!("Could not create HTTP client: {error}"))?;
    let head_response = client
        .head(url.clone())
        .send()
        .await
        .map_err(|error| format!("Metadata request failed: {error}"))?;
    if head_response.status().is_client_error() || head_response.status().is_server_error() {
        return Err(format!(
            "Remote server returned HTTP {}",
            head_response.status().as_u16()
        ));
    }

    let mut total_bytes = parse_content_length(&head_response);
    let mut content_type = header_string(&head_response, CONTENT_TYPE);
    let mut supports_range = header_string(&head_response, ACCEPT_RANGES)
        .is_some_and(|value| value.eq_ignore_ascii_case("bytes"));
    let mut status_code = head_response.status().as_u16();

    if total_bytes.is_none() || !supports_range {
        let range_response = client
            .get(url.clone())
            .header(RANGE, "bytes=0-0")
            .send()
            .await
            .map_err(|error| format!("Range metadata request failed: {error}"))?;
        if range_response.status().is_client_error() || range_response.status().is_server_error() {
            return Err(format!(
                "Remote server returned HTTP {}",
                range_response.status().as_u16()
            ));
        }
        total_bytes = total_bytes.or_else(|| parse_content_range_total(&range_response));
        content_type = content_type.or_else(|| header_string(&range_response, CONTENT_TYPE));
        supports_range = supports_range
            || range_response.status().as_u16() == 206
            || header_string(&range_response, ACCEPT_RANGES)
                .is_some_and(|value| value.eq_ignore_ascii_case("bytes"));
        status_code = range_response.status().as_u16();
    }

    Ok(RemoteMetadata {
        url: url.to_string(),
        total_bytes,
        content_type,
        supports_range,
        status_code,
    })
}

fn parse_content_length(response: &reqwest::Response) -> Option<i64> {
    response
        .headers()
        .get(CONTENT_LENGTH)?
        .to_str()
        .ok()?
        .parse()
        .ok()
}

fn parse_content_range_total(response: &reqwest::Response) -> Option<i64> {
    let value = response.headers().get(CONTENT_RANGE)?.to_str().ok()?;
    value.split('/').nth(1)?.parse().ok()
}

fn header_string(
    response: &reqwest::Response,
    name: reqwest::header::HeaderName,
) -> Option<String> {
    response
        .headers()
        .get(name)?
        .to_str()
        .ok()
        .map(ToOwned::to_owned)
}

fn validate_url(raw_url: &str) -> Result<Url, String> {
    let trimmed = raw_url.trim();
    if trimmed.is_empty() {
        return Err("URL is required".to_string());
    }
    let parsed = Url::parse(trimmed).map_err(|_| "Enter a valid URL".to_string())?;
    if !matches!(parsed.scheme(), "http" | "https") {
        return Err("Only HTTP and HTTPS URLs are supported".to_string());
    }
    if parsed.host_str().is_none() {
        return Err("URL must include a host".to_string());
    }
    if !parsed.username().is_empty() || parsed.password().is_some() {
        return Err("URLs with embedded credentials are not allowed".to_string());
    }
    Ok(parsed)
}

fn validate_destination(destination: &str) -> Result<String, String> {
    match destination {
        "Downloads" | "Desktop" | "Documents" => Ok(destination.to_string()),
        _ => Err("Unsupported destination".to_string()),
    }
}

fn category_for_filename(filename: &str) -> String {
    let extension = Path::new(filename)
        .extension()
        .and_then(|value| value.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase();
    match extension.as_str() {
        "zip" | "rar" | "7z" | "tar" | "gz" | "bz2" | "xz" => "archives",
        "mp3" | "wav" | "flac" | "aac" | "ogg" | "m4a" => "audio",
        "mp4" | "mkv" | "avi" | "mov" | "webm" | "m4v" => "video",
        "jpg" | "jpeg" | "png" | "gif" | "webp" | "svg" | "bmp" => "images",
        "pdf" | "doc" | "docx" | "xls" | "xlsx" | "ppt" | "pptx" | "txt" | "md" => "documents",
        "exe" | "msi" | "msix" | "dmg" | "deb" | "rpm" => "applications",
        _ => "other",
    }
    .to_string()
}

fn filename_from_url(url: &Url) -> String {
    let candidate = url
        .path_segments()
        .and_then(|segments| segments.filter(|segment| !segment.is_empty()).next_back())
        .unwrap_or("download");
    let decoded = percent_encoding::percent_decode_str(candidate).decode_utf8_lossy();
    let sanitized: String = decoded
        .chars()
        .map(|character| {
            if "<>:\"/\\|?*".contains(character) || character.is_control() {
                '_'
            } else {
                character
            }
        })
        .collect();
    let sanitized = sanitized.trim_matches('.').trim();
    if sanitized.is_empty() {
        "download".to_string()
    } else {
        sanitized.chars().take(180).collect()
    }
}
