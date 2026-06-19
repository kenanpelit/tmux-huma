use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "huma", version, about = "tmux status-bar awareness widgets.")]
pub struct Cli {
    #[command(subcommand)]
    pub cmd: Cmd,
}

#[derive(Subcommand)]
pub enum Cmd {
    /// Run the update daemon (writes @huma_online/_battery/_load periodically)
    Daemon,
    /// Run one update cycle and exit
    Once,
    /// Print the @huma_mode format string
    Mode,
    /// Print the online widget
    Online,
    /// Print the battery widget
    Battery,
    /// Print the load widget
    Load,
    /// Refresh @huma_ssh for the active pane (pane-focus hook / daemon)
    Ssh,
    /// Print the crypto-price widget (TTL-cached CoinGecko fetch)
    Kripto,
    /// Print the now-playing widget (playerctl)
    Player,
    /// Print the Nerd Font icon for a command (window-name helper)
    Icon {
        /// Command name, e.g. #{pane_current_command}
        command: String,
    },
    /// Apply a modern tmux baseline (only options still at their default)
    Sensible,
    /// Suspend tmux: disable the prefix so keys pass through (port of tmux-suspend)
    Suspend,
    /// Resume from the suspended pass-through state
    Resume,
}
