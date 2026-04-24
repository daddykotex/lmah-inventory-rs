use axum::{Router, extract::State, routing::get};
use maud::Markup;
use sqlx::SqlitePool;

use crate::server::{
    routes::{bootstrap::AppState, errors::AppError},
    services::{
        admin::load_all_factures,
        config::{load_event_types, load_extra_large_amount},
    },
    templates::misc::{page_admin, page_help, page_wait},
};
async fn wait() -> Result<Markup, AppError> {
    Ok(page_wait())
}

async fn help(State(db_pool): State<SqlitePool>) -> Result<Markup, AppError> {
    let event_types = load_event_types(&db_pool).await?;
    let extra = load_extra_large_amount(&db_pool).await?;
    Ok(page_help(event_types, extra))
}

async fn admin(State(db_pool): State<SqlitePool>) -> Result<Markup, AppError> {
    let factures = load_all_factures(&db_pool).await?;
    Ok(page_admin(factures))
}

pub fn misc_router() -> Router<AppState> {
    Router::new()
        .route("/wait", get(wait))
        .route("/help", get(help))
        .route("/admin", get(admin))
}
