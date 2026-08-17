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
            commands::inspect_url
        ])
        .setup(|app| {
            let database = database::DatabaseState::open(
                app.path()
                    .app_data_dir()
                    .map_err(|error| format!("Could not resolve app data directory: {error}"))?,
            )?;
            app.manage(database);
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
