//! A modern tmux baseline (replaces tmux-sensible), so huma is a sane plugin
//! on its own — not just inside one person's config.
//!
//! **Only-if-default**: each option is written only when it still holds tmux's
//! compiled-in default, so anything you set yourself always wins (huma.tmux runs
//! after your config is sourced). Keybindings are left alone — that's yours.

use crate::tmux;

enum Scope {
    Server,
    Global,
    Window,
}

struct Tweak {
    name: &'static str,
    scope: Scope,
    default: &'static str,
    desired: &'static str,
}

/// `(option, scope, tmux default, huma's value)`. tmux 3.x defaults.
const TWEAKS: &[Tweak] = &[
    Tweak { name: "default-terminal", scope: Scope::Global, default: "screen", desired: "tmux-256color" },
    Tweak { name: "escape-time", scope: Scope::Server, default: "500", desired: "10" },
    Tweak { name: "history-limit", scope: Scope::Global, default: "2000", desired: "50000" },
    Tweak { name: "display-time", scope: Scope::Global, default: "750", desired: "4000" },
    Tweak { name: "status-interval", scope: Scope::Global, default: "15", desired: "5" },
    Tweak { name: "focus-events", scope: Scope::Global, default: "off", desired: "on" },
    Tweak { name: "set-clipboard", scope: Scope::Global, default: "external", desired: "on" },
    Tweak { name: "aggressive-resize", scope: Scope::Window, default: "off", desired: "on" },
];

fn current(t: &Tweak) -> String {
    match t.scope {
        Scope::Server => tmux::server_option(t.name),
        Scope::Global => tmux::global_option(t.name),
        Scope::Window => tmux::window_option(t.name),
    }
}

fn set(t: &Tweak) {
    match t.scope {
        Scope::Server => tmux::set_server_option(t.name, t.desired),
        Scope::Global => tmux::set_global_option(t.name, t.desired),
        Scope::Window => tmux::set_window_option(t.name, t.desired),
    }
}

pub fn apply() {
    for t in TWEAKS {
        if current(t) == t.default {
            set(t);
        }
    }
}
