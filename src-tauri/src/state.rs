use crate::config::Config;
use crate::parser::{is_localhost_ip, parse_hosts, serialize_hosts, ParsedHosts};
use parking_lot::Mutex;
use std::collections::{BTreeSet, HashMap};
use std::fs;
use std::net::IpAddr;
use std::path::PathBuf;
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub blocking: Arc<Mutex<HashMap<IpAddr, BTreeSet<String>>>>,
    pub preserved_lines: Arc<Mutex<Vec<crate::parser::PreservedLine>>>,
    pub config: Arc<Mutex<Config>>,
}

impl AppState {
    #[inline]
    pub fn new(config: Config) -> Self {
        AppState {
            blocking: Arc::new(Mutex::new(HashMap::new())),
            preserved_lines: Arc::new(Mutex::new(Vec::new())),
            config: Arc::new(Mutex::new(config)),
        }
    }

    /// Load state from hosts file
    pub fn load_from_file(&self, path: &PathBuf) -> Result<(), anyhow::Error> {
        let content = fs::read_to_string(path)?;
        let parsed = parse_hosts(&content)?;

        *self.blocking.lock() = parsed.blocking;
        *self.preserved_lines.lock() = parsed.preserved_lines;

        Ok(())
    }

    /// Add a domain to blocking (only accepts localhost IPs)
    pub fn add_block(&self, ip: IpAddr, hostname: String) -> Result<(), anyhow::Error> {
        if !is_localhost_ip(&ip) {
            return Err(anyhow::anyhow!(
                "Only localhost IPs can be added. IP {} is not a localhost address.",
                ip
            ));
        }

        let mut blocking = self.blocking.lock();
        blocking.entry(ip).or_default().insert(hostname);

        Ok(())
    }

    /// Remove a domain from blocking (only operates on localhost IPs)
    pub fn remove_block(&self, ip: IpAddr, hostname: &str) -> Result<(), anyhow::Error> {
        if !is_localhost_ip(&ip) {
            return Err(anyhow::anyhow!(
                "Only localhost IPs can be removed. IP {} is not a localhost address.",
                ip
            ));
        }

        let mut blocking = self.blocking.lock();
        if let Some(hostnames) = blocking.get_mut(&ip) {
            hostnames.remove(hostname);
            if hostnames.is_empty() {
                blocking.remove(&ip);
            }
        }

        Ok(())
    }

    /// Get all blocked domains (only returns localhost entries)
    /// Returns domains sorted alphabetically by hostname
    pub fn get_all_blocks(&self) -> Vec<(IpAddr, String)> {
        let blocking = self.blocking.lock();
        let mut result = Vec::new();
        for (ip, hostnames) in blocking.iter() {
            // BTreeSet is already sorted, so hostnames are in order
            for hostname in hostnames {
                result.push((*ip, hostname.clone()));
            }
        }
        // Sort by hostname only (all IPs are localhost anyway)
        result.sort_by(|a, b| a.1.cmp(&b.1));
        result
    }

    /// Serialize state to hosts file content
    pub fn serialize(&self) -> String {
        let blocking = self.blocking.lock().clone();
        let preserved_lines = self.preserved_lines.lock().clone();
        let parsed = ParsedHosts {
            blocking,
            preserved_lines,
        };
        serialize_hosts(&parsed)
    }

    /// Get statistics (only counts localhost entries)
    #[inline]
    pub fn get_statistics(&self) -> (usize, usize) {
        let blocking = self.blocking.lock();
        let total_blocked: usize = blocking.values().map(|h| h.len()).sum();
        let unique_ips = blocking.len();
        (total_blocked, unique_ips)
    }

    /// Get config (read-only)
    #[inline]
    pub fn get_config(&self) -> Config {
        self.config.lock().clone()
    }

    /// Update config
    #[inline]
    pub fn update_config(&self, config: Config) {
        *self.config.lock() = config;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::IpAddr;
    use std::str::FromStr;

    #[test]
    fn test_add_block() {
        let state = AppState::new(Config::default());
        let ip = IpAddr::from_str("127.0.0.1").unwrap();

        state.add_block(ip, "example.com".to_string()).unwrap();
        let blocking = state.blocking.lock();
        assert!(blocking.contains_key(&ip));
        assert!(blocking[&ip].contains("example.com"));
    }

    #[test]
    fn test_add_block_rejects_non_localhost() {
        let state = AppState::new(Config::default());
        let ip = IpAddr::from_str("192.168.1.1").unwrap();

        assert!(state.add_block(ip, "example.com".to_string()).is_err());
    }

    #[test]
    fn test_remove_block() {
        let state = AppState::new(Config::default());
        let ip = IpAddr::from_str("127.0.0.1").unwrap();

        state.add_block(ip, "example.com".to_string()).unwrap();
        state.remove_block(ip, "example.com").unwrap();

        let blocking = state.blocking.lock();
        assert!(!blocking.contains_key(&ip));
    }

    #[test]
    fn test_get_all_blocks() {
        let state = AppState::new(Config::default());
        let ip = IpAddr::from_str("127.0.0.1").unwrap();

        state.add_block(ip, "example.com".to_string()).unwrap();
        state.add_block(ip, "test.com".to_string()).unwrap();

        let blocks = state.get_all_blocks();
        assert_eq!(blocks.len(), 2);
    }

    #[test]
    fn test_get_statistics() {
        let state = AppState::new(Config::default());
        let ip1 = IpAddr::from_str("127.0.0.1").unwrap();
        let ip2 = IpAddr::from_str("0.0.0.0").unwrap();

        state.add_block(ip1, "example.com".to_string()).unwrap();
        state.add_block(ip1, "test.com".to_string()).unwrap();
        state.add_block(ip2, "blocked.com".to_string()).unwrap();

        let (total, unique_ips) = state.get_statistics();
        assert_eq!(total, 3);
        assert_eq!(unique_ips, 2);
    }
}
