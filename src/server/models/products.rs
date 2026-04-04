use sqlx::prelude::FromRow;

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

#[derive(Debug)]
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
