pub mod rules;
pub mod stats;

use time::OffsetDateTime;
use std::net::IpAddr;

fn get_date() -> String {
    let now = OffsetDateTime::now_utc();
    format!("{:4}-{:02}-{:02}-{:02}:{:02}",
        now.year(), now.month(), now.day(), now.hour(), now.minute())
}

fn is_valid_domain(s: &str) -> bool {
    if s.len() > 253 {
        return false;
    }

    if !s.chars().all(|c|
        c.is_ascii_alphanumeric() || c == '-' || c == '.')
    {
        return false;
    }

    let labels: Vec<&str> = s.split('.').collect();
    if let Some(tld) = labels.last()
        && tld.len() < 2
    {
        return false;
    }

    if labels.iter().any(|label| label.is_empty()
        || label.len() > 63
        || label.starts_with('-')
        || label.ends_with('-')
    ) {
        return false;
    } 
    true
}

fn has_redis_wildcard(s: &str) -> bool {
    if s.contains(['*', '?', '[', ']', '^']) {
        return true;
    }
    false
}

fn is_public_ip(ip: &IpAddr) -> bool {
    !match ip {
        IpAddr::V4(ipv4) => ipv4.is_loopback()
            || ipv4.is_private()
            || ipv4.is_multicast()
            || ipv4.is_broadcast()
            || ipv4.is_unspecified()
            || ipv4.is_link_local(),
        IpAddr::V6(ipv6) => ipv6.is_loopback()
            || ipv6.is_unique_local()
            || ipv6.is_multicast()
            || ipv6.is_unspecified()
            || ipv6.is_unicast_link_local()
    }
}
