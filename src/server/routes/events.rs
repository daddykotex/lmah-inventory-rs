use axum::{
    Form, Router,
    extract::{Path, State},
    response::Redirect,
    routing::{get, post},
};
use maud::Markup;
use sqlx::SqlitePool;

use crate::server::{
    models::events::{EventForm, EventView},
    routes::errors::AppError,
    services::{
        config::load_event_types,
        events::{insert_event, select_all, select_one, update_event},
    },
    templates::events,
};

async fn list_events(State(pool): State<SqlitePool>) -> Result<Markup, AppError> {
    let events = select_all(&pool).await?;
    let event_views = events.into_iter().map(EventView::from).collect();
    let rendered = events::page_events(event_views);

    Ok(rendered)
}

async fn new_event(State(pool): State<SqlitePool>) -> Result<Markup, AppError> {
    let event_types = load_event_types(&pool).await?;
    Ok(events::page_new_event(event_types))
}

async fn one_event(
    State(pool): State<SqlitePool>,
    Path(event_id): Path<i64>,
) -> Result<Markup, AppError> {
    let maybe_event = select_one(&pool, event_id).await?;
    let event_types = load_event_types(&pool).await?;
    let event = maybe_event.ok_or(anyhow::Error::msg(format!(
        "event with id {} not found",
        event_id
    )))?;
    let event_view = EventView::from(event);
    Ok(events::page_one_event(event_view, event_types))
}

async fn update_one_event(
    State(pool): State<SqlitePool>,
    Path(event_id): Path<i64>,
    Form(update): Form<EventForm>,
) -> Result<Redirect, AppError> {
    update_event(&pool, event_id, update).await?;
    let url = format!("/events/{}?success=true", event_id);
    Ok(Redirect::to(&url))
}

async fn create_one_event(
    State(pool): State<SqlitePool>,
    Form(create): Form<EventForm>,
) -> Result<Redirect, AppError> {
    let id = insert_event(&pool, create).await?;
    let url = format!("/events/{}?success=true", id);
    Ok(Redirect::to(&url))
}

pub fn event_router() -> Router<SqlitePool> {
    Router::new()
        .route("/events", get(list_events))
        .route("/events/new", get(new_event))
        .route("/events/{event_id}", get(one_event))
        .route("/events/new", post(create_one_event))
        .route("/events/{event_id}/update", post(update_one_event))
}
