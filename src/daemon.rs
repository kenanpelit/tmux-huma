//! Update daemon: periodically write the widget user options. Single-instance.

use anyhow::{bail, Result};
use std::fs;
use std::path::{Path, PathBuf};
use std::thread;
use std::time::Duration;

use crate::config::Config;
use crate::{battery, load, online, ssh, tmux};

pub fn once(cfg: &Config) -> Result<()> {
    if !tmux::server_running() {
        bail!("no tmux server running");
    }
    tmux::set_global_option("@huma_online", &online::widget(cfg));
    tmux::set_global_option("@huma_battery", &battery::widget(cfg));
    tmux::set_global_option("@huma_load", &load::widget(cfg));
    tmux::set_global_option("@huma_ssh", &ssh::target(cfg));
    Ok(())
}

pub fn run(cfg: &Config) -> Result<()> {
    let lock = lock_path();
    if another_running(&lock) {
        return Ok(());
    }
    write_pid(&lock);
    let interval = interval(cfg);
    loop {
        let _ = once(cfg);
        thread::sleep(interval);
        if !tmux::server_running() {
            break;
        }
    }
    let _ = fs::remove_file(&lock);
    Ok(())
}

fn interval(cfg: &Config) -> Duration {
    if let Ok(ms) = std::env::var("HUMA_DAEMON_INTERVAL_MS") {
        if let Ok(ms) = ms.parse::<u64>() {
            return Duration::from_millis(ms);
        }
    }
    Duration::from_secs(cfg.interval_secs)
}

fn lock_path() -> PathBuf {
    let dir = std::env::var("XDG_RUNTIME_DIR").unwrap_or_else(|_| "/tmp".into());
    PathBuf::from(dir).join("huma.daemon.lock")
}

fn another_running(lock: &Path) -> bool {
    fs::read_to_string(lock)
        .ok()
        .and_then(|s| s.trim().parse::<i32>().ok())
        .is_some_and(|pid| Path::new(&format!("/proc/{pid}")).exists())
}

fn write_pid(lock: &Path) {
    let _ = fs::write(lock, std::process::id().to_string());
}
