mod cli;
mod files;
mod git;
mod init;
mod lifecycle;
mod profile;
mod pull;
mod push;
mod sanitize;
mod state;
mod sync;

use anyhow::Result;
use clap::Parser;
use cli::{Cli, Commands};

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Init { repo_url } => init::run(&repo_url),
        Commands::Push => push::run(),
        Commands::Pull => pull::run(),
    }
}
