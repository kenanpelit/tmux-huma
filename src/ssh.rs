//! SSH target of the active pane → `@huma_ssh`.
//!
//! Detects whether the active pane is running `ssh` by walking `/proc` from the
//! pane's shell (robust, vs parsing `ps` output), and shows the destination
//! host (or `user@host` with `@huma-ssh-format user@host`). No window renaming.

use std::fs;

use crate::config::Config;
use crate::tmux;

/// ssh options that consume the following argv token.
const VALUE_OPTS: &[&str] = &[
    "-b", "-c", "-D", "-E", "-e", "-F", "-I", "-i", "-J", "-L", "-l", "-m", "-O", "-o", "-p",
    "-Q", "-R", "-S", "-W", "-w",
];

/// Parse an `ssh` argv into the display target. `user_at_host` controls whether
/// the username is included.
pub fn parse_ssh_target(argv: &[String], user_at_host: bool) -> Option<String> {
    let mut i = 1; // skip argv[0] (ssh)
    let mut user: Option<String> = None;
    let mut dest: Option<String> = None;
    while i < argv.len() {
        let a = &argv[i];
        if a == "-l" {
            user = argv.get(i + 1).cloned();
            i += 2;
            continue;
        }
        if VALUE_OPTS.contains(&a.as_str()) {
            i += 2;
            continue;
        }
        if a.starts_with('-') {
            i += 1;
            continue;
        }
        dest = Some(a.clone());
        break;
    }
    let dest = dest?;
    let dest = dest.strip_prefix("ssh://").unwrap_or(&dest);
    let (u, host) = match dest.split_once('@') {
        Some((u, h)) => (Some(u.to_string()), h),
        None => (user, dest),
    };
    let host = host.split(['/', ':']).next().unwrap_or(host);
    if host.is_empty() {
        return None;
    }
    match u {
        Some(u) if user_at_host && !u.is_empty() => Some(format!("{u}@{host}")),
        _ => Some(host.to_string()),
    }
}

fn comm(pid: i32) -> String {
    fs::read_to_string(format!("/proc/{pid}/comm"))
        .map(|s| s.trim().to_string())
        .unwrap_or_default()
}

fn children(pid: i32) -> Vec<i32> {
    fs::read_to_string(format!("/proc/{pid}/task/{pid}/children"))
        .ok()
        .map(|s| s.split_whitespace().filter_map(|x| x.parse().ok()).collect())
        .unwrap_or_default()
}

fn cmdline(pid: i32) -> Vec<String> {
    fs::read(format!("/proc/{pid}/cmdline"))
        .ok()
        .map(|b| {
            b.split(|c| *c == 0)
                .filter(|p| !p.is_empty())
                .map(|p| String::from_utf8_lossy(p).into_owned())
                .collect()
        })
        .unwrap_or_default()
}

/// Find an `ssh` process under `pid` (shallow walk), return its argv.
fn find_ssh(pid: i32, depth: u8) -> Option<Vec<String>> {
    for c in children(pid) {
        if comm(c) == "ssh" {
            let argv = cmdline(c);
            if !argv.is_empty() {
                return Some(argv);
            }
        }
        if depth > 0 {
            if let Some(a) = find_ssh(c, depth - 1) {
                return Some(a);
            }
        }
    }
    None
}

/// The active pane's ssh target, or empty if it isn't in ssh.
pub fn target(cfg: &Config) -> String {
    let pane_pid = tmux::run(&["display-message", "-p", "#{pane_pid}"])
        .ok()
        .and_then(|s| s.trim().parse::<i32>().ok());
    let Some(pid) = pane_pid else {
        return String::new();
    };
    match find_ssh(pid, 2) {
        Some(argv) => parse_ssh_target(&argv, cfg.ssh_user_at_host).unwrap_or_default(),
        None => String::new(),
    }
}

/// `huma ssh`: refresh `@huma_ssh` for the active pane (called by the
/// pane-focus-in hook and the daemon).
pub fn update(cfg: &Config) {
    tmux::set_global_option("@huma_ssh", &target(cfg));
}

#[cfg(test)]
mod tests {
    use super::*;

    fn argv(s: &str) -> Vec<String> {
        s.split_whitespace().map(String::from).collect()
    }

    #[test]
    fn plain_host() {
        assert_eq!(parse_ssh_target(&argv("ssh grid"), false), Some("grid".into()));
    }
    #[test]
    fn user_at_host_form() {
        assert_eq!(parse_ssh_target(&argv("ssh kenan@grid"), true), Some("kenan@grid".into()));
        assert_eq!(parse_ssh_target(&argv("ssh kenan@grid"), false), Some("grid".into()));
    }
    #[test]
    fn skips_value_options() {
        assert_eq!(
            parse_ssh_target(&argv("ssh -p 2222 -i ~/.ssh/k -o StrictHostKeyChecking=no grid"), false),
            Some("grid".into())
        );
    }
    #[test]
    fn dash_l_user() {
        assert_eq!(parse_ssh_target(&argv("ssh -l deploy grid"), true), Some("deploy@grid".into()));
    }
    #[test]
    fn strips_port_and_path() {
        assert_eq!(parse_ssh_target(&argv("ssh kenan@grid:22"), false), Some("grid".into()));
    }
    #[test]
    fn flags_without_value() {
        assert_eq!(parse_ssh_target(&argv("ssh -4 -t grid htop"), false), Some("grid".into()));
    }
    #[test]
    fn no_dest_is_none() {
        assert_eq!(parse_ssh_target(&argv("ssh -V"), false), None);
    }
}
