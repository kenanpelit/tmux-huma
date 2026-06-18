//! Now-playing widget wrapping the `playerctl` CLI.
//!
//! The daemon polls each tick and writes `@huma_player`. With several MPRIS
//! players around (browser, Spotify, …) the bare `playerctl status` reports
//! whichever is "first", which is usually the wrong, stopped one — so we list
//! the players and pick the one that is actually Playing (or Paused), honouring
//! an optional `@huma-player-name` preference. Empty when nothing is playing.

use std::process::Command;

use crate::config::Config;
use crate::tmux;

fn run(args: &[&str]) -> Option<String> {
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

fn players() -> Vec<String> {
    run(&["-l"])
        .map(|s| {
            s.lines()
                .map(|l| l.trim().to_string())
                .filter(|l| !l.is_empty())
                .collect()
        })
        .unwrap_or_default()
}

/// Pick which player to show: a configured `preferred` name wins (prefix match,
/// so `spotify` matches `spotify.instance123`), otherwise the first Playing
/// player, otherwise the first Paused one.
pub fn choose<'a>(
    statuses: &'a [(String, String)],
    preferred: &str,
) -> Option<&'a (String, String)> {
    if !preferred.is_empty() {
        if let Some(s) = statuses
            .iter()
            .find(|(n, _)| n == preferred || n.starts_with(preferred))
        {
            return Some(s);
        }
    }
    statuses
        .iter()
        .find(|(_, s)| s == "Playing")
        .or_else(|| statuses.iter().find(|(_, s)| s == "Paused"))
}

/// Tidy metadata: podcasts/streams often have an empty artist, leaving a
/// dangling `-` around the title.
pub fn clean(text: &str) -> String {
    text.trim()
        .trim_start_matches('-')
        .trim_end_matches('-')
        .trim()
        .to_string()
}

fn truncate(text: &str, max: usize) -> String {
    if max == 0 || text.chars().count() <= max {
        return text.to_string();
    }
    let kept: String = text.chars().take(max.saturating_sub(1)).collect();
    format!("{kept}…")
}

/// Build the widget from a player status + cleaned metadata. Stopped/unknown
/// states render nothing; empty metadata still shows the play/pause icon.
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
    let names = players();
    if names.is_empty() {
        return String::new();
    }
    let statuses: Vec<(String, String)> = names
        .iter()
        .filter_map(|p| run(&["-p", p, "status"]).map(|s| (p.clone(), s)))
        .collect();
    let Some((name, status)) = choose(&statuses, &cfg.player_name) else {
        return String::new();
    };
    let text = run(&["-p", name, "metadata", "--format", &cfg.player_format]).unwrap_or_default();
    format_player(status, &clean(&text), cfg)
}

pub fn update(cfg: &Config) {
    tmux::set_global_option("@huma_player", &value(cfg));
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cfg(max: usize, preferred: &str) -> Config {
        let mut c = Config::test();
        c.player_max = max;
        c.player_name = preferred.into();
        c
    }

    fn s(pairs: &[(&str, &str)]) -> Vec<(String, String)> {
        pairs
            .iter()
            .map(|(a, b)| (a.to_string(), b.to_string()))
            .collect()
    }

    #[test]
    fn prefers_playing_over_first() {
        let st = s(&[("chromium.x", "Stopped"), ("spotify", "Playing")]);
        assert_eq!(choose(&st, "").unwrap().0, "spotify");
    }

    #[test]
    fn falls_back_to_paused() {
        let st = s(&[("chromium.x", "Stopped"), ("mpv", "Paused")]);
        assert_eq!(choose(&st, "").unwrap().0, "mpv");
    }

    #[test]
    fn none_when_all_stopped() {
        let st = s(&[("a", "Stopped"), ("b", "Stopped")]);
        assert!(choose(&st, "").is_none());
    }

    #[test]
    fn preferred_wins_by_prefix() {
        let st = s(&[("spotify.instance7", "Paused"), ("mpv", "Playing")]);
        assert_eq!(choose(&st, "spotify").unwrap().0, "spotify.instance7");
    }

    #[test]
    fn clean_strips_dangling_dash() {
        assert_eq!(clean("- Rabarba 1597"), "Rabarba 1597");
        assert_eq!(clean("Daft Punk - Aerodynamic"), "Daft Punk - Aerodynamic");
        assert_eq!(clean("Title -"), "Title");
    }

    #[test]
    fn format_and_truncate() {
        assert_eq!(format_player("Playing", "x - y", &cfg(40, "")), "▶ x - y");
        assert_eq!(format_player("Stopped", "x", &cfg(40, "")), "");
        assert_eq!(format_player("Playing", "abcdefghij", &cfg(5, "")), "▶ abcd…");
        assert_eq!(format_player("Playing", "", &cfg(40, "")), "▶");
    }
}
