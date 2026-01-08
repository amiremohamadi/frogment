use std::sync::Arc;

use anyhow::{Result, anyhow};
use clap::Parser;

use frogment::Resolver;
use frogment::config::Config;

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<()> {
    let args = Args::parse();

    let path = args.config;
    let config = match std::fs::read_to_string(&path) {
        Ok(c) => Config::new(&c),
        Err(e) => Err(anyhow!("failed to read config '{}': {}", path, e)),
    }?;

    env_logger::Builder::new()
        .filter_level(config.log_level.into())
        .format_timestamp_secs()
        .init();

    let resolver = Arc::new(Resolver::new(config));
    resolver.run().await
}

#[derive(Debug, Parser)]
#[clap(author, version)]
pub struct Args {
    #[clap(short, long)]
    pub config: String,
}
