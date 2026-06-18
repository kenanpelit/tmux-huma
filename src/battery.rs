//! Battery status from /sys/class/power_supply.

use std::fs;
use std::path::PathBuf;

use crate::config::Config;

pub fn format(capacity: u32, status: &str, cfg: &Config) -> String {
    let mark = match status {
        "Charging" => "⚡",
        _ if capacity <= cfg.battery_low => "!",
        _ => "",
    };
    if mark.is_empty() {
        format!("{capacity}%")
    } else {
        format!("{mark}{capacity}%")
    }
}

fn battery_dir() -> Option<PathBuf> {
    fs::read_dir("/sys/class/power_supply")
        .ok()?
        .flatten()
        .map(|e| e.path())
        .find(|p| {
            p.file_name()
                .and_then(|n| n.to_str())
                .map(|n| n.starts_with("BAT"))
                .unwrap_or(false)
        })
}

pub fn widget(cfg: &Config) -> String {
    let Some(dir) = battery_dir() else {
        return String::new();
    };
    let cap = fs::read_to_string(dir.join("capacity"))
        .ok()
        .and_then(|s| s.trim().parse::<u32>().ok());
    let status = fs::read_to_string(dir.join("status"))
        .map(|s| s.trim().to_string())
        .unwrap_or_default();
    match cap {
        Some(c) => format(c, &status, cfg),
        None => String::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cfg() -> Config {
        Config::test()
    }

    #[test]
    fn charging() {
        assert_eq!(format(50, "Charging", &cfg()), "⚡50%");
    }
    #[test]
    fn low() {
        assert_eq!(format(12, "Discharging", &cfg()), "!12%");
    }
    #[test]
    fn normal() {
        assert_eq!(format(80, "Discharging", &cfg()), "80%");
    }
}
