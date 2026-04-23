use anyhow::Context;
use axum::{
    Form, Json, Router,
    extract::{Path, Query, State},
    response::Redirect,
    routing::{get, post},
};
use google_cloud_auth::signer::Signer;
use google_cloud_storage::client::Storage;
use maud::Markup;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use time::OffsetDateTime;

use crate::server::{
    database::select::Selectable,
    models::{
        clients::{ClientForm, ClientView},
        events::{EventForm, EventView},
        facture_items::FactureItemForm,
        factures::{FactureRow, FactureUpdateForm, SelectClientForm, SelectEventForm},
        payments::PaymentForm,
        products::ProductForm,
        refunds::RefundForm,
        statuts::StatusForm,
    },
    routes::{RouterConfig, bootstrap::AppState, errors::AppError, redirect::RedirectOr},
    services::{
        clients,
        config::load_event_types,
        events,
        factures::{
            blank_facture_item, delete_facture_item, insert_facture, insert_facture_item,
            link_event, load_add_product_data, load_print_data, load_products_to_add, select_all,
            select_one, select_one_facture_item, select_transactions, update_facture_item,
        },
        filename::pdf_name_for,
        payments::{delete_payment, insert_payment, update_payment},
        print::{print_to_pdf},
        products,
        refunds::{delete_refund, insert_refund, update_refund},
        statuts::insert_status, storage::bytes_to_storage,
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

async fn create_facture_with_client_handler(
    State(pool): State<SqlitePool>,
    Form(form): Form<SelectClientForm>,
) -> Result<Redirect, AppError> {
    let mut tx = pool.begin().await.context("Failed to begin transaction")?;

    let facture_id = insert_facture(&mut tx, form.selected_client, form.facture_type).await?;

    tx.commit().await.context("Failed to commit transaction")?;

    let url = format!("/factures/{}/select-event?success=true", facture_id);
    Ok(Redirect::to(&url))
}

async fn create_facture_with_new_client_handler(
    State(pool): State<SqlitePool>,
    Query(facture_type_query): Query<FactureTypeQuery>,
    Form(client_form): Form<ClientForm>,
) -> Result<Redirect, AppError> {
    let mut tx = pool.begin().await.context("Failed to begin transaction")?;

    let client_id = clients::insert_client(&mut tx, client_form).await?;

    let facture_type = facture_type_query
        .facture_type
        .unwrap_or_else(|| "Product".to_string());
    let facture_id = insert_facture(&mut tx, client_id, facture_type).await?;

    tx.commit().await.context("Failed to commit transaction")?;

    let url = format!("/factures/{}/select-event?success=true", facture_id);
    Ok(Redirect::to(&url))
}

async fn create_and_link_event_handler(
    State(pool): State<SqlitePool>,
    Path(facture_id): Path<i64>,
    Form(event_form): Form<EventForm>,
) -> Result<Redirect, AppError> {
    let mut tx = pool.begin().await.context("Failed to begin transaction")?;

    // Create the event
    let event_id = events::insert_event(&pool, event_form).await?;

    // Link it to the facture
    link_event(&mut tx, facture_id, event_id).await?;

    tx.commit().await.context("Failed to commit transaction")?;

    // Redirect to items page
    let url = format!("/factures/{}/items?success=true", facture_id);
    Ok(Redirect::to(&url))
}

async fn update_facture_handler(
    State(pool): State<SqlitePool>,
    Path(facture_id): Path<i64>,
    Form(form): Form<FactureUpdateForm>,
) -> Result<Redirect, AppError> {
    crate::server::services::factures::update_facture_details(
        &pool,
        facture_id,
        form.date,
        form.paper_ref,
    )
    .await?;
    let url = format!("/factures/{}/items?success=true", facture_id);
    Ok(Redirect::to(&url))
}

async fn create_product_handler(
    State(pool): State<SqlitePool>,
    Path(facture_id): Path<i64>,
    Form(form): Form<ProductForm>,
) -> Result<Redirect, AppError> {
    let product_id = products::insert_product(&pool, form).await?;
    let url = format!(
        "/factures/{}/add-item/{}?success=true",
        facture_id, product_id
    );
    Ok(Redirect::to(&url))
}

async fn create_facture_item_handler(
    State(pool): State<SqlitePool>,
    Path(facture_id): Path<i64>,
    Form(form): Form<FactureItemForm>,
) -> Result<Redirect, AppError> {
    insert_facture_item(&pool, facture_id, form).await?;
    let url = format!("/factures/{}/items?success=true", facture_id);
    Ok(Redirect::to(&url))
}

async fn update_facture_item_handler(
    State(pool): State<SqlitePool>,
    Path((facture_id, item_id)): Path<(i64, i64)>,
    Form(form): Form<FactureItemForm>,
) -> Result<Redirect, AppError> {
    update_facture_item(&pool, item_id, form).await?;
    let url = format!("/factures/{}/items?success=true", facture_id);
    Ok(Redirect::to(&url))
}

async fn delete_facture_item_handler(
    State(pool): State<SqlitePool>,
    Path((facture_id, item_id)): Path<(i64, i64)>,
) -> Result<Redirect, AppError> {
    delete_facture_item(&pool, item_id).await?;
    let url = format!("/factures/{}/items?success=true", facture_id);
    Ok(Redirect::to(&url))
}

async fn update_item_state_handler(
    State(pool): State<SqlitePool>,
    Path((facture_id, item_id)): Path<(i64, i64)>,
    Form(form): Form<StatusForm>,
) -> Result<Redirect, AppError> {
    insert_status(&pool, facture_id, item_id, form).await?;
    let url = format!("/factures/{}/items/{}?success=true", facture_id, item_id);
    Ok(Redirect::to(&url))
}

async fn create_payment_handler(
    State(pool): State<SqlitePool>,
    Path(facture_id): Path<i64>,
    Form(form): Form<PaymentForm>,
) -> Result<Redirect, AppError> {
    insert_payment(&pool, facture_id, form).await?;
    let url = format!("/factures/{}/transactions?success=true", facture_id);
    Ok(Redirect::to(&url))
}

async fn update_payment_handler(
    State(pool): State<SqlitePool>,
    Path((facture_id, payment_id)): Path<(i64, i64)>,
    Form(form): Form<PaymentForm>,
) -> Result<Redirect, AppError> {
    update_payment(&pool, payment_id, form).await?;
    let url = format!("/factures/{}/transactions?success=true", facture_id);
    Ok(Redirect::to(&url))
}

async fn delete_payment_handler(
    State(pool): State<SqlitePool>,
    Path((facture_id, payment_id)): Path<(i64, i64)>,
) -> Result<Redirect, AppError> {
    delete_payment(&pool, payment_id).await?;
    let url = format!("/factures/{}/transactions?success=true", facture_id);
    Ok(Redirect::to(&url))
}

async fn create_refund_handler(
    State(pool): State<SqlitePool>,
    Path(facture_id): Path<i64>,
    Form(form): Form<RefundForm>,
) -> Result<Redirect, AppError> {
    insert_refund(&pool, facture_id, form).await?;
    let url = format!("/factures/{}/transactions?success=true", facture_id);
    Ok(Redirect::to(&url))
}

async fn update_refund_handler(
    State(pool): State<SqlitePool>,
    Path((facture_id, refund_id)): Path<(i64, i64)>,
    Form(form): Form<RefundForm>,
) -> Result<Redirect, AppError> {
    update_refund(&pool, refund_id, form).await?;
    let url = format!("/factures/{}/transactions?success=true", facture_id);
    Ok(Redirect::to(&url))
}

async fn delete_refund_handler(
    State(pool): State<SqlitePool>,
    Path((facture_id, refund_id)): Path<(i64, i64)>,
) -> Result<Redirect, AppError> {
    delete_refund(&pool, refund_id).await?;
    let url = format!("/factures/{}/transactions?success=true", facture_id);
    Ok(Redirect::to(&url))
}

#[derive(Deserialize)]
struct PrintOptions {
    #[serde(rename = "admin")]
    for_admin: bool,
}
#[derive(Serialize)]
struct PrintResponse {
    url: String,
}
async fn generate_print_handler(
    State(pool): State<SqlitePool>,
    State(http_client): State<Client>,
    State(config): State<RouterConfig>,
    State(storage): State<Storage>,
    State(signer): State<Signer>,
    print_options: Query<PrintOptions>,
    Path(facture_id): Path<i64>,
) -> Result<Json<PrintResponse>, AppError> {
    let page_data = load_print_data(&pool, facture_id).await?;
    let rendered = factures::page_print(page_data);

    let bucket_name = config.google_bucket_name();
    let pdf_rocket_api_key = config.pdf_rocket_api_key();

    let pdf_bytes = print_to_pdf(&http_client, pdf_rocket_api_key, rendered).await?;
    let now = OffsetDateTime::now_utc();
    let file_name = pdf_name_for(facture_id, &now);
    let url = bytes_to_storage(&storage, &signer, bucket_name, &file_name, pdf_bytes, Some("application/pdf"))
        .await
        .context("Uploading to storage failed.")?;

    Ok(Json(PrintResponse { url: url }))
}

pub fn facture_router() -> Router<AppState> {
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
            "/factures/{facture_id}/generate-print",
            post(generate_print_handler),
        )
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
        .route(
            "/factures/new/select-client",
            post(create_facture_with_client_handler),
        )
        .route(
            "/factures/new/new-client",
            post(create_facture_with_new_client_handler),
        )
        .route(
            "/factures/{facture_id}/new-event",
            post(create_and_link_event_handler),
        )
        .route(
            "/factures/{facture_id}/update",
            post(update_facture_handler),
        )
        .route(
            "/factures/{facture_id}/add-product",
            post(create_product_handler),
        )
        // Phase 6: Facture items
        .route(
            "/factures/{facture_id}/items",
            post(create_facture_item_handler),
        )
        .route(
            "/factures/{facture_id}/items/{item_id}/update",
            post(update_facture_item_handler),
        )
        .route(
            "/factures/{facture_id}/items/{item_id}/delete",
            post(delete_facture_item_handler),
        )
        .route(
            "/factures/{facture_id}/items/{item_id}/update-state",
            post(update_item_state_handler),
        )
        // Phase 7: Payments
        .route(
            "/factures/{facture_id}/payments",
            post(create_payment_handler),
        )
        .route(
            "/factures/{facture_id}/payments/{payment_id}/update",
            post(update_payment_handler),
        )
        .route(
            "/factures/{facture_id}/payments/{payment_id}/delete",
            post(delete_payment_handler),
        )
        // Phase 8: Refunds
        .route(
            "/factures/{facture_id}/refunds",
            post(create_refund_handler),
        )
        .route(
            "/factures/{facture_id}/refunds/{refund_id}/update",
            post(update_refund_handler),
        )
        .route(
            "/factures/{facture_id}/refunds/{refund_id}/delete",
            post(delete_refund_handler),
        )
}
