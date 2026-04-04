use axum::{
    Router,
    extract::{Path, State},
    routing::get,
};
use maud::Markup;
use sqlx::SqlitePool;

use crate::server::{
    routes::errors::AppError,
    services::factures::{select_all, select_one},
    templates::factures,
};

async fn list_factures(State(pool): State<SqlitePool>) -> Result<Markup, AppError> {
    let factures_data = select_all(&pool).await?;
    let rendered = factures::page_factures(factures_data);

    Ok(rendered)
}

async fn facture_items(
    State(pool): State<SqlitePool>,
    Path(facture_id): Path<i64>,
) -> Result<Markup, AppError> {
    let facture_items_data = select_one(&pool, facture_id).await?;
    let rendered = factures::page_facture_items(facture_items_data);

    Ok(rendered)
}

pub fn facture_router() -> Router<SqlitePool> {
    Router::new()
        .route("/factures/{facture_id}/items", get(facture_items))
        .route("/factures", get(list_factures))
}
