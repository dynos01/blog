use std::env::remove_var;

use anyhow::Result;
use log::info;
use tracing_subscriber::EnvFilter;

pub fn setup_logger(loglevel: u8) -> Result<()> {
    let level = match loglevel {
        0 => "warn",
        1 => "info",
        2 => "debug",
        _ => "trace",
    };

    // We control logging exclusively here
    remove_var("RUST_LOG");

    let filter_str = format!("{}={level}", env!("CARGO_PKG_NAME"));

    let filter = EnvFilter::builder()
        .with_default_directive(tracing::Level::ERROR.into())
        .from_env()?
        .add_directive(filter_str.parse()?);

    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .compact()
        .init();

    info!("Log level set to {level}");

    Ok(())
}
