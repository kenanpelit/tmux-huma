# tmux-huma 🪶

> Status-bar awareness widgets for tmux — online, mode, battery, load, ssh,
> crypto, now-playing — plus window-name icons and sensible defaults.

**Huma** (the mythical bird that watches from above) is a single modern plugin
that provides the status-bar widgets people reach for — connectivity, the
current mode, battery, and system load — as one zero-runtime-dependency Rust
binary, mirroring [`tmux-anka`](https://github.com/kenanpelit/tmux-anka).

It folds in `tmux-online-status`, `tmux-prefix-highlight`, `tmux-ssh-status`,
`tmux-kripto`, `tmux-plugin-playerctl`, `tmux-nerd-font-window-name`,
`tmux-sensible`, `tmux-suspend` and `tmux-autoreload`, with a **non-blocking**
design: a tiny background daemon updates
the values, so your status bar never waits on a connectivity check. Linux-only.

## Features

- **Online** — connectivity up/down via a fast TCP probe (not `ping`, no root),
  with optional latency. `#{@huma_online}`
- **Mode badge** — prefix / copy-mode / sync / mouse in one widget.
  `#{@huma_mode}`
- **Battery** — percentage + charging/low marker (`/sys`). `#{@huma_battery}`
- **Load** — `/proc/loadavg` 1-minute, optional used-RAM%. `#{@huma_load}`
- **SSH** — the active pane's ssh host (or `user@host`), detected via `/proc`,
  refreshed instantly on pane focus. Empty when not in ssh. `#{@huma_ssh}`
- **Kripto** — live crypto prices (CoinGecko), each coin shown with a currency
  glyph (₿, Ξ, …) or its ticker. TTL-cached so the API is hit at most once per
  `@huma-kripto-ttl`. Off until you set coins. `#{@huma_kripto}`
- **Player** — now-playing from `playerctl`; auto-picks the player that is
  actually playing (handy with a browser + Spotify open), play/pause marker,
  truncation. Empty when nothing is playing. `#{@huma_player}`
- **Window icons** — a Nerd Font glyph per command for `automatic-rename-format`
  (nvim/git/docker/…), native and dependency-free. Replaces
  `tmux-nerd-font-window-name` (no `yq`/YAML). `huma icon <command>`
- **Sensible defaults** — a modern tmux baseline (`tmux-256color`, focus-events,
  bigger history, …) applied **only where you haven't set the option yourself**,
  so huma is a sane plugin even outside one person's config. Replaces
  `tmux-sensible`.
- **Suspend** — `F12` disables the prefix and switches to a `suspended` key
  table so keys pass through to a nested session (ssh + tmux/byobu); a badge
  shows in the mode widget while suspended, and `F12` resumes. Replaces
  `tmux-suspend`. `@huma-suspend-key`
- **Autoreload** — the daemon watches your tmux config files and re-sources them
  on save (mtime poll, no `entr`). Off until `@huma-autoreload on`. Replaces
  `tmux-autoreload`.
- **Non-blocking** — a background daemon writes the values; the status bar just
  reads user options. No per-refresh blocking.
- **Theme-agnostic** — emits value + icon only; you wrap it in your own
  `#[fg=…]` / theme blocks.

## Requirements

- **Linux** (`/proc`, `/sys`); **tmux 3.x+**.
- Core widgets need nothing else: `huma` is a single static binary. Building from
  source needs Rust; installing a release does not.
- Optional: `curl` for the **kripto** widget, `playerctl` for the **player**
  widget. Missing either just leaves that one widget empty.

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
| `#{@huma_ssh}` | `grid` (or `user@host`) when the pane is in ssh, else empty |
| `#{@huma_kripto}` | `₿ $62,500  Ξ $1,680` (one or more coins; empty until configured) |
| `#{@huma_player}` | `▶ Artist - Title` / `⏸ …` (empty when not playing) |

Example:

```tmux
set -g status-right "#{@huma_mode} #{@huma_load} #{@huma_battery} #{@huma_online} %H:%M"
```

**Window-name icons** — compose the name yourself; `huma icon` prints just the
glyph for the current command:

```tmux
set -g automatic-rename on
set -g automatic-rename-format "#(~/.config/tmux/plugins/tmux-huma/bin/huma icon '#{pane_current_command}') #{b:pane_current_path}"
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
| `@huma-ssh-format` | `host` | SSH widget: `host` or `user@host` |
| `@huma-kripto-coins` | _(empty)_ | Comma list of CoinGecko ids, e.g. `bitcoin,ethereum`. Empty = off |
| `@huma-kripto-currency` | `usd` | Quote currency (CoinGecko `vs_currency`) |
| `@huma-kripto-symbol` | `$` | Prefix shown before each price |
| `@huma-kripto-ttl` | `300` | Min seconds between API fetches (cache TTL) |
| `@huma-player-format` | `{{artist}} - {{title}}` | `playerctl metadata --format` template |
| `@huma-player-max` | `40` | Truncate the now-playing text to N chars |
| `@huma-player-playing` / `-paused` | `▶` / `⏸` | Player state icons |
| `@huma-player-name` | _(auto)_ | Force an MPRIS player by name (prefix, e.g. `spotify`); empty = first playing |
| `@huma-icon-shell` | _(auto)_ | Override the glyph for all shells (empty = per-shell built-in) |
| `@huma-icon-editor` | _(auto)_ | Override the glyph for all editors (empty = per-editor built-in) |
| `@huma-icon-default` | `?` | Fallback glyph for unknown commands |
| `@huma-mode-suspend` | `󰒲` | Badge shown in the mode widget while suspended |
| `@huma-suspend-key` | `F12` | Key (root + suspended tables) toggling suspend; `none` disables |
| `@huma-autoreload` | `off` | Watch config files and re-source on change |
| `@huma-autoreload-configs` | _(empty)_ | Extra files to watch (comma list), beyond `#{config_files}` |

`#{@huma_ssh}` needs `focus-events on` for instant updates (the daemon refreshes
it each tick regardless).

## CLI

```
huma daemon     Run the background update daemon
huma once       Run one update cycle
huma mode       Print the @huma_mode format string
huma online     Print the online widget
huma battery    Print the battery widget
huma load       Print the load widget
huma kripto     Print the crypto-price widget (TTL-cached CoinGecko fetch)
huma player     Print the now-playing widget (playerctl)
huma icon CMD   Print the Nerd Font icon for a command (window-name helper)
huma sensible   Apply a modern tmux baseline (only options still at default)
huma suspend    Disable the prefix; pass keys through (port of tmux-suspend)
huma resume     Resume from the suspended pass-through state
```

`huma kripto` needs `curl`; `huma player` needs `playerctl`. Both are optional —
the widget is simply empty if the tool or data is missing.

## Design

See [`docs/DESIGN.md`](docs/DESIGN.md).

## License

MIT © Kenan Pelit
