use axum::{Router, extract::State, routing::get};
use maud::Markup;
use sqlx::SqlitePool;

use crate::server::{
    routes::errors::AppError, services::factures::select_all, templates::factures,
};

async fn list_factures(State(pool): State<SqlitePool>) -> Result<Markup, AppError> {
    let factures_data = select_all(&pool).await?;
    let rendered = factures::page_factures(factures_data);

    Ok(rendered)
}
pub fn facture_router() -> Router<SqlitePool> {
    Router::new().route("/factures", get(list_factures))
}
