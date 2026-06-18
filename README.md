# tmux-huma 🪶

> Status-bar awareness widgets for tmux — online, mode, battery, load.

**Huma** (the mythical bird that watches from above) is a single modern plugin
that provides the status-bar widgets people reach for — connectivity, the
current mode, battery, and system load — as one zero-runtime-dependency Rust
binary, mirroring [`tmux-anka`](https://github.com/kenanpelit/tmux-anka).

It replaces `tmux-online-status` and `tmux-prefix-highlight` and adds battery +
load, with a **non-blocking** design: a tiny background daemon updates the
values, so your status bar never waits on a connectivity check. Linux-only.

## Features

- **Online** — connectivity up/down via a fast TCP probe (not `ping`, no root),
  with optional latency. `#{@huma_online}`
- **Mode badge** — prefix / copy-mode / sync / mouse in one widget.
  `#{@huma_mode}`
- **Battery** — percentage + charging/low marker (`/sys`). `#{@huma_battery}`
- **Load** — `/proc/loadavg` 1-minute, optional used-RAM%. `#{@huma_load}`
- **Non-blocking** — a background daemon writes the values; the status bar just
  reads user options. No per-refresh blocking.
- **Theme-agnostic** — emits value + icon only; you wrap it in your own
  `#[fg=…]` / theme blocks.

## Requirements

- **Linux** (`/proc`, `/sys`); **tmux 3.x+**.
- Nothing else at runtime: `huma` is a single static binary. Building from source
  needs Rust; installing a release does not.

## Install (TPM)

```tmux
set -g @plugin 'kenanpelit/tmux-huma'
```

Hit `prefix + I`. On first load the plugin resolves the binary **inside the
plugin directory** (never `PATH`): it downloads the prebuilt release asset for
your architecture (`x86_64` / `aarch64`), or compiles with `cargo` if needed.

## Widgets

Place these in your `status-left` / `status-right` and style them yourself:

| Widget | Shows |
|--------|-------|
| `#{@huma_online}` | `✓ 23ms` when up, `✗` when down |
| `#{@huma_mode}` | `⌨`/`❐`/`⚏`/`↗` for prefix/copy/sync/mouse |
| `#{@huma_battery}` | `⚡50%` / `!12%` / `80%` (empty on desktops) |
| `#{@huma_load}` | `▟ 0.42` (+ ` · 38%` with `@huma-load-mem on`) |

Example:

```tmux
set -g status-right "#{@huma_mode} #{@huma_load} #{@huma_battery} #{@huma_online} %H:%M"
```

## Configuration

| Option | Default | Meaning |
|--------|---------|---------|
| `@huma-interval` | `5` | Daemon update period (seconds) |
| `@huma-online-host` | `1.1.1.1:443` | `host:port` probed by TCP connect |
| `@huma-online-timeout` | `1` | Connect timeout (seconds) |
| `@huma-online-latency` | `on` | Append `<ms>ms` when up |
| `@huma-online-icon-up` / `-down` | `✓` / `✗` | Online icons |
| `@huma-battery-low` | `20` | Low-battery threshold (%) |
| `@huma-load-mem` | `off` | Append used-RAM% to the load widget |
| `@huma-load-icon` | `▟` | Load prefix icon |
| `@huma-mode-prefix/copy/sync/mouse` | `⌨` / `❐` / `⚏` / `↗` | Mode-badge icons |

## CLI

```
huma daemon     Run the background update daemon
huma once       Run one update cycle
huma mode       Print the @huma_mode format string
huma online     Print the online widget
huma battery    Print the battery widget
huma load       Print the load widget
```

## Design

See [`docs/DESIGN.md`](docs/DESIGN.md).

## License

MIT © Kenan Pelit
