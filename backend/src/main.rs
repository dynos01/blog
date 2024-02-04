#![feature(try_blocks)]

mod article_manager;
mod config;
mod db;
mod logging;
mod server;
mod site;
mod text;
mod util;

use std::process::exit;

use clap::Parser;
use log::{error, info};

use crate::{config::Args, logging::setup_logger, server::serve, site::Site};

#[tokio::main]
async fn main() {
    let args = Args::parse();

    if let Err(e) = setup_logger(args.verbose) {
        eprintln!("Failed to setup logger: {e}");
        exit(1);
    }

    let config = match config::build_config(args).await {
        Ok(config) => config,
        Err(e) => {
            error!("Failed to build config: {e}");
            return;
        }
    };

    let site = match Site::new(&config).await {
        Ok(site) => site,
        Err(e) => {
            error!("Failed to load site: {e}");
            return;
        }
    };

    info!(
        "Started {} version {}",
        env!("CARGO_PKG_NAME"),
        env!("CARGO_PKG_VERSION")
    );

    if let Err(e) = serve(config.bind, site).await {
        error!("Failed to start server: {e}");
        return;
    }
}
