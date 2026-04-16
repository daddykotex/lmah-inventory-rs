/// FactureItem model with Toasty ORM
/// Polymorphic table with type-specific fields
#[derive(Debug, toasty::Model)]
#[table = "facture_items"]
pub struct FactureItem {
    #[key]
    #[auto]
    id: u64,

    #[index]
    facture_id: u64,
    #[belongs_to(key = facture_id, references = id)]
    facture: toasty::BelongsTo<crate::server::models::factures::Facture>,

    #[index]
    product_id: u64,
    #[belongs_to(key = product_id, references = id)]
    product: toasty::BelongsTo<crate::server::models::products::Product>,

    item_type: String, // "Produit", "Location", or "Alteration"

    // Common fields (all types)
    price: Option<i64>, // in cents
    notes: Option<String>,
    quantity: i64, // Default 1

    // Produit-specific fields
    extra_large_size: Option<i64>, // in cents
    rebate_percent: Option<i64>,
    size: Option<String>,
    chest: Option<i64>,
    waist: Option<i64>,
    hips: Option<i64>,
    color: Option<String>,
    beneficiary: Option<String>,
    floor_item: bool, // Default false

    // Location-specific fields
    insurance: Option<i64>,   // in cents
    other_costs: Option<i64>, // in cents

    // Alteration-specific fields
    rebate_dollar: Option<i64>, // in cents

    created_at: String,
    updated_at: String,

    #[has_many]
    statuts: toasty::HasMany<crate::server::models::statuts::Statut>,
}

/// Database row structure for facture_items table (kept for migration)
#[derive(Debug)]
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
    pub facture_id: u64,   // Required FK to factures
    pub product_id: u64,   // Required FK to products
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
    pub price: Option<i64>, // in cents
    pub notes: Option<String>,
    pub quantity: i64, // Default 1
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
    pub price: Option<i64>, // in cents
    pub notes: Option<String>,
    pub quantity: i64, // Default 1
    //
    pub beneficiary: Option<String>,
    pub insurance: Option<i64>,   // in cents
    pub other_costs: Option<i64>, // in cents
}
pub struct FactureItemAlteration {
    pub price: Option<i64>, // in cents
    pub notes: Option<String>,
    pub quantity: i64, // Default 1
    //
    pub rebate_dollar: Option<i64>, // in cents
}

pub enum FactureItemValue<Location, Alteration, Product> {
    FactureItemProduct(Product),
    FactureItemLocation(Location),
    FactureItemAlteration(Alteration),
}

pub type FactureItemType =
    FactureItemValue<FactureItemLocation, FactureItemAlteration, FactureItemProduct>;
pub struct FactureItemView {
    pub id: u64,
    pub facture_id: u64, // Required FK to factures
    pub product_id: u64, // Required FK to products
    pub created_at: String,
    pub updated_at: String,
    pub value: FactureItemType,
}

impl FactureItemView {
    pub fn blank(
        product_name: &str,
    ) -> FactureItemValue<FactureItemLocation, FactureItemAlteration, FactureItemProduct> {
        match product_name {
            "Alteration" => FactureItemValue::FactureItemAlteration(FactureItemAlteration {
                price: None,
                notes: None,
                quantity: 1,
                rebate_dollar: None,
            }),
            "Location" => FactureItemValue::FactureItemLocation(FactureItemLocation {
                price: None,
                notes: None,
                quantity: 1,
                beneficiary: None,
                insurance: None,
                other_costs: None,
            }),
            _ => FactureItemValue::FactureItemProduct(FactureItemProduct {
                price: None,
                notes: None,
                quantity: 1,
                extra_large_size: None,
                rebate_percent: None,
                size: None,
                chest: None,
                waist: None,
                hips: None,
                color: None,
                beneficiary: None,
                floor_item: false,
            }),
        }
    }
    pub fn price(&self) -> Option<i64> {
        match &self.value {
            FactureItemValue::FactureItemProduct(i) => i.price,
            FactureItemValue::FactureItemLocation(i) => i.price,
            FactureItemValue::FactureItemAlteration(i) => i.price,
        }
    }
}

impl TryFrom<FactureItemRow> for FactureItemView {
    type Error = anyhow::Error;

    fn try_from(value: FactureItemRow) -> Result<Self, Self::Error> {
        match value.item_type.as_str() {
            "Alteration" => Ok(FactureItemView {
                id: value.id as u64,
                facture_id: value.facture_id as u64,
                product_id: value.product_id as u64,
                created_at: value.created_at,
                updated_at: value.updated_at,
                value: FactureItemValue::FactureItemAlteration(FactureItemAlteration {
                    price: value.price,
                    notes: value.notes,
                    quantity: value.quantity,
                    rebate_dollar: value.rebate_dollar,
                }),
            }),
            "Location" => Ok(FactureItemView {
                id: value.id as u64,
                facture_id: value.facture_id as u64,
                product_id: value.product_id as u64,
                created_at: value.created_at,
                updated_at: value.updated_at,
                value: FactureItemValue::FactureItemLocation(FactureItemLocation {
                    price: value.price,
                    notes: value.notes,
                    quantity: value.quantity,
                    beneficiary: value.beneficiary,
                    insurance: value.insurance,
                    other_costs: value.other_costs,
                }),
            }),
            "Product" => Ok(FactureItemView {
                id: value.id as u64,
                facture_id: value.facture_id as u64,
                product_id: value.product_id as u64,
                created_at: value.created_at,
                updated_at: value.updated_at,
                value: FactureItemValue::FactureItemProduct(FactureItemProduct {
                    price: value.price,
                    notes: value.notes,
                    quantity: value.quantity,
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

#[derive(Debug)]
pub struct ItemFactureFlowType {
    pub facture_id: u64,
    pub facture_item_id: u64,
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
