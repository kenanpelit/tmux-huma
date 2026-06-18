//! Thin wrapper around the `tmux` CLI.

use anyhow::{bail, Context, Result};
use std::process::Command;

pub fn run(args: &[&str]) -> Result<String> {
    let out = Command::new("tmux").args(args).output().context("spawn tmux")?;
    if !out.status.success() {
        bail!(
            "tmux {} failed: {}",
            args.join(" "),
            String::from_utf8_lossy(&out.stderr).trim()
        );
    }
    Ok(String::from_utf8_lossy(&out.stdout)
        .trim_end_matches('\n')
        .to_string())
}

pub fn run_ok(args: &[&str]) {
    let _ = Command::new("tmux").args(args).output();
}

pub fn server_running() -> bool {
    Command::new("tmux")
        .arg("list-sessions")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

pub fn global_option(name: &str) -> String {
    run(&["show-options", "-gqv", name]).unwrap_or_default()
}

pub fn set_global_option(name: &str, value: &str) {
    run_ok(&["set-option", "-g", name, value]);
}

pub fn server_option(name: &str) -> String {
    run(&["show-options", "-sqv", name]).unwrap_or_default()
}

pub fn window_option(name: &str) -> String {
    run(&["show-options", "-gwqv", name]).unwrap_or_default()
}

pub fn set_server_option(name: &str, value: &str) {
    run_ok(&["set-option", "-s", name, value]);
}

pub fn set_window_option(name: &str, value: &str) {
    run_ok(&["set-option", "-gw", name, value]);
}
