//! Single-connection streaming download worker with resumable temp files.

use crate::database::{DatabaseState, StoredDownload};
use futures_util::StreamExt;
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
    time::{Duration, Instant},
};
#[cfg(not(test))]
use tauri::{AppHandle, Emitter};
use tokio::{
    fs::{self, OpenOptions},
    io::AsyncWriteExt,
};

#[derive(Clone)]
pub struct DownloadManager {
    database: DatabaseState,
    controls: Arc<Mutex<HashMap<String, Arc<Control>>>>,
    client: reqwest::Client,
    #[cfg(not(test))]
    app_handle: Arc<Mutex<Option<AppHandle>>>,
}

struct Control {
    paused: AtomicBool,
    cancelled: AtomicBool,
}

impl DownloadManager {
    pub fn new(database: DatabaseState) -> Result<Self, String> {
        let client = reqwest::Client::builder()
            .user_agent("ZYNERO/0.1.0")
            .connect_timeout(Duration::from_secs(15))
            .timeout(Duration::from_secs(60))
            .build()
            .map_err(|error| format!("Could not create download client: {error}"))?;
        Ok(Self {
            database,
            controls: Arc::new(Mutex::new(HashMap::new())),
            client,
            #[cfg(not(test))]
            app_handle: Arc::new(Mutex::new(None)),
        })
    }

    #[cfg(not(test))]
    pub fn set_app_handle(&self, handle: AppHandle) -> Result<(), String> {
        *self
            .app_handle
            .lock()
            .map_err(|_| "Download manager lock poisoned".to_string())? = Some(handle);
        Ok(())
    }

    #[cfg(not(test))]
    fn emit_progress(&self, id: &str) {
        let handle = self.app_handle.lock().ok().and_then(|value| value.clone());
        if let (Some(handle), Ok(Some(download))) = (handle, self.database.find_download(id)) {
            let _ = handle.emit("download-progress", download);
        }
    }

    #[cfg(test)]
    fn emit_progress(&self, _id: &str) {}
    pub fn start(&self, download: StoredDownload) -> Result<(), String> {
        let control = Arc::new(Control {
            paused: AtomicBool::new(false),
            cancelled: AtomicBool::new(false),
        });
        self.controls
            .lock()
            .map_err(|_| "Download manager lock poisoned".to_string())?
            .insert(download.id.clone(), control.clone());
        let manager = self.clone();
        tokio::spawn(async move {
            let result = manager.run(download.clone(), control).await;
            if let Err(error) = result {
                let _ = manager.database.update_runtime(
                    &download.id,
                    "failed",
                    download.downloaded_bytes,
                    0,
                    0,
                    Some(&error),
                );
                manager.emit_progress(&download.id);
            }
            let _ = manager
                .controls
                .lock()
                .map(|mut controls| controls.remove(&download.id));
        });
        Ok(())
    }

    pub fn start_queued(&self, max_concurrent: i64) -> Result<usize, String> {
        let active = self.database.count_active_downloads()?;
        let capacity = (max_concurrent.clamp(1, 32) - active).max(0);
        if capacity == 0 {
            return Ok(0);
        }
        let queued = self.database.list_queued_downloads(capacity)?;
        let mut started = 0;
        for download in queued {
            self.start(download)?;
            started += 1;
        }
        Ok(started)
    }

    pub fn pause(&self, id: &str) -> Result<(), String> {
        let controls = self
            .controls
            .lock()
            .map_err(|_| "Download manager lock poisoned".to_string())?;
        controls
            .get(id)
            .ok_or_else(|| "Download is not active".to_string())?
            .paused
            .store(true, Ordering::Relaxed);
        Ok(())
    }

    pub fn cancel(&self, id: &str) -> Result<(), String> {
        let controls = self
            .controls
            .lock()
            .map_err(|_| "Download manager lock poisoned".to_string())?;
        controls
            .get(id)
            .ok_or_else(|| "Download is not active".to_string())?
            .cancelled
            .store(true, Ordering::Relaxed);
        Ok(())
    }

