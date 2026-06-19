//! `huma suspend` / `huma resume`: a pass-through toggle (port of tmux-suspend).
//!
//! Suspending disables the prefix and switches the key table to `suspended`, so
//! every key except the resume key falls through to whatever runs in the pane —
//! handy when a nested multiplexer (ssh + byobu/tmux) wants the same prefix.
//! Operates at the global scope, matching huma's model; while suspended the
//! session-switching keys are inactive anyway, so global vs per-session is moot.

use anyhow::Result;

use crate::tmux;

const SAVED_PREFIX: &str = "@huma_suspend_prefix";

pub fn suspend() -> Result<()> {
    // Remember the active prefix so resume can put it back exactly.
    let prefix = tmux::global_option("prefix");
    tmux::set_global_option(SAVED_PREFIX, &prefix);

    tmux::set_global_option("prefix", "none");
    tmux::set_global_option("key-table", "suspended");

    // Leave copy-mode and stop synchronized input so the pass-through is clean.
    tmux::run_ok(&["if-shell", "-F", "#{pane_in_mode}", "send-keys -X cancel"]);
    tmux::run_ok(&[
        "if-shell",
        "-F",
        "#{pane_synchronized}",
        "set -w synchronize-panes off",
    ]);

    // Force an immediate status redraw so the mode badge appears at once.
    tmux::run_ok(&["refresh-client", "-S"]);
    Ok(())
}

pub fn resume() -> Result<()> {
    let prefix = tmux::global_option(SAVED_PREFIX);
    if prefix.is_empty() {
        tmux::run_ok(&["set-option", "-gu", "prefix"]);
    } else {
        tmux::set_global_option("prefix", &prefix);
    }
    tmux::run_ok(&["set-option", "-gu", "key-table"]);
    tmux::run_ok(&["refresh-client", "-S"]);
    Ok(())
}
