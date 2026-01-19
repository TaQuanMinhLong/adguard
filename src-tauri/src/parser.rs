use crate::utils::{is_local_domain, is_localhost_ip};
use pest::Parser;
use pest_derive::Parser;
use std::collections::{BTreeSet, HashMap};
use std::net::IpAddr;
use std::str::FromStr;
use std::sync::Arc;

#[derive(Parser)]
#[grammar = "grammar/hosts.pest"]
pub struct HostsParser;

#[derive(Debug, Clone)]
pub enum PreservedLine {
    Comment(Arc<str>),
    NonLocalhostEntry(Arc<str>),
    LocalhostEntry { ip: IpAddr, hostname: Arc<str> },
}

#[derive(Debug)]
pub struct ParsedHosts {
    pub blocking: BTreeSet<Arc<str>>,
    pub preserved_lines: Vec<PreservedLine>,
}

/// Parse a hosts file content into managed entries and preserved lines
pub fn parse_hosts(content: &str) -> Result<ParsedHosts, pest::error::Error<Rule>> {
    let file = HostsParser::parse(Rule::file, content)?
        .next()
        .ok_or_else(|| {
            pest::error::Error::new_from_span(
                pest::error::ErrorVariant::CustomError {
                    message: "Empty file".to_string(),
                },
                pest::Span::new(content, 0, 0).unwrap(),
            )
        })?;

    let mut blocking: BTreeSet<Arc<str>> = BTreeSet::new();
    let mut preserved_lines: Vec<PreservedLine> = Vec::new();

    // Iterate over lines in the file
    for node in file.into_inner() {
        match node.as_rule() {
            Rule::line => {
                // Process the line's inner content (entry, comment, or empty for NEWLINE)
                let mut line_inner = node.into_inner();
                let line_content = line_inner.next();

                match line_content {
                    Some(content) => {
                        match content.as_rule() {
                            Rule::entry => {
                                let original_line_str = content.as_str();
                                let mut inner = content.into_inner();
                                let ip_str = inner.next().unwrap().as_str();

                                match IpAddr::from_str(ip_str) {
                                    Ok(ip) => {
                                        if is_localhost_ip(&ip) {
                                            for hostname in inner.map(|pair| pair.as_str()) {
                                                if !is_local_domain(hostname) {
                                                    blocking.insert(hostname.into());
                                                } else {
                                                    // Localhost entry - preserve as-is
                                                    preserved_lines.push(
                                                        PreservedLine::LocalhostEntry {
                                                            ip,
                                                            hostname: hostname.into(),
                                                        },
                                                    );
                                                }
                                            }
                                        } else {
                                            // Non-localhost entry - preserve as-is
                                            let original_line = original_line_str.trim();
                                            preserved_lines.push(PreservedLine::NonLocalhostEntry(
                                                original_line.into(),
                                            ));
                                        }
                                    }
                                    Err(_) => {
                                        // Invalid IP - preserve as-is
                                        let original_line = original_line_str.trim();
                                        preserved_lines.push(PreservedLine::NonLocalhostEntry(
                                            original_line.into(),
                                        ));
                                    }
                                }
                            }
                            Rule::comment => {
                                let comment_text = content.as_str().trim();
                                preserved_lines.push(PreservedLine::Comment(comment_text.into()));
                            }
                            _ => {}
                        }
                    }
                    None => {
                        // Empty line (NEWLINE is silent, so line with no content is empty)
                        // Don't preserve empty lines - they're just separators
                        // If we need to preserve intentional empty lines, we can add logic later
                    }
                }
            }
            Rule::EOI => break,
            _ => {}
        }
    }

    Ok(ParsedHosts {
        blocking,
        preserved_lines,
    })
}

