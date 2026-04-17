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
    routes::{bootstrap::AppState, errors::AppError},
    services::{
        config::load_event_types,
        events::{insert_event, load_one_event, select_all, update_event},
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
    let page_data = load_one_event(&pool, event_id).await?;
    Ok(events::page_one_event(
        page_data.event,
        page_data.event_types,
        page_data.related_factures,
    ))
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

pub fn event_router() -> Router<AppState> {
    Router::new()
        .route("/events", get(list_events))
        .route("/events/new", get(new_event))
        .route("/events/{event_id}", get(one_event))
        .route("/events/new", post(create_one_event))
        .route("/events/{event_id}/update", post(update_one_event))
}
