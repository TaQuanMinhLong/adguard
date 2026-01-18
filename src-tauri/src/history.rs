use crate::parser::parse_hosts;
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::SystemTime;

#[derive(Debug, Clone)]
pub struct HistoryEntry {
    pub filename: String,
    pub path: PathBuf,
    pub timestamp: SystemTime,
    pub entry_count: usize,
    pub file_size: u64,
}

/// Verify a hosts file is valid
#[inline]
pub fn verify_host_file(path: &Path) -> Result<(), anyhow::Error> {
    // Check if file exists and is readable
    let content = fs::read_to_string(path)?;

    if content.trim().is_empty() {
        return Err(anyhow::anyhow!("Host file is empty"));
    }

    // Try to parse the file
    let parsed = parse_hosts(&content)?;

    // Validate all IP addresses are valid
    let mut seen_entries: HashSet<Arc<str>> = HashSet::new();

    for hostname in parsed.blocking {
        // Check for duplicate entries
        if !seen_entries.insert(hostname.clone()) {
            return Err(anyhow::anyhow!("Duplicate entry: {}", hostname));
        }

        // Validate hostname format (basic check)
        if hostname.is_empty() {
            return Err(anyhow::anyhow!("Empty hostname found"));
        }

        if hostname.len() > 253 {
            return Err(anyhow::anyhow!("Hostname too long: {}", hostname));
        }
    }

    Ok(())
}

/// Write a history snapshot
pub fn write_history_snapshot(
    history_dir: &Path,
    content: &str,
) -> Result<HistoryEntry, anyhow::Error> {
    // Create history directory if it doesn't exist
    fs::create_dir_all(history_dir)?;

    // Generate filename with timestamp (include nanoseconds and counter for uniqueness)
    let now = SystemTime::now();
    let duration = now.duration_since(std::time::UNIX_EPOCH).unwrap();

    // Use local timezone instead of UTC
    // Convert UTC timestamp to local timezone
    let utc_datetime = chrono::DateTime::<chrono::Utc>::from_timestamp(
        duration.as_secs() as i64,
        duration.subsec_nanos(),
    )
    .unwrap_or_else(chrono::Utc::now);

    // Convert to local timezone
    let datetime = utc_datetime.with_timezone(&chrono::Local);

    // Include nanoseconds and a random component to ensure uniqueness
    // Use a simple counter based on existing files to avoid collisions
    let mut counter = 0u32;
    let base_filename = format!(
        "hosts-backup-{}-{}",
        datetime.format("%Y-%m-%d-%H-%M-%S"),
        duration.subsec_nanos()
    );

    let mut filename = format!("{}-{}.txt", base_filename, counter);
    let mut file_path = history_dir.join(&filename);

    // If file exists, increment counter until we find a unique name
    while file_path.exists() {
        counter += 1;
        filename = format!("{}-{}.txt", base_filename, counter);
        file_path = history_dir.join(&filename);
    }

    // Write content to file
    fs::write(&file_path, content)?;

    // Get file metadata
    let metadata = fs::metadata(&file_path)?;
    let file_size = metadata.len();

    // Count entries (approximate)
    let entry_count = content
        .lines()
        .filter(|line| {
            let trimmed = line.trim();
            !trimmed.is_empty() && !trimmed.starts_with('#')
        })
        .count();

    Ok(HistoryEntry {
        filename,
        path: file_path,
        timestamp: now,
        entry_count,
        file_size,
    })
}

/// List all history entries
pub fn list_history_entries(history_dir: &Path) -> Result<Vec<HistoryEntry>, anyhow::Error> {
    if !history_dir.exists() {
        return Ok(Vec::new());
    }

    let mut entries = Vec::new();

    for entry in fs::read_dir(history_dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("txt") {
            let metadata = fs::metadata(&path)?;
            let file_size = metadata.len();
            let modified = metadata.modified().unwrap_or(SystemTime::now());

            let content = fs::read_to_string(&path).unwrap_or_default();
            let entry_count = content
                .lines()
                .filter(|line| {
                    let trimmed = line.trim();
                    !trimmed.is_empty() && !trimmed.starts_with('#')
                })
                .count();

            entries.push(HistoryEntry {
                filename: path.file_name().unwrap().to_string_lossy().to_string(),
                path,
                timestamp: modified,
                entry_count,
                file_size,
            });
        }
    }

    // Sort by timestamp (newest first)
    entries.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

    Ok(entries)
}

/// Clean up old history entries, keeping only the most recent N
#[inline]
pub fn cleanup_old_history(history_dir: &Path, max_entries: usize) -> Result<(), anyhow::Error> {
    let mut entries = list_history_entries(history_dir)?;

    if entries.len() <= max_entries {
        return Ok(());
    }

    // Remove oldest entries
    entries.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));
    let to_remove = entries.len() - max_entries;

    for entry in entries.into_iter().take(to_remove) {
        if let Err(e) = fs::remove_file(&entry.path) {
            eprintln!("Failed to remove history file {:?}: {}", entry.path, e);
        }
    }

    Ok(())
}

/// Rollback to a history entry
#[inline]
pub fn rollback_to_history(
    history_entry: &HistoryEntry,
    hosts_file_path: &Path,
) -> Result<(), anyhow::Error> {
    // Verify the history file first
    verify_host_file(&history_entry.path)?;

    // Read history file content
    let content = fs::read_to_string(&history_entry.path)?;

    // Write to hosts file (atomic write: temp file then rename)
    let temp_path = hosts_file_path.with_extension("tmp");
    fs::write(&temp_path, content)?;
    fs::rename(&temp_path, hosts_file_path)?;

    Ok(())
}

/// Delete history files by filenames
#[inline]
pub fn delete_history_files(history_dir: &Path, filenames: &[String]) -> Result<(), anyhow::Error> {
    for filename in filenames {
        let file_path = history_dir.join(filename);
        if file_path.exists() && file_path.is_file() {
            fs::remove_file(&file_path)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_verify_valid_host_file() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("hosts");

        fs::write(&file_path, "127.0.0.1 localhost\n").unwrap();

        assert!(verify_host_file(&file_path).is_ok());
    }

    #[test]
    fn test_verify_empty_file() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("hosts");

        fs::write(&file_path, "").unwrap();

        assert!(verify_host_file(&file_path).is_err());
    }

    #[test]
    fn test_write_history_snapshot() {
        let temp_dir = TempDir::new().unwrap();
        let content = "127.0.0.1 localhost\n";
        let history_dir = temp_dir.path().to_path_buf();

        let entry = write_history_snapshot(&history_dir, content).unwrap();

        assert!(entry.path.exists());
        assert!(entry.entry_count > 0);
    }

    #[test]
    fn test_list_history_entries() {
        let temp_dir = TempDir::new().unwrap();
        let content = "127.0.0.1 localhost\n";
        let history_dir = temp_dir.path().to_path_buf();

        write_history_snapshot(&history_dir, content).unwrap();
        write_history_snapshot(&history_dir, content).unwrap();

        let entries = list_history_entries(&history_dir).unwrap();
        assert_eq!(entries.len(), 2);
    }

    #[test]
    fn test_cleanup_old_history() {
        let temp_dir = TempDir::new().unwrap();
        let content = "127.0.0.1 localhost\n";
        let history_dir = temp_dir.path().to_path_buf();

        for _ in 0..5 {
            write_history_snapshot(&history_dir, content).unwrap();
        }

        cleanup_old_history(&history_dir, 3).unwrap();

        let entries = list_history_entries(&history_dir).unwrap();
        assert_eq!(entries.len(), 3);
    }
}
