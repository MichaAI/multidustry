use clap::{Parser, Subcommand};
use clap_complete::Shell;

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
    Completions {
        #[arg(value_enum)]
        shell: Shell,
    },
    Kv {
        #[command(subcommand)]
        command: KvSubcommand,
    },
}

#[derive(Subcommand)]
pub enum GetSubcommand {
    Worlds,
}

#[derive(Subcommand)]
pub enum KvSubcommand {
    Set { key: String, value: String },
}
