//! Which server names lead to this computer. The account form turns
//! certificate checking off only for these, and says so; the engine refuses
//! an unverified or unencrypted connection to any other host. The rule is the
//! engine's: `localhost` in any letter case, with or without a trailing dot,
//! and every loopback address, with or without brackets.

use std::net::IpAddr;

/// True for `localhost` and for loopback addresses (all of 127.0.0.0/8 and
/// `::1`, also written as an IPv4-mapped address), with or without brackets.
pub fn is_local_host(host: &str) -> bool {
    let bare = host.trim_start_matches('[').trim_end_matches(']');
    if bare.trim_end_matches('.').eq_ignore_ascii_case("localhost") {
        return true;
    }
    match bare.parse::<IpAddr>() {
        Ok(IpAddr::V4(v4)) => v4.is_loopback(),
        Ok(IpAddr::V6(v6)) => v6.is_loopback() || v6.to_ipv4_mapped().is_some_and(|v4| v4.is_loopback()),
        Err(_) => false,
    }
}

#[cfg(test)]
include!("local_host.tests.rs");
