# tmux-huma — Design

Status: approved 2026-06-18. Sibling to `tmux-anka`.

## Goal

One small plugin for tmux status-bar "awareness" widgets — online, mode badge,
battery, load — as a single zero-runtime-dependency Rust binary. Replaces
`tmux-online-status` and `tmux-prefix-highlight`, adds battery + load, and is
**non-blocking** (a background daemon updates values; the status bar reads tmux
user options). Linux-only (`/proc`, `/sys`). URL/file opening is out of scope —
that belongs to `extrakto`.

## Components

```
huma.tmux            TPM entrypoint (bash): resolve/install binary, set
                     @huma_mode, paint initial values, start the daemon.
scripts/install-binary.sh   download-release-or-cargo-build
src/
  main.rs / cli.rs   clap dispatch: daemon | once | mode | online | battery | load
  tmux.rs            tmux CLI wrapper (run + set/show user options)
  config.rs          read @huma-* via `tmux show-options -gqv`
  online.rs          TCP-connect reachability + latency
  battery.rs         /sys/class/power_supply parse
  load.rs            /proc/loadavg (+ optional /proc/meminfo)
  mode.rs            build the @huma_mode format string
  daemon.rs          single-instance loop; write @huma_online/_battery/_load
.github/workflows/release.yml   tag v* → static x86_64/aarch64 binaries
```

## Data flow

Time-varying widgets (online, battery, load) are produced by a background
**daemon** that writes tmux user options every `@huma-interval` seconds; the
status bar reads `#{@huma_online}` / `#{@huma_battery}` / `#{@huma_load}`
instantly (no per-refresh blocking — the key win over status-time `#(...)`
checks). The **mode** badge is pure tmux state, so it is a static format string
`huma mode` prints once into `@huma_mode` (re-evaluated by tmux each refresh; no
process).

## Widgets

- **online**: `TcpStream::connect_timeout` to `@huma-online-host` (default
  `1.1.1.1:443`) within `@huma-online-timeout`. Up → up-icon (+ ` <ms>ms` when
  `@huma-online-latency on`); down/refused/timeout → down-icon.
- **battery**: first `/sys/class/power_supply/BAT*` (`capacity` + `status`):
  `⚡<pct>%` charging, `!<pct>%` ≤ `@huma-battery-low`, else `<pct>%`. No battery
  → empty.
- **load**: `/proc/loadavg` 1-minute; with `@huma-load-mem on`, append used-RAM%
  from `/proc/meminfo`.
- **mode**: `#{?client_prefix,…}#{?pane_in_mode,…}#{?pane_synchronized,…}#{?mouse,…}`
  with icons from `@huma-mode-*`.

## Daemon

Single-instance via a lockfile under `$XDG_RUNTIME_DIR` (stale pid reclaimed).
Loops: update → sleep `@huma-interval` → exit when the tmux server is gone. A
`HUMA_DAEMON_INTERVAL_MS` env override keeps it testable.

## Theming

huma emits value + icon only — never colours. Style the widgets in your config
(`#[fg=…,bg=…]` / theme blocks), so it fits any palette.

## Distribution

Source of truth: `github.com/kenanpelit/tmux-huma`. CI publishes static
x86_64/aarch64 binaries on `v*` tags. TPM installs it; `huma.tmux` resolves the
binary inside the plugin dir, never touching `PATH` — exactly like anka.

## Testing

- Unit: online display formatting; battery formatting (charging/low/normal);
  `/proc/loadavg` + `/proc/meminfo` parsing; load formatting; mode string build.
- Integration (throwaway tmux server): `huma once` writes the three `@huma_*`
  options; `huma daemon` updates them on its interval and exits when the server
  is killed.

## Roadmap

- **v0.1.0** ✅ — online, mode, battery, load widgets; daemon; CI release binaries.
- Later — per-widget icon sets / nerd-font presets; more probes (VPN, multiple
  hosts) if wanted.
