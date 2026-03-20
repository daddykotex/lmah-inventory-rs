use anyhow::{Context, Result};
use async_trait::async_trait;
use sqlx::{Sqlite, Transaction};

/// Database row structure for clients table
#[derive(Debug)]
pub struct ClientRow {
    pub airtable_id: String,
    pub first_name: String,
    pub last_name: String,
    pub street: Option<String>,
    pub city: Option<String>,
    pub phone1: String,
    pub phone2: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[async_trait]
impl crate::cli::migration::ImportableRecord for ClientRow {
    fn table_name() -> &'static str {
        "clients"
    }

    async fn insert_record(&self, tx: &mut Transaction<'_, Sqlite>) -> Result<()> {
        sqlx::query(
            "INSERT INTO clients (airtable_id, first_name, last_name, street, city, phone1, phone2, created_at, updated_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(&self.airtable_id)
        .bind(&self.first_name)
        .bind(&self.last_name)
        .bind(&self.street)
        .bind(&self.city)
        .bind(&self.phone1)
        .bind(&self.phone2)
        .bind(&self.created_at)
        .bind(&self.updated_at)
        .execute(&mut **tx)
        .await
        .with_context(|| {
            format!(
                "Failed to insert client: {} {}",
                self.first_name, self.last_name
            )
        })?;
        Ok(())
    }

    fn display_name(&self) -> String {
        format!("client: {} {}", self.first_name, self.last_name)
    }
}
