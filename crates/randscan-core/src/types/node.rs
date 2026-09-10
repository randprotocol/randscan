use serde::{Deserialize, Serialize};

/// Geolocation of a node's public IP.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeoInfo {
    pub lat: f64,
    pub lon: f64,
    pub city: Option<String>,
    pub region: Option<String>,
    pub country: Option<String>,
    pub country_code: Option<String>,
    pub org: Option<String>,
}

/// A full node known to the explorer's own node (itself plus its libp2p peers).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeInfo {
    pub peer_id: String,
    pub ip: Option<String>,
    pub port: Option<u16>,
    pub connected_secs: Option<i64>,
    pub is_self: bool,
    /// "validator" | "observer" | "peer" (role of remote peers is not known)
    pub role: String,
    pub geo: Option<GeoInfo>,
}

/// Extract the first IPv4/IPv6 address and TCP port from a libp2p multiaddr.
pub fn parse_multiaddr(addr: &str) -> (Option<String>, Option<u16>) {
    let parts: Vec<&str> = addr.split('/').collect();
    let mut ip = None;
    let mut port = None;
    let mut i = 1;
    while i + 1 < parts.len() {
        match parts[i] {
            "ip4" | "ip6" | "dns" | "dns4" | "dns6" => ip = Some(parts[i + 1].to_string()),
            "tcp" | "udp" => port = parts[i + 1].parse().ok(),
            _ => {}
        }
        i += 2;
    }
    (ip, port)
}

/// True for loopback, link-local and RFC1918 addresses (not worth geolocating).
pub fn is_private_ip(ip: &str) -> bool {
    match ip.parse::<std::net::IpAddr>() {
        Ok(std::net::IpAddr::V4(v4)) => {
            v4.is_private() || v4.is_loopback() || v4.is_link_local() || v4.is_unspecified()
        }
        Ok(std::net::IpAddr::V6(v6)) => v6.is_loopback() || v6.is_unspecified(),
        Err(_) => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_multiaddrs() {
        assert_eq!(
            parse_multiaddr("/ip4/167.172.65.63/tcp/30303/p2p/12D3KooWBK"),
            (Some("167.172.65.63".into()), Some(30303))
        );
        assert_eq!(
            parse_multiaddr("/ip4/37.19.201.133/tcp/12383"),
            (Some("37.19.201.133".into()), Some(12383))
        );
        assert_eq!(parse_multiaddr("/p2p/12D3"), (None, None));
    }

    #[test]
    fn private_ips() {
        assert!(is_private_ip("192.168.100.79"));
        assert!(is_private_ip("127.0.0.1"));
        assert!(!is_private_ip("167.172.65.63"));
    }
}
