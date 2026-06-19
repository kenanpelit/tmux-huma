#!/usr/bin/env bash
# tmux-huma — TPM entrypoint.
# Resolves the binary inside the plugin dir (never touches PATH), installs it if
# missing/stale, sets the @huma_mode badge format, paints initial values, then
# starts the update daemon. Provides the widgets #{@huma_online},
# #{@huma_battery}, #{@huma_load}, #{@huma_mode} for use in your status bar.

set -u

CURRENT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

resolve_binary() {
    if [ -x "$CURRENT_DIR/bin/huma" ]; then
        echo "$CURRENT_DIR/bin/huma"
    elif [ -x "$CURRENT_DIR/target/release/huma" ]; then
        echo "$CURRENT_DIR/target/release/huma"
    fi
}

VERSION="$(grep -m1 '^version' "$CURRENT_DIR/Cargo.toml" | sed -E 's/.*"(.*)".*/\1/')"
BINARY="$(resolve_binary)"

needs_install() {
    [ -z "$BINARY" ] && return 0
    local have
    have="$("$BINARY" --version 2>/dev/null | awk '{print $2}')"
    [ "$have" != "$VERSION" ]
}

if needs_install; then
    tmux display-message "tmux-huma: installing binary…" 2>/dev/null || true
    "$CURRENT_DIR/scripts/install-binary.sh" >/dev/null 2>&1
    BINARY="$(resolve_binary)"
fi

if [ -z "$BINARY" ]; then
    tmux display-message "tmux-huma: binary could not be installed (need cargo or a release asset)" 2>/dev/null || true
    exit 0
fi

# Sane tmux baseline (replaces tmux-sensible). Only-if-default, so anything you
# set in your own config — sourced before this — always wins.
"$BINARY" sensible >/dev/null 2>&1

# Mode badge: a static format string (no daemon).
tmux set -g @huma_mode "$("$BINARY" mode)"

# Suspend toggle (replaces tmux-suspend): @huma-suspend-key (default F12) disables
# the prefix and switches to the `suspended` key-table so keys pass through to a
# nested session; the same key resumes. Set the option to 'none' to skip binding.
SUSPEND_KEY="$(tmux show-option -gqv @huma-suspend-key)"
SUSPEND_KEY="${SUSPEND_KEY:-F12}"
if [ "$SUSPEND_KEY" != "none" ] && [ "$SUSPEND_KEY" != "off" ]; then
    tmux bind -T root "$SUSPEND_KEY" run-shell "$BINARY suspend"
    tmux bind -T suspended "$SUSPEND_KEY" run-shell "$BINARY resume"
fi

# SSH widget (#{@huma_ssh}): refresh the instant a pane gains focus (needs
# focus-events on), so switching to/from an ssh pane updates immediately. The
# daemon also refreshes it each tick as a fallback.
tmux set-hook -g pane-focus-in "run-shell -b \"$BINARY ssh\""

# Immediate first values, then the background update daemon.
"$BINARY" once >/dev/null 2>&1
"$BINARY" daemon >/dev/null 2>&1 &
