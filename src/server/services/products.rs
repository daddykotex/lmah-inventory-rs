use anyhow::{Context, Result};
use axum::body::Bytes;
use futures_util::stream;
use google_cloud_storage::client::Storage;
use sqlx::SqlitePool;

use crate::server::{
    database::{
        insert::Insertable,
        queries::{replace_product_types, select_type_names_for_product},
        select::Selectable,
        update::Updatable,
    },
    models::{
        product_types::ProductTypeRow,
        products::{
            ProductAdminForm, ProductAdminListEntry, ProductForm, ProductImageInsert,
            ProductImageRow, ProductImageView, ProductInsert, ProductRow, ProductView,
        },
    },
    services::storage::bytes_to_public_storage,
    utils::money::parse_money,
};

/// Create a new product from form data
pub async fn insert_product(pool: &SqlitePool, form: ProductForm) -> Result<i64> {
    let mut tx = pool.begin().await.context("Failed to begin transaction")?;

    let price = form
        .price
        .as_deref()
        .map(parse_money)
        .transpose()
        .map_err(anyhow::Error::msg)?;

    let to_insert = ProductInsert {
        name: form.name,
        price,
        liquidation: form.liquidation.unwrap_or(false),
        visible_on_site: form.visible_on_site.unwrap_or(true),
    };

    let inserted_id = to_insert
        .insert_one(&mut tx)
        .await?
        .expect("An ID should be generated for a new Product");

    tx.commit().await.context("Failed to commit transaction")?;

    Ok(inserted_id)
}

fn parse_optional_price(raw: Option<&str>) -> Result<Option<i64>> {
    match raw.map(str::trim).filter(|s| !s.is_empty()) {
        Some(s) => parse_money(s).map(Some).map_err(anyhow::Error::msg),
        None => Ok(None),
    }
}

/// Create a product from the admin form (with types + visibility).
pub async fn create_admin_product(pool: &SqlitePool, form: ProductAdminForm) -> Result<i64> {
    let price = parse_optional_price(form.price.as_deref())?;
    let mut tx = pool.begin().await.context("Failed to begin transaction")?;

    let to_insert = ProductInsert {
        name: form.name.clone(),
        price,
        liquidation: form.is_liquidation(),
        visible_on_site: form.is_visible(),
    };

    let inserted_id = to_insert
        .insert_one(&mut tx)
        .await?
        .expect("An ID should be generated for a new Product");

    replace_product_types(inserted_id, &form.types, &mut tx).await?;

    tx.commit().await.context("Failed to commit transaction")?;

    Ok(inserted_id)
}

/// Update a product from the admin form.
pub async fn update_admin_product(
    pool: &SqlitePool,
    id: i64,
    form: ProductAdminForm,
) -> Result<u64> {
    let price = parse_optional_price(form.price.as_deref())?;
    let mut tx = pool.begin().await.context("Failed to begin transaction")?;

    let existing = ProductRow::select_one(id, &mut *tx)
        .await?
        .ok_or_else(|| anyhow::anyhow!("Product with id {id} not found"))?;

    let updated = ProductRow {
        name: form.name.clone(),
        price,
        liquidation: form.is_liquidation(),
        visible_on_site: form.is_visible(),
        ..existing
    };

    let rows_affected = updated.update_one(&mut tx).await?;
    replace_product_types(id, &form.types, &mut tx).await?;

    tx.commit().await.context("Failed to commit transaction")?;

    Ok(rows_affected)
}

/// List all products for the admin index (name, visibility, types).
pub async fn list_admin_products(pool: &SqlitePool) -> Result<Vec<ProductAdminListEntry>> {
    let mut tx = pool.begin().await.context("Failed to begin transaction")?;
    let rows = ProductRow::select_all(&mut *tx).await?;

    let type_pairs: Vec<(i64, String)> = sqlx::query_as(
        "SELECT product_id, product_type_name FROM product_product_types ORDER BY product_id",
    )
    .fetch_all(&mut *tx)
    .await
    .context("Failed to load product-type associations")?;

    tx.commit().await.context("Failed to commit transaction")?;

    let mut entries = Vec::with_capacity(rows.len());
    for row in rows {
        let types = type_pairs
            .iter()
            .filter(|(pid, _)| *pid == row.id)
            .map(|(_, n)| n.clone())
            .collect();
        entries.push(ProductAdminListEntry {
            id: row.id,
            name: row.name,
            visible_on_site: row.visible_on_site,
            types,
        });
    }
    Ok(entries)
}

pub struct ProductEdit {
    pub product: ProductView,
    pub selected_types: Vec<String>,
    pub all_types: Vec<String>,
}

pub async fn load_product_for_edit(pool: &SqlitePool, id: i64) -> Result<ProductEdit> {
    let mut tx = pool.begin().await.context("Failed to begin transaction")?;

    let row = ProductRow::select_one(id, &mut *tx)
        .await?
        .ok_or_else(|| anyhow::anyhow!("Product with id {id} not found"))?;

    let selected_types = select_type_names_for_product(id, &mut *tx).await?;
    let all_types: Vec<String> = ProductTypeRow::select_all(&mut *tx)
        .await?
        .into_iter()
        .map(|t| t.name)
        .collect();

    tx.commit().await.context("Failed to commit transaction")?;

    Ok(ProductEdit {
        product: row.into(),
        selected_types,
        all_types,
    })
}

pub async fn load_all_product_type_names(pool: &SqlitePool) -> Result<Vec<String>> {
    Ok(ProductTypeRow::select_all(pool)
        .await?
        .into_iter()
        .map(|t| t.name)
        .collect())
}

pub struct ProductImagesPage {
    pub product: ProductView,
    pub images: Vec<ProductImageView>,
}

pub async fn load_product_images_page(pool: &SqlitePool, id: i64) -> Result<ProductImagesPage> {
    let mut tx = pool.begin().await.context("Failed to begin transaction")?;
    let row = ProductRow::select_one(id, &mut *tx)
        .await?
        .ok_or_else(|| anyhow::anyhow!("Product with id {id} not found"))?;
    let images = ProductImageRow::select_all_for_product(id, &mut *tx).await?;
    tx.commit().await.context("Failed to commit transaction")?;

    Ok(ProductImagesPage {
        product: row.into(),
        images: images.into_iter().map(ProductImageView::from).collect(),
    })
}

/// Upload a single image at `position` for a product. Existing image at the
/// same position (if any) is deleted from the DB.
pub async fn upload_product_image(
    pool: &SqlitePool,
    storage: &Storage,
    bucket_name: &str,
    product_id: i64,
    position: &str,
    filename: &str,
    content_type: Option<&str>,
    bytes: Bytes,
) -> Result<()> {
    let object_name = format!("products/{product_id}/{position}-{filename}");
    let byte_stream = stream::once(async move { Ok::<_, reqwest::Error>(bytes) });
    let url = Box::pin(bytes_to_public_storage(
        storage,
        bucket_name,
        &object_name,
        byte_stream,
        content_type,
    ))
    .await
    .context("Failed to upload product image to storage")?;

    let mut tx = pool.begin().await.context("Failed to begin transaction")?;
    ProductImageRow::delete_for_product_position(product_id, position, &mut tx).await?;

    let to_insert = ProductImageInsert {
        product_id,
        url,
        filename: object_name,
        position: position.to_string(),
    };
    to_insert.insert_one(&mut tx).await?;
    tx.commit().await.context("Failed to commit transaction")?;

    Ok(())
}
