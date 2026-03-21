/// Database row structure for products table
/// Note: Airtable ID mapping is stored in the airtable_mapping table
#[derive(Debug)]
pub struct ProductRow {
    pub name: String,
    pub price: Option<i64>, // Price in cents
    pub liquidation: bool,
    pub visible_on_site: bool,
    pub created_at: String,
    pub updated_at: String,
}

/// Database row structure for product_images table
#[derive(Debug)]
pub struct ProductImageRow {
    pub product_id: i64,
    pub url: String,
    pub filename: String,
    pub position: String, // "front" or "back"
    pub created_at: String,
}
