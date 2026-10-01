use anyhow::Context;
use axum::{
    Router,
    extract::{Multipart, Path, State},
    response::Redirect,
    routing::get,
};
use axum_extra::extract::Form;
use google_cloud_storage::client::Storage;
use maud::Markup;
use sqlx::SqlitePool;

use serde::Deserialize;

use crate::server::{
    models::products::ProductAdminForm,
    routes::{RouterConfig, bootstrap::AppState, errors::AppError},
    services::products::{
        create_admin_product, list_admin_products, load_all_product_type_names,
        load_ordering_cards, load_product_for_edit, load_product_images_page,
        ordering_table_for_slug, save_ordering, update_admin_product, upload_product_image,
    },
    templates::products,
};

#[derive(Deserialize, Debug)]
struct OrderingForm {
    #[serde(default)]
    ids: Vec<i64>,
}

async fn ordering_page(
    State(pool): State<SqlitePool>,
    Path(slug): Path<String>,
) -> Result<Markup, AppError> {
    let (table, url_slug, label) = ordering_table_for_slug(&slug)
        .ok_or_else(|| anyhow::anyhow!("Unknown ordering table: {slug}"))?;
    let cards = load_ordering_cards(&pool, table).await?;
    Ok(products::page_admin_product_ordering(
        url_slug, label, cards,
    ))
}

async fn save_ordering_handler(
    State(pool): State<SqlitePool>,
    Path(slug): Path<String>,
    Form(form): Form<OrderingForm>,
) -> Result<Redirect, AppError> {
    let (table, url_slug, _) = ordering_table_for_slug(&slug)
        .ok_or_else(|| anyhow::anyhow!("Unknown ordering table: {slug}"))?;
    save_ordering(&pool, table, &form.ids).await?;
    Ok(Redirect::to(&format!(
        "/admin/products/order/{url_slug}?success=true"
    )))
}

async fn list_products(State(pool): State<SqlitePool>) -> Result<Markup, AppError> {
    let entries = list_admin_products(&pool).await?;
    Ok(products::page_admin_products_list(entries))
}

async fn new_product_form(State(pool): State<SqlitePool>) -> Result<Markup, AppError> {
    let all_types = load_all_product_type_names(&pool).await?;
    Ok(products::page_admin_product_new(all_types))
}

async fn create_product(
    State(pool): State<SqlitePool>,
    Form(form): Form<ProductAdminForm>,
) -> Result<Redirect, AppError> {
    let id = create_admin_product(&pool, form).await?;
    Ok(Redirect::to(&format!("/admin/products/{id}?success=true")))
}

async fn edit_product_form(
    State(pool): State<SqlitePool>,
    Path(id): Path<i64>,
) -> Result<Markup, AppError> {
    let data = load_product_for_edit(&pool, id).await?;
    Ok(products::page_admin_product_edit(
        data.product,
        data.selected_types,
        data.all_types,
    ))
}

async fn update_product(
    State(pool): State<SqlitePool>,
    Path(id): Path<i64>,
    Form(form): Form<ProductAdminForm>,
) -> Result<Redirect, AppError> {
    update_admin_product(&pool, id, form).await?;
    Ok(Redirect::to(&format!("/admin/products/{id}?success=true")))
}

async fn images_page(
    State(pool): State<SqlitePool>,
    Path(id): Path<i64>,
) -> Result<Markup, AppError> {
    let data = load_product_images_page(&pool, id).await?;
    Ok(products::page_admin_product_images(data))
}

async fn upload_images(
    State(pool): State<SqlitePool>,
    State(storage): State<Storage>,
    State(config): State<RouterConfig>,
    Path(id): Path<i64>,
    mut multipart: Multipart,
) -> Result<Redirect, AppError> {
    let mut position: Option<String> = None;
    let mut file_bytes: Option<axum::body::Bytes> = None;
    let mut file_name: Option<String> = None;
    let mut content_type: Option<String> = None;

    while let Some(field) = multipart
        .next_field()
        .await
        .context("Failed to read multipart field")?
    {
        match field.name() {
            Some("position") => {
                position = Some(field.text().await.context("Failed to read position")?);
            }
            Some("file") => {
                file_name = field.file_name().map(str::to_string);
                content_type = field.content_type().map(str::to_string);
                let bytes = field
                    .bytes()
                    .await
                    .context("Failed to read uploaded file bytes")?;
                file_bytes = Some(bytes);
            }
            _ => {}
        }
    }

    let position = position.ok_or_else(|| anyhow::anyhow!("Missing position field"))?;
    if position != "front" && position != "back" {
        return Err(AppError::from(anyhow::anyhow!(
            "Invalid position: {position}"
        )));
    }
    let bytes = file_bytes.ok_or_else(|| anyhow::anyhow!("Missing file"))?;
    if bytes.is_empty() {
        return Err(AppError::from(anyhow::anyhow!("Uploaded file is empty")));
    }
    let filename = file_name.unwrap_or_else(|| format!("{position}.bin"));

    let bucket_name = config.google_public_bucket_name();
    upload_product_image(
        &pool,
        &storage,
        &bucket_name,
        id,
        &position,
        &filename,
        content_type.as_deref(),
        bytes,
    )
    .await?;

    Ok(Redirect::to(&format!(
        "/admin/products/{id}/images?success=true"
    )))
}

pub fn product_router() -> Router<AppState> {
    Router::new()
        .route("/admin/products", get(list_products))
        .route(
            "/admin/products/add-product",
            get(new_product_form).post(create_product),
        )
        .route(
            "/admin/products/order/{slug}",
            get(ordering_page).post(save_ordering_handler),
        )
        .route(
            "/admin/products/{id}",
            get(edit_product_form).post(update_product),
        )
        .route(
            "/admin/products/{id}/images",
            get(images_page).post(upload_images),
        )
}
