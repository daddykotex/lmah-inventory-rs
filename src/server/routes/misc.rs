use axum::{Router, extract::State, routing::get};
use maud::Markup;
use sqlx::SqlitePool;

use crate::server::{
    routes::{bootstrap::AppState, errors::AppError},
    services::config::{load_event_types, load_extra_large_amount},
    templates::misc::page_help,
};

async fn help(State(db_pool): State<SqlitePool>) -> Result<Markup, AppError> {
    let event_types = load_event_types(&db_pool).await?;
    let extra = load_extra_large_amount(&db_pool).await?;
    Ok(page_help(event_types, extra))
}

pub fn misc_router() -> Router<AppState> {
    Router::new().route("/help", get(help))
}
