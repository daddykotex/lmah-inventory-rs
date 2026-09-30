use anyhow::{Context, Result};

use crate::server::models::{clients::ClientRow, events::EventRow, products::ProductRow};

pub trait Updatable {
    fn update_one(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    ) -> impl Future<Output = Result<u64>>;
}

impl Updatable for ClientRow {
    async fn update_one(&self, tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>) -> Result<u64> {
        // Insert client row
        let result = sqlx::query(
            "UPDATE clients
                 SET
                     first_name = ?,
                     last_name = ?,
                     street = ?,
                     city = ?,
                     phone1 = ?,
                     phone2 = ?,
                     updated_at = datetime('now')
                 WHERE id = ?",
        )
        .bind(&self.first_name)
        .bind(&self.last_name)
        .bind(&self.street)
        .bind(&self.city)
        .bind(&self.phone1)
        .bind(&self.phone2)
        .bind(self.id)
        .execute(&mut **tx)
        .await
        .with_context(|| {
            format!(
                "Failed to update client: {} {}",
                self.first_name, self.last_name
            )
        })?;

        Ok(result.rows_affected())
    }
}

impl Updatable for EventRow {
    async fn update_one(&self, tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>) -> Result<u64> {
        // Insert client row
        let result = sqlx::query(
            "UPDATE events
                 SET
                     name = ?,
                     date = ?,
                     event_type = ?
                 WHERE id = ?",
        )
        .bind(&self.name)
        .bind(&self.date)
        .bind(&self.event_type)
        .bind(self.id)
        .execute(&mut **tx)
        .await
        .with_context(|| format!("Failed to update event: {}", self.name))?;

        Ok(result.rows_affected())
    }
}

impl Updatable for ProductRow {
    async fn update_one(&self, tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>) -> Result<u64> {
        let result = sqlx::query(
            "UPDATE products
                 SET
                     name = ?,
                     price = ?,
                     liquidation = ?,
                     visible_on_site = ?,
                     updated_at = datetime('now')
                 WHERE id = ?",
        )
        .bind(&self.name)
        .bind(self.price)
        .bind(if self.liquidation { 1 } else { 0 })
        .bind(if self.visible_on_site { 1 } else { 0 })
        .bind(self.id)
        .execute(&mut **tx)
        .await
        .with_context(|| format!("Failed to update product: {}", self.name))?;

        Ok(result.rows_affected())
    }
}
