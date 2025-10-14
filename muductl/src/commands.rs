use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(version, about, long_about = None)]
pub struct Cli {
    /// Server from ~/.muductl/config.toml to use
    #[arg(long)]
    pub context: Option<String>,

    /// Is server should generate fake data
    #[arg(long)]
    pub fake: bool,

    #[command(subcommand)]
    pub command: Subcommands,
}

#[derive(Subcommand)]
pub enum Subcommands {
    Get {
        #[command(subcommand)]
        command: GetSubcommand,
    },
}

#[derive(Subcommand)]
pub enum GetSubcommand {
    Worlds,
}
