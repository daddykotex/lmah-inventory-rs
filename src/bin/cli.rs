use anyhow::{Context, Result};
use clap::{Args, Parser, Subcommand};
use lmah_inventory_rs::cli::migration::{
    ClientFields, ConfigFields, EventFields, ProductTypeFields, load_and_insert_facture_items,
    load_and_insert_factures, load_and_insert_payments, load_and_insert_products,
    load_and_insert_refunds, load_and_insert_statuts, load_data, load_records,
};
use lmah_inventory_rs::server::models::clients::ClientRow;
use lmah_inventory_rs::server::models::config::ConfigRow;
use lmah_inventory_rs::server::models::events::EventRow;
use lmah_inventory_rs::server::models::product_types::ProductTypeRow;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePool};
use std::path::{Path, PathBuf};
use std::str::FromStr;

/// CLI tool to load the data from a JSON database into a SQL database
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    /// Command to run
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Load the data from the JSON file into the SQLite database
    Load(LoadArgs),
}
/// Options for the load command
#[derive(Args, Debug)]
struct MigrateArgs {
    /// Location of the SQLite database
    #[arg(short, long)]
    target: PathBuf,
}

/// Options for the load command
#[derive(Args, Debug)]
struct LoadArgs {
    /// Location of the JSON file
    #[arg(short, long)]
    src: PathBuf,

    /// Location of the SQLite database
    #[arg(short, long)]
    target: PathBuf,
}

fn assert_args(args: &LoadArgs) {
    assert!(args.src.exists(), "The source JSON file does not exist.");
    assert!(
        args.src
            .extension()
            .expect("No extension on the source JSON file.")
            == "json",
        "The extension is not `.json`."
    );

    assert!(
        args.target.exists(),
        "The target SQLite database file does not exist."
    );
    assert!(
        args.target
            .extension()
            .expect("No extension on the target file.")
            == "db",
        "The extension is not `.db`."
    );
}

async fn connect_to_database(db_path: &Path) -> Result<SqlitePool> {
    let connection_string = format!("sqlite://{}", db_path.display());

    let options = SqliteConnectOptions::from_str(&connection_string)?
        .foreign_keys(true)
        .create_if_missing(false);

    let pool = SqlitePool::connect_with(options)
        .await
        .context("Failed to connect to SQLite database")?;

    println!("Connected to database: {}", db_path.display());

    Ok(pool)
}

async fn verify_import(pool: &SqlitePool) -> Result<()> {
    let (count,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM config")
        .fetch_one(pool)
        .await?;

    println!("Total config records in database: {}", count);

    let type_counts: Vec<(String, i64)> =
        sqlx::query_as("SELECT type, COUNT(*) as count FROM config GROUP BY type ORDER BY type")
            .fetch_all(pool)
            .await?;

    println!("\nRecords by type:");
    for (config_type, count) in type_counts {
        println!("  {}: {}", config_type, count);
    }

    Ok(())
}

async fn verify_clients_import(pool: &SqlitePool) -> Result<()> {
    let (count,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM clients")
        .fetch_one(pool)
        .await?;

    println!("Total client records in database: {}", count);

    Ok(())
}

async fn load(args: &LoadArgs) -> Result<()> {
    assert_args(&args);

    println!("LMAH Inventory - Data Loader");
    println!("===========================");
    println!("Source: {}", args.src.display());
    println!("Target: {}", args.target.display());
    println!();

    // ===== LOAD JSON =====
    println!("Step 1: Loading JSON...");
    let export = load_data(&args.src).await?;
    println!("✓ JSON loaded and validated successfully");

    // ===== CONNECT TO DATABASE =====
    println!("\nStep 2: Connecting to database...");
    let pool = connect_to_database(&args.target).await?;
    println!("✓ Database connection established");

    // ===== INSERT CONFIG =====
    println!("\nStep 4: Inserting config records...");
    load_records::<ConfigFields, ConfigRow>(&pool, export.config).await?;

    // ===== INSERT CLIENTS =====
    println!("\nStep 5: Inserting client records...");
    load_records::<ClientFields, ClientRow>(&pool, export.clients).await?;

    // ===== INSERT PRODUCT_TYPES =====
    println!("\nStep 6: Inserting product_types records...");
    load_records::<ProductTypeFields, ProductTypeRow>(&pool, export.product_types).await?;

    // ===== INSERT EVENTS =====
    println!("\nStep 7: Inserting events records...");
    load_records::<EventFields, EventRow>(&pool, export.events).await?;

    // ===== INSERT PRODUCTS (with types and images) =====
    println!("\nStep 8: Inserting products with related data...");
    load_and_insert_products(&pool, export.products).await?;

    // ===== INSERT FACTURES (with FK resolution) =====
    println!("\nStep 9: Inserting factures with foreign key resolution...");
    load_and_insert_factures(&pool, export.factures).await?;

    // ===== INSERT FACTURE_ITEMS (with FK resolution) =====
    println!("\nStep 10: Inserting facture_items with foreign key resolution...");
    load_and_insert_facture_items(&pool, export.facture_items).await?;

    // ===== INSERT PAYMENTS (with FK resolution) =====
    println!("\nStep 11: Inserting payments with foreign key resolution...");
    load_and_insert_payments(&pool, export.payments).await?;

    // ===== INSERT REFUNDS (with FK resolution) =====
    println!("\nStep 12: Inserting refunds with foreign key resolution...");
    load_and_insert_refunds(&pool, export.refunds).await?;

    // ===== INSERT STATUTS (with FK resolution) =====
    println!("\nStep 13: Inserting statuts with foreign key resolution...");
    load_and_insert_statuts(&pool, export.statuts).await?;

    // ===== VERIFY IMPORTS =====
    println!("\nStep 14: Verifying imports...");
    println!("\nConfig verification:");
    verify_import(&pool).await?;

    println!("\nClients verification:");
    verify_clients_import(&pool).await?;

    println!("\n✓ Import completed successfully!");

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Load(load_args) => load(load_args).await?,
    }

    Ok(())
}
