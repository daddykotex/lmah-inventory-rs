use anyhow::{Context, Result};

use crate::server::models::{
    clients::{ClientInsert, ClientRow},
    config::ConfigInsert,
    events::{EventInsert, EventRow},
    facture_items::FactureItemInsert,
    factures::{FactureInsert, FactureRow},
    payments::PaymentInsert,
    product_types::ProductTypeRow,
    products::{ProductImageInsert, ProductInsert},
    refunds::RefundInsert,
    statuts::StatutInsert,
};

pub trait Insertable {
    fn insert_one(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    ) -> impl Future<Output = Result<Option<i64>>>;
}

impl Insertable for ConfigInsert {
    async fn insert_one(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    ) -> Result<Option<i64>> {
        sqlx::query(
            "INSERT INTO config (key, value, config_type, created_at, updated_at)
             VALUES (?, ?, ?, datetime('now'), datetime('now'))",
        )
        .bind(&self.key)
        .bind(&self.value)
        .bind(&self.config_type)
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

impl Insertable for ClientInsert {
    async fn insert_one(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    ) -> Result<Option<i64>> {
        // Insert client row
        let result = sqlx::query(
            "INSERT INTO clients (first_name, last_name, street, city, phone1, phone2, created_at, updated_at)
             VALUES (?, ?, ?, ?, ?, ?, datetime('now'), datetime('now'))",
        )
        .bind(&self.first_name)
        .bind(&self.last_name)
        .bind(&self.street)
        .bind(&self.city)
        .bind(&self.phone1)
        .bind(&self.phone2)
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

impl Insertable for ClientRow {
    async fn insert_one(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    ) -> Result<Option<i64>> {
        // Insert client row
        let result = sqlx::query(
            "INSERT INTO clients (id, first_name, last_name, street, city, phone1, phone2, created_at, updated_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(self.id)
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

        Ok(None)
    }
}

impl Insertable for EventInsert {
    async fn insert_one(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    ) -> Result<Option<i64>> {
        // Insert client row
        let result = sqlx::query(
            "INSERT INTO events (name, date, event_type, created_at, updated_at)
             VALUES (?, ?, ?, datetime('now'), datetime('now'))",
        )
        .bind(&self.name)
        .bind(&self.date)
        .bind(&self.event_type)
        .execute(&mut **tx)
        .await
        .with_context(|| format!("Failed to insert event: {}", self.name,))?;

        Ok(Some(result.last_insert_rowid()))
    }
}

impl Insertable for EventRow {
    async fn insert_one(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    ) -> Result<Option<i64>> {
        // Insert event row
        let result = sqlx::query(
            "INSERT INTO events (name, event_type, date, created_at, updated_at)
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

        Ok(Some(db_id))
    }
}

impl Insertable for ProductInsert {
    async fn insert_one(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    ) -> Result<Option<i64>> {
        // Insert product row
        let result = sqlx::query(
            "INSERT INTO products (name, price, liquidation, visible_on_site, created_at, updated_at)
             VALUES (?, ?, ?, ?, datetime('now'), datetime('now'))",
        )
        .bind(&self.name)
        .bind(self.price)
        .bind(self.liquidation)
        .bind(if self.visible_on_site { 1 } else { 0 })
        .execute(&mut **tx)
        .await
        .with_context(|| format!("Failed to insert product: {}", self.name))?;

        // Get the database ID
        let db_id = result.last_insert_rowid();

        Ok(Some(db_id))
    }
}

impl Insertable for ProductImageInsert {
    async fn insert_one(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    ) -> Result<Option<i64>> {
        sqlx::query(
            "INSERT INTO product_images (product_id, url, filename, position, created_at)
             VALUES (?, ?, ?, ?, datetime('now'))",
        )
        .bind(self.product_id)
        .bind(&self.url)
        .bind(&self.filename)
        .bind(&self.position)
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
            "INSERT INTO factures (client_id, facture_type, date, event_id, fixed_total, cancelled, paper_ref, created_at, updated_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(self.client_id)
        .bind(&self.facture_type)
        .bind(&self.date)
        .bind(self.event_id)
        .bind(self.fixed_total)
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

        Ok(Some(db_id))
    }
}

impl Insertable for FactureInsert {
    async fn insert_one(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    ) -> Result<Option<i64>> {
        // Insert facture
        let result = sqlx::query(
            "INSERT INTO factures (client_id, facture_type, event_id, fixed_total, cancelled, paper_ref, date, created_at, updated_at)
             VALUES (?, ?, ?, ?, ?, ?, date('now'), datetime('now'), datetime('now'))",
        )
        .bind(self.client_id)
        .bind(&self.facture_type)
        .bind(self.event_id)
        .bind(self.fixed_total)
        .bind(if self.cancelled { 1 } else { 0 })
        .bind(&self.paper_ref)
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

        Ok(Some(db_id))
    }
}

impl Insertable for FactureItemInsert {
    async fn insert_one(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    ) -> Result<Option<i64>> {
        // Insert facture_item row
        let result = sqlx::query(
            "INSERT INTO facture_items (
                facture_id, product_id, item_type,
                price, notes, quantity,
                extra_large_size, rebate_percent, size, chest, waist, hips, color, beneficiary, floor_item,
                insurance, other_costs,
                rebate_dollar,
                created_at, updated_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, datetime('now'), datetime('now'))",
        )
        .bind(self.facture_id)
        .bind(self.product_id)
        .bind(&self.item_type)
        .bind(self.price)
        .bind(&self.notes)
        .bind(self.quantity)
        .bind(self.extra_large_size)
        .bind(self.rebate_percent)
        .bind(&self.size)
        .bind(self.chest)
        .bind(self.waist)
        .bind(self.hips)
        .bind(&self.color)
        .bind(&self.beneficiary)
        .bind(if self.floor_item { 1 } else { 0 })
        .bind(self.insurance)
        .bind(self.other_costs)
        .bind(self.rebate_dollar)
        .execute(&mut **tx)
        .await
        .with_context(|| {
            format!(
                "Failed to insert facture_item for facture_id={}, product_id={}",
                self.facture_id, self.product_id
            )
        })?;

        // Get the database ID
        let db_id = result.last_insert_rowid();

        Ok(Some(db_id))
    }
}

impl Insertable for PaymentInsert {
    async fn insert_one(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    ) -> Result<Option<i64>> {
        let result = sqlx::query(
            "INSERT INTO payments (facture_id, amount, date, payment_type, cheque_number, created_at, updated_at)
             VALUES (?, ?, ?, ?, ?, datetime('now'), datetime('now'))",
        )
        .bind(self.facture_id)
        .bind(self.amount)
        .bind(&self.date)
        .bind(&self.payment_type)
        .bind(&self.cheque_number)
        .execute(&mut **tx)
        .await
        .with_context(|| {
            format!(
                "Failed to insert payment for facture_id={}",
                self.facture_id
            )
        })?;

        let db_id = result.last_insert_rowid();

        Ok(Some(db_id))
    }
}

impl Insertable for RefundInsert {
    async fn insert_one(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    ) -> Result<Option<i64>> {
        let result = sqlx::query(
            "INSERT INTO refunds (facture_id, amount, date, refund_type, cheque_number, created_at, updated_at)
             VALUES (?, ?, ?, ?, ?, datetime('now'), datetime('now'))",
        )
        .bind(self.facture_id)
        .bind(self.amount)
        .bind(&self.date)
        .bind(&self.refund_type)
        .bind(&self.cheque_number)
        .execute(&mut **tx)
        .await
        .with_context(|| {
            format!(
                "Failed to insert refund for facture_id={}",
                self.facture_id
            )
        })?;

        let db_id = result.last_insert_rowid();

        Ok(Some(db_id))
    }
}

impl Insertable for StatutInsert {
    async fn insert_one(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    ) -> Result<Option<i64>> {
        let result = sqlx::query(
            "INSERT INTO statuts (facture_id, facture_item_id, statut_type, date, seamstress, created_at, updated_at)
             VALUES (?, ?, ?, ?, ?, datetime('now'), datetime('now'))",
        )
        .bind(self.facture_id)
        .bind(self.facture_item_id)
        .bind(&self.statut_type)
        .bind(&self.date)
        .bind(&self.seamstress)
        .execute(&mut **tx)
        .await
        .with_context(|| {
            format!(
                "Failed to insert statut for facture_id={}, facture_item_id={}",
                self.facture_id, self.facture_item_id
            )
        })?;

        let db_id = result.last_insert_rowid();

        Ok(Some(db_id))
    }
}
