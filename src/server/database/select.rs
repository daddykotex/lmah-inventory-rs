use anyhow::{Context, Result};
use sqlx::{Executor, Sqlite};

use crate::server::{
    database::has_table::{HasTable, TableName},
    models::{
        clients::ClientRow, config::ConfigRow, events::EventRow, facture_items::FactureItemRow,
        factures::FactureRow, products::ProductRow, statuts::StatutRow,
    },
};

pub trait Selectable<T> {
    fn select_one<'c, E>(id: i64, e: E) -> impl Future<Output = Result<Option<T>>>
    where
        E: Executor<'c, Database = Sqlite>;

    fn select_all<'c, E>(e: E) -> impl Future<Output = Result<Vec<T>>>
    where
        E: Executor<'c, Database = Sqlite>;
}

impl Selectable<ConfigRow> for ConfigRow {
    async fn select_one<'c, E>(id: i64, e: E) -> Result<Option<ConfigRow>>
    where
        E: Executor<'c, Database = Sqlite>,
    {
        let table = ConfigRow::table();
        let result: Option<ConfigRow> = sqlx::query_as(&format!(
            "SELECT * FROM {} WHERE id = ?",
            table.table_name()
        ))
        .bind(id)
        .fetch_optional(e)
        .await
        .context(format!("Failed to retrieve one config with id {}", id))?;

        Ok(result)
    }

    async fn select_all<'c, E>(e: E) -> Result<Vec<ConfigRow>>
    where
        E: Executor<'c, Database = Sqlite>,
    {
        let table = ConfigRow::table();
        let result: Vec<ConfigRow> =
            sqlx::query_as(&format!("SELECT * FROM {}", table.table_name()))
                .fetch_all(e)
                .await
                .context("Failed to retrieve clients")?;

        Ok(result)
    }
}

impl Selectable<ClientRow> for ClientRow {
    async fn select_one<'c, E>(id: i64, e: E) -> Result<Option<ClientRow>>
    where
        E: Executor<'c, Database = Sqlite>,
    {
        let table = ClientRow::table();
        let result: Option<ClientRow> = sqlx::query_as(&format!(
            "SELECT * FROM {} WHERE id = ?",
            table.table_name()
        ))
        .bind(id)
        .fetch_optional(e)
        .await
        .context(format!("Failed to retrieve one client with id {}", id))?;

        Ok(result)
    }

    async fn select_all<'c, E>(e: E) -> Result<Vec<ClientRow>>
    where
        E: Executor<'c, Database = Sqlite>,
    {
        let table = ClientRow::table();
        let result: Vec<ClientRow> =
            sqlx::query_as(&format!("SELECT * FROM {}", table.table_name()))
                .fetch_all(e)
                .await
                .context("Failed to retrieve clients")?;

        Ok(result)
    }
}

impl Selectable<EventRow> for EventRow {
    async fn select_one<'c, E>(id: i64, e: E) -> Result<Option<EventRow>>
    where
        E: Executor<'c, Database = Sqlite>,
    {
        let table = EventRow::table();
        let result: Option<EventRow> = sqlx::query_as(&format!(
            "SELECT * FROM {} WHERE id = ?",
            table.table_name()
        ))
        .bind(id)
        .fetch_optional(e)
        .await
        .context(format!("Failed to retrieve one client with id {}", id))?;

        Ok(result)
    }

    async fn select_all<'c, E>(e: E) -> Result<Vec<EventRow>>
    where
        E: Executor<'c, Database = Sqlite>,
    {
        let table = EventRow::table();
        let result: Vec<EventRow> =
            sqlx::query_as(&format!("SELECT * FROM {}", table.table_name()))
                .fetch_all(e)
                .await
                .context("Failed to retrieve events")?;

        Ok(result)
    }
}

