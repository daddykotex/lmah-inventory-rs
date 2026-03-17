use clap::Parser;

/// CLI tool to load the data from a JSON database into a SQL database
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Location of the JSON file
    #[arg(short, long)]
    location: String,
}

fn main() {
    let args = Args::parse();
    println!("Hello {}!", args.location);
}