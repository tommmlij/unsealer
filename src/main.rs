mod cli;
mod api;
mod runtime;
mod utils;

use clap::Parser;
use cli::Cli;
use crate::api::pre_run;
use crate::runtime::run;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    let config = pre_run(cli.clone()).await?;

    run(config, cli.command).await?;

    Ok(())
}
