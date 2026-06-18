#![allow(dead_code)]
use std::path::PathBuf;
use std::process::{Child, Command, Output};
use std::time::{Duration, Instant};

pub fn has_tmux() -> bool {
    Command::new("tmux")
        .arg("-V")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

fn tmux_raw(socket: &str, args: &[&str]) -> Output {
    Command::new("tmux")
        .args(["-f", "/dev/null", "-L", socket])
        .args(args)
        .output()
        .expect("spawn tmux")
}

pub fn tmux(socket: &str, args: &[&str]) -> String {
    let o = tmux_raw(socket, args);
    assert!(
        o.status.success(),
        "tmux -L {socket} {args:?}: {}",
        String::from_utf8_lossy(&o.stderr)
    );
    String::from_utf8_lossy(&o.stdout)
        .trim_end_matches('\n')
        .to_string()
}

pub fn wait_for_exit(child: &mut Child, timeout: Duration) -> bool {
    let start = Instant::now();
    while start.elapsed() < timeout {
        match child.try_wait() {
            Ok(Some(_)) => return true,
            Ok(None) => std::thread::sleep(Duration::from_millis(50)),
            Err(_) => return true,
        }
    }
    let _ = child.kill();
    false
}

pub struct Server {
    pub socket: String,
    pub dir: PathBuf,
    pub socket_path: String,
    pub pid: String,
}

impl Server {
    pub fn start(tag: &str) -> Self {
        let uniq = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let socket = format!("huma-it-{uniq}-{tag}");
        let dir = std::env::temp_dir().join(format!("huma-it-{uniq}-{tag}"));
        let _ = std::fs::create_dir_all(&dir);
        tmux_raw(&socket, &["kill-server"]);
        tmux(&socket, &["new-session", "-d", "-s", "scratch", "-x", "200", "-y", "50"]);
        let socket_path = tmux(&socket, &["display-message", "-p", "#{socket_path}"]);
        let pid = tmux(&socket, &["display-message", "-p", "#{pid}"]);
        Server { socket, dir, socket_path, pid }
    }

    pub fn tmux(&self, args: &[&str]) -> String {
        tmux(&self.socket, args)
    }

    pub fn huma_command(&self, args: &[&str]) -> Command {
        let mut c = Command::new(env!("CARGO_BIN_EXE_huma"));
        c.args(args)
            .env("TMUX", format!("{},{},0", self.socket_path, self.pid))
            .env("XDG_RUNTIME_DIR", &self.dir); // isolate the daemon lockfile
        c
    }

    pub fn huma(&self, args: &[&str]) -> Output {
        self.huma_command(args).output().expect("spawn huma")
    }

    pub fn opt(&self, name: &str) -> String {
        self.tmux(&["show-options", "-gqv", name])
    }
}

impl Drop for Server {
    fn drop(&mut self) {
        tmux_raw(&self.socket, &["kill-server"]);
        let _ = std::fs::remove_file(&self.socket_path);
        let _ = std::fs::remove_dir_all(&self.dir);
    }
}
