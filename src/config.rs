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
    pub icon_shell: String,
    pub icon_editor: String,
    pub icon_default: String,
    pub mode_suspend: String,
    pub autoreload: bool,
    pub autoreload_configs: Vec<String>,
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
            icon_shell: opt_or("@huma-icon-shell", ""),
            icon_editor: opt_or("@huma-icon-editor", ""),
            icon_default: opt_or("@huma-icon-default", "?"),
            mode_suspend: opt_or("@huma-mode-suspend", "󰒲"),
            autoreload: opt_bool("@huma-autoreload", false),
            autoreload_configs: opt_or("@huma-autoreload-configs", "")
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect(),
        }
    }

    #[cfg(test)]
    pub fn test() -> Self {
        Config {
            interval_secs: 5,
            online_host: "x".into(),
            online_timeout_secs: 1,
            online_latency: true,
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
            kripto_coins: vec![],
            kripto_currency: "usd".into(),
            kripto_symbol: "$".into(),
            kripto_ttl: 300,
            player_format: "{{artist}} - {{title}}".into(),
            player_max: 40,
            player_playing: "▶".into(),
            player_paused: "⏸".into(),
            player_name: String::new(),
            icon_shell: String::new(),
            icon_editor: String::new(),
            icon_default: "?".into(),
            mode_suspend: "Z".into(),
            autoreload: false,
            autoreload_configs: vec![],
        }
    }
}
