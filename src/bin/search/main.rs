mod data;
mod exact;
mod fuzzy;

use std::fmt::Display;
use std::io::stdout;
use std::path::PathBuf;

use clap::builder::{PathBufValueParser, TypedValueParser};
use clap::Parser;
use comfy_table::TableComponent;
use eyre::Context;
use eyre::Result;
use rippkgs::Package;
use rusqlite::OpenFlags;
use xdg::BaseDirectories;

/// Custom type because clap needs to use Display to print the default value.
#[derive(Clone, Debug)]
struct IndexPath(PathBuf);

impl AsRef<PathBuf> for IndexPath {
    fn as_ref(&self) -> &PathBuf {
        &self.0
    }
}

impl Display for IndexPath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.display().fmt(f)
    }
}

#[derive(Clone, Debug, Default)]
struct IndexPathValueParser(PathBufValueParser);

impl TypedValueParser for IndexPathValueParser {
    type Value = IndexPath;

    fn parse_ref(
        &self,
        cmd: &clap::Command,
        arg: Option<&clap::Arg>,
        value: &std::ffi::OsStr,
    ) -> std::prelude::v1::Result<Self::Value, clap::Error> {
        self.0.parse_ref(cmd, arg, value).map(IndexPath)
    }
}

#[derive(Debug, Parser)]
struct Opts {
    /// The location of the rippkgs index to use.
    #[arg(short, long, default_value_t = get_default_index_path(), value_parser = IndexPathValueParser::default())]
    index: IndexPath,

    /// The number of results to return.
    #[arg(short, long, default_value = "30")]
    num_results: u32,

    /// Whether to return information about an exact attribute.
    #[arg(long)]
    exact: bool,

    /// Filter results by whether the /nix/store path already exists. Only applies when doing fuzzy
    /// matching.
    #[arg(long)]
    filter_built: bool,

    /// Print the results as json
    #[arg(long)]
    json: bool,

    /// The search query.
    query: String,
}

fn get_default_index_path() -> IndexPath {
    let dirs = BaseDirectories::new()
        .context("rippkgs isn't supported on Windows.")
        .unwrap();

    IndexPath(dirs.get_data_home().join("rippkgs-index.sqlite"))
}

fn main() -> Result<()> {
    let opts = Opts::parse();

    let conn = rusqlite::Connection::open_with_flags(
        opts.index.0,
        OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX,
    )
    .context("reading index")?;

    let results: Box<dyn Iterator<Item = Package>> = if opts.exact {
        let result =
            exact::search(opts.query.as_str(), &conn).context("searching for exact query")?;

        Box::new(result.into_iter())
    } else {
        let results = fuzzy::search(
            opts.query.as_str(),
            &conn,
            opts.num_results,
            opts.filter_built,
        )
        .context("searching for fuzzy query")?;

        Box::new(results.into_iter())
    };

    if opts.json {
        serde_json::to_writer(stdout(), &results.collect::<Vec<_>>()).context("printing results")?
    } else {
        let mut table = comfy_table::Table::new();

        table
            .set_header(vec!["attribute", "version", "description"])
            .remove_style(TableComponent::HorizontalLines)
            .remove_style(TableComponent::MiddleIntersections)
            .remove_style(TableComponent::LeftBorderIntersections)
            .remove_style(TableComponent::RightBorderIntersections);
        results.for_each(
            |Package {
                 attribute,
                 version,
                 description,
                 ..
             }| {
                table.add_row(vec![
                    attribute,
                    version.unwrap_or_default(),
                    description.unwrap_or_default(),
                ]);
            },
        );

        println!("{table}")
    }

    Ok(())
}
