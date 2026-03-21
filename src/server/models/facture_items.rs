/// Database row structure for facture_items table
/// This is a polymorphic table with type-specific fields
#[derive(Debug)]
pub struct FactureItemRow {
    pub facture_id: i64,   // Required FK to factures
    pub product_id: i64,   // Required FK to products
    pub item_type: String, // "Produit", "Location", or "Alteration"

    // Common fields (all types)
    pub price: Option<i64>, // in cents
    pub notes: Option<String>,
    pub quantity: i64, // Default 1

    // Produit-specific fields
    pub extra_large_size: Option<i64>, // in cents
    pub rebate_percent: Option<i64>,
    pub size: Option<String>,
    pub chest: Option<i64>,
    pub waist: Option<i64>,
    pub hips: Option<i64>,
    pub color: Option<String>,
    pub beneficiary: Option<String>,
    pub floor_item: bool, // Default false

    // Location-specific fields
    pub insurance: Option<i64>,   // in cents
    pub other_costs: Option<i64>, // in cents

    // Alteration-specific fields
    pub rebate_dollar: Option<i64>, // in cents

    pub created_at: String,
    pub updated_at: String,
}
