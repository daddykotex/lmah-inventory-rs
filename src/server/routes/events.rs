use axum::{
    Form, Router,
    extract::{Path, State},
    response::Redirect,
    routing::{get, post},
};
use maud::Markup;
use toasty::Db;

use crate::server::{
    models::events::{EventForm, EventView},
    routes::errors::AppError,
    services::{
        config::load_event_types,
        events::{insert_event, /* load_one_event, */ select_all, update_event},
    },
    templates::events,
};

async fn list_events(State(mut db): State<Db>) -> Result<Markup, AppError> {
    let events = select_all(&mut db).await?;
    let event_views = events.into_iter().map(EventView::from).collect();
    let rendered = events::page_events(event_views);

    Ok(rendered)
}

async fn new_event(State(mut db): State<Db>) -> Result<Markup, AppError> {
    let event_types = load_event_types(&mut db).await?;
    Ok(events::page_new_event(event_types))
}

async fn one_event(
    State(_db): State<Db>,
    Path(_event_id): Path<u64>,
) -> Result<Markup, AppError> {
    // TODO: Re-enable when Factures and Config are migrated
    // let page_data = load_one_event(&db, event_id).await?;
    // Ok(events::page_one_event(
    //     page_data.event,
    //     page_data.event_types,
    //     page_data.related_factures,
    // ))
    Err(anyhow::Error::msg("Event detail page not yet migrated").into())
}

async fn update_one_event(
    State(mut db): State<Db>,
    Path(event_id): Path<u64>,
    Form(update): Form<EventForm>,
) -> Result<Redirect, AppError> {
    update_event(&mut db, event_id, update).await?;
    let url = format!("/events/{}?success=true", event_id);
    Ok(Redirect::to(&url))
}

async fn create_one_event(
    State(mut db): State<Db>,
    Form(create): Form<EventForm>,
) -> Result<Redirect, AppError> {
    let id = insert_event(&mut db, create).await?;
    let url = format!("/events/{}?success=true", id);
    Ok(Redirect::to(&url))
}

pub fn event_router() -> Router<Db> {
    Router::new()
        .route("/events", get(list_events))
        .route("/events/new", get(new_event))
        .route("/events/{event_id}", get(one_event))
        .route("/events/new", post(create_one_event))
        .route("/events/{event_id}/update", post(update_one_event))
}
