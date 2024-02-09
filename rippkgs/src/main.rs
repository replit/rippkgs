use std::path::PathBuf;

use clap::{Parser, ValueEnum};
use eyre::Context;
use eyre::Result;
use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
use rusqlite::functions::Context as FunctionContext;
use rusqlite::functions::FunctionFlags;
use sqlx::Connection;
use sqlx::Executor;
use sqlx::Row;
use sqlx::SqliteConnection;

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

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    let opts = Opts::parse();

    let conn = &mut SqliteConnection::connect(format!("sqlite:{}", opts.index.display()).as_str())
        .await
        .context("unable to read index")?;

    install_functions(&mut *conn)
        .await
        .context("unable to install functions")?;

    let query = sqlx::query(include_str!("../../queries/search.sql"))
        .bind("rust")
        .bind(5);

    let results = conn
        .fetch_all(query)
        .await
        .context("unable to query index")?;

    for (ii, row) in results.into_iter().enumerate() {
        println!("{ii}: {} @ {}", row.get::<i64, _>("score"), row.get::<String, _>("attribute"));
    }

    Ok(())
}

async fn install_functions(conn: &mut SqliteConnection) -> eyre::Result<()> {
    let mut handle_lock = conn.lock_handle().await?;
    let handle = handle_lock.as_raw_handle().as_ptr();

    let install_conn = unsafe {
        // SAFETY: the original handle is locked for the duration of this function and this handle
        // doesn't outlast the handle lock. The functions are cleaned up in standard sqlite shutdown
        rusqlite::Connection::from_handle(handle)
            .context("unable to create sqlite connection from sqlx connection handle")?
    };

    install_conn
        .create_scalar_function(
            "fuzzy_score",
            2,
            FunctionFlags::SQLITE_UTF8,
            scalar_fuzzy_score,
        )
        .context("unable to install `fuzzy_score` scalar function")?;

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
