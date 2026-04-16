use crate::server::models::product_types::ProductTypeView;

/// Product model with Toasty ORM
#[derive(Debug, toasty::Model)]
pub struct Product {
    #[key]
    #[auto]
    id: u64,

    name: String,
    price: Option<i64>, // Price in cents
    liquidation: bool,
    visible_on_site: bool,
    created_at: String,
    updated_at: String,

    #[has_many]
    images: toasty::HasMany<ProductImage>,

    #[has_many]
    product_types: toasty::HasMany<ProductProductType>,
}

/// ProductImage model with Toasty ORM
#[derive(Debug, toasty::Model)]
#[table = "product_images"]
pub struct ProductImage {
    #[key]
    #[auto]
    id: u64,

    #[index]
    product_id: u64,
    #[belongs_to(key = product_id, references = id)]
    product: toasty::BelongsTo<Product>,

    url: String,
    filename: String,
    position: String, // "front" or "back"
    created_at: String,
}

/// Junction table model for Product <-> ProductType M:N relationship
#[derive(Debug, toasty::Model)]
#[table = "product_product_types"]
pub struct ProductProductType {
    #[key]
    #[auto]
    id: u64,

    #[index]
    product_id: u64,
    #[belongs_to(key = product_id, references = id)]
    product: toasty::BelongsTo<Product>,

    #[index]
    product_type_name: String,
    #[belongs_to(key = product_type_name, references = name)]
    product_type: toasty::BelongsTo<crate::server::models::product_types::ProductType>,
}

/// Database row structure for products table (kept for migration)
#[derive(Debug)]
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

/// Database row structure for product_images table (kept for migration)
#[derive(Debug)]
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
    pub id: u64,
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
            id: value.id as u64,
            name: value.name,
            price: value.price,
            liquidation: value.liquidation,
            visible_on_site: value.visible_on_site,
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}

impl From<Product> for ProductView {
    fn from(value: Product) -> Self {
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
