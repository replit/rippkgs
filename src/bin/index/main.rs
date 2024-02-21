mod data;

use std::{
    collections::{HashMap, VecDeque},
    fs::File,
    io::{self, Write},
    path::PathBuf,
    process::Command,
    time::Instant,
};

use clap::Parser;
use data::PackageInfo;
use eyre::{Context, Result};
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
    nixpkgs_config: Option<String>,
}

fn main() -> Result<()> {
    let opts = Opts::parse();

    let registry = get_registry(&opts).context("unable to get nixpkgs registry")?;

    match std::fs::remove_file(opts.output.as_path()) {
        Ok(()) => (),
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => (),
        Err(err) => Err(err).context("unable to remove previous index db")?,
    }

    let conn = rusqlite::Connection::open_with_flags(
        opts.output,
        OpenFlags::SQLITE_OPEN_CREATE
            | OpenFlags::SQLITE_OPEN_READ_WRITE
            | OpenFlags::SQLITE_OPEN_NO_MUTEX,
    )
    .context("unable to connect to index database")?;

    conn.execute(
        r#"
CREATE TABLE packages (
    attribute TEXT NOT NULL,
    store_path TEXT NOT NULL,
    name TEXT,
    version TEXT,
    description TEXT,
    homepage TEXT,
    long_description TEXT,
    PRIMARY KEY (attribute)
)
        "#,
        [],
    )
    .context("unable to create table in database")?;

    let mut create_row_query = conn
        .prepare(
            r#"
INSERT INTO packages (attribute, store_path, name, version, description, homepage, long_description)
VALUES (?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .context("unable to prepare INSERT query")?;

    let start = Instant::now();

    for (attr, info) in registry.into_iter() {
        let store_path = match info.outputs.get("out") {
            Some(out) => out.display().to_string(),
            None => continue,
        };

        let name = info.pname.as_ref().unwrap_or(&attr).as_str();
        let version = info.version.as_ref().unwrap().as_str();
        let description = info
            .meta
            .as_ref()
            .map(|meta| meta.description.clone())
            .flatten();
        let long_description = info
            .meta
            .as_ref()
            .map(|meta| meta.long_description.clone())
            .flatten();

        create_row_query
            .execute(rusqlite::params![
                attr,
                store_path,
                name,
                version,
                description,
                None::<String>,
                long_description
            ])
            .context("could not insert package into database")?;
    }

    println!("wrote index in {:.4} seconds", start.elapsed().as_secs_f64());

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
        let nixpkgs_var = format!("nixpkgs={}", nixpkgs);

        let mut args = vec![
            "--json",
            "-f",
            "<nixpkgs>",
            "-I",
            nixpkgs_var.as_str(),
            "-qa",
            "--meta",
            // TODO: get the out paths. unfortunately this can cause evaluation errors
            // since attributes can be missing but still be valid...
            // "--out-path",
        ];

        if let Some(config) = nixpkgs_config.as_ref() {
            args.push("--arg");
            args.push("config");
            args.push(config.as_str());
        }

        let start = Instant::now();

        let output = Command::new("nix-env")
            .args(args.iter())
            .output()
            .with_context(|| format!("failed to get nixpkgs packages from {nixpkgs}"))?;

        println!("evaluated registry in {:.4} seconds", start.elapsed().as_secs_f64());

        if !output.status.success() {
            panic!(
                "nix-env failed: {}",
                String::from_utf8_lossy(&output.stderr)
            );
        }

        if let Some(registry) = registry {
            File::options()
                .write(true)
                .truncate(true)
                .create(true)
                .open(registry)
                .context("couldn't open registry file")?
                .write(&output.stdout)
                .context("couldn't write registry file")?;
        }

        Box::new(VecDeque::from(output.stdout))
    } else if let Some(registry) = registry {
        let f = File::options()
            .read(true)
            .open(registry)
            .context("couldn't open registry file")?;

        Box::new(f)
    } else {
        return Err(eyre::eyre!("expected nixpkgs location or cached registry"));
    };

    let start = Instant::now();
    let res = serde_json::from_reader(registry_reader).context("unable to read registry JSON");
    println!("parsed registry in {:.4} seconds", start.elapsed().as_secs_f64());

    res
}
