mod commands;
mod commit;
mod config;
mod history;
mod parser;
mod platform;
mod state;
mod watcher;

use crate::config::Config;
use crate::platform::default_hosts_file_path;
use crate::state::AppState;
use crate::watcher::start_watcher;
use std::path::PathBuf;
use std::sync::Arc;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            // Get config file path using Tauri PathResolver
            let config_path = app
                .path()
                .app_config_dir()
                .unwrap_or_else(|_| {
                    // Fallback to a default location if app_config_dir fails
                    app.path()
                        .config_dir()
                        .unwrap_or_else(|_| PathBuf::from("."))
                        .join("adguard")
                })
                .join("config.ini");

            // Load or create config
            let mut config = Config::load_from_file(&config_path).unwrap_or_else(|_| {
                // If loading fails, create default config
                let default_config = Config::default();
                // Try to save it, but don't fail if we can't
                let _ = default_config.save_to_file(&config_path);
                default_config
            });

            // Set default history directory if not configured
            if config.history_dir.is_none() {
                let default_history = app
                    .path()
                    .app_data_dir()
                    .unwrap_or_else(|_| {
                        // Fallback to app_config_dir if app_data_dir fails
                        app.path().app_config_dir().unwrap_or_else(|_| {
                            // Final fallback
                            app.path()
                                .config_dir()
                                .unwrap_or_else(|_| PathBuf::from("."))
                                .join("adguard")
                        })
                    })
                    .join("history");
                config.history_dir = Some(default_history);
                // Try to save updated config, but don't fail if we can't
                let _ = config.save_to_file(&config_path);
            }

            // Create app state
            let app_state = Arc::new(AppState::new(config.clone()));

            // Get hosts file path
            let hosts_file_path = config
                .host_file_path
                .clone()
                .unwrap_or_else(default_hosts_file_path);

            // Load initial state from hosts file
            if hosts_file_path.exists() {
                if let Err(e) = app_state.load_from_file(&hosts_file_path) {
                    eprintln!("Failed to load hosts file: {}", e);
                }
            }

            // Start file watcher
            if let Err(e) = start_watcher(
                app.handle().clone(),
                hosts_file_path.clone(),
                app_state.clone(),
            ) {
                eprintln!("Failed to start file watcher: {}", e);
            }

            // Register state with Tauri
            app.manage(app_state);

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_blocked_domains,
            commands::add_domain,
            commands::remove_domain,
            commands::save_changes,
            commands::get_history_list,
            commands::rollback_to,
            commands::delete_history_files,
            commands::get_config,
            commands::update_config,
            commands::get_host_file_path,
            commands::get_statistics,
            commands::check_admin_privileges,
            commands::export_hosts,
            commands::import_hosts,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
