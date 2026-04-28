use anyhow::{Context, Result};
use sqlx::{SqlitePool, sqlite::SqliteConnectOptions};
use std::{str::FromStr, time::Duration};

pub mod has_table;
pub mod insert;
pub mod queries;
pub mod select;
pub mod update;

pub async fn connect_to_url(db_url: &String) -> Result<SqlitePool> {
    let options = SqliteConnectOptions::from_str(&db_url)?
        .foreign_keys(true)
        .create_if_missing(false)
        // litestream recommended options: https://litestream.io/tips/
        .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal)
        .busy_timeout(Duration::from_secs(5))
        .foreign_keys(true)
        .synchronous(sqlx::sqlite::SqliteSynchronous::Normal);

    let pool = SqlitePool::connect_with(options)
        .await
        .context("Failed to connect to SQLite database")?;

    println!("Connected to database: {}", db_url);

    Ok(pool)
}
