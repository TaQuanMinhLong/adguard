use crate::config::Config;
use crate::parser::{parse_hosts, serialize_hosts, PreservedLine};
use crate::utils::is_local_domain;
use parking_lot::Mutex;
use std::collections::BTreeSet;
use std::fs;
use std::path::Path;
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub blocking: Arc<Mutex<BTreeSet<Arc<str>>>>,
    pub preserved_lines: Arc<Mutex<Vec<PreservedLine>>>,
    pub config: Arc<Mutex<Config>>,
}

impl AppState {
    #[inline]
    pub fn new(config: Config) -> Self {
        AppState {
            blocking: Arc::new(Mutex::new(BTreeSet::new())),
            preserved_lines: Arc::new(Mutex::new(Vec::new())),
            config: Arc::new(Mutex::new(config)),
        }
    }

    /// Load state from hosts file
    #[inline]
    pub fn load_from_file(&self, path: &Path) -> Result<(), anyhow::Error> {
        let content = fs::read_to_string(path)?;
        let parsed = parse_hosts(&content)?;

        {
            let mut blocking = self.blocking.lock();
            for hostname in parsed.blocking {
                if !is_local_domain(&hostname) {
                    blocking.insert(hostname);
                }
            }
        }
        {
            let mut preserved_lines = self.preserved_lines.lock();
            for line in parsed.preserved_lines {
                preserved_lines.push(line);
            }
        };

        Ok(())
    }

    /// Add a domain to blocking
    #[inline]
    pub fn add_block(&self, hostname: &str) {
        if !is_local_domain(hostname) {
            self.blocking.lock().insert(hostname.into());
        }
    }

    /// Remove a domain from blocking
    #[inline]
    pub fn remove_block(&self, hostname: &str) {
        if !is_local_domain(hostname) {
            self.blocking.lock().remove(&Arc::from(hostname));
        }
    }

    /// Get all blocked domains (only returns localhost entries)
    /// Returns domains sorted alphabetically by hostname
    #[inline]
    pub fn get_all_blocks(&self) -> BTreeSet<Arc<str>> {
        self.blocking.lock().clone()
    }

    /// Serialize state to hosts file content
    #[inline]
    pub fn serialize(&self) -> String {
        let preserved_lines = self.preserved_lines.lock();
        let blocking = self.blocking.lock();
        serialize_hosts(&preserved_lines, &blocking)
    }

    /// Get statistics (only counts localhost entries)
    #[inline]
    pub fn get_total_blocked(&self) -> usize {
        self.blocking.lock().len()
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

    #[test]
    fn test_add_block() {
        let state = AppState::new(Config::default());
        state.add_block("example.com");
        let blocking = state.blocking.lock();
        assert!(blocking.contains(&Arc::from("example.com")));
    }

    #[test]
    fn test_remove_block() {
        let state = AppState::new(Config::default());

        state.add_block("example.com");
        state.remove_block("example.com");

        let blocking = state.blocking.lock();
        assert!(!blocking.contains(&Arc::from("example.com")));
    }

    #[test]
    fn test_get_all_blocks() {
        let state = AppState::new(Config::default());

        state.add_block("example.com");
        state.add_block("test.com");

        let blocks = state.get_all_blocks();
        assert_eq!(blocks.len(), 2);
    }

    #[test]
    fn test_get_statistics() {
        let state = AppState::new(Config::default());

        state.add_block("example.com");
        state.add_block("test.com");
        state.add_block("blocked.com");

        let total = state.get_total_blocked();
        assert_eq!(total, 3);
    }

    #[test]
    fn test_localhost_domains_not_blocked() {
        let state = AppState::new(Config::default());

        // Try to add localhost domains
        state.add_block("localhost");
        state.add_block("localhost.localdomain");
        state.add_block("subdomain.localhost");
        state.add_block("example.localhost");

        // These should not be added
        {
            let blocking = state.blocking.lock();
            assert!(!blocking.contains(&Arc::from("localhost")));
            assert!(!blocking.contains(&Arc::from("localhost.localdomain")));
            assert!(!blocking.contains(&Arc::from("subdomain.localhost")));
            assert!(!blocking.contains(&Arc::from("example.localhost")));
        }

        // Regular domains should still work
        state.add_block("example.com");
        {
            let blocking = state.blocking.lock();
            assert!(blocking.contains(&Arc::from("example.com")));
        }
    }
}
