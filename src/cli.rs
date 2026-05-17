use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "zensync",
    version,
    about = "Sync Zen Browser configurations across machines via Git"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Clone a Git repo of Zen configs into the local state directory
    Init {
        /// Git URL (HTTPS or SSH) of the repo holding your Zen configs
        repo_url: String,
    },
    /// Copy local profile files into the repo, commit, and push
    Push,
    /// Pull from the repo and apply to local profile (Zen must be closed)
    Pull,
}
