use crate::server::{
    database::{insert::Insertable, select::Selectable, update::Updatable},
    models::clients::{ClientForm, ClientInsert, ClientRow},
};
use anyhow::{Context, Result};
use sqlx::SqlitePool;

pub async fn insert_client(pool: &SqlitePool, form: ClientForm) -> Result<i64> {
    let to_insert = ClientInsert::from(form);

    let mut tx = pool.begin().await.context("Failed to begin transaction")?;
    let inserted_id = to_insert.insert_one(&mut tx).await?;
    tx.commit().await.context("Failed to commit transaction")?;

    Ok(inserted_id.expect("An ID should be generated for a new Client"))
}

pub async fn select_one(pool: &SqlitePool, id: i64) -> Result<Option<ClientRow>> {
    Ok(ClientRow::select_one(id, pool).await?)
}

pub async fn select_all(pool: &SqlitePool) -> Result<Vec<ClientRow>> {
    Ok(ClientRow::select_all(pool).await?)
}

pub async fn update_client(pool: &SqlitePool, id: i64, form: ClientForm) -> Result<u64> {
    let mut tx = pool.begin().await.context("Failed to begin transaction")?;
    let maybe_client: Option<ClientRow> = ClientRow::select_one(id, &mut *tx).await?;

    let client =
        maybe_client.ok_or(anyhow::Error::msg(format!("User with id {} not found", id)))?;
    let updated_client = ClientRow {
        first_name: form.first_name,
        last_name: form.last_name,
        street: form.street,
        city: form.city,
        phone1: form.phone1,
        phone2: form.phone2,
        ..client
    };
    let rows_affected = updated_client.update_one(&mut tx).await?;
    tx.commit().await.context("Failed to commit transaction")?;

    Ok(rows_affected)
}
