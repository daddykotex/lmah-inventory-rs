use axum::{
    Router,
    extract::{Path, State},
    routing::get,
};
use maud::Markup;
use sqlx::SqlitePool;

use crate::server::{
    routes::errors::AppError,
    services::factures::{select_all, select_one, select_one_facture_item},
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

async fn the_facture_item(
    State(pool): State<SqlitePool>,
    Path((facture_id, facture_item_id)): Path<(i64, i64)>,
) -> Result<Markup, AppError> {
    let facture_item_data = select_one_facture_item(&pool, facture_id, facture_item_id).await?;
    let rendered = factures::page_one_facture_item(facture_item_data);

    Ok(rendered)
}

pub fn facture_router() -> Router<SqlitePool> {
    Router::new()
        .route("/factures/{facture_id}/items", get(facture_items))
        .route(
            "/factures/{facture_id}/items/{facture_item_id}",
            get(the_facture_item),
        )
        .route("/factures", get(list_factures))
}
