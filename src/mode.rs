//! The `@huma_mode` badge: a static tmux format string (no daemon).

use crate::config::Config;

pub fn build_mode(cfg: &Config) -> String {
    format!(
        "#{{?client_prefix,{p} ,}}#{{?pane_in_mode,{c} ,}}#{{?pane_synchronized,{s} ,}}#{{?mouse,{m} ,}}#{{?#{{==:#{{client_key_table}},suspended}},{z} ,}}",
        p = cfg.mode_prefix,
        c = cfg.mode_copy,
        s = cfg.mode_sync,
        m = cfg.mode_mouse,
        z = cfg.mode_suspend,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cfg() -> Config {
        Config::test()
    }

    #[test]
    fn mode_embeds_conditionals_and_icons() {
        let s = build_mode(&cfg());
        assert!(s.contains("#{?client_prefix,P ,}"));
        assert!(s.contains("#{?pane_in_mode,C ,}"));
        assert!(s.contains("#{?pane_synchronized,S ,}"));
        assert!(s.contains("#{?mouse,M ,}"));
        assert!(s.contains("#{?#{==:#{client_key_table},suspended},Z ,}"));
    }
}
