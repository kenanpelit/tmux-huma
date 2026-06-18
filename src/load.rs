//! System load from /proc/loadavg (+ optional /proc/meminfo).

use crate::config::Config;

pub fn parse_loadavg(s: &str) -> Option<f64> {
    s.split_whitespace().next()?.parse().ok()
}

pub fn mem_used_pct(meminfo: &str) -> Option<u32> {
    let mut total = 0u64;
    let mut avail = 0u64;
    for line in meminfo.lines() {
        if let Some(v) = line.strip_prefix("MemTotal:") {
            total = v.split_whitespace().next()?.parse().ok()?;
        } else if let Some(v) = line.strip_prefix("MemAvailable:") {
            avail = v.split_whitespace().next()?.parse().ok()?;
        }
    }
    if total == 0 {
        return None;
    }
    let used = total.saturating_sub(avail);
    Some(((used as f64 / total as f64) * 100.0).round() as u32)
}

pub fn format(load: f64, mem: Option<u32>, cfg: &Config) -> String {
    let mut s = format!("{} {:.2}", cfg.load_icon, load);
    if let Some(m) = mem {
        s.push_str(&format!(" · {m}%"));
    }
    s
}

pub fn widget(cfg: &Config) -> String {
    let load = std::fs::read_to_string("/proc/loadavg")
        .ok()
        .and_then(|s| parse_loadavg(&s))
        .unwrap_or(0.0);
    let mem = if cfg.load_mem {
        std::fs::read_to_string("/proc/meminfo")
            .ok()
            .and_then(|s| mem_used_pct(&s))
    } else {
        None
    };
    format(load, mem, cfg)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cfg(mem: bool) -> Config {
        Config {
            interval_secs: 5,
            online_host: "x".into(),
            online_timeout_secs: 1,
            online_latency: true,
            online_up: "✓".into(),
            online_down: "✗".into(),
            battery_low: 20,
            load_mem: mem,
            load_icon: "▟".into(),
            mode_prefix: "P".into(),
            mode_copy: "C".into(),
            mode_sync: "S".into(),
            mode_mouse: "M".into(),
            ssh_user_at_host: false,
        }
    }

    #[test]
    fn loadavg() {
        assert_eq!(parse_loadavg("0.42 0.10 0.05 1/100 1234"), Some(0.42));
    }
    #[test]
    fn mem() {
        assert_eq!(mem_used_pct("MemTotal: 1000 kB\nMemAvailable: 250 kB\n"), Some(75));
    }
    #[test]
    fn fmt_plain() {
        assert_eq!(format(0.42, None, &cfg(false)), "▟ 0.42");
    }
    #[test]
    fn fmt_mem() {
        assert_eq!(format(0.42, Some(38), &cfg(true)), "▟ 0.42 · 38%");
    }
}
