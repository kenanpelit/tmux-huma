# tmux-huma — native status/awareness widgets — design

Status: approved 2026-06-18. Drives implementation for **v0.1.0**.

## Goal

One small plugin that provides the status-bar "awareness" widgets currently
spread across `tmux-online-status` (and `tmux-prefix-highlight`, which is already
done natively) — **even better**, as a single zero-runtime-dependency Rust
binary, mirroring `tmux-anka`'s shape. Named **huma** (the Hüma bird that watches
from above). Linux-only (`/proc`, `/sys`).

URL/file extraction & opening is **out of scope** — `extrakto` owns it (see
Migration for how `prefix+u` is preserved).

## Decisions (from brainstorming)

- **Widgets:** `online` (core), `mode` badge, `battery`, `load`.
- **Rust binary + daemon**, mirroring anka (non-blocking; the status bar never
  waits on a check) — not pure shell.
- **Theme-agnostic:** huma emits only a value + icon; the user wraps it in their
  own `#[...]` / theme blocks (like `online_status` does). No colours baked in.
- **prefix+u stays** but is served by extrakto's URL engine + the user's browser,
  not by huma.

## Non-goals

- URL/file opening (extrakto).
- Colour/theming (the user styles the widgets).
- macOS/BSD.

## Architecture

Mirror anka. The data that changes over time (online, battery, load) is produced
by a background **daemon** that writes tmux user options; the status bar reads
them instantly. The `mode` badge is pure tmux state, so it is a **static format
string** huma sets once — no process.

```
huma.tmux            TPM entrypoint (bash): resolve/install binary, set
                     @huma_mode, run `huma once`, start `huma daemon`.
scripts/install-binary.sh   download-release-or-cargo-build (anka's twin)
src/
  main.rs / cli.rs   clap dispatch: daemon | once
  tmux.rs            tmux CLI wrapper + set/show user options
  config.rs          read @huma-* via `tmux show-options -gqv`
  online.rs          TCP-connect reachability + latency
  battery.rs         /sys/class/power_supply parse
  load.rs            /proc/loadavg (+ optional /proc/meminfo)
  daemon.rs          single-instance loop; write @huma_online/_battery/_load
  render.rs          format a widget value + icon (theme-agnostic)
.github/workflows/release.yml   tag v* → static x86_64/aarch64 binaries
```

### Binary subcommands

