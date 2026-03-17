use clap::Parser;
use std::{fs, path::PathBuf};

/// CLI tool to load the data from a JSON database into a SQL database
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Location of the JSON file
    #[arg(short, long)]
    src: PathBuf,

    
    /// Location of the SQLite database
    #[arg(short, long)]
    target: PathBuf,
}

fn assert_args(args: &Args) {
    assert!(args.src.exists(), "The source JSON file does not exist.");
    assert!(
        args.src.extension().expect("No extension on the source JSON file.") == "json",
        "The extension is not `.json`."
    );

    assert!(args.target.exists(), "The target SQLite database file does not exist.");
    assert!(
        args.target.extension().expect("No extension on the target file.") == "db",
        "The extension is not `.db`."
    );
}

fn main() {
    let args = Args::parse();

    assert_args(&args);

    println!("Hello {}!", args.src.display());
}