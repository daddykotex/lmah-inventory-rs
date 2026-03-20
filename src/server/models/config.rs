use anyhow::{Context, Result};
use async_trait::async_trait;
use sqlx::{Sqlite, Transaction};

/// Database row structure for config table
#[derive(Debug)]
pub struct ConfigRow {
    pub key: String,
    pub value: String,
    pub config_type: String,
    pub created_at: String,
    pub updated_at: String,
}

#[async_trait]
impl crate::cli::migration::ImportableRecord for ConfigRow {
    fn table_name() -> &'static str {
        "config"
    }

    async fn insert_record(&self, tx: &mut Transaction<'_, Sqlite>) -> Result<()> {
        sqlx::query(
            "INSERT INTO config (key, value, type, created_at, updated_at)
             VALUES (?, ?, ?, ?, ?)",
        )
        .bind(&self.key)
        .bind(&self.value)
        .bind(&self.config_type)
        .bind(&self.created_at)
        .bind(&self.updated_at)
        .execute(&mut **tx)
        .await
        .with_context(|| format!("Failed to insert config key: {}", self.key))?;
        Ok(())
    }

    fn display_name(&self) -> String {
        format!("config key: {}", self.key)
    }
}
