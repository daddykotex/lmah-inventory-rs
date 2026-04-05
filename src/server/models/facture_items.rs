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
    pub beneficiary: Option<String>,
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

impl FactureItemView {
    pub fn id(&self) -> i64 {
        match &self.value {
            FactureItemType::FactureItemProduct(i) => i.id,
            FactureItemType::FactureItemLocation(i) => i.id,
            FactureItemType::FactureItemAlteration(i) => i.id,
        }
    }
    pub fn facture_id(&self) -> i64 {
        match &self.value {
            FactureItemType::FactureItemProduct(i) => i.facture_id,
            FactureItemType::FactureItemLocation(i) => i.facture_id,
            FactureItemType::FactureItemAlteration(i) => i.facture_id,
        }
    }
    pub fn product_id(&self) -> i64 {
        match &self.value {
            FactureItemType::FactureItemProduct(i) => i.product_id,
            FactureItemType::FactureItemLocation(i) => i.product_id,
            FactureItemType::FactureItemAlteration(i) => i.product_id,
        }
    }
    pub fn price(&self) -> Option<i64> {
        match &self.value {
            FactureItemType::FactureItemProduct(i) => i.price,
            FactureItemType::FactureItemLocation(i) => i.price,
            FactureItemType::FactureItemAlteration(i) => i.price,
        }
    }
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
                    beneficiary: value.beneficiary,
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

#[derive(Debug, FromRow)]
pub struct ItemFactureFlowType {
    pub facture_id: i64,
    pub facture_item_id: i64,
    pub flow_type: String,
}

pub struct FactureComputed {
    pub total: i64,
    pub tvq: i64,
    pub tps: i64,
    pub tax_total: i64,
    pub balance: i64,
    pub total_payments: i64,
    pub total_refunds: i64,
    pub total_refundable: i64,
}

pub struct FactureItemComputed {
    pub calculated_rebate: i64,
    pub total: i64,
    pub measurements: String,
}