impl Selectable<FactureRow> for FactureRow {
    async fn select_one<'c, E>(id: i64, e: E) -> Result<Option<FactureRow>>
    where
        E: Executor<'c, Database = Sqlite>,
    {
        let table = FactureRow::table();
        let result: Option<FactureRow> = sqlx::query_as(&format!(
            "SELECT * FROM {} WHERE id = ?",
            table.table_name()
        ))
        .bind(id)
        .fetch_optional(e)
        .await
        .context(format!("Failed to retrieve one facture with id {}", id))?;

        Ok(result)
    }

    async fn select_all<'c, E>(e: E) -> Result<Vec<FactureRow>>
    where
        E: Executor<'c, Database = Sqlite>,
    {
        let table = FactureRow::table();
        let result: Vec<FactureRow> =
            sqlx::query_as(&format!("SELECT * FROM {}", table.table_name()))
                .fetch_all(e)
                .await
                .context("Failed to retrieve factures")?;

        Ok(result)
    }
}

impl Selectable<FactureItemRow> for FactureItemRow {
    async fn select_one<'c, E>(id: i64, e: E) -> Result<Option<FactureItemRow>>
    where
        E: Executor<'c, Database = Sqlite>,
    {
        let table = FactureItemRow::table();
        let result: Option<FactureItemRow> = sqlx::query_as(&format!(
            "SELECT * FROM {} WHERE id = ?",
            table.table_name()
        ))
        .bind(id)
        .fetch_optional(e)
        .await
        .context(format!(
            "Failed to retrieve one facture item with id {}",
            id
        ))?;

        Ok(result)
    }

    async fn select_all<'c, E>(e: E) -> Result<Vec<FactureItemRow>>
    where
        E: Executor<'c, Database = Sqlite>,
    {
        let table = FactureItemRow::table();
        let result: Vec<FactureItemRow> = sqlx::query_as(&format!(
            "SELECT * FROM {} ORDER BY facture_id DESC, id DESC",
            table.table_name()
        ))
        .fetch_all(e)
        .await
        .context("Failed to retrieve facture items")?;

        Ok(result)
    }
}

impl Selectable<StatutRow> for StatutRow {
    async fn select_one<'c, E>(id: i64, e: E) -> Result<Option<StatutRow>>
    where
        E: Executor<'c, Database = Sqlite>,
    {
        let table = StatutRow::table();
        let result: Option<StatutRow> = sqlx::query_as(&format!(
            "SELECT * FROM {} WHERE id = ?",
            table.table_name()
        ))
        .bind(id)
        .fetch_optional(e)
        .await
        .context(format!("Failed to retrieve one statut with id {}", id))?;

        Ok(result)
    }

    async fn select_all<'c, E>(e: E) -> Result<Vec<StatutRow>>
    where
        E: Executor<'c, Database = Sqlite>,
    {
        let table = StatutRow::table();
        let result: Vec<StatutRow> = sqlx::query_as(&format!(
            "SELECT * FROM {} ORDER BY id ASC",
            table.table_name()
        ))
        .fetch_all(e)
        .await
        .context("Failed to retrieve statuts")?;

        Ok(result)
    }
}

impl Selectable<ProductRow> for ProductRow {
    async fn select_one<'c, E>(id: i64, e: E) -> Result<Option<ProductRow>>
    where
        E: Executor<'c, Database = Sqlite>,
    {
        let table = ProductRow::table();
        let result: Option<ProductRow> = sqlx::query_as(&format!(
            "SELECT * FROM {} WHERE id = ?",
            table.table_name()
        ))
        .bind(id)
        .fetch_optional(e)
        .await
        .context(format!("Failed to retrieve one product with id {}", id))?;

        Ok(result)
    }

    async fn select_all<'c, E>(e: E) -> Result<Vec<ProductRow>>
    where
        E: Executor<'c, Database = Sqlite>,
    {
        let table = ProductRow::table();
        let result: Vec<ProductRow> = sqlx::query_as(&format!(
            "SELECT * FROM {} ORDER BY facture_id DESC, facture_item_id DESC",
            table.table_name()
        ))
        .fetch_all(e)
        .await
        .context("Failed to retrieve product row")?;

        Ok(result)
    }
}
