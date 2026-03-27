use crate::server::{
    database::{insert::Insertable, select::Selectable, update::Updatable},
    models::events::{EventForm, EventInsert, EventRow},
};
use anyhow::{Context, Result};
use sqlx::SqlitePool;

pub async fn insert_event(pool: &SqlitePool, form: EventForm) -> Result<i64> {
    let to_insert = EventInsert::from(form);

    let mut tx = pool.begin().await.context("Failed to begin transaction")?;
    let inserted_id = to_insert.insert_one(&mut tx).await?;
    tx.commit().await.context("Failed to commit transaction")?;

    Ok(inserted_id.expect("An ID should be generated for a new Event"))
}

pub async fn select_one(pool: &SqlitePool, id: i64) -> Result<Option<EventRow>> {
    let mut tx = pool.begin().await.context("Failed to begin transaction")?;
    let res = EventRow::select_one(id, &mut tx).await;
    tx.commit().await.context("Failed to commit transaction")?;
    res
}

pub async fn select_all(pool: &SqlitePool) -> Result<Vec<EventRow>> {
    let mut tx = pool.begin().await.context("Failed to begin transaction")?;
    let res = EventRow::select_all(&mut tx).await;
    tx.commit().await.context("Failed to commit transaction")?;
    res
}

pub async fn update_event(pool: &SqlitePool, id: i64, form: EventForm) -> Result<u64> {
    let mut tx = pool.begin().await.context("Failed to begin transaction")?;
    let maybe_event: Option<EventRow> = EventRow::select_one(id, &mut tx).await?;

    let event = maybe_event.ok_or(anyhow::Error::msg(format!("User with id {} not found", id)))?;
    let updated_event = EventRow {
        name: form.name,
        event_type: form.event_type,
        date: form.date,
        ..event
    };
    let rows_affected = updated_event.update_one(&mut tx).await?;
    tx.commit().await.context("Failed to commit transaction")?;

    Ok(rows_affected)
}
