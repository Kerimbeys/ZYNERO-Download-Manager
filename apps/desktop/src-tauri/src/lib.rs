mod browser;
mod commands;
mod database;
mod download;
mod scheduler;
mod security;
mod utils;

use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            commands::add_download,
            commands::get_downloads,
            commands::evaluate_queue_schedule,
            commands::get_queues,
            commands::save_queue,
            commands::get_setting,
            commands::set_setting,
            commands::inspect_url,
            commands::pause_download,
            commands::resume_download,
            commands::cancel_download,
            commands::delete_download,
            commands::open_download_file,
            commands::open_download_folder
        ])
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
            app.manage(database);
            app.manage(manager);
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
