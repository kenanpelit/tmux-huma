# huma: suspend + autoreload (design)

Date: 2026-06-19 · Target: tmux-huma v0.6.0

Port two small tmux plugins into huma as native subcommands, with **zero new
crate dependencies** (std + the existing `tmux.rs` wrapper):

- **tmux-suspend** → `huma suspend` / `huma resume`
- **tmux-autoreload** → folded into the existing `huma daemon` loop

## 1. Suspend (port of tmux-suspend)

A pass-through toggle: disable the prefix and switch the key table to `suspended`
so every key except the resume key passes through to a nested (e.g. SSH/byobu)
session. The user's fzf launchers are full of `ssh … byobu` sessions, so prefix
collisions with the inner multiplexer are the motivating case.

**`huma suspend`**
1. Save the active prefix into `@huma_suspend_prefix`.
2. `set -g prefix none` and `set -g key-table suspended`.
3. If in copy-mode, cancel it; if `synchronize-panes` is on, turn it off.
4. `refresh-client -S` (forces an immediate status redraw so the badge appears).

**`huma resume`**
1. Restore `prefix` from `@huma_suspend_prefix` (unset if it was empty).
2. `set -gu key-table` (back to `root`).
3. `refresh-client -S`.

**Scope decision:** operates at the **global/server** scope (`-g`), not the
original's per-session scope. This fits huma's global model and is correct for
this user (single client per session; while suspended, session-switching keys
are inactive anyway). Noted as a deliberate simplification.

**Visual indicator:** the original drove a generic `@suspend_suspended_options`
DSL aimed at `tmux-mode-indicator` (not used here). Dropped. Instead the
huma-native **mode badge** gains a suspend segment:

```
#{?#{==:#{client_key_table},suspended},<badge> ,}
```

appended in `mode::build_mode`, badge from `@huma-mode-suspend` (default `󰒲`).
Because the status bar renders `#{E:@huma_mode}` every interval, the badge shows
while suspended and clears on resume — no daemon work.

**Key binding (huma.tmux):** `@huma-suspend-key` (default `F12`):

```
tmux bind -T root      F12 run-shell "$BINARY suspend"
tmux bind -T suspended F12 run-shell "$BINARY resume"
```

Set `@huma-suspend-key 'none'` (or `off`) to disable the binding.

**Dropped (YAGNI):** the options snapshot/restore DSL, and the
`@suspend_on_suspend_command` / `@suspend_on_resume_command` hooks.

## 2. Autoreload (port of tmux-autoreload)

The original forks a background watcher built on **`entr`** (external dep). huma
already runs a single-instance daemon loop, so autoreload folds in there: no
extra process, no `entr`.

- **Mechanism:** mtime polling (zero deps; more robust than inotify against
  editor rename-saves). Reuses the **daemon's existing 5 s tick** — no separate
  thread or interval.
- **Watched files:** `#{config_files}` (the user's `~/.config/tmux/tmux.conf`)
  plus optional `@huma-autoreload-configs` (comma list).
- **Each tick:** stat every watched file. The first observation primes the
  baseline (no reload). Afterwards, if any mtime changed, `tmux source-file
  <files>` and `display-message "huma: reloaded …"`; on error, show the error.
- **Gate:** `@huma-autoreload` (default `off`; the user enables `on`).
- **No reload loop:** the new mtime is recorded before sourcing; re-sourcing
  re-runs `huma daemon &`, but the single-instance lockfile makes the second
  daemon exit immediately.

**Dropped (YAGNI):** the `entr` dependency, the `entrypoints` DSL, and the
standalone PID / `-k` / `-s` lifecycle (the daemon already owns lifecycle).

## 3. Files touched (canonical `~/.kod/tmux/tmux-huma`)

| File | Change |
|------|--------|
| `Cargo.toml` | version 0.5.0 → 0.6.0 (bump **before** `--release` build) |
| `src/cli.rs` | `Suspend`, `Resume` subcommands |
| `src/main.rs` | `mod suspend; mod autoreload;` + wire the two arms |
| `src/suspend.rs` | new: `suspend()` / `resume()` |
| `src/autoreload.rs` | new: `Watcher` + pure `detect_changes` |
| `src/mode.rs` | append the suspend badge segment |
| `src/config.rs` | `mode_suspend`, `autoreload`, `autoreload_configs` (+ `test()`) |
| `src/daemon.rs` | construct a `Watcher`, call `tick(cfg)` each loop |
| `huma.tmux` | bind `@huma-suspend-key` (root/suspended) |
| `README.md` | suspend + autoreload docs |

## 4. Tests

- `mode.rs`: assert the rendered format contains the `suspended` conditional.
- `autoreload.rs`: unit-test `detect_changes` (pure: new file → changed once;
  unchanged mtime → not changed; changed mtime → changed).
- `tests/suspend_integration.rs`: start a server, `huma suspend` → assert
  `prefix == none` and `key-table == suspended`; `huma resume` → assert both
  restored. Gated on `has_tmux()` like the existing integration tests.

## 5. User config (cachy `tmux.conf`)

- `@huma-autoreload on` in the huma settings block.
- Cheatsheet (`prefix + ?`): add `F12  Suspend / Resume (huma)`.

## 6. Ship (per repo-layout notes)

Bump → `cargo test`/`clippy` → `cargo build --release` → commit + annotated tag
`v0.6.0` → push HEAD + tag (CI builds release assets). Then in the `.cachy`
gitlink clone: `git fetch --tags && git checkout v0.6.0`, `pkill -x huma`,
refresh `bin/huma`. Then `.cachy`: add the gitlink + tmux.conf, commit, push.
Live-apply: source the config, restart the daemon, verify the binds, the
`@huma-autoreload` option, and `@huma_mode`.