    async fn run(&self, download: StoredDownload, control: Arc<Control>) -> Result<(), String> {
        let (temp_path, final_path) = resolve_paths(&download)?;
        let connections = self
            .database
            .get_setting("connections_per_download")?
            .and_then(|value| value.parse::<i64>().ok())
            .unwrap_or(3)
            .clamp(1, 32);
        if download.supports_range && download.total_bytes.unwrap_or(0) > 0 && connections > 1 {
            return self
                .run_segmented(download, control, temp_path, final_path, connections)
                .await;
        }
        self.database.set_paths(
            &download.id,
            temp_path.to_string_lossy().as_ref(),
            final_path.to_string_lossy().as_ref(),
        )?;
        let mut offset = if temp_path.exists() {
            fs::metadata(&temp_path)
                .await
                .map_err(|error| format!("Could not inspect temp file: {error}"))?
                .len() as i64
        } else {
            0
        };
        let mut response = self.request(&download, offset).await?;
        if offset > 0 && response.status().as_u16() != 206 {
            offset = 0;
            let _ = fs::remove_file(&temp_path).await;
            response = self.request(&download, 0).await?;
        }
        if !response.status().is_success() {
            return Err(format!(
                "Download server returned HTTP {}",
                response.status().as_u16()
            ));
        }
        let total_bytes = download
            .total_bytes
            .or_else(|| response.content_length().map(|value| value as i64 + offset));
        let mut file = if offset > 0 {
            OpenOptions::new()
                .create(true)
                .append(true)
                .open(&temp_path)
                .await
        } else {
            OpenOptions::new()
                .create(true)
                .write(true)
                .truncate(true)
                .open(&temp_path)
                .await
        }
        .map_err(|error| format!("Could not open temp file: {error}"))?;
        let mut downloaded = offset;
        let mut last_update = Instant::now();
        let started = Instant::now();
        let mut stream = response.bytes_stream();
        while let Some(chunk) = stream.next().await {
            if control.cancelled.load(Ordering::Relaxed) {
                self.database
                    .update_runtime(&download.id, "cancelled", downloaded, 0, 0, None)?;
                return Ok(());
            }
            if control.paused.load(Ordering::Relaxed) {
                self.database
                    .update_runtime(&download.id, "paused", downloaded, 0, 0, None)?;
                return Ok(());
            }
            let chunk = chunk.map_err(|error| format!("Download stream failed: {error}"))?;
            file.write_all(&chunk)
                .await
                .map_err(|error| format!("Could not write temp file: {error}"))?;
            downloaded += chunk.len() as i64;
            if last_update.elapsed() >= Duration::from_millis(250) {
                let elapsed = started.elapsed().as_secs_f64().max(0.001);
                let speed = ((downloaded - offset) as f64 / elapsed) as i64;
                let eta = total_bytes
                    .and_then(|total| {
                        if speed > 0 {
                            Some(((total - downloaded).max(0) / speed).max(0))
                        } else {
                            None
                        }
                    })
                    .unwrap_or(0);
                self.database.update_runtime(
                    &download.id,
                    "active",
                    downloaded,
                    speed,
                    eta,
                    None,
                )?;
                self.emit_progress(&download.id);
                last_update = Instant::now();
            }
        }
        file.flush()
            .await
            .map_err(|error| format!("Could not flush temp file: {error}"))?;
        drop(file);
        if let Some(parent) = final_path.parent() {
            fs::create_dir_all(parent)
                .await
                .map_err(|error| format!("Could not create final directory: {error}"))?;
        }
        fs::rename(&temp_path, &final_path)
            .await
            .map_err(|error| format!("Could not finalize download: {error}"))?;
        self.database
            .update_runtime(&download.id, "completed", downloaded, 0, 0, None)?;
        self.emit_progress(&download.id);
        Ok(())
    }

