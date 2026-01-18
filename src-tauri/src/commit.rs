use crate::history::{cleanup_old_history, verify_host_file, write_history_snapshot};
use crate::platform::flush_dns;
use crate::state::AppState;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use tauri::async_runtime;

/// Commit current state to hosts file
pub async fn commit_changes(
    state: Arc<AppState>,
    hosts_file_path: PathBuf,
    history_dir: Option<PathBuf>,
    max_history_entries: usize,
) -> Result<(), anyhow::Error> {
    // Serialize current state
    let content = state.serialize();

    // Write to history directory if enabled
    if let Some(ref history_dir) = history_dir {
        let history_entry = async_runtime::spawn_blocking({
            let content = content.clone();
            let history_dir = history_dir.clone();
            move || write_history_snapshot(&history_dir, &content)
        })
        .await??;

        // Verify the history file
        verify_host_file(&history_entry.path)?;

        // Cleanup old history entries
        async_runtime::spawn_blocking({
            let history_dir = history_dir.clone();
            move || cleanup_old_history(&history_dir, max_history_entries)
        })
        .await??;
    }

    // Write to actual hosts file (atomic write)
    async_runtime::spawn_blocking({
        let content = content.clone();
        let hosts_file_path = hosts_file_path.clone();
        move || {
            let temp_path = hosts_file_path.with_extension("tmp");
            fs::write(&temp_path, content)?;
            fs::rename(&temp_path, &hosts_file_path)?;
            Ok::<(), anyhow::Error>(())
        }
    })
    .await??;

    // Flush DNS cache
    async_runtime::spawn_blocking(|| {
        flush_dns().map_err(|e| anyhow::anyhow!("Failed to flush DNS: {}", e))
    })
    .await??;

    Ok(())
}
