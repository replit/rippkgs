mod data;

use std::{
    collections::{HashMap, VecDeque},
    fs::File,
    io::{self, Write},
    path::{Path, PathBuf},
    process::Command,
    time::Instant,
};

use clap::Parser;
use data::PackageInfo;
use eyre::{Context, Result};
use rippkgs::Package;
use rusqlite::OpenFlags;

#[derive(Debug, Parser)]
struct Opts {
    /// The location to write the saved index to.
    #[arg(short, long)]
    output: PathBuf,

    /// The flake URI of the nixpkgs to index.
    ///
    /// If this is provided, then the registry will optionally be cached at `--registry`.
    ///
    /// If this is empty, `--registry` must be provided.
    #[arg(short, long)]
    nixpkgs: Option<String>,

    /// The file for the cached registry.
    ///
    /// If `--nixpkgs` is provided, then this will cache the registry at the given path.
    ///
    /// If `--nixpkgs` is empty, then this file will be used in lieu of evaluating nixpkgs.
    #[arg(short, long)]
    registry: Option<PathBuf>,

    /// The value to pass as the config parameter to nixpkgs.
    ///
    /// Only used if `--nixpkgs` is provided.
    #[arg(short = 'c', long)]
    nixpkgs_config: Option<Box<Path>>,
}

fn main() -> Result<()> {
    let opts = Opts::parse();

    let registry = get_registry(&opts).context("getting nixpkgs registry")?;

    match std::fs::remove_file(opts.output.as_path()) {
        Ok(()) => (),
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => (),
        Err(err) => Err(err).context("removing previous index db")?,
    }

    let mut conn = rusqlite::Connection::open_with_flags(
        opts.output,
        OpenFlags::SQLITE_OPEN_CREATE
            | OpenFlags::SQLITE_OPEN_READ_WRITE
            | OpenFlags::SQLITE_OPEN_NO_MUTEX,
    )
    .context("connecting to index database")?;

    conn.execute(Package::create_table(), [])
        .context("creating table in database")?;

    let start = Instant::now();
    let tx = conn.transaction().context("starting transaction")?;

    {
        let mut create_row_query = tx
            .prepare(
                r#"
INSERT INTO packages (attribute, name, version, storePath, description, long_description)
VALUES (?, ?, ?, ?, ?, ?)
            "#,
            )
            .context("preparing INSERT query")?;

        registry
            .into_iter()
            .map(|(attr, info)| info.into_rippkgs_package(attr))
            .try_for_each(
                |Package {
                     attribute,
                     name,
                     version,
                     store_path,
                     description,
                     long_description,
                 }| {
                    create_row_query
                        .execute(rusqlite::params![
                            attribute,
                            name,
                            version,
                            store_path,
                            description,
                            long_description
                        ])
                        .context("inserting package into database")
                        .map(|_| ())
                },
            )?;
    }

    tx.commit().context("committing database")?;

    println!(
        "wrote index in {:.4} seconds",
        start.elapsed().as_secs_f64()
    );

    Ok(())
}

fn get_registry(
    Opts {
        nixpkgs,
        registry,
        nixpkgs_config,
        ..
    }: &Opts,
) -> eyre::Result<HashMap<String, PackageInfo>> {
    let registry_reader: Box<dyn io::Read> = if let Some(nixpkgs) = nixpkgs {
        let nixpkgs_include_arg = format!("nixpkgs={nixpkgs}");
        let nixpkgs_config = nixpkgs_config
            .as_ref()
            .map(|config| config.display().to_string())
            .unwrap_or("{}".to_string());
        let apply_arg = format!(
            r#"
genRegistry:

let pkgs = import <nixpkgs> {{ config = {nixpkgs_config}; }};
    genRegistry' = genRegistry pkgs;
in genRegistry' pkgs
            "#
        );

        let args = vec![
            "eval",
            "--impure",
            "--json",
            "-I",
            nixpkgs_include_arg.as_str(),
            "--expr",
            include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/lib/genRegistry.nix")),
            "--apply",
            apply_arg.as_str(),
        ];

        let start = Instant::now();

        let output = Command::new("nix")
            .args(args.iter())
            .output()
            .with_context(|| format!("getting nixpkgs packages from {nixpkgs}"))?;

        println!(
            "evaluated registry in {:.4} seconds",
            start.elapsed().as_secs_f64()
        );

        if !output.status.success() {
            panic!(
                "`nix eval` failed: {}",
                String::from_utf8_lossy(&output.stderr)
            );
        }

        if let Some(registry) = registry {
            File::options()
                .write(true)
                .truncate(true)
                .create(true)
                .open(registry)
                .context("opening registry file")?
                .write(&output.stdout)
                .context("writing registry file")?;
        }

        Box::new(VecDeque::from(output.stdout))
    } else if let Some(registry) = registry {
        let f = File::options()
            .read(true)
            .open(registry)
            .context("opening registry file")?;

        Box::new(f)
    } else {
        return Err(eyre::eyre!("expected nixpkgs location or cached registry"));
    };

    let start = Instant::now();
    let res = serde_json::from_reader(registry_reader).context("reading registry JSON");
    println!(
        "parsed registry in {:.4} seconds",
        start.elapsed().as_secs_f64()
    );

    res
}
