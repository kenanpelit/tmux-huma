//! Now-playing widget wrapping the `playerctl` CLI.
//!
//! The daemon polls `playerctl status` + `playerctl metadata` each tick and
//! writes `@huma_player`. Empty when no player is running or it is stopped.

use std::process::Command;

use crate::config::Config;
use crate::tmux;

fn playerctl(args: &[&str]) -> Option<String> {
    let out = Command::new("playerctl").args(args).output().ok()?;
    if !out.status.success() {
        return None;
    }
    let s = String::from_utf8_lossy(&out.stdout).trim().to_string();
    if s.is_empty() {
        None
    } else {
        Some(s)
    }
}

fn truncate(text: &str, max: usize) -> String {
    if max == 0 || text.chars().count() <= max {
        return text.to_string();
    }
    let kept: String = text.chars().take(max.saturating_sub(1)).collect();
    format!("{kept}…")
}

/// Build the widget from a playerctl status + metadata string. Stopped/unknown
/// states render nothing; an empty metadata still shows the play/pause icon.
pub fn format_player(status: &str, text: &str, cfg: &Config) -> String {
    let icon = match status {
        "Playing" => &cfg.player_playing,
        "Paused" => &cfg.player_paused,
        _ => return String::new(),
    };
    let t = truncate(text, cfg.player_max);
    if t.is_empty() {
        icon.clone()
    } else {
        format!("{icon} {t}")
    }
}

pub fn value(cfg: &Config) -> String {
    let Some(status) = playerctl(&["status"]) else {
        return String::new();
    };
    let text = playerctl(&["metadata", "--format", &cfg.player_format]).unwrap_or_default();
    format_player(&status, &text, cfg)
}

pub fn update(cfg: &Config) {
    tmux::set_global_option("@huma_player", &value(cfg));
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cfg(max: usize) -> Config {
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
            player_max: max,
            player_playing: "▶".into(),
            player_paused: "⏸".into(),
        }
    }

    #[test]
    fn playing() {
        assert_eq!(format_player("Playing", "Daft Punk - Aerodynamic", &cfg(40)), "▶ Daft Punk - Aerodynamic");
    }

    #[test]
    fn paused() {
        assert_eq!(format_player("Paused", "x - y", &cfg(40)), "⏸ x - y");
    }

    #[test]
    fn stopped_is_blank() {
        assert_eq!(format_player("Stopped", "x - y", &cfg(40)), "");
        assert_eq!(format_player("No players found", "", &cfg(40)), "");
    }

    #[test]
    fn icon_only_when_no_metadata() {
        assert_eq!(format_player("Playing", "", &cfg(40)), "▶");
    }

    #[test]
    fn truncates_long_titles() {
        assert_eq!(format_player("Playing", "abcdefghij", &cfg(5)), "▶ abcd…");
    }
}
