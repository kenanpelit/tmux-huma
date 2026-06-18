//! Window-name icons (replaces tmux-nerd-font-window-name, no `yq`/YAML).
//!
//! `huma icon <command>` prints a single Nerd Font glyph for the given command
//! (typically `#{pane_current_command}`); compose the window name yourself in
//! `automatic-rename-format`, e.g.
//! `"#(… huma icon '#{pane_current_command}') #{b:pane_current_path}"`.
//!
//! `@huma-icon-shell` / `@huma-icon-editor` override the glyph for all shells /
//! editors (empty = use the built-in per-command glyph); `@huma-icon-default`
//! is the fallback for unknown commands.

use crate::config::Config;
use crate::icon_map;

fn is_shell(cmd: &str) -> bool {
    matches!(
        cmd,
        "bash" | "zsh" | "fish" | "sh" | "dash" | "nu" | "tcsh" | "csh" | "ksh" | "pwsh"
            | "powershell"
    )
}

fn is_editor(cmd: &str) -> bool {
    matches!(
        cmd,
        "nvim" | "vim" | "vi" | "hx" | "helix" | "nano" | "emacs" | "lvim" | "code"
            | "code-insiders" | "micro" | "kak"
    )
}

/// Resolve the icon for a command: category override → built-in map → fallback.
pub fn icon(cmd: &str, cfg: &Config) -> String {
    let c = cmd.trim().to_lowercase();
    if is_shell(&c) && !cfg.icon_shell.is_empty() {
        return cfg.icon_shell.clone();
    }
    if is_editor(&c) && !cfg.icon_editor.is_empty() {
        return cfg.icon_editor.clone();
    }
    match icon_map::lookup(&c) {
        Some(g) => g.to_string(),
        None => cfg.icon_default.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cfg(shell: &str, editor: &str, default: &str) -> Config {
        let mut c = Config::test();
        c.icon_shell = shell.into();
        c.icon_editor = editor.into();
        c.icon_default = default.into();
        c
    }

    #[test]
    fn shell_override_wins() {
        assert_eq!(icon("zsh", &cfg("S", "", "?")), "S");
        assert_eq!(icon("ZSH", &cfg("S", "", "?")), "S");
    }

    #[test]
    fn editor_override_wins() {
        assert_eq!(icon("nvim", &cfg("", "E", "?")), "E");
    }

    #[test]
    fn empty_override_falls_back_to_builtin() {
        // no override → built-in glyph (non-empty, not the fallback)
        let g = icon("zsh", &cfg("", "", "?"));
        assert!(!g.is_empty() && g != "?");
    }

    #[test]
    fn known_command_uses_map() {
        assert_eq!(icon("git", &cfg("", "", "?")), icon_map::lookup("git").unwrap());
    }

    #[test]
    fn unknown_uses_default() {
        assert_eq!(icon("definitelynotacommand", &cfg("", "", "?")), "?");
    }
}
