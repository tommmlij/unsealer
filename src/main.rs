mod cli;
mod api;
mod runtime;

use clap::Parser;
use cli::Cli;
use crate::api::pre_run;
use crate::runtime::run;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    let config = pre_run(cli.clone()).await?;

    run(config, cli.executable_path).await?;

    Ok(())
}
