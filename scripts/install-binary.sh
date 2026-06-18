#!/usr/bin/env bash
# Install the huma binary into the plugin directory: try a prebuilt release
# asset first, fall back to compiling with cargo. Never writes to PATH.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

REPO="kenanpelit/tmux-huma"
VERSION="$(grep -m1 '^version' Cargo.toml | sed -E 's/.*"(.*)".*/\1/')"

try_download() {
    local arch os asset url
    arch="$(uname -m)"
    os="$(uname -s | tr '[:upper:]' '[:lower:]')"
    asset="huma-${VERSION}-${arch}-${os}.tar.gz"
    url="https://github.com/${REPO}/releases/download/v${VERSION}/${asset}"
    command -v curl >/dev/null 2>&1 || return 1
    mkdir -p bin
    curl -fsSL "$url" -o /tmp/huma-dl.tgz 2>/dev/null || return 1
    tar -xzf /tmp/huma-dl.tgz -C bin 2>/dev/null || return 1
    [ -x bin/huma ]
}

try_compile() {
    command -v cargo >/dev/null 2>&1 || return 1
    cargo build --release
    mkdir -p bin
    cp target/release/huma bin/huma
}

if try_download; then
    echo "huma: installed prebuilt binary v${VERSION}"
elif try_compile; then
    echo "huma: compiled binary v${VERSION}"
else
    echo "huma: could not install (no release asset and no cargo)" >&2
    exit 1
fi
