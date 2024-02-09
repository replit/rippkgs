#![feature(assert_matches)]
#![feature(unix_sigpipe)]

mod data;

use std::{collections::HashMap, fs::File, path::PathBuf};

use clap::Parser;
use rippkgs_db::package;
use sea_orm::{ActiveModelTrait, ActiveValue, ConnectionTrait, Database, Schema};

#[derive(Debug, Parser)]
struct Opts {
    /// The location to write the saved index to.
    #[arg(short, long)]
    output: PathBuf,

    /// The flake URI of the nixpkgs to index.
    #[arg(short, long)]
    nixpkgs: String,

    /// The value to pass as the config parameter to nixpkgs.
    #[arg(short = 'c', long)]
    nixpkgs_config: Option<String>,
}

#[unix_sigpipe = "inherit"]
#[tokio::main]
async fn main() {
    let opts = Opts::parse();

    // let nixpkgs_var = format!("nixpkgs={}", opts.nixpkgs);
    // let mut args = vec![
    //     "--json",
    //     "-f",
    //     "<nixpkgs>",
    //     "-I",
    //     nixpkgs_var.as_str(),
    //     "-qa",
    //     "--meta",
    //     "--out-path",
    // ];
    //
    // if let Some(config) = opts.nixpkgs_config.as_ref() {
    //     args.push("--arg");
    //     args.push("config");
    //     args.push(config.as_str());
    // }
    //
    // let output = Command::new("nix-env")
    //     .args(args.iter())
    //     .output()
    //     .expect("failed to get nixpkgs packages");
    //
    // if !output.status.success() {
    //     panic!(
    //         "nix-env failed: {}",
    //         String::from_utf8_lossy(&output.stderr)
    //     );
    // }
    //
    // File::options()
    //     .write(true)
    //     .truncate(true)
    //     .create(true)
    //     .open("nixpkgs.json")
    //     .expect("couldn't open nixpkgs.json")
    //     .write(&output.stdout)
    //     .expect("couldn't write nixpkgs.json");
    //
    // let registry = serde_json::from_slice::<HashMap<String, data::PackageInfo>>(&output.stdout)
    //     .expect("unable to read nixpkgs registry JSON");

    let registry = serde_json::from_reader::<_, HashMap<String, data::PackageInfo>>(
        File::options()
            .read(true)
            .open("nixpkgs.json")
            .expect("couldn't open nixpkgs.json"),
    )
    .expect("unable to read nixpkgs registry JSON");

    // println!(
    //     "{}",
    //     registry
    //         .iter()
    //         .map(|(attr, info)| format!(
    //             "{attr}: meta={} outs={}",
    //             info.meta.is_some(),
    //             !info.outputs.is_empty()
    //         ))
    //         .collect::<Vec<_>>()
    //         .join("\n")
    // );

    // eprintln!("{output}");

    match std::fs::remove_file(opts.output.as_path()) {
        Ok(()) => (),
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => (),
        Err(err) => panic!("error deleting previous index: {:?}", err),
    }

    std::fs::File::options()
        .write(true)
        .create(true)
        .open(dbg!(opts.output.as_path()))
        .unwrap();

    let mut database = Database::connect(format!("sqlite:{}", opts.output.display()))
        .await
        .expect("unable to open sqlite db");
    let db = &mut database;

    let db_backend = db.get_database_backend();
    let schema = Schema::new(db_backend);
    db.execute(db_backend.build(&schema.create_table_from_entity(package::Entity)).into())
      .await
      .expect("unable to create package table");

    for (attr, info) in registry.into_iter() {
        let attribute = ActiveValue::set(attr.clone());
        let store_path = match info.outputs.get("out") {
            Some(out) => ActiveValue::set(out.display().to_string()),
            None => continue,
        };
        let name = ActiveValue::set(info.pname.unwrap_or_else(|| attr.clone()));
        let version = ActiveValue::set(info.version.unwrap());
        let description = ActiveValue::set(
            info.meta
                .as_ref()
                .map(|meta| meta.description.clone())
                .flatten(),
        );
        let homepage = ActiveValue::set(None);
        let long_description = ActiveValue::set(
            info.meta
                .as_ref()
                .map(|meta| meta.long_description.clone())
                .flatten(),
        );

        println!("saving {attr}: sp={store_path:?} n={name:?} v={version:?} d={description:?}");

        package::ActiveModel {
            attribute,
            store_path,
            name,
            version,
            description,
            homepage,
            long_description,
        }
        .insert(db)
        .await
        .expect("unable to save package");
    }
}
