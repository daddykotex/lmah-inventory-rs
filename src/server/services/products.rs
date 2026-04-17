use anyhow::{Context, Result};
use sqlx::SqlitePool;

use crate::server::{
    database::insert::Insertable,
    models::products::{ProductForm, ProductInsert},
};

/// Create a new product from form data
pub async fn insert_product(pool: &SqlitePool, form: ProductForm) -> Result<i64> {
    let mut tx = pool.begin().await.context("Failed to begin transaction")?;

    let to_insert = ProductInsert {
        name: form.name,
        price: form.price,
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
