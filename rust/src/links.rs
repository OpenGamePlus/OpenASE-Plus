//! `openase://connect/<game>/<host>:<port>` links, from a browser or a chat.
//!
//! A link comes from outside: [`parse`] accepts exactly that shape (a catalog
//! game id, an IPv4 address or host name, a port) and nothing else. Opening
//! a link never joins by itself; the GUI asks first.

use crate::catalog;

pub const SCHEME: &str = "openase";
const MAX_LENGTH: usize = 300;

/// The link to a server.
pub fn build(game_id: &str, address: &str) -> String {
    format!("{SCHEME}://connect/{game_id}/{address}")
}

/// Game id and `host:port` of a valid link.
pub fn parse(url: &str) -> Option<(String, String)> {
    if url.len() > MAX_LENGTH {
        return None;
    }
    let (scheme, rest) = url.split_once("://")?;
    if !scheme.eq_ignore_ascii_case(SCHEME) {
        return None;
    }
    let rest = rest.strip_suffix('/').unwrap_or(rest);
    let mut parts = rest.split('/');
    let (action, game, address) = (parts.next()?, parts.next()?, parts.next()?);
    if parts.next().is_some() || !action.eq_ignore_ascii_case("connect") {
        return None;
    }
    let game = catalog::game(game)?;
    let (host, port) = address.rsplit_once(':')?;
    let port: u16 = port.parse().ok().filter(|&p| p != 0)?;
    if !port_digits(address) || !valid_host(host) {
        return None;
    }
    Some((game.id.clone(), format!("{host}:{port}")))
}

fn port_digits(address: &str) -> bool {
    address
        .rsplit_once(':')
        .is_some_and(|(_, p)| !p.is_empty() && p.chars().all(|c| c.is_ascii_digit()))
}

/// An IPv4 address or a DNS name: letters, digits, `-` and `.`, labels not
/// starting or ending with `-`.
pub fn valid_host(host: &str) -> bool {
    if host.parse::<std::net::Ipv4Addr>().is_ok() {
        return true;
    }
    !host.is_empty()
        && host.len() <= 253
        && host.split('.').all(|label| {
            !label.is_empty()
                && label.len() <= 63
                && !label.starts_with('-')
                && !label.ends_with('-')
                && label.chars().all(|c| c.is_ascii_alphanumeric() || c == '-')
        })
}
