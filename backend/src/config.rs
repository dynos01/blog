use std::net::SocketAddr;

use anyhow::Result;
use clap::{ArgAction, Parser};
use serde::Deserialize;
use tokio::fs::read_to_string;

#[derive(Parser, Debug)]
#[command(name = env!("CARGO_PKG_NAME"))]
#[command(version = env!("CARGO_PKG_VERSION"))]
pub struct Args {
    /// Path to configuration file
    #[arg(short, long, value_name = "/path/to/config.yaml")]
    config: String,

    /// Verbose level
    #[arg(short, long, action = ArgAction::Count)]
    pub verbose: u8,
}

#[derive(Deserialize)]
pub enum Locale {
    #[serde(rename(deserialize = "en"))]
    En,

    #[serde(rename(deserialize = "zh"))]
    Zh,
}

#[derive(Deserialize)]
pub struct Config {
    pub database_path: String,
    pub bind: SocketAddr,
    pub locale: Locale,
}

pub async fn build_config(args: Args) -> Result<Config> {
    let file = read_to_string(&args.config).await?;
    let config = serde_yaml::from_str(&file)?;

    Ok(config)
}
