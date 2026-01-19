use std::net::IpAddr;

#[inline]
pub fn is_local_domain(hostname: &str) -> bool {
    let h = hostname.trim_end_matches('.');

    if !h.contains('.') {
        // Treat single-label hostnames as local (e.g. mDNS / /etc/hosts)
        return true;
    }

    let h = h.to_lowercase();

    return h == "localhost" || h == "localhost.localdomain" || h.ends_with(".localhost");
}

/// Check if an IP address is a localhost address
#[inline]
pub fn is_localhost_ip(ip: &IpAddr) -> bool {
    match ip {
        IpAddr::V4(ipv4) => {
            // 127.0.0.0/8 range (127.0.0.1 to 127.255.255.255)
            // or 0.0.0.0
            ipv4.octets()[0] == 127 || *ipv4 == std::net::Ipv4Addr::new(0, 0, 0, 0)
        }
        IpAddr::V6(ipv6) => {
            // ::1 (IPv6 loopback)
            // or :: (unspecified)
            *ipv6 == std::net::Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 1)
                || *ipv6 == std::net::Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 0)
        }
    }
}
