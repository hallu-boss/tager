mod cli;
mod db;
mod tager_manager;

use cli::Cli;
use clap::Parser;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    cli.run().await?;
    Ok(())
}