//! `@huma-*` configuration, read from tmux global options.

use crate::tmux;

pub struct Config {
    pub interval_secs: u64,
    pub online_host: String,
    pub online_timeout_secs: u64,
    pub online_latency: bool,
    pub online_up: String,
    pub online_down: String,
    pub battery_low: u32,
    pub load_mem: bool,
    pub load_icon: String,
    pub mode_prefix: String,
    pub mode_copy: String,
    pub mode_sync: String,
    pub mode_mouse: String,
    pub ssh_user_at_host: bool,
    pub kripto_coins: Vec<String>,
    pub kripto_currency: String,
    pub kripto_symbol: String,
    pub kripto_ttl: u64,
    pub player_format: String,
    pub player_max: usize,
    pub player_playing: String,
    pub player_paused: String,
    pub player_name: String,
}

fn opt_or(name: &str, default: &str) -> String {
    let v = tmux::global_option(name);
    if v.is_empty() {
        default.to_string()
    } else {
        v
    }
}

fn opt_bool(name: &str, default: bool) -> bool {
    match tmux::global_option(name).as_str() {
        "" => default,
        "on" | "1" | "true" | "yes" => true,
        _ => false,
    }
}

impl Config {
    pub fn load() -> Self {
        Config {
            interval_secs: opt_or("@huma-interval", "5").parse().unwrap_or(5),
            online_host: opt_or("@huma-online-host", "1.1.1.1:443"),
            online_timeout_secs: opt_or("@huma-online-timeout", "1").parse().unwrap_or(1),
            online_latency: opt_bool("@huma-online-latency", true),
            online_up: opt_or("@huma-online-icon-up", "✓"),
            online_down: opt_or("@huma-online-icon-down", "✗"),
            battery_low: opt_or("@huma-battery-low", "20").parse().unwrap_or(20),
            load_mem: opt_bool("@huma-load-mem", false),
            load_icon: opt_or("@huma-load-icon", "▟"),
            mode_prefix: opt_or("@huma-mode-prefix", "⌨"),
            mode_copy: opt_or("@huma-mode-copy", "❐"),
            mode_sync: opt_or("@huma-mode-sync", "⚏"),
            mode_mouse: opt_or("@huma-mode-mouse", "↗"),
            ssh_user_at_host: opt_or("@huma-ssh-format", "host") == "user@host",
            kripto_coins: opt_or("@huma-kripto-coins", "")
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect(),
            kripto_currency: opt_or("@huma-kripto-currency", "usd"),
            kripto_symbol: opt_or("@huma-kripto-symbol", "$"),
            kripto_ttl: opt_or("@huma-kripto-ttl", "300").parse().unwrap_or(300),
            player_format: opt_or("@huma-player-format", "{{artist}} - {{title}}"),
            player_max: opt_or("@huma-player-max", "40").parse().unwrap_or(40),
            player_playing: opt_or("@huma-player-playing", "▶"),
            player_paused: opt_or("@huma-player-paused", "⏸"),
            player_name: opt_or("@huma-player-name", ""),
        }
    }
}
