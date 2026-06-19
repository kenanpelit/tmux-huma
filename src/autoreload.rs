//! Config auto-reload (port of tmux-autoreload, no `entr`).
//!
//! Folds into the daemon's existing tick: stat the watched config files and
//! `source-file` them when an mtime changes. mtime polling (vs inotify) needs no
//! dependency and survives editor rename-saves.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use crate::config::Config;
use crate::tmux;

#[derive(Default)]
pub struct Watcher {
    seen: HashMap<PathBuf, SystemTime>,
    primed: bool,
}

impl Watcher {
    pub fn new() -> Self {
        Watcher {
            seen: HashMap::new(),
            primed: false,
        }
    }

    /// One poll cycle. The first cycle only records the baseline (no reload).
    pub fn tick(&mut self, cfg: &Config) {
        if !cfg.autoreload {
            return;
        }
        let files = watched_files(cfg);
        let current: Vec<(PathBuf, Option<SystemTime>)> =
            files.iter().map(|f| (f.clone(), mtime(f))).collect();
        let changed = detect_changes(&mut self.seen, &current);
        if !self.primed {
            self.primed = true;
            return;
        }
        if !changed.is_empty() {
            reload(&files);
        }
    }
}

fn watched_files(cfg: &Config) -> Vec<PathBuf> {
    let mut files: Vec<PathBuf> = tmux::run(&["display-message", "-p", "#{config_files}"])
        .unwrap_or_default()
        .split(',')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .map(PathBuf::from)
        .collect();
    files.extend(cfg.autoreload_configs.iter().map(PathBuf::from));
    files.sort();
    files.dedup();
    files
}

fn mtime(p: &Path) -> Option<SystemTime> {
    std::fs::metadata(p).and_then(|m| m.modified()).ok()
}

/// Update `seen` with the current mtimes and return the files whose mtime
/// changed (a newly-seen file with an mtime counts as changed once).
pub fn detect_changes(
    seen: &mut HashMap<PathBuf, SystemTime>,
    current: &[(PathBuf, Option<SystemTime>)],
) -> Vec<PathBuf> {
    let mut changed = Vec::new();
    for (path, mt) in current {
        let Some(mt) = mt else { continue };
        let differs = seen.get(path) != Some(mt);
        if differs {
            seen.insert(path.clone(), *mt);
            changed.push(path.clone());
        }
    }
    changed
}

fn reload(files: &[PathBuf]) {
    let mut args: Vec<String> = vec!["source-file".to_string()];
    args.extend(files.iter().map(|f| f.to_string_lossy().into_owned()));
    let refs: Vec<&str> = args.iter().map(String::as_str).collect();
    match tmux::run(&refs) {
        Ok(_) => {
            let names: Vec<String> = files
                .iter()
                .filter_map(|f| f.file_name().map(|n| n.to_string_lossy().into_owned()))
                .collect();
            tmux::run_ok(&["display-message", &format!("huma: reloaded {}", names.join(", "))]);
        }
        Err(e) => {
            tmux::run_ok(&["display-message", &format!("huma: reload error: {e}")]);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn t(secs: u64) -> SystemTime {
        SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(secs)
    }

    #[test]
    fn new_file_changes_once_then_is_stable() {
        let mut seen = HashMap::new();
        let cur = vec![(PathBuf::from("/a.conf"), Some(t(100)))];
        assert_eq!(detect_changes(&mut seen, &cur), vec![PathBuf::from("/a.conf")]);
        // same mtime → no change
        assert!(detect_changes(&mut seen, &cur).is_empty());
    }

    #[test]
    fn changed_mtime_is_detected() {
        let mut seen = HashMap::new();
        let _ = detect_changes(&mut seen, &[(PathBuf::from("/a.conf"), Some(t(100)))]);
        let changed = detect_changes(&mut seen, &[(PathBuf::from("/a.conf"), Some(t(200)))]);
        assert_eq!(changed, vec![PathBuf::from("/a.conf")]);
    }

    #[test]
    fn missing_mtime_is_ignored() {
        let mut seen = HashMap::new();
        assert!(detect_changes(&mut seen, &[(PathBuf::from("/gone.conf"), None)]).is_empty());
    }
}
