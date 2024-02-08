#![feature(assert_matches)]
#![feature(unix_sigpipe)]

mod data;

use std::{collections::HashMap, fs::File, io::Write, path::PathBuf, process::Command};

use clap::Parser;

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
fn main() {
    let opts = Opts::parse();

    let nixpkgs_var = format!("nixpkgs={}", opts.nixpkgs);
    let mut args = vec![
        "--json",
        "-f",
        "<nixpkgs>",
        "-I",
        nixpkgs_var.as_str(),
        "-qa",
        "--meta",
        "--out-path",
    ];

    if let Some(config) = opts.nixpkgs_config.as_ref() {
        args.push("--arg");
        args.push("config");
        args.push(config.as_str());
    }

    let output = Command::new("nix-env")
        .args(args.iter())
        .output()
        .expect("failed to get nixpkgs packages");

    if !output.status.success() {
        panic!(
            "nix-env failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    File::options()
        .write(true)
        .truncate(true)
        .create(true)
        .open("nixpkgs.json")
        .expect("couldn't open nixpkgs.json")
        .write(&output.stdout)
        .expect("couldn't write nixpkgs.json");

    let registry = serde_json::from_slice::<HashMap<String, data::PackageInfo>>(&output.stdout)
        .expect("unable to read nixpkgs registry JSON");

    // let registry = serde_json::from_reader::<_, HashMap<String, data::PackageInfo>>(
    //     File::options()
    //         .read(true)
    //         .open("nixpkgs.json")
    //         .expect("couldn't open nixpkgs.json"),
    // )
    // .expect("unable to read nixpkgs registry JSON");

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
}