    async fn run_segmented(
        &self,
        download: StoredDownload,
        control: Arc<Control>,
        temp_path: PathBuf,
        final_path: PathBuf,
        connections: i64,
    ) -> Result<(), String> {
        let total = download
            .total_bytes
            .ok_or_else(|| "Segmented download requires a known content length".to_string())?;
        let ranges = plan_segments(total, connections)?;
        let segment_dir = temp_path.with_file_name(format!(".{}.segments", download.filename));
        fs::create_dir_all(&segment_dir)
            .await
            .map_err(|error| format!("Could not create segment directory: {error}"))?;
        self.database.set_paths(
            &download.id,
            temp_path.to_string_lossy().as_ref(),
            final_path.to_string_lossy().as_ref(),
        )?;
        self.database
            .update_runtime(&download.id, "active", 0, 0, 0, None)?;
        self.emit_progress(&download.id);

        let total_downloaded = Arc::new(std::sync::atomic::AtomicI64::new(0));
        let last_update = Arc::new(tokio::sync::Mutex::new(Instant::now()));
        let mut workers = tokio::task::JoinSet::new();
        for (index, (start, end)) in ranges.iter().copied().enumerate() {
            let manager = self.clone();
            let worker_download = download.clone();
            let worker_control = control.clone();
            let worker_dir = segment_dir.clone();
            let shared_downloaded = total_downloaded.clone();
            let shared_last_update = last_update.clone();
            workers.spawn(async move {
                let segment_path = worker_dir.join(format!("segment-{index:02}.part"));
                let existing = match fs::metadata(&segment_path).await {
                    Ok(metadata) => metadata.len() as i64,
                    Err(_) => 0,
                };
                let expected = end - start + 1;
                let completed = existing.clamp(0, expected);
                if completed == expected {
                    shared_downloaded.fetch_add(completed, Ordering::Relaxed);
                    return Ok::<(), String>(());
                }
                if completed > 0 {
                    let _ = fs::remove_file(&segment_path).await;
                }
                let response = manager.request_range(&worker_download, start, end).await?;
                if response.status().as_u16() != 206 {
                    return Err(format!(
                        "Segment {index} expected HTTP 206, got {}",
                        response.status().as_u16()
                    ));
                }
                let mut file = OpenOptions::new()
                    .create(true)
                    .write(true)
                    .truncate(true)
                    .open(&segment_path)
                    .await
                    .map_err(|error| format!("Could not open segment {index}: {error}"))?;
                let mut stream = response.bytes_stream();
                let mut segment_downloaded = 0i64;
                while let Some(chunk) = stream.next().await {
                    if worker_control.cancelled.load(Ordering::Relaxed) {
                        return Err("Download cancelled".to_string());
                    }
                    if worker_control.paused.load(Ordering::Relaxed) {
                        return Err("Download paused".to_string());
                    }
                    let chunk =
                        chunk.map_err(|error| format!("Segment {index} stream failed: {error}"))?;
                    segment_downloaded += chunk.len() as i64;
                    if segment_downloaded > expected {
                        return Err(format!("Segment {index} exceeded its requested byte range"));
                    }
                    file.write_all(&chunk)
                        .await
                        .map_err(|error| format!("Could not write segment {index}: {error}"))?;
                    let current = shared_downloaded
                        .fetch_add(chunk.len() as i64, Ordering::Relaxed)
                        + chunk.len() as i64;
                    let mut timestamp = shared_last_update.lock().await;
                    if timestamp.elapsed() >= Duration::from_millis(250) {
                        let speed =
                            (current as f64 / timestamp.elapsed().as_secs_f64().max(0.001)) as i64;
                        let eta = if speed > 0 {
                            ((total - current).max(0) / speed).max(0)
                        } else {
                            0
                        };
                        manager.database.update_runtime(
                            &worker_download.id,
                            "active",
                            current,
                            speed,
                            eta,
                            None,
                        )?;
                        manager.emit_progress(&worker_download.id);
                        *timestamp = Instant::now();
                    }
                }
                file.flush()
                    .await
                    .map_err(|error| format!("Could not flush segment {index}: {error}"))?;
                if segment_downloaded != expected {
                    return Err(format!(
                        "Segment {index} incomplete: {segment_downloaded}/{expected} bytes"
                    ));
                }
                Ok::<(), String>(())
            });
        }
        while let Some(result) = workers.join_next().await {
            if let Err(error) = result.map_err(|error| format!("Segment worker failed: {error}"))? {
                let status = if control.cancelled.load(Ordering::Relaxed) {
                    "cancelled"
                } else if control.paused.load(Ordering::Relaxed) {
                    "paused"
                } else {
                    "failed"
                };
                let downloaded = total_downloaded.load(Ordering::Relaxed);
                self.database.update_runtime(
                    &download.id,
                    status,
                    downloaded,
                    0,
                    0,
                    if status == "failed" {
                        Some(error.as_str())
                    } else {
                        None
                    },
                )?;
                self.emit_progress(&download.id);
                return if status == "failed" {
                    Err(error)
                } else {
                    Ok(())
                };
            }
        }
        let mut output = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&temp_path)
            .await
            .map_err(|error| format!("Could not open merged temp file: {error}"))?;
        for index in 0..ranges.len() {
            let segment_path = segment_dir.join(format!("segment-{index:02}.part"));
            let mut input = fs::File::open(&segment_path)
                .await
                .map_err(|error| format!("Could not open segment {index} for merge: {error}"))?;
            tokio::io::copy(&mut input, &mut output)
                .await
                .map_err(|error| format!("Could not merge segment {index}: {error}"))?;
        }
        output
            .flush()
            .await
            .map_err(|error| format!("Could not flush merged file: {error}"))?;
        drop(output);
        let _ = fs::remove_dir_all(&segment_dir).await;
        if let Some(parent) = final_path.parent() {
            fs::create_dir_all(parent)
                .await
                .map_err(|error| format!("Could not create final directory: {error}"))?;
        }
        fs::rename(&temp_path, &final_path)
            .await
            .map_err(|error| format!("Could not finalize segmented download: {error}"))?;
        self.database
            .update_runtime(&download.id, "completed", total, 0, 0, None)?;
        self.emit_progress(&download.id);
        Ok(())
    }

    async fn request_range(
        &self,
        download: &StoredDownload,
        start: i64,
        end: i64,
    ) -> Result<reqwest::Response, String> {
        for attempt in 0..3u32 {
            match self
                .client
                .get(&download.url)
                .header(reqwest::header::RANGE, format!("bytes={start}-{end}"))
                .send()
                .await
            {
                Ok(response) if response.status().as_u16() == 206 => return Ok(response),
                Ok(response)
                    if (response.status().as_u16() == 408
                        || response.status().as_u16() == 429
                        || response.status().is_server_error())
                        && attempt < 2 =>
                {
                    tokio::time::sleep(Duration::from_millis(250 * 2u64.pow(attempt))).await;
                }
                Ok(response) => return Ok(response),
                Err(_error) if attempt < 2 => {
                    tokio::time::sleep(Duration::from_millis(250 * 2u64.pow(attempt))).await;
                }
                Err(error) => return Err(format!("Segment request failed: {error}")),
            }
        }
        Err("Segment request exhausted retry attempts".to_string())
    }

    async fn request(
        &self,
        download: &StoredDownload,
        offset: i64,
    ) -> Result<reqwest::Response, String> {
        for attempt in 0..3u32 {
            let mut request = self.client.get(&download.url);
            if offset > 0 && download.supports_range {
                request = request.header(reqwest::header::RANGE, format!("bytes={offset}-"));
            }
            match request.send().await {
                Ok(response)
                    if response.status().is_success() || response.status().as_u16() == 206 =>
                {
                    return Ok(response)
                }
                Ok(response)
                    if (response.status().as_u16() == 408
                        || response.status().as_u16() == 429
                        || response.status().is_server_error())
                        && attempt < 2 =>
                {
                    tokio::time::sleep(Duration::from_millis(250 * 2u64.pow(attempt))).await;
                }
                Ok(response) => return Ok(response),
                Err(error) if attempt < 2 => {
                    tokio::time::sleep(Duration::from_millis(250 * 2u64.pow(attempt))).await;
                    if attempt == 2 {
                        return Err(format!("Download request failed: {error}"));
                    }
                }
                Err(error) => return Err(format!("Download request failed: {error}")),
            }
        }
        Err("Download request exhausted retry attempts".to_string())
    }
}

