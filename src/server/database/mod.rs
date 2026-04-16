use anyhow::{Context, Result};
use sqlx::{SqlitePool, sqlite::SqliteConnectOptions};
use std::str::FromStr;

pub mod has_table;
pub mod insert;
pub mod queries;
pub mod select;
pub mod update;

pub async fn connect_to_url(db_url: &String) -> Result<SqlitePool> {
    let options = SqliteConnectOptions::from_str(&db_url)?
        .foreign_keys(true)
        .create_if_missing(false)
        .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal);

    let pool = SqlitePool::connect_with(options)
        .await
        .context("Failed to connect to SQLite database")?;

    println!("Connected to database: {}", db_url);

    Ok(pool)
}
