use anyhow::{Context, Result};

use crate::server::models::{
    clients::ClientRow,
    config::ConfigRow,
    events::EventRow,
    factures::FactureRow,
    product_types::ProductTypeRow,
    products::{ProductImageRow, ProductRow},
};

pub trait Insertable {
    fn insert_one(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    ) -> impl Future<Output = Result<Option<i64>>>;
}

impl Insertable for ConfigRow {
    async fn insert_one(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    ) -> Result<Option<i64>> {
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

        Ok(None)
    }
}

impl Insertable for ClientRow {
    async fn insert_one(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    ) -> Result<Option<i64>> {
        // Insert client row
        let result = sqlx::query(
            "INSERT INTO clients (first_name, last_name, street, city, phone1, phone2, created_at, updated_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
        )
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

        Ok(Some(result.last_insert_rowid()))
    }
}

impl Insertable for ProductTypeRow {
    async fn insert_one(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    ) -> Result<Option<i64>> {
        sqlx::query("INSERT INTO product_types (name) VALUES (?)")
            .bind(&self.name)
            .execute(&mut **tx)
            .await
            .with_context(|| format!("Failed to insert product_type: {}", self.name))?;

        return Ok(None);
    }
}

impl Insertable for EventRow {
    async fn insert_one(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    ) -> Result<Option<i64>> {
        // Insert event row
        let result = sqlx::query(
            "INSERT INTO events (name, type, date, created_at, updated_at)
             VALUES (?, ?, ?, ?, ?)",
        )
        .bind(&self.name)
        .bind(&self.event_type)
        .bind(&self.date)
        .bind(&self.created_at)
        .bind(&self.updated_at)
        .execute(&mut **tx)
        .await
        .with_context(|| format!("Failed to insert event: {}", self.name))?;

        // Get the database ID
        let db_id = result.last_insert_rowid();

        return Ok(Some(db_id));
    }
}

impl Insertable for ProductRow {
    async fn insert_one(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    ) -> Result<Option<i64>> {
        // Insert product row
        let result = sqlx::query(
            "INSERT INTO products (name, price, liquidation, visible_on_site, created_at, updated_at)
             VALUES (?, ?, ?, ?, ?, ?)",
        )
        .bind(&self.name)
        .bind(&self.price)
        .bind(&self.liquidation)
        .bind(if self.visible_on_site { 1 } else { 0 })
        .bind(&self.created_at)
        .bind(&self.updated_at)
        .execute(&mut **tx)
        .await
        .with_context(|| format!("Failed to insert product: {}", self.name))?;

        // Get the database ID
        let db_id = result.last_insert_rowid();

        return Ok(Some(db_id));
    }
}

impl Insertable for ProductImageRow {
    async fn insert_one(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    ) -> Result<Option<i64>> {
        sqlx::query(
            "INSERT INTO product_images (product_id, url, filename, position, created_at)
             VALUES (?, ?, ?, ?, ?)",
        )
        .bind(&self.product_id)
        .bind(&self.url)
        .bind(&self.filename)
        .bind(&self.position)
        .bind(&self.created_at)
        .execute(&mut **tx)
        .await
        .with_context(|| {
            format!(
                "Failed to insert product image: {} (position: {})",
                self.filename, self.position
            )
        })?;

        Ok(None)
    }
}

impl Insertable for FactureRow {
    async fn insert_one(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    ) -> Result<Option<i64>> {
        // Insert facture row
        let result = sqlx::query(
            "INSERT INTO factures (client_id, type, date, event_id, fixed_total, cancelled, paper_ref, created_at, updated_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(&self.client_id)
        .bind(&self.facture_type)
        .bind(&self.date)
        .bind(&self.event_id)
        .bind(&self.fixed_total)
        .bind(if self.cancelled { 1 } else { 0 })
        .bind(&self.paper_ref)
        .bind(&self.created_at)
        .bind(&self.updated_at)
        .execute(&mut **tx)
        .await
        .with_context(|| {
            format!(
                "Failed to insert facture for client_id={}",
                self.client_id
            )
        })?;

        // Get the database ID
        let db_id = result.last_insert_rowid();

        return Ok(Some(db_id));
    }
}
