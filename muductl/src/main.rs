use clap::{Parser, Subcommand};
use tokio::{fs::File, io::AsyncReadExt};

use crate::{
    commands::{Cli, GetSubcommand, Subcommands},
    config_model::Config,
};

pub mod commands;
pub mod config_model;
pub mod get;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let command = Cli::parse();
    let mut path = home::home_dir().expect("Cant find home dir");
    path.push(".muductl");
    if !path.exists() {
        tokio::fs::create_dir(&path)
            .await
            .expect("Cant create configuration directory");
    }
    path.push("config.toml");

    if !path.exists() {
        File::create(&path)
            .await
            .expect(&format!("Cant create {:?}", &path));
    }
    let mut file = File::open(&path).await.expect("Cant read config file");
    let mut config_string = String::new();
    file.read_to_string(&mut config_string)
        .await
        .expect("Failed to read config file");

    let config: Config = toml::from_str(&config_string)
        .map_err(|e| println!("{:?}", e))
        .expect("Failed to parce config ^^^^^^^");

    match &command.command {
        Subcommands::Get { command } => match &command {
            GetSubcommand::Worlds => {
                get::worlds::get_worlds(&config).await;
            }
        },
    }

    Ok(())
}
