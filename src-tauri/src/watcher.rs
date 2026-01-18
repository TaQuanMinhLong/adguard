use crate::state::AppState;
use notify::{Event, EventKind, RecursiveMode, Watcher};
use std::path::Path;
use std::sync::Arc;
use tauri::async_runtime;
use tauri::{AppHandle, Manager};

/// Start watching the hosts file for external changes
pub fn start_watcher(
    app: AppHandle,
    hosts_file_path: Arc<Path>,
    state: Arc<AppState>,
) -> Result<(), anyhow::Error> {
    let path_for_watch = hosts_file_path.clone();
    let app_for_manage = app.clone();

    let mut watcher = notify::recommended_watcher(move |result: Result<Event, notify::Error>| {
        match result {
            Ok(event) => {
                // Only react to modify events (not create/remove)
                if matches!(event.kind, EventKind::Modify(_)) {
                    // Debounce: spawn async task to handle the change
                    let path_clone = path_for_watch.clone();
                    let state_clone = state.clone();

                    async_runtime::spawn(async move {
                        // Small delay to debounce rapid changes
                        tokio::time::sleep(std::time::Duration::from_millis(500)).await;

                        // Re-parse the file and update state
                        if let Err(e) = state_clone.load_from_file(&path_clone) {
                            eprintln!("Failed to reload hosts file: {}", e);
                        }
                        // Note: Frontend can poll for updates or user can refresh manually
                        // Event emission can be added later when needed
                    });
                }
            }
            Err(e) => {
                eprintln!("Watcher error: {}", e);
            }
        }
    })?;

    watcher.watch(&hosts_file_path, RecursiveMode::NonRecursive)?;

    // Store watcher in app state so it doesn't get dropped
    app_for_manage.manage(watcher);

    Ok(())
}
