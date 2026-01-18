use crate::commit::commit_changes;
use crate::history::{list_history_entries, rollback_to_history};
use crate::parser::parse_hosts;
use crate::platform::{default_hosts_file_path, is_elevated};
use crate::state::AppState;
use std::collections::BTreeSet;
use std::path::PathBuf;
use std::sync::Arc;
use tauri::State;

#[tauri::command]
pub async fn get_blocked_domains(
    state: State<'_, Arc<AppState>>,
) -> Result<BTreeSet<Arc<str>>, ()> {
    Ok(state.get_all_blocks())
}

#[tauri::command]
pub async fn remove_domain(state: State<'_, Arc<AppState>>, hostname: &str) -> Result<(), ()> {
    state.remove_block(hostname);
    Ok(())
}

#[tauri::command]
pub async fn add_domain(state: State<'_, Arc<AppState>>, hostname: &str) -> Result<(), String> {
    state.add_block(hostname);
    Ok(())
}

#[tauri::command]
pub async fn save_changes(state: State<'_, Arc<AppState>>) -> Result<(), String> {
    let config = state.get_config();
    let hosts_path = config
        .host_file_path
        .unwrap_or_else(default_hosts_file_path);

    let history_dir = config.history_dir.clone();
    let max_history = config.max_history_entries;

    commit_changes(state.inner().clone(), hosts_path, history_dir, max_history)
        .await
        .map_err(|e| {
            // Provide more detailed error message
            let error_msg = format!("Failed to save changes: {}", e);
            eprintln!("{}", error_msg);
            error_msg
        })?;

    Ok(())
}

#[tauri::command]
pub async fn get_history_list(
    state: State<'_, Arc<AppState>>,
) -> Result<Vec<serde_json::Value>, String> {
    let config = state.get_config();
    let history_dir = config
        .history_dir
        .ok_or_else(|| "History directory not configured".to_string())?;

    let entries =
        list_history_entries(&history_dir).map_err(|e| format!("Failed to list history: {}", e))?;

    Ok(entries
        .into_iter()
        .map(|entry| {
            serde_json::json!({
                "filename": entry.filename,
                "path": entry.path.to_string_lossy(),
                "entry_count": entry.entry_count,
                "file_size": entry.file_size,
                "timestamp": entry.timestamp.duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
            })
        })
        .collect())
}

#[tauri::command]
pub async fn rollback_to(state: State<'_, Arc<AppState>>, filename: &str) -> Result<(), String> {
    let config = state.get_config();
    let history_dir = config
        .history_dir
        .ok_or_else(|| "History directory not configured".to_string())?;

    let entries =
        list_history_entries(&history_dir).map_err(|e| format!("Failed to list history: {}", e))?;

    let entry = entries
        .into_iter()
        .find(|e| e.filename == filename)
        .ok_or_else(|| "History entry not found".to_string())?;

    let hosts_path = config
        .host_file_path
        .unwrap_or_else(default_hosts_file_path);

    rollback_to_history(&entry, &hosts_path).map_err(|e| format!("Failed to rollback: {}", e))?;

    Ok(())
}

#[tauri::command]
pub async fn delete_history_files(
    state: State<'_, Arc<AppState>>,
    filenames: Vec<String>,
) -> Result<(), String> {
    let config = state.get_config();
    let history_dir = config
        .history_dir
        .ok_or_else(|| "History directory not configured".to_string())?;

    crate::history::delete_history_files(&history_dir, &filenames)
        .map_err(|e| format!("Failed to delete history files: {}", e))?;

    Ok(())
}

#[tauri::command]
pub fn get_config(state: State<'_, Arc<AppState>>) -> Result<serde_json::Value, String> {
    let config = state.get_config();
    Ok(serde_json::json!({
        "host_file_path": config.host_file_path.as_ref().map(|p| p.to_string_lossy().to_string()),
        "history_dir": config.history_dir.as_ref().map(|p| p.to_string_lossy().to_string()),
        "max_history_entries": config.max_history_entries,
        "theme": config.theme.to_str(),
    }))
}

#[tauri::command]
pub fn update_config(
    state: State<'_, Arc<AppState>>,
    config_json: serde_json::Value,
) -> Result<(), String> {
    let mut config = state.get_config();

    if let Some(host_path) = config_json.get("host_file_path").and_then(|v| v.as_str()) {
        if host_path.is_empty() {
            config.host_file_path = None;
        } else {
            config.host_file_path = Some(PathBuf::from(host_path).as_path().into());
        }
    }

    if let Some(history_path) = config_json.get("history_dir").and_then(|v| v.as_str()) {
        if history_path.is_empty() {
            config.history_dir = None;
        } else {
            config.history_dir = Some(PathBuf::from(history_path).as_path().into());
        }
    }

    if let Some(max_entries) = config_json
        .get("max_history_entries")
        .and_then(|v| v.as_u64())
    {
        config.max_history_entries = max_entries as usize;
    }

    if let Some(theme_str) = config_json.get("theme").and_then(|v| v.as_str()) {
        config.theme = crate::config::Theme::from_str(theme_str);
    }

    state.update_config(config);
    Ok(())
}

#[tauri::command]
pub fn get_host_file_path(state: State<'_, Arc<AppState>>) -> String {
    let config = state.get_config();
    config
        .host_file_path
        .unwrap_or_else(default_hosts_file_path)
        .to_string_lossy()
        .to_string()
}

#[tauri::command]
pub fn get_statistics(state: State<'_, Arc<AppState>>) -> Result<serde_json::Value, String> {
    let total_blocked = state.get_total_blocked();
    Ok(serde_json::json!({
        "total_blocked": total_blocked,
    }))
}

#[tauri::command]
pub fn check_admin_privileges() -> bool {
    is_elevated()
}

#[tauri::command]
pub fn export_hosts(state: State<'_, Arc<AppState>>) -> String {
    state.serialize()
}

#[tauri::command]
pub async fn import_hosts(state: State<'_, Arc<AppState>>, content: String) -> Result<(), String> {
    let parsed = parse_hosts(&content).map_err(|e| format!("Failed to parse hosts file: {}", e))?;
    *state.blocking.lock() = parsed.blocking;
    *state.preserved_lines.lock() = parsed.preserved_lines;
    Ok(())
}
