use std::path::PathBuf;

use clap::{Parser, ValueEnum};
use eyre::Context;
use eyre::Result;
use rippkgs_db::package;
use sea_orm::Database;
use sea_orm::EntityTrait;

#[derive(Debug, Parser)]
struct Opts {
    /// The location of the nixpkgs index to use
    #[arg(short, long)]
    index: PathBuf,

    #[arg(short, long)]
    sort: Sort,
}

#[derive(Clone, Debug, ValueEnum)]
enum Sort {
    Relevant,
}

#[tokio::main]
async fn main() -> Result<()> {
    let opts = Opts::parse();

    let db = &mut Database::connect(format!("sqlite:{}", opts.index.display()))
        .await
        .context("unable to open nixpkgs database")?;


    Ok(())
}
