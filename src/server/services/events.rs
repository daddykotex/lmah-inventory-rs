use crate::server::{
    models::{
        // FactureAndClient, PageOneEvent,
        // clients::Client,
        events::{Event, EventForm, EventView},
        // factures::Facture,
    },
    // services::config::load_event_types,
};
use anyhow::{Context, Result};
use toasty::Db;

pub async fn insert_event(db: &mut Db, form: EventForm) -> Result<u64> {
    let mut tx = db.transaction().await.context("Failed to begin transaction")?;
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
    let event = Event::filter_by_id(id)
        .first()
        .exec(db)
        .await?;
    Ok(event.map(|e|e.into()))
}

// TODO: Re-enable this when Factures is migrated
// pub async fn load_one_event(db: &Db, event_id: u64) -> Result<PageOneEvent> {
//     let mut tx = db.transaction().await.context("Failed to begin transaction")?;

//     let event = Event::filter_by_id(event_id)
//         .get(&db)
//         .await?
//         .ok_or(anyhow::Error::msg(format!(
//             "event with id {} not found",
//             event_id,
//         )))?;
//     let event_types = load_event_types(&mut *tx).await?;
//     let related_factures = Facture::select_for_event(event_id, &mut *tx).await?;
//     let mut related_clients = Client::select_for_facture_event(event_id, &mut *tx).await?;

//     let related_factures: Result<Vec<FactureAndClient>> = related_factures
//         .into_iter()
//         .map(|f| {
//             let idx = related_clients.iter().position(|c| c.id == f.client_id);
//             idx.ok_or(anyhow::Error::msg("Unable to find client for facture"))
//                 .map(|i| {
//                     let client = related_clients.swap_remove(i);
//                     FactureAndClient {
//                         facture: f.into(),
//                         client: client.into(),
//                     }
//                 })
//         })
//         .collect();

//     Ok(PageOneEvent {
//         event: event.into(),
//         event_types,
//         related_factures: related_factures?,
//     })
// }

pub async fn select_all(db: &mut Db) -> Result<Vec<Event>> {
    let events = Event::all()
        .exec(db)
        .await?;
    Ok(events)
}

pub async fn update_event(db: &mut Db, id: u64, form: EventForm) -> Result<u64> {
    let mut tx = db.transaction().await.context("Failed to begin transaction")?;
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
