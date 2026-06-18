mod common;
use common::*;
use std::time::Duration;

#[test]
fn once_sets_widget_options() {
    if !has_tmux() {
        eprintln!("skip");
        return;
    }
    let s = Server::start("once");
    assert!(s.huma(&["once"]).status.success());
    // load is always available on Linux; assert it got written (icon prefix ▟)
    assert!(
        s.opt("@huma_load").starts_with('▟'),
        "load='{}'",
        s.opt("@huma_load")
    );
    // online is either ✓… or ✗ — non-empty
    assert!(!s.opt("@huma_online").is_empty());
}

#[test]
fn daemon_updates_then_exits_when_server_gone() {
    if !has_tmux() {
        eprintln!("skip");
        return;
    }
    let s = Server::start("daemon");
    let mut child = s
        .huma_command(&["daemon"])
        .env("HUMA_DAEMON_INTERVAL_MS", "100")
        .spawn()
        .expect("spawn");
    std::thread::sleep(Duration::from_millis(400));
    assert!(
        s.opt("@huma_load").starts_with('▟'),
        "daemon did not write @huma_load"
    );
    s.tmux(&["kill-server"]);
    assert!(
        wait_for_exit(&mut child, Duration::from_secs(3)),
        "daemon did not exit after server gone"
    );
}
