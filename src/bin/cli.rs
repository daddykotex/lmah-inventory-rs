use anyhow::Result;
use clap::{Args, Parser, Subcommand};
use lmah_inventory_rs::cli::export_airtable::{ExportAirtableOptions, run as export_airtable_run};
use lmah_inventory_rs::cli::sync_site::{SyncSiteOptions, run as sync_site_run};
use std::path::PathBuf;

/// CLI tool for the LMAH inventory app
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Fetch all data from Airtable and insert it into the SQLite database
    ExportAirtable(ExportAirtableArgs),
    /// Export product data as JSON for the marieealhonneur Hugo site
    SyncSite(SyncSiteArgs),
}

#[derive(Args, Debug)]
struct ExportAirtableArgs {
    /// Location of the SQLite database
    #[arg(short, long, env = "DATABASE_URL")]
    target: String,

    /// Airtable personal access token
    #[arg(long, env = "LMAH_AIRTABLE_PAT", hide_env_values = true)]
    airtable_pat: String,

    /// Airtable base URL (e.g. https://api.airtable.com/v0/appXXXXX/)
    #[arg(long, env = "LMAH_AIRTABLE_URL")]
    airtable_url: String,

    /// GCS public bucket used to rewrite product image URLs
    #[arg(long, env = "LMAH_GOOGLE_PUBLIC_BUCKET_NAME")]
    public_bucket: Option<String>,
}

#[derive(Args, Debug)]
struct SyncSiteArgs {
    /// Location of the SQLite database
    #[arg(long, env = "DATABASE_URL")]
    db_url: String,

    /// Directory where the JSON files will be written
    #[arg(short, long)]
    out: PathBuf,
}

async fn export_airtable(args: &ExportAirtableArgs) -> Result<()> {
    export_airtable_run(ExportAirtableOptions {
        target: args.target.clone(),
        airtable_pat: args.airtable_pat.clone(),
        airtable_url: args.airtable_url.clone(),
        public_bucket: args.public_bucket.clone(),
    })
    .await
}

async fn sync_site(args: &SyncSiteArgs) -> Result<()> {
    sync_site_run(SyncSiteOptions {
        db_url: &args.db_url,
        out_dir: &args.out,
    })
    .await
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::ExportAirtable(args) => export_airtable(args).await?,
        Commands::SyncSite(args) => sync_site(args).await?,
    }

    Ok(())
}
