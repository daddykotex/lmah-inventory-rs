use sqlx::prelude::FromRow;

/// Database row structure for facture_items table
/// This is a polymorphic table with type-specific fields
#[derive(Debug, FromRow)]
pub struct FactureItemRow {
    pub id: i64,
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

#[derive(Debug)]
pub struct FactureItemInsert {
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
}

pub struct FactureItemProduct {
    pub id: i64,
    pub facture_id: i64,    // Required FK to factures
    pub product_id: i64,    // Required FK to products
    pub price: Option<i64>, // in cents
    pub notes: Option<String>,
    pub quantity: i64, // Default 1
    pub created_at: String,
    pub updated_at: String,
    //
    pub extra_large_size: Option<i64>, // in cents
    pub rebate_percent: Option<i64>,
    pub size: Option<String>,
    pub chest: Option<i64>,
    pub waist: Option<i64>,
    pub hips: Option<i64>,
    pub color: Option<String>,
    pub beneficiary: Option<String>,
    pub floor_item: bool, // Default false
}
pub struct FactureItemLocation {
    pub id: i64,
    pub facture_id: i64,    // Required FK to factures
    pub product_id: i64,    // Required FK to products
    pub price: Option<i64>, // in cents
    pub notes: Option<String>,
    pub quantity: i64, // Default 1
    pub created_at: String,
    pub updated_at: String,
    //
    pub insurance: Option<i64>,   // in cents
    pub other_costs: Option<i64>, // in cents
}
pub struct FactureItemAlteration {
    pub id: i64,
    pub facture_id: i64,    // Required FK to factures
    pub product_id: i64,    // Required FK to products
    pub price: Option<i64>, // in cents
    pub notes: Option<String>,
    pub quantity: i64, // Default 1
    pub created_at: String,
    pub updated_at: String,
    //
    pub rebate_dollar: Option<i64>, // in cents
}

pub enum FactureItemType<Location, Alteration, Product> {
    FactureItemProduct(Product),
    FactureItemLocation(Location),
    FactureItemAlteration(Alteration),
}

pub struct FactureItemView {
    pub value: FactureItemType<FactureItemLocation, FactureItemAlteration, FactureItemProduct>,
}

impl TryFrom<FactureItemRow> for FactureItemView {
    type Error = anyhow::Error;

    fn try_from(value: FactureItemRow) -> Result<Self, Self::Error> {
        match value.item_type.as_str() {
            "Alteration" => Ok(FactureItemView {
                value: FactureItemType::FactureItemAlteration(FactureItemAlteration {
                    id: value.id,
                    facture_id: value.facture_id,
                    product_id: value.product_id,
                    price: value.price,
                    notes: value.notes,
                    quantity: value.quantity,
                    created_at: value.created_at,
                    updated_at: value.updated_at,
                    rebate_dollar: value.rebate_dollar,
                }),
            }),
            "Location" => Ok(FactureItemView {
                value: FactureItemType::FactureItemLocation(FactureItemLocation {
                    id: value.id,
                    facture_id: value.facture_id,
                    product_id: value.product_id,
                    price: value.price,
                    notes: value.notes,
                    quantity: value.quantity,
                    created_at: value.created_at,
                    updated_at: value.updated_at,
                    insurance: value.insurance,
                    other_costs: value.other_costs,
                }),
            }),
            "Product" => Ok(FactureItemView {
                value: FactureItemType::FactureItemProduct(FactureItemProduct {
                    id: value.id,
                    facture_id: value.facture_id,
                    product_id: value.product_id,
                    price: value.price,
                    notes: value.notes,
                    quantity: value.quantity,
                    created_at: value.created_at,
                    updated_at: value.updated_at,
                    extra_large_size: value.extra_large_size,
                    rebate_percent: value.rebate_percent,
                    size: value.size,
                    chest: value.chest,
                    waist: value.waist,
                    hips: value.hips,
                    color: value.color,
                    beneficiary: value.beneficiary,
                    floor_item: value.floor_item,
                }),
            }),

            _ => Err(anyhow::Error::msg("Invalid facture item type")),
        }
    }
}

#[derive(FromRow)]
pub struct ItemFactureFlowType {
    pub facture_id: i64,
    pub facture_item_id: i64,
    pub flow_type: String,
}
