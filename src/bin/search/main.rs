mod data;
mod exact;
mod fuzzy;

use std::io::stdout;
use std::path::PathBuf;

use clap::{Parser, ValueEnum};
use eyre::Context;
use eyre::Result;
use rusqlite::OpenFlags;

#[derive(Debug, Parser)]
struct Opts {
    /// The location of the nixpkgs index to use
    #[arg(short, long)]
    index: PathBuf,

    query: String,

    /// The number of results to return
    #[arg(default_value = "30")]
    num_results: u32,

    #[arg(short, long, default_value = "relevant")]
    sort: Sort,

    /// Whether to return information about an exact attribute.
    #[arg(long)]
    exact: bool,
}

#[derive(Clone, Debug, ValueEnum)]
enum Sort {
    Relevant,
}

fn main() -> Result<()> {
    let opts = Opts::parse();

    let conn = rusqlite::Connection::open_with_flags(
        opts.index,
        OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX,
    )
    .context("unable to read index")?;

    let results = if opts.exact {
        let result =
            exact::search(opts.query.as_str(), &conn).context("error searching for exact query")?;
        serde_json::to_value(result).context("error serializing exact result")?
    } else {
        let results = fuzzy::search(opts.query.as_str(), &conn, opts.num_results)
            .context("error searching for fuzzy query")?;
        serde_json::to_value(results).context("error serializing fuzzy results")?
    };

    serde_json::to_writer(stdout(), &results).context("error printing results")?;

    Ok(())
}
