use anyhow::Result;
use clap::{Args, Parser, Subcommand};
use lmah_inventory_rs::cli::migration::{
    ClientFields, ConfigFields, EventFields, ProductTypeFields, check_counts,
    load_and_insert_facture_items, load_and_insert_factures, load_and_insert_payments,
    load_and_insert_products, load_and_insert_refunds, load_and_insert_statuts, load_data,
    load_records, sort_export_by_created_time,
};
use lmah_inventory_rs::server::database::connect_to_url;
use lmah_inventory_rs::server::models::clients::ClientInsert;
use lmah_inventory_rs::server::models::config::ConfigInsert;
use lmah_inventory_rs::server::models::events::EventInsert;
use lmah_inventory_rs::server::models::product_types::ProductTypeRow;
use sqlx::sqlite::SqlitePool;
use std::path::PathBuf;

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
    #[arg(short, long, env = "DATABASE_URL")]
    db_url: String,
}

/// Options for the load command
#[derive(Args, Debug)]
struct LoadArgs {
    /// Location of the JSON file
    #[arg(short, long)]
    src: PathBuf,

    /// Location of the SQLite database
    #[arg(short, long, env = "DATABASE_URL")]
    target: String,
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
}

async fn verify_import(pool: &SqlitePool) -> Result<()> {
    let (count,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM config")
        .fetch_one(pool)
        .await?;

    println!("Total config records in database: {}", count);

    let type_counts: Vec<(String, i64)> =
        sqlx::query_as("SELECT config_type, COUNT(*) as count FROM config GROUP BY config_type ORDER BY config_type")
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
    assert_args(args);

    println!("LMAH Inventory - Data Loader");
    println!("===========================");
    println!("Source: {}", args.src.display());
    println!("Target: {}", args.target);
    println!();

    // ===== LOAD JSON =====
    println!("Step 1: Loading JSON...");
    let export = load_data(&args.src).await?;
    let export = sort_export_by_created_time(export);
    println!("✓ JSON loaded and validated successfully");

    // ===== CONNECT TO DATABASE =====
    println!("\nStep 2: Connecting to database...");
    let pool = connect_to_url(&args.target).await?;
    check_counts(&pool).await?;
    println!("✓ Database connection established");

    // ===== INSERT CONFIG =====
    println!("\nStep 4: Inserting config records...");
    load_records::<ConfigFields, ConfigInsert>(&pool, export.config).await?;

    // ===== INSERT CLIENTS =====
    println!("\nStep 5: Inserting client records...");
    load_records::<ClientFields, ClientInsert>(&pool, export.clients).await?;

    // ===== INSERT PRODUCT_TYPES =====
    println!("\nStep 6: Inserting product_types records...");
    load_records::<ProductTypeFields, ProductTypeRow>(&pool, export.product_types).await?;

    // ===== INSERT EVENTS =====
    println!("\nStep 7: Inserting events records...");
    load_records::<EventFields, EventInsert>(&pool, export.events).await?;

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
