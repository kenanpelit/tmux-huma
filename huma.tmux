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

# Mode badge: a static format string (no daemon).
tmux set -g @huma_mode "$("$BINARY" mode)"

# Immediate first values, then the background update daemon.
"$BINARY" once >/dev/null 2>&1
"$BINARY" daemon >/dev/null 2>&1 &