fn resolve_paths(download: &StoredDownload) -> Result<(PathBuf, PathBuf), String> {
    let root = match download.destination.as_str() {
        "Downloads" => dirs::download_dir(),
        "Desktop" => dirs::desktop_dir(),
        "Documents" => dirs::document_dir(),
        _ => None,
    }
    .ok_or_else(|| "Could not resolve download destination".to_string())?;
    let final_path = unique_path(root.join(&download.filename));
    let temp_path = final_path.with_file_name(format!(".{}.zynero.part", download.filename));
    Ok((temp_path, final_path))
}

fn unique_path(path: PathBuf) -> PathBuf {
    if !path.exists() {
        return path;
    }
    let stem = path
        .file_stem()
        .and_then(|value| value.to_str())
        .unwrap_or("download");
    let extension = path
        .extension()
        .and_then(|value| value.to_str())
        .map(|value| format!(".{value}"))
        .unwrap_or_default();
    for index in 1..10000 {
        let candidate = path.with_file_name(format!("{stem} ({index}){extension}"));
        if !candidate.exists() {
            return candidate;
        }
    }
    path.with_file_name(format!("{stem}-{}{}", uuid::Uuid::new_v4(), extension))
}

#[allow(dead_code)]
fn _is_safe_path(path: &Path) -> bool {
    !path
        .components()
        .any(|component| matches!(component, std::path::Component::ParentDir))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::DatabaseState;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::TcpListener;

    fn sample(url: String) -> StoredDownload {
        StoredDownload {
            id: "worker-test".into(),
            url,
            filename: "sample.bin".into(),
            destination: "Downloads".into(),
            status: "queued".into(),
            total_bytes: None,
            downloaded_bytes: 0,
            content_type: None,
            supports_range: false,
            temp_path: None,
            final_path: None,
            error_message: None,
            speed_bps: 0,
            eta_seconds: 0,
            created_at: String::new(),
            updated_at: String::new(),
        }
    }

    #[tokio::test]
    async fn request_retries_transient_http_errors() {
        let listener = TcpListener::bind("127.0.0.1:0")
            .await
            .expect("bind local server");
        let address = listener.local_addr().expect("local address");
        tokio::spawn(async move {
            for attempt in 0..3 {
                let (mut socket, _) = listener.accept().await.expect("accept request");
                let mut buffer = [0u8; 512];
                let _ = socket.read(&mut buffer).await;
                let response = if attempt < 2 {
                    "HTTP/1.1 503 Service Unavailable\r\nContent-Length: 0\r\n\r\n"
                } else {
                    "HTTP/1.1 200 OK\r\nContent-Length: 0\r\n\r\n"
                };
                socket
                    .write_all(response.as_bytes())
                    .await
                    .expect("write response");
            }
        });
        let path =
            std::env::temp_dir().join(format!("zynero-worker-test-{}", uuid::Uuid::new_v4()));
        let database = DatabaseState::open(path.clone()).expect("database opens");
        let manager = DownloadManager::new(database).expect("manager creates");
        let response = manager
            .request(&sample(format!("http://{address}/file")), 0)
            .await
            .expect("request eventually succeeds");
        assert_eq!(response.status(), reqwest::StatusCode::OK);
        let _ = std::fs::remove_dir_all(path);
    }

    #[test]
    fn generated_temp_path_is_hidden_sibling_of_final_file() {
        let path = std::env::temp_dir().join(format!("zynero-path-test-{}", uuid::Uuid::new_v4()));
        let database = DatabaseState::open(path.clone()).expect("database opens");
        let download = sample("https://example.com/file".into());
        let (temp, final_path) = resolve_paths(&download).expect("paths resolve");
        assert!(temp
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or_default()
            .ends_with(".zynero.part"));
        assert_eq!(temp.parent(), final_path.parent());
        drop(database);
        let _ = std::fs::remove_dir_all(path);
    }
    #[tokio::test]
    async fn resumable_download_completes_from_existing_part() {
        let payload = b"ZYNERO-C11-resumable-payload".to_vec();
        let prefix_len = 7usize;
        let listener = TcpListener::bind("127.0.0.1:0")
            .await
            .expect("bind local server");
        let address = listener.local_addr().expect("local address");
        let expected = payload.clone();
        tokio::spawn(async move {
            let (mut socket, _) = listener.accept().await.expect("accept request");
            let mut request = [0u8; 2048];
            let size = socket.read(&mut request).await.expect("read request");
            let request = String::from_utf8_lossy(&request[..size]).to_ascii_lowercase();
            assert!(request.contains("range: bytes=7-"));
            let remaining = &expected[prefix_len..];
            let response = format!(
                "HTTP/1.1 206 Partial Content\r\nContent-Length: {}\r\nContent-Range: bytes 7-{}/{}\r\nConnection: close\r\n\r\n",
                remaining.len(),
                expected.len() - 1,
                expected.len()
            );
            socket
                .write_all(response.as_bytes())
                .await
                .expect("write headers");
            socket.write_all(remaining).await.expect("write body");
        });

        let root = std::env::temp_dir().join(format!("zynero-c11-db-{}", uuid::Uuid::new_v4()));
        let database = DatabaseState::open(root.clone()).expect("open test database");
        database
            .set_setting("connections_per_download", "1")
            .expect("set single connection");
        let filename = format!("zynero-c11-{}.bin", uuid::Uuid::new_v4());
        let url = format!("http://{address}/payload");
        let mut download = sample(url);
        download.id = uuid::Uuid::new_v4().to_string();
        download.filename = filename;
        download.total_bytes = Some(payload.len() as i64);
        download.downloaded_bytes = prefix_len as i64;
        download.supports_range = true;
        database
            .insert_download(&download)
            .expect("insert download");

        let (temp_path, final_path) = resolve_paths(&download).expect("resolve paths");
        if let Some(parent) = temp_path.parent() {
            tokio::fs::create_dir_all(parent)
                .await
                .expect("create download dir");
        }
        tokio::fs::write(&temp_path, &payload[..prefix_len])
            .await
            .expect("write partial file");
        let manager = DownloadManager::new(database.clone()).expect("create manager");
        let control = Arc::new(Control {
            paused: AtomicBool::new(false),
            cancelled: AtomicBool::new(false),
        });
        manager
            .run(download.clone(), control)
            .await
            .expect("complete resume");

        let actual = tokio::fs::read(&final_path).await.expect("read final file");
        assert_eq!(actual, payload);
        let stored = database
            .find_download(&download.id)
            .expect("read stored download")
            .expect("download exists");
        assert_eq!(stored.status, "completed");
        assert_eq!(stored.downloaded_bytes, payload.len() as i64);
        drop(database);
        let _ = std::fs::remove_file(final_path);
        let _ = std::fs::remove_dir_all(root);
    }
    #[tokio::test]
    async fn download_pauses_before_writing_when_control_paused() {
        let listener = TcpListener::bind("127.0.0.1:0")
            .await
            .expect("bind local server");
        let address = listener.local_addr().expect("local address");
        tokio::spawn(async move {
            let (mut socket, _) = listener.accept().await.expect("accept request");
            let mut request = [0u8; 1024];
            let _ = socket.read(&mut request).await;
            socket
                .write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 12\r\nConnection: close\r\n\r\nhello world!")
                .await
                .expect("write response");
        });
        let root = std::env::temp_dir().join(format!("zynero-c10-pause-{}", uuid::Uuid::new_v4()));
        let database = DatabaseState::open(root.clone()).expect("open test database");
        database
            .set_setting("connections_per_download", "1")
            .expect("set single connection");
        let mut download = sample(format!("http://{address}/pause"));
        download.id = uuid::Uuid::new_v4().to_string();
        download.filename = format!("zynero-c10-pause-{}.bin", uuid::Uuid::new_v4());
        database
            .insert_download(&download)
            .expect("insert download");
        let manager = DownloadManager::new(database.clone()).expect("create manager");
        let control = Arc::new(Control {
            paused: AtomicBool::new(true),
            cancelled: AtomicBool::new(false),
        });
        manager
            .run(download.clone(), control)
            .await
            .expect("pause is a normal state");
        let stored = database
            .find_download(&download.id)
            .expect("read stored download")
            .expect("download exists");
        assert_eq!(stored.status, "paused");
        assert_eq!(stored.downloaded_bytes, 0);
        drop(database);
        let _ = std::fs::remove_dir_all(root);
    }

    #[tokio::test]
    async fn download_rejects_local_http_failure() {
        let listener = TcpListener::bind("127.0.0.1:0")
            .await
            .expect("bind local server");
        let address = listener.local_addr().expect("local address");
        tokio::spawn(async move {
            let (mut socket, _) = listener.accept().await.expect("accept request");
            let mut request = [0u8; 1024];
            let _ = socket.read(&mut request).await;
            socket
                .write_all(
                    b"HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\nConnection: close\r\n\r\n",
                )
                .await
                .expect("write failure response");
        });
        let root =
            std::env::temp_dir().join(format!("zynero-c10-failure-{}", uuid::Uuid::new_v4()));
        let database = DatabaseState::open(root.clone()).expect("open test database");
        database
            .set_setting("connections_per_download", "1")
            .expect("set single connection");
        let mut download = sample(format!("http://{address}/missing"));
        download.id = uuid::Uuid::new_v4().to_string();
        download.filename = format!("zynero-c10-failure-{}.bin", uuid::Uuid::new_v4());
        database
            .insert_download(&download)
            .expect("insert download");
        let manager = DownloadManager::new(database.clone()).expect("create manager");
        let control = Arc::new(Control {
            paused: AtomicBool::new(false),
            cancelled: AtomicBool::new(false),
        });
        let error = manager
            .run(download, control)
            .await
            .expect_err("404 must fail");
        assert!(error.contains("HTTP 404"));
        drop(database);
        let _ = std::fs::remove_dir_all(root);
    }
}

