mod browser;
mod commands;
mod database;
mod download;
mod scheduler;
mod security;
mod utils;

use std::time::Duration;

use tauri::Manager;

use crate::scheduler::{evaluate_window, ScheduleDecision};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            commands::add_download,
            commands::get_downloads,
            commands::evaluate_queue_schedule,
            commands::start_queued_downloads,
            commands::get_queues,
            commands::save_queue,
            commands::get_setting,
            commands::set_setting,
            commands::inspect_url,
            commands::get_file_category,
            commands::pause_download,
            commands::resume_download,
            commands::cancel_download,
            commands::verify_download_hash,
            commands::delete_download,
            commands::open_download_file,
            commands::open_download_folder
        ])
        .plugin(tauri_plugin_notification::init())
        .setup(|app| {
            let database = database::DatabaseState::open(
                app.path()
                    .app_data_dir()
                    .map_err(|error| format!("Could not resolve app data directory: {error}"))?,
            )?;
            database
                .recover_incomplete()
                .map_err(|error| Box::<dyn std::error::Error>::from(error))?;
            let manager = download::DownloadManager::new(database.clone())
                .map_err(|error| Box::<dyn std::error::Error>::from(error))?;
            manager
                .set_app_handle(app.handle().clone())
                .map_err(|error| Box::<dyn std::error::Error>::from(error))?;
            app.manage(database.clone());
            app.manage(manager.clone());
            let scheduler_database = database;
            let scheduler_manager = manager;
            std::thread::spawn(move || loop {
                let auto_start = scheduler_database
                    .get_setting("auto_start_downloads")
                    .ok()
                    .flatten()
                    .map(|value| value != "false")
                    .unwrap_or(true);
                let schedule_allows = scheduler_database
                    .list_queues()
                    .ok()
                    .map(|queues| {
                        let configured = queues
                            .iter()
                            .filter(|queue| queue.auto_start)
                            .collect::<Vec<_>>();
                        configured.is_empty()
                            || configured.iter().any(|queue| {
                                matches!(
                                    evaluate_window(
                                        chrono::Local::now(),
                                        queue.start_at.as_deref(),
                                        queue.stop_at.as_deref(),
                                    ),
                                    Ok(ScheduleDecision::Ready)
                                )
                            })
                    })
                    .unwrap_or(true);
                if auto_start && schedule_allows {
                    let max_concurrent = scheduler_database
                        .get_setting("max_concurrent_downloads")
                        .ok()
                        .flatten()
                        .and_then(|value| value.parse::<i64>().ok())
                        .unwrap_or(3);
                    let _ = scheduler_manager.start_queued(max_concurrent);
                }
                std::thread::sleep(Duration::from_secs(5));
            });
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
