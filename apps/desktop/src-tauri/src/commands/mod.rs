use serde::{Deserialize, Serialize};
use tauri::command;
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
pub struct DownloadInfo {
    pub id: String,
    pub url: String,
    pub filename: String,
    pub destination: String,
    pub status: String,
}

/// Validates the URL and creates the first durable-domain download record.
/// Actual streaming I/O will be added by the download worker milestone.
#[command]
pub fn add_download(request: AddDownloadRequest) -> Result<DownloadInfo, String> {
    let url = validate_url(&request.url)?;
    let destination = validate_destination(&request.destination)?;
    let filename = filename_from_url(&url);

    Ok(DownloadInfo {
        id: Uuid::new_v4().to_string(),
        url: url.to_string(),
        filename,
        destination,
        status: "queued".to_string(),
    })
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
    if parsed.username() != "" || parsed.password().is_some() {
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

fn filename_from_url(url: &Url) -> String {
    let candidate = url
        .path_segments()
        .and_then(|segments| segments.filter(|segment| !segment.is_empty()).next_back())
        .unwrap_or("download");
    let decoded = percent_encoding::percent_decode_str(candidate).decode_utf8_lossy();
    let sanitized: String = decoded
        .chars()
        .map(|character| if "<>:\"/\\|?*".contains(character) || character.is_control() { '_' } else { character })
        .collect();
    let sanitized = sanitized.trim_matches('.').trim();
    if sanitized.is_empty() { "download".to_string() } else { sanitized.chars().take(180).collect() }
}
