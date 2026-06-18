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
}