- `huma daemon` — single-instance (lockfile under `@huma-dir` or
  `${XDG_RUNTIME_DIR}`); every `@huma-interval` seconds: compute online,
  battery, load → `tmux set -g @huma_online/@huma_battery/@huma_load <text>`;
  exit when the tmux server is gone. A `HUMA_DAEMON_INTERVAL_MS` env override
  keeps it testable without second-long waits (anka's pattern).
- `huma once` — run exactly one update cycle (immediate first paint + tests).

### Widgets

- **online** (`@huma_online`): TCP-connect to `@huma-online-host` (default
  `1.1.1.1:53`) with `@huma-online-timeout` (default `1s`). Up → `@huma-online-icon-up`
  (default `✓`) plus, when `@huma-online-latency on`, ` 23ms`. Down →
  `@huma-online-icon-down` (default `✗`).
- **battery** (`@huma_battery`): read the first `/sys/class/power_supply/BAT*`
  (`capacity` + `status`). Output `<icon> <pct>%`; icon by state
  (charging/full/discharging) and a low marker under `@huma-battery-low`
  (default `20`). No battery present → empty string (desktops show nothing).
- **load** (`@huma_load`): `/proc/loadavg` 1-minute value, e.g. `0.42`. With
  `@huma-load-mem on`, append used-memory percent from `/proc/meminfo`
  (` · 38%`). Prefixed with `@huma-load-icon` (default `▟`).
- **mode** (`@huma_mode`): a **static format string** huma.tmux sets, combining
  tmux state into one badge:
  `#{?client_prefix,<prefix>,}#{?pane_in_mode,<copy>,}#{?pane_synchronized,<sync>,}#{?mouse,<mouse>,}`
  with icons from `@huma-mode-prefix/copy/sync/mouse`. Re-evaluated by tmux each
  refresh; no daemon involvement.

## Configuration (additions; all optional, defaulted)

| Option | Default | Meaning |
|--------|---------|---------|
| `@huma-interval` | `5` | Daemon update period (seconds) |
| `@huma-online-host` | `1.1.1.1:53` | host:port probed by TCP connect |
| `@huma-online-timeout` | `1` | Connect timeout (seconds) |
| `@huma-online-latency` | `on` | Append `<ms>ms` when up |
| `@huma-online-icon-up` / `-down` | `✓` / `✗` | Online icons |
| `@huma-battery-low` | `20` | Low-battery threshold (%) |
| `@huma-load-mem` | `off` | Append used-RAM % to the load widget |
| `@huma-load-icon` | `▟` | Load prefix icon |
| `@huma-mode-prefix/copy/sync/mouse` | `⌨` / `❐` / `⚏` / `↗` | Mode-badge icons |

(`@huma-dir` optional, for the lockfile; defaults to `${XDG_RUNTIME_DIR}` or `/tmp`.)

## Distribution

Standalone repo `github.com/kenanpelit/tmux-huma`; `set -g @plugin
'kenanpelit/tmux-huma'`. `huma.tmux` resolves the binary inside the plugin dir
(never `PATH`), downloading the matching release asset or compiling with cargo —
exactly like anka. Tagging `v*` triggers CI that publishes static
x86_64/aarch64 Linux binaries.

## Migration (in `~/.cachy`)

Remove `tmux-online-status`, `tmux-prefix-highlight`, `tmux-fzf-url`, `tmux-open`
(gitlinks + `@plugin` lines + their `@*` settings). Add
`@plugin 'kenanpelit/tmux-huma'` + the `@huma-*` config.

- **status-left:** `#{online_status}` → `#{@huma_online}`.
- **status-right:** the scattered native conditionals
  (`#{?client_prefix,…}#{?mouse,…}` + `#{?pane_synchronized,…}`) → `#{@huma_mode}`;
  add `#{@huma_battery}` and `#{@huma_load}` where wanted (styled with the
  existing `@theme_b*` blocks).
- **extrakto (kept):** set `@extrakto_open_tool 'helium-kenp-default'` and
  `@extrakto_filter_order 'all url path word line'` so `Ctrl-o` opens URLs in the
  browser and the url/path filters are reachable.
- **prefix+u (preserved, dedicated URL→browser):** a popup binding reusing
  extrakto's URL engine —
  `display-popup -E "tmux capture-pane -p -J -t '#{pane_id}' -S -3000 | python3 ~/.config/tmux/plugins/extrakto/extrakto.py -u | sort -u | fzf --multi --prompt 'url> ' | xargs -r -n1 helium-kenp-default"`.
  Replaces tmux-fzf-url with no plugin.

## Error handling

- No tmux server → daemon exits (and `once` is a no-op with a message).
- Online check fails/refused/times out → the "down" icon (not an error).
- No `/sys/.../BAT*` → `@huma_battery` is empty.
- Unreadable `/proc` → that widget is empty; other widgets continue.
- Single-instance daemon (stale lockfile from a dead pid is reclaimed).

## Testing

- **Unit (pure):** online result → display string (up+latency / down); battery
  parse from sample `capacity`/`status` text (path injected) incl. low/charging;
  `/proc/loadavg` parse; `/proc/meminfo` used-% calc; mode format-string builder.
- **Integration (Server harness, anka's twin):** `huma once` writes the three
  `@huma_*` options on a throwaway server; `huma daemon` updates them on its
  interval (via `HUMA_DAEMON_INTERVAL_MS`) and exits when the server is killed.

## Rollout

Ships as **v0.1.0** (tagged → CI binaries). README + DESIGN written. `.cachy`
migrated in the same pass; extrakto reconfigured and `prefix+u` preserved.
