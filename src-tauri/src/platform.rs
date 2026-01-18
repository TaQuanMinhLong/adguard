use std::path::PathBuf;
use std::process::Command;

/// Get the default hosts file path for the current platform
#[inline]
pub fn default_hosts_file_path() -> PathBuf {
    #[cfg(target_os = "windows")]
    {
        PathBuf::from(r"C:\Windows\System32\drivers\etc\hosts")
    }

    #[cfg(any(target_os = "linux", target_os = "macos"))]
    {
        PathBuf::from("/etc/hosts")
    }

    #[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
    {
        PathBuf::from("/etc/hosts") // Default fallback
    }
}

/// Check if the application is running with administrator/elevated privileges
/// On Windows, this checks if we can write to the hosts file
/// On Unix, this checks if running as root
pub fn is_elevated() -> bool {
    #[cfg(target_os = "windows")]
    {
        // Try to open the hosts file for writing to check permissions
        let hosts_path = default_hosts_file_path();
        if let Ok(file) = std::fs::OpenOptions::new().write(true).open(&hosts_path) {
            // If we can open it for writing, we likely have admin privileges
            drop(file);
            true
        } else {
            false
        }
    }

    #[cfg(any(target_os = "linux", target_os = "macos"))]
    {
        // Check if we can write to the hosts file (requires root)
        let hosts_path = default_hosts_file_path();
        std::fs::OpenOptions::new()
            .write(true)
            .open(&hosts_path)
            .is_ok()
    }

    #[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
    {
        // For other platforms, assume not elevated
        false
    }
}

/// Flush DNS cache using platform-specific command
pub fn flush_dns() -> Result<(), anyhow::Error> {
    #[cfg(target_os = "windows")]
    {
        Command::new("ipconfig").arg("/flushdns").output()?;
    }

    #[cfg(target_os = "macos")]
    {
        // Try dscacheutil first, fallback to killall
        let _ = Command::new("dscacheutil").arg("-flushcache").output();

        Command::new("killall")
            .arg("-HUP")
            .arg("mDNSResponder")
            .output()?;
    }

    #[cfg(target_os = "linux")]
    {
        // Try systemd-resolve first
        let result = Command::new("systemd-resolve")
            .arg("--flush-caches")
            .output();

        if result.is_err() {
            // Fallback: try restarting nscd
            let _ = Command::new("sudo")
                .arg("service")
                .arg("nscd")
                .arg("restart")
                .output();
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_hosts_file_path() {
        let path = default_hosts_file_path();
        assert!(!path.as_os_str().is_empty());
    }
}
