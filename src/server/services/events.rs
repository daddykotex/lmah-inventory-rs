use crate::server::{
    models::{
        FactureAndClient, PageOneEvent,
        events::{Event, EventForm, EventView},
    },
    services::config::load_event_types,
};
use anyhow::{Context, Result};
use toasty::Db;

pub async fn insert_event(db: &mut Db, form: EventForm) -> Result<u64> {
    let mut tx = db
        .transaction()
        .await
        .context("Failed to begin transaction")?;
    let event = toasty::create!(Event {
        name: form.name,
        event_type: form.event_type,
        date: form.date,
    })
    .exec(&mut tx)
    .await?;
    tx.commit().await.context("Failed to commit transaction")?;
    let event_view: EventView = event.into();

    Ok(event_view.id)
}

pub async fn select_one(db: &mut Db, id: u64) -> Result<Option<EventView>> {
    let event = Event::filter_by_id(id).first().exec(db).await?;
    Ok(event.map(|e| e.into()))
}

pub async fn load_one_event(db: &mut Db, event_id: u64) -> Result<PageOneEvent> {
    // Get the event
    let event = Event::filter_by_id(event_id)
        .include(Event::fields().factures())
        .include(Event::fields().factures().client())
        .first()
        .exec(db)
        .await?
        .ok_or(anyhow::Error::msg(format!(
            "event with id {} not found",
            event_id,
        )))?;

    let event_types = load_event_types(db).await?;

    let factures_with_clients = event
        .factures
        .get()
        .iter()
        .map(|f| FactureAndClient {
            facture: f.into(),
            client: f.client.get().into(),
        })
        .collect();

    Ok(PageOneEvent {
        event: EventView::from(event),
        event_types,
        related_factures: factures_with_clients,
    })
}

pub async fn select_all(db: &mut Db) -> Result<Vec<Event>> {
    let events = Event::all().exec(db).await?;
    Ok(events)
}

pub async fn update_event(db: &mut Db, id: u64, form: EventForm) -> Result<u64> {
    let mut tx = db
        .transaction()
        .await
        .context("Failed to begin transaction")?;
    Event::filter_by_id(id)
        .update()
        .name(form.name)
        .event_type(form.event_type)
        .date(form.date)
        .exec(&mut tx)
        .await?;
    tx.commit().await.context("Failed to commit transaction")?;

    Ok(1) // TODO: Get actual rows affected
}
