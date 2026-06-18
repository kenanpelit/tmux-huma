//! The `@huma_mode` badge: a static tmux format string (no daemon).

use crate::config::Config;

pub fn build_mode(cfg: &Config) -> String {
    format!(
        "#{{?client_prefix,{p} ,}}#{{?pane_in_mode,{c} ,}}#{{?pane_synchronized,{s} ,}}#{{?mouse,{m} ,}}",
        p = cfg.mode_prefix,
        c = cfg.mode_copy,
        s = cfg.mode_sync,
        m = cfg.mode_mouse,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cfg() -> Config {
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
        }
    }

    #[test]
    fn mode_embeds_conditionals_and_icons() {
        let s = build_mode(&cfg());
        assert!(s.contains("#{?client_prefix,P ,}"));
        assert!(s.contains("#{?pane_in_mode,C ,}"));
        assert!(s.contains("#{?pane_synchronized,S ,}"));
        assert!(s.contains("#{?mouse,M ,}"));
    }
}
