use anyhow::{Context, Result};

use crate::server::models::{
    clients::ClientRowWithId, config::ConfigRow, events::EventRowWithId,
    product_types::ProductTypeRow,
};

pub trait Insertable {
    fn insert_one(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    ) -> impl Future<Output = Result<()>>;
}

impl Insertable for ConfigRow {
    async fn insert_one(&self, tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>) -> Result<()> {
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
        .with_context(|| {
            format!(
                "Failed to insert config key: {}, value: {}",
                self.key, self.value
            )
        })?;

        return Ok(());
    }
}

impl Insertable for ClientRowWithId {
    async fn insert_one(&self, tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>) -> Result<()> {
        // Insert client row
        let result = sqlx::query(
            "INSERT INTO clients (first_name, last_name, street, city, phone1, phone2, created_at, updated_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(&self.row.first_name)
        .bind(&self.row.last_name)
        .bind(&self.row.street)
        .bind(&self.row.city)
        .bind(&self.row.phone1)
        .bind(&self.row.phone2)
        .bind(&self.row.created_at)
        .bind(&self.row.updated_at)
        .execute(&mut **tx)
        .await
        .with_context(|| {
            format!(
                "Failed to insert client: {} {}",
                self.row.first_name, self.row.last_name
            )
        })?;

        // Get the database ID
        let db_id = result.last_insert_rowid();

        // Insert mapping into airtable_mapping table
        sqlx::query(
            "INSERT INTO airtable_mapping (table_name, airtable_id, db_id)
             VALUES (?, ?, ?)",
        )
        .bind("clients")
        .bind(&self.airtable_id)
        .bind(db_id)
        .execute(&mut **tx)
        .await
        .with_context(|| {
            format!(
                "Failed to insert airtable mapping for client: {} {} (airtable_id: {})",
                self.row.first_name, self.row.last_name, self.airtable_id
            )
        })?;

        return Ok(());
    }
}

impl Insertable for ProductTypeRow {
    async fn insert_one(&self, tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>) -> Result<()> {
        sqlx::query("INSERT INTO product_types (name) VALUES (?)")
            .bind(&self.name)
            .execute(&mut **tx)
            .await
            .with_context(|| format!("Failed to insert product_type: {}", self.name))?;

        return Ok(());
    }
}

impl Insertable for EventRowWithId {
    async fn insert_one(&self, tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>) -> Result<()> {
        // Insert event row
        let result = sqlx::query(
            "INSERT INTO events (name, type, date, created_at, updated_at)
             VALUES (?, ?, ?, ?, ?)",
        )
        .bind(&self.row.name)
        .bind(&self.row.event_type)
        .bind(&self.row.date)
        .bind(&self.row.created_at)
        .bind(&self.row.updated_at)
        .execute(&mut **tx)
        .await
        .with_context(|| format!("Failed to insert event: {}", self.row.name))?;

        // Get the database ID
        let db_id = result.last_insert_rowid();

        // Insert mapping into airtable_mapping table
        sqlx::query(
            "INSERT INTO airtable_mapping (table_name, airtable_id, db_id)
             VALUES (?, ?, ?)",
        )
        .bind("events")
        .bind(&self.airtable_id)
        .bind(db_id)
        .execute(&mut **tx)
        .await
        .with_context(|| {
            format!(
                "Failed to insert airtable mapping for event: {} (airtable_id: {})",
                self.row.name, self.airtable_id
            )
        })?;

        return Ok(());
    }
}
