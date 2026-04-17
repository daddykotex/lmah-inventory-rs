use anyhow::Context;
use axum::{
    Form, Router,
    extract::{Path, Query, State},
    response::Redirect,
    routing::{get, post},
};
use maud::Markup;
use serde::Deserialize;
use sqlx::SqlitePool;

use crate::server::{
    database::select::Selectable,
    models::{
        clients::ClientView,
        events::EventView,
        factures::{FactureRow, SelectEventForm},
    },
    routes::{errors::AppError, redirect::RedirectOr},
    services::{
        clients,
        config::load_event_types,
        events,
        factures::{
            blank_facture_item, link_event, load_add_product_data, load_print_data,
            load_products_to_add, select_all, select_one, select_one_facture_item,
            select_transactions,
        },
    },
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

#[derive(Deserialize)]
struct FactureTypeQuery {
    #[serde(rename = "facture-type")]
    facture_type: Option<String>,
}

async fn new_facture_new_client(
    State(pool): State<SqlitePool>,
    facture_type: Query<FactureTypeQuery>,
) -> Result<Markup, AppError> {
    let clients = clients::select_all(&pool).await?;
    let clients = clients.into_iter().map(ClientView::from).collect();
    let rendered = factures::page_new_facture_new_client(
        facture_type.facture_type.as_ref().map(|a| a.as_str()),
        clients,
    );
    Ok(rendered)
}

async fn new_facture_the_client(
    State(pool): State<SqlitePool>,
    facture_type: Query<FactureTypeQuery>,
) -> Result<Markup, AppError> {
    let clients = clients::select_all(&pool).await?;
    let clients = clients.into_iter().map(ClientView::from).collect();
    let rendered = factures::page_new_facture_the_client(
        facture_type.facture_type.as_ref().map(|a| a.as_str()),
        clients,
    );
    Ok(rendered)
}

async fn new_facture_the_event(
    State(pool): State<SqlitePool>,
    Path(facture_id): Path<i64>,
) -> Result<Markup, AppError> {
    select_one(&pool, facture_id).await?; // ensure the facture exists

    let events = events::select_all(&pool).await?;
    let events = events.into_iter().map(EventView::from).collect();
    let rendered = factures::page_new_facture_the_event(facture_id, events);
    Ok(rendered)
}

async fn new_facture_new_event(
    State(pool): State<SqlitePool>,
    Path(facture_id): Path<i64>,
) -> Result<Markup, AppError> {
    select_one(&pool, facture_id).await?; // ensure the facture exists
    let event_types = load_event_types(&pool).await?;
    let rendered = factures::page_new_facture_new_event(facture_id, event_types);
    Ok(rendered)
}

async fn select_item(
    State(pool): State<SqlitePool>,
    Path(facture_id): Path<i64>,
) -> Result<RedirectOr<Markup>, AppError> {
    let result = load_products_to_add(&pool, facture_id).await?;
    match result {
        Ok(products) => {
            let rendered = factures::page_select_item(facture_id, products);
            Ok(RedirectOr::Response(rendered))
        }
        Err(product) => {
            let url = format!("/factures/{}/add-item/{}", facture_id, product.id);
            Ok(RedirectOr::Redirect(url))
        }
    }
}

async fn add_item(
    State(pool): State<SqlitePool>,
    Path((facture_id, product_id)): Path<(i64, i64)>,
) -> Result<Markup, AppError> {
    let facture_item_data = blank_facture_item(&pool, facture_id, product_id).await?;
    let rendered = factures::page_add_item(facture_item_data);

    Ok(rendered)
}

async fn transactions(
    State(pool): State<SqlitePool>,
    Path(facture_id): Path<i64>,
) -> Result<Markup, AppError> {
    let transactions_data = select_transactions(&pool, facture_id).await?;
    let rendered = factures::page_transactions(transactions_data);

    Ok(rendered)
}

async fn add_product(
    State(pool): State<SqlitePool>,
    Path(facture_id): Path<i64>,
) -> Result<Markup, AppError> {
    let page_data = load_add_product_data(&pool, facture_id).await?;
    let rendered = factures::page_add_product(page_data);
    Ok(rendered)
}

async fn print(
    State(pool): State<SqlitePool>,
    Path(facture_id): Path<i64>,
) -> Result<Markup, AppError> {
    let page_data = load_print_data(&pool, facture_id).await?;
    let rendered = factures::page_print(page_data);
    Ok(rendered)
}

async fn cancel_facture_handler(
    State(pool): State<SqlitePool>,
    Path(facture_id): Path<i64>,
) -> Result<Redirect, AppError> {
    crate::server::services::factures::cancel_facture(&pool, facture_id).await?;
    let url = format!("/factures/{}/items?success=true", facture_id);
    Ok(Redirect::to(&url))
}

async fn uncancel_facture_handler(
    State(pool): State<SqlitePool>,
    Path(facture_id): Path<i64>,
) -> Result<Redirect, AppError> {
    crate::server::services::factures::uncancel_facture(&pool, facture_id).await?;
    let url = format!("/factures/{}/items?success=true", facture_id);
    Ok(Redirect::to(&url))
}

async fn unlink_event_handler(
    State(pool): State<SqlitePool>,
    Path(facture_id): Path<i64>,
) -> Result<Redirect, AppError> {
    let facture_id = facture_id;
    let mut tx = pool.begin().await.context("Failed to begin transaction")?;

    let maybe_facture = FactureRow::select_one(facture_id, &mut *tx).await?;
    maybe_facture.ok_or(anyhow::Error::msg(format!(
        "Facture with id {} not found",
        facture_id
    )))?;

    sqlx::query("UPDATE factures SET event_id = NULL, updated_at = datetime('now') WHERE id = ?")
        .bind(facture_id)
        .execute(&mut *tx)
        .await
        .with_context(|| format!("Failed to unlink event from facture {}", facture_id))?;
    tx.commit().await.context("Failed to commit transaction")?;

    let url = format!("/factures/{}/items?success=true", facture_id);
    Ok(Redirect::to(&url))
}

async fn link_event_handler(
    State(pool): State<SqlitePool>,
    Path(facture_id): Path<i64>,
    Form(form): Form<SelectEventForm>,
) -> Result<Redirect, AppError> {
    let mut tx = pool.begin().await.context("Failed to begin transaction")?;

    link_event(&mut tx, facture_id, form.selected_event).await?;

    let maybe_facture = FactureRow::select_one(facture_id, &mut *tx).await?;
    maybe_facture.ok_or(anyhow::Error::msg(format!(
        "Facture with id {} not found",
        facture_id
    )))?;

    tx.commit().await.context("Failed to commit transaction")?;

    let url = format!("/factures/{}/items", facture_id);
    Ok(Redirect::to(&url))
}

pub fn facture_router() -> Router<SqlitePool> {
    Router::new()
        // GET routes
        .route("/factures/new", get(new_facture_the_client))
        .route("/factures/new/new-client", get(new_facture_new_client))
        .route(
            "/factures/{facture_id}/new-event",
            get(new_facture_new_event),
        )
        .route(
            "/factures/{facture_id}/select-event",
            get(new_facture_the_event),
        )
        .route("/factures/{facture_id}/add-item", get(select_item))
        .route(
            "/factures/{facture_id}/add-item/{product_id}",
            get(add_item),
        )
        .route("/factures/{facture_id}/add-product", get(add_product))
        .route("/factures/{facture_id}/print", get(print))
        .route("/factures/{facture_id}/items", get(facture_items))
        .route("/factures/{facture_id}/transactions", get(transactions))
        .route(
            "/factures/{facture_id}/items/{facture_item_id}",
            get(the_facture_item),
        )
        .route("/factures", get(list_factures))
        // POST routes
        .route(
            "/factures/{facture_id}/cancel",
            post(cancel_facture_handler),
        )
        .route(
            "/factures/{facture_id}/uncancel",
            post(uncancel_facture_handler),
        )
        .route(
            "/factures/{facture_id}/unlink-event",
            post(unlink_event_handler),
        )
        .route(
            "/factures/{facture_id}/select-event",
            post(link_event_handler),
        )
}
