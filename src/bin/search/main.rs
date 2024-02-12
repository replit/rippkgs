use std::path::PathBuf;
use std::time::Instant;

use clap::{Parser, ValueEnum};
use eyre::Context;
use eyre::Result;
use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
use rusqlite::functions::Context as FunctionContext;
use rusqlite::functions::FunctionFlags;
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

    #[arg(short, long)]
    sort: Sort,
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

    conn.create_scalar_function(
        "fuzzy_score",
        2,
        FunctionFlags::SQLITE_UTF8,
        scalar_fuzzy_score,
    )
    .context("unable to install `fuzzy_score` function")?;

    let mut query = conn
        .prepare(
            r#"
SELECT *, fuzzy_score(name, ?1) as score
FROM packages
ORDER BY score DESC
LIMIT ?2
            "#,
        )
        .context("unable to prepare search query")?;

    let start = Instant::now();

    let mut results = query
        .query(rusqlite::params![opts.query, opts.num_results])
        .context("unable to execute query")?;

    loop {
        let row = results.next().context("error collecting query results")?;

        let Some(row) = row else {
            break;
        };

        let attribute: String = row.get("attribute").context("error reading column")?;
        // let store_path: String = row.get("store_path").context("error reading column")?;
        // let name: String = row.get("name").context("error reading column")?;
        // let version: String = row.get("version").context("error reading column")?;
        // let description: String = row.get("description").context("error reading column")?;
        // let homepage: String = row.get("homepage").context("error reading column")?;
        // let long_description: String = row
        //     .get("long_description")
        //     .context("error reading column")?;
        let score: i32 = row.get("score").context("error reading column")?;

        println!("({score}) {attribute}");
    }

    let elapsed = start.elapsed();
    println!("finished in {} ms", elapsed.as_millis());

    Ok(())
}

fn scalar_fuzzy_score(ctx: &FunctionContext) -> rusqlite::Result<i64> {
    lazy_static::lazy_static! {
      static ref MATCHER: SkimMatcherV2 = SkimMatcherV2::default();
    }

    let choice = ctx.get::<String>(0)?;
    let pattern = ctx.get::<String>(1)?;

    Ok(MATCHER.fuzzy_match(&choice, &pattern).unwrap_or(0))
}