/// Plans contiguous byte ranges for future concurrent workers.
/// The final segment absorbs any remainder so the ranges cover the complete file.
pub fn plan_segments(
    total_bytes: i64,
    requested_connections: i64,
) -> Result<Vec<(i64, i64)>, String> {
    if total_bytes <= 0 {
        return Err("Total bytes must be positive".to_string());
    }
    let connections = requested_connections.clamp(1, 32).min(total_bytes);
    let base = total_bytes / connections;
    let remainder = total_bytes % connections;
    let mut segments = Vec::with_capacity(connections as usize);
    let mut start = 0;
    for index in 0..connections {
        let length = base + i64::from(index < remainder);
        let end = start + length - 1;
        segments.push((start, end));
        start = end + 1;
    }
    Ok(segments)
}

#[cfg(test)]
mod segment_tests {
    use super::plan_segments;

    #[test]
    fn segment_plan_covers_file_without_gaps() {
        let segments = plan_segments(100, 3).unwrap();
        assert_eq!(segments, vec![(0, 33), (34, 66), (67, 99)]);
        assert_eq!(segments.first().unwrap().0, 0);
        assert_eq!(segments.last().unwrap().1, 99);
        for pair in segments.windows(2) {
            assert_eq!(pair[0].1 + 1, pair[1].0);
        }
    }

    #[test]
    fn segment_connections_are_bounded_and_small_files_are_not_oversplit() {
        assert_eq!(plan_segments(4, 99).unwrap().len(), 4);
        assert_eq!(plan_segments(100, 0).unwrap().len(), 1);
        assert!(plan_segments(0, 4).is_err());
    }
}
