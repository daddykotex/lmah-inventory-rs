use serde::Deserialize;
use sqlx::prelude::FromRow;

use crate::server::models::product_types::ProductTypeView;

/// Database row structure for products table
/// Note: Airtable ID mapping is stored in the airtable_mapping table
#[derive(Debug, FromRow)]
pub struct ProductRow {
    pub id: i64,
    pub name: String,
    pub price: Option<i64>, // Price in cents
    pub liquidation: bool,
    pub visible_on_site: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug)]
pub struct ProductInsert {
    pub name: String,
    pub price: Option<i64>, // Price in cents
    pub liquidation: bool,
    pub visible_on_site: bool,
}

/// Database row structure for product_images table
#[derive(Debug, FromRow)]
pub struct ProductImageRow {
    pub id: i64,
    pub product_id: i64,
    pub url: String,
    pub filename: String,
    pub position: String, // "front" or "back"
    pub created_at: String,
}

#[derive(Debug)]
pub struct ProductImageInsert {
    pub product_id: i64,
    pub url: String,
    pub filename: String,
    pub position: String, // "front" or "back"
}

#[derive(Debug, PartialEq)]
pub struct ProductView {
    pub id: i64,
    pub name: String,
    pub price: Option<i64>, // Price in cents
    pub liquidation: bool,
    pub visible_on_site: bool,
    pub created_at: String,
    pub updated_at: String,
}

impl From<ProductRow> for ProductView {
    fn from(value: ProductRow) -> Self {
        ProductView {
            id: value.id,
            name: value.name,
            price: value.price,
            liquidation: value.liquidation,
            visible_on_site: value.visible_on_site,
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct ProductInfo {
    pub product: ProductView,
    pub types: Vec<ProductTypeView>,
}

impl ProductInfo {
    pub fn reduced_price(&self) -> Option<i64> {
        if self.product.liquidation
            && let Some(p) = self.product.price
        {
            let is_wedding = self.types.iter().any(|t| t.is_wedding());
            let rate = if is_wedding { 0.7 } else { 0.5 };
            let res: f64 = (p as f64 * rate).ceil();
            Some(res as i64)
        } else {
            None
        }
    }
}

// Form structure for POST endpoint
#[derive(Deserialize, Debug)]
pub struct ProductForm {
    pub name: String,
    pub price: Option<String>,
    pub liquidation: Option<bool>,
    #[serde(rename = "visible-on-site")]
    pub visible_on_site: Option<bool>,
}
