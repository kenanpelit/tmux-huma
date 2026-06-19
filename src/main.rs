mod autoreload;
mod battery;
mod cli;
mod config;
mod daemon;
mod icon;
mod icon_map;
mod kripto;
mod load;
mod mode;
mod online;
mod player;
mod sensible;
mod ssh;
mod suspend;
mod tmux;

use anyhow::Result;
use clap::Parser;

use cli::{Cli, Cmd};
use config::Config;

fn main() {
    if let Err(e) = run() {
        eprintln!("huma: {e:#}");
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let cfg = Config::load();
    match Cli::parse().cmd {
        Cmd::Daemon => daemon::run(&cfg),
        Cmd::Once => daemon::once(&cfg),
        Cmd::Mode => {
            println!("{}", mode::build_mode(&cfg));
            Ok(())
        }
        Cmd::Online => {
            println!("{}", online::widget(&cfg));
            Ok(())
        }
        Cmd::Battery => {
            println!("{}", battery::widget(&cfg));
            Ok(())
        }
        Cmd::Load => {
            println!("{}", load::widget(&cfg));
            Ok(())
        }
        Cmd::Ssh => {
            ssh::update(&cfg);
            Ok(())
        }
        Cmd::Kripto => {
            println!("{}", kripto::value(&cfg));
            Ok(())
        }
        Cmd::Player => {
            println!("{}", player::value(&cfg));
            Ok(())
        }
        Cmd::Icon { command } => {
            println!("{}", icon::icon(&command, &cfg));
            Ok(())
        }
        Cmd::Sensible => {
            sensible::apply();
            Ok(())
        }
        Cmd::Suspend => suspend::suspend(),
        Cmd::Resume => suspend::resume(),
    }
}
