use pest::Parser;
use pest_derive::Parser;
use std::collections::{BTreeSet, HashMap};
use std::net::IpAddr;
use std::str::FromStr;

#[derive(Parser)]
#[grammar = "grammar/hosts.pest"]
pub struct HostsParser;

#[derive(Debug, Clone)]
pub enum PreservedLine {
    Comment(String),
    NonLocalhostEntry(String),
}

#[derive(Debug)]
pub struct ParsedHosts {
    pub blocking: HashMap<IpAddr, BTreeSet<String>>,
    pub preserved_lines: Vec<PreservedLine>,
}

/// Check if an IP address is a localhost address
#[inline]
pub fn is_localhost_ip(ip: &IpAddr) -> bool {
    match ip {
        IpAddr::V4(ipv4) => {
            // 127.0.0.0/8 range (127.0.0.1 to 127.255.255.255)
            ipv4.octets()[0] == 127 || 
            // 0.0.0.0
            *ipv4 == std::net::Ipv4Addr::new(0, 0, 0, 0)
        }
        IpAddr::V6(ipv6) => {
            // ::1 (IPv6 loopback)
            *ipv6 == std::net::Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 1) ||
            // :: (unspecified)
            *ipv6 == std::net::Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 0)
        }
    }
}

/// Parse a hosts file content into managed entries and preserved lines
pub fn parse_hosts(content: &str) -> Result<ParsedHosts, pest::error::Error<Rule>> {
    let file = HostsParser::parse(Rule::file, content)?
        .next()
        .ok_or_else(|| pest::error::Error::new_from_span(
            pest::error::ErrorVariant::CustomError { message: "Empty file".to_string() },
            pest::Span::new(content, 0, 0).unwrap()
        ))?;

    let mut blocking: HashMap<IpAddr, BTreeSet<String>> = HashMap::new();
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
                                let original_line_str = content.as_str().to_string();
                                let mut inner = content.into_inner();
                                let ip_str = inner.next().unwrap().as_str();
                                
                                if let Ok(ip) = IpAddr::from_str(ip_str) {
                                    if is_localhost_ip(&ip) {
                                        // This is a localhost entry - add to blocking map
                                        let hostnames = inner
                                            .map(|pair| pair.as_str().to_string())
                                            .collect::<Vec<_>>();
                                        
                                        let entry = blocking.entry(ip).or_default();
                                        for hostname in hostnames {
                                            entry.insert(hostname);
                                        }
                                    } else {
                                        // Non-localhost entry - preserve as-is
                                        let original_line = original_line_str.trim().to_string();
                                        preserved_lines.push(PreservedLine::NonLocalhostEntry(original_line));
                                    }
                                } else {
                                    // Invalid IP - preserve as-is
                                    let original_line = original_line_str.trim().to_string();
                                    preserved_lines.push(PreservedLine::NonLocalhostEntry(original_line));
                                }
                            }
                            Rule::comment => {
                                let comment_text = content.as_str().trim().to_string();
                                preserved_lines.push(PreservedLine::Comment(comment_text));
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

/// Serialize parsed hosts back to hosts file format
pub fn serialize_hosts(parsed: &ParsedHosts) -> String {
    let mut result = String::new();
    
    // First, write preserved lines
    for line in &parsed.preserved_lines {
        match line {
            PreservedLine::Comment(comment) => {
                result.push_str(comment);
                result.push('\n');
            }
            PreservedLine::NonLocalhostEntry(entry) => {
                result.push_str(entry);
                result.push('\n');
            }
        }
    }
    
    // Then, write localhost blocking entries
    let mut entries: Vec<_> = parsed.blocking.iter().collect();
    entries.sort_by_key(|(ip, _)| *ip);
    
    for (ip, hostnames) in entries {
        // BTreeSet is already sorted, so we can iterate directly
        result.push_str(&format!("{}", ip));
        for hostname in hostnames {
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
        let content = "127.0.0.1 localhost\n127.0.0.1 example.com\n";
        let parsed = parse_hosts(content).unwrap();
        
        assert_eq!(parsed.blocking.len(), 1);
        assert!(parsed.blocking.contains_key(&IpAddr::from_str("127.0.0.1").unwrap()));
        let hostnames = &parsed.blocking[&IpAddr::from_str("127.0.0.1").unwrap()];
        assert!(hostnames.contains("localhost"));
        assert!(hostnames.contains("example.com"));
    }

    #[test]
    fn test_parse_with_comments() {
        let content = "# This is a comment\n127.0.0.1 localhost\n";
        let parsed = parse_hosts(content).unwrap();
        
        assert_eq!(parsed.preserved_lines.len(), 1);
        assert!(matches!(parsed.preserved_lines[0], PreservedLine::Comment(_)));
    }

    #[test]
    fn test_parse_non_localhost() {
        let content = "192.168.1.1 router\n127.0.0.1 localhost\n";
        let parsed = parse_hosts(content).unwrap();
        
        // Non-localhost should be preserved
        assert!(parsed.preserved_lines.iter().any(|line| {
            matches!(line, PreservedLine::NonLocalhostEntry(s) if s.contains("192.168.1.1"))
        }));
        
        // Localhost should be in blocking
        assert!(parsed.blocking.contains_key(&IpAddr::from_str("127.0.0.1").unwrap()));
    }

    #[test]
    fn test_round_trip() {
        let original = "# Comment\n127.0.0.1 localhost example.com\n192.168.1.1 router\n\n";
        let parsed = parse_hosts(original).unwrap();
        let serialized = serialize_hosts(&parsed);
        
        // Re-parse to verify
        let reparsed = parse_hosts(&serialized).unwrap();
        
        // Check that localhost entries are preserved
        assert_eq!(parsed.blocking.len(), reparsed.blocking.len());
        assert!(reparsed.blocking.contains_key(&IpAddr::from_str("127.0.0.1").unwrap()));
    }
}
