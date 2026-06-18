//! Connectivity probe + display.

use std::net::{TcpStream, ToSocketAddrs};
use std::time::{Duration, Instant};

use crate::config::Config;

/// Latency in ms if a TCP connection to `host` ("ip:port") succeeds, else None.
pub fn probe(host: &str, timeout: Duration) -> Option<u32> {
    let addr = host.to_socket_addrs().ok()?.next()?;
    let start = Instant::now();
    TcpStream::connect_timeout(&addr, timeout).ok()?;
    Some(start.elapsed().as_millis() as u32)
}

pub fn format(latency: Option<u32>, cfg: &Config) -> String {
    match latency {
        Some(ms) if cfg.online_latency => format!("{} {}ms", cfg.online_up, ms),
        Some(_) => cfg.online_up.clone(),
        None => cfg.online_down.clone(),
    }
}

pub fn widget(cfg: &Config) -> String {
    let latency = probe(&cfg.online_host, Duration::from_secs(cfg.online_timeout_secs));
    format(latency, cfg)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;

    fn cfg(latency: bool) -> Config {
        Config {
            interval_secs: 5,
            online_host: "x".into(),
            online_timeout_secs: 1,
            online_latency: latency,
            online_up: "✓".into(),
            online_down: "✗".into(),
            battery_low: 20,
            load_mem: false,
            load_icon: "▟".into(),
            mode_prefix: "P".into(),
            mode_copy: "C".into(),
            mode_sync: "S".into(),
            mode_mouse: "M".into(),
            ssh_user_at_host: false,
        }
    }

    #[test]
    fn up_with_latency() {
        assert_eq!(format(Some(23), &cfg(true)), "✓ 23ms");
    }
    #[test]
    fn up_without_latency() {
        assert_eq!(format(Some(23), &cfg(false)), "✓");
    }
    #[test]
    fn down() {
        assert_eq!(format(None, &cfg(true)), "✗");
    }
}