#[inline]
pub fn serialize_hosts(preserved_lines: &[PreservedLine], blocking: &BTreeSet<Arc<str>>) -> String {
    let mut result = String::new();
    let mut localhost_entries: HashMap<IpAddr, BTreeSet<Arc<str>>> = HashMap::new();

    // First, write preserved lines
    for line in preserved_lines {
        match line {
            PreservedLine::Comment(comment) => {
                result.push_str(comment);
                result.push('\n');
            }
            PreservedLine::NonLocalhostEntry(entry) => {
                result.push_str(entry);
                result.push('\n');
            }
            PreservedLine::LocalhostEntry { ip, hostname } => {
                localhost_entries
                    .entry(*ip)
                    .or_insert_with(BTreeSet::new)
                    .insert(hostname.clone());
            }
        }
    }

    // Write localhost entries grouped by IP
    for (ip, hostnames) in localhost_entries {
        result.push_str(&ip.to_string());
        for hostname in hostnames {
            result.push(' ');
            result.push_str(hostname.as_ref());
        }
        result.push('\n');
    }

    // Write blocking entries (non-localhost domains)
    if !blocking.is_empty() {
        result.push_str("127.0.0.1");
        for hostname in blocking {
            result.push(' ');
            result.push_str(hostname);
        }
        result.push('\n');
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_localhost_ip() {
        assert!(is_localhost_ip(&IpAddr::from_str("127.0.0.1").unwrap()));
        assert!(is_localhost_ip(&IpAddr::from_str("127.0.0.0").unwrap()));
        assert!(is_localhost_ip(&IpAddr::from_str("0.0.0.0").unwrap()));
        assert!(is_localhost_ip(&IpAddr::from_str("::1").unwrap()));
        assert!(!is_localhost_ip(&IpAddr::from_str("192.168.1.1").unwrap()));
        assert!(!is_localhost_ip(&IpAddr::from_str("8.8.8.8").unwrap()));
    }

    #[test]
    fn test_parse_simple_hosts() {
        let content = "127.0.0.1 localhost example.com";
        let parsed = parse_hosts(content).unwrap();

        // localhost should be preserved, not blocked
        assert_eq!(parsed.blocking.len(), 1);
        assert!(!parsed.blocking.contains("localhost"));
        assert!(parsed.blocking.contains("example.com"));

        // Check that localhost is preserved
        assert!(parsed.preserved_lines.iter().any(|line| {
            matches!(line, PreservedLine::LocalhostEntry { hostname, .. } if hostname.as_ref() == "localhost")
        }));
    }

    #[test]
    fn test_parse_with_comments() {
        let content = "# This is a comment\n127.0.0.1 localhost\n";
        let parsed = parse_hosts(content).unwrap();

        // Should have comment and localhost entry (preserved, not blocked)
        assert_eq!(parsed.preserved_lines.len(), 2);
        assert!(parsed
            .preserved_lines
            .iter()
            .any(|line| { matches!(line, PreservedLine::Comment(_)) }));
        assert!(parsed.preserved_lines.iter().any(|line| {
            matches!(line, PreservedLine::LocalhostEntry { hostname, .. } if hostname.as_ref() == "localhost")
        }));
        // localhost should not be in blocking
        assert!(!parsed.blocking.contains("localhost"));
    }

    #[test]
    fn test_parse_non_localhost() {
        let content = "192.168.1.1 router\n127.0.0.1 localhost\n";
        let parsed = parse_hosts(content).unwrap();

        // Non-localhost should be preserved
        assert!(parsed.preserved_lines.iter().any(|line| {
            matches!(line, PreservedLine::NonLocalhostEntry(s) if s.contains("192.168.1.1"))
        }));

        // Localhost should be preserved, not blocked
        assert!(!parsed.blocking.contains("localhost"));
        assert!(parsed.preserved_lines.iter().any(|line| {
            matches!(line, PreservedLine::LocalhostEntry { hostname, .. } if hostname.as_ref() == "localhost")
        }));
    }

    #[test]
    fn test_round_trip() {
        let original = "# Comment\n127.0.0.1 localhost example.com\n192.168.1.1 router\n\n";
        let parsed = parse_hosts(original).unwrap();
        let serialized = serialize_hosts(&parsed.preserved_lines, &parsed.blocking);

        // Re-parse to verify
        let reparsed = parse_hosts(&serialized).unwrap();

        // Check that localhost entries are preserved (not in blocking)
        assert_eq!(parsed.blocking.len(), reparsed.blocking.len());
        assert!(!reparsed.blocking.contains("localhost"));
        assert!(reparsed.blocking.contains(&Arc::from("example.com")));

        // Check that localhost is preserved
        assert!(reparsed.preserved_lines.iter().any(|line| {
            matches!(line, PreservedLine::LocalhostEntry { hostname, .. } if hostname.as_ref() == "localhost")
        }));
    }

    #[test]
    fn test_parse_ipv6_addresses() {
        // Test various IPv6 address formats
        let content = "fe00::0 ip6-localnet\nff00::0 ip6-mcastprefix\nff02::1 ip6-allnodes\nff02::2 ip6-allrouters\n::1 localhost\n";
        let parsed = parse_hosts(content).unwrap();

        // All these should be preserved as non-localhost entries (except ::1 localhost)
        assert_eq!(parsed.preserved_lines.len(), 5); // 4 non-localhost IPv6 entries + 1 localhost entry
        assert_eq!(parsed.blocking.len(), 0); // localhost should not be in blocking
        assert!(!parsed.blocking.contains("localhost"));

        // Check that localhost is preserved
        assert!(parsed.preserved_lines.iter().any(|line| {
            matches!(line, PreservedLine::LocalhostEntry { hostname, .. } if hostname.as_ref() == "localhost")
        }));
    }
}
