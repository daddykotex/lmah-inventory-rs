use anyhow::{Context, Result};
use clap::Parser;
use lmah_inventory_rs::cli::migration::load_config_from_json;
use lmah_inventory_rs::server::models::config::ConfigRow;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePool};
use std::path::PathBuf;
use std::str::FromStr;

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

    /// Clear existing config records before importing
    #[arg(long)]
    clear_existing: bool,
}

fn assert_args(args: &Args) {
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

async fn connect_to_database(db_path: &std::path::Path) -> Result<SqlitePool> {
    let connection_string = format!("sqlite://{}", db_path.display());

    let options = SqliteConnectOptions::from_str(&connection_string)?
        .foreign_keys(true)
        .create_if_missing(false);

    let pool = SqlitePool::connect_with(options)
        .await
        .context("Failed to connect to SQLite database")?;

    println!("Connected to database: {}", db_path.display());

    verify_config_table_exists(&pool).await?;

    Ok(pool)
}

async fn verify_config_table_exists(pool: &SqlitePool) -> Result<()> {
    let result: (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='config'")
            .fetch_one(pool)
            .await
            .context("Failed to verify config table")?;

    if result.0 == 0 {
        anyhow::bail!("config table does not exist. Run migration.sql first.");
    }

    Ok(())
}

async fn check_existing_records(pool: &SqlitePool) -> Result<i64> {
    let (count,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM config")
        .fetch_one(pool)
        .await?;
    Ok(count)
}

async fn insert_config_records(
    pool: &SqlitePool,
    records: Vec<ConfigRow>,
    clear_existing: bool,
) -> Result<()> {
    let existing_count = check_existing_records(pool).await?;

    if existing_count > 0 && !clear_existing {
        anyhow::bail!(
            "Config table already contains {} records. Use --clear-existing flag to clear and reload.",
            existing_count
        );
    }

    let mut tx = pool.begin().await.context("Failed to begin transaction")?;

    if clear_existing && existing_count > 0 {
        sqlx::query("DELETE FROM config")
            .execute(&mut *tx)
            .await
            .context("Failed to clear config table")?;
        println!("Cleared {} existing config records", existing_count);
    }

    let record_count = records.len();
    for record in records {
        sqlx::query(
            "INSERT INTO config (key, value, type, created_at, updated_at)
             VALUES (?, ?, ?, ?, ?)",
        )
        .bind(&record.key)
        .bind(&record.value)
        .bind(&record.config_type)
        .bind(&record.created_at)
        .bind(&record.updated_at)
        .execute(&mut *tx)
        .await
        .with_context(|| format!("Failed to insert config key: {}", record.key))?;
    }

    tx.commit().await.context("Failed to commit transaction")?;

    println!("Inserted {} new config records", record_count);
    Ok(())
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

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    assert_args(&args);

    println!("LMAH Inventory - Config Data Loader");
    println!("===================================");
    println!("Source: {}", args.src.display());
    println!("Target: {}", args.target.display());
    println!();

    println!("Step 1: Loading JSON...");
    let records = load_config_from_json(&args.src).await?;

    println!("\nStep 2: Connecting to database...");
    let pool = connect_to_database(&args.target).await?;

    println!("\nStep 3: Inserting records...");
    insert_config_records(&pool, records, args.clear_existing).await?;

    println!("\nStep 4: Verifying import...");
    verify_import(&pool).await?;

    println!("\n✓ Import completed successfully!");

    Ok(())
}
