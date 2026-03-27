use anyhow::{Context, Result};

use crate::server::{
    database::has_table::{HasTable, TableName},
    models::{clients::ClientRow, events::EventRow},
};

pub trait Selectable<T> {
    fn select_one(
        id: i64,
        tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    ) -> impl Future<Output = Result<Option<T>>>;

    fn select_all(
        tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    ) -> impl Future<Output = Result<Vec<T>>>;
}

impl Selectable<ClientRow> for ClientRow {
    async fn select_one(
        id: i64,
        tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    ) -> Result<Option<ClientRow>> {
        let table = ClientRow::table();
        let result: Option<ClientRow> = sqlx::query_as(&format!(
            "SELECT * FROM {} WHERE id = ?",
            table.table_name()
        ))
        .bind(id)
        .fetch_optional(&mut **tx)
        .await
        .context(format!("Failed to retrieve one client with id {}", id))?;

        Ok(result)
    }

    async fn select_all(tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>) -> Result<Vec<ClientRow>> {
        let table = ClientRow::table();
        let result: Vec<ClientRow> =
            sqlx::query_as(&format!("SELECT * FROM {}", table.table_name()))
                .fetch_all(&mut **tx)
                .await
                .context("Failed to retrieve clients")?;

        Ok(result)
    }
}

impl Selectable<EventRow> for EventRow {
    async fn select_one(
        id: i64,
        tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    ) -> Result<Option<EventRow>> {
        let table = EventRow::table();
        let result: Option<EventRow> = sqlx::query_as(&format!(
            "SELECT * FROM {} WHERE id = ?",
            table.table_name()
        ))
        .bind(id)
        .fetch_optional(&mut **tx)
        .await
        .context(format!("Failed to retrieve one client with id {}", id))?;

        Ok(result)
    }

    async fn select_all(tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>) -> Result<Vec<EventRow>> {
        let table = EventRow::table();
        let result: Vec<EventRow> =
            sqlx::query_as(&format!("SELECT * FROM {}", table.table_name()))
                .fetch_all(&mut **tx)
                .await
                .context("Failed to retrieve events")?;

        Ok(result)
    }
}
