use lmah_inventory_rs::server::models::{
    facture_items::FactureItemRow, factures::FactureRow, product_types::ProductTypeRow,
    products::ProductRow, statuts::StatutInsert,
};

#[derive(Clone)]
pub struct FactureFixture;

impl FactureFixture {
    /// A complete facture with a product item
    pub fn simple_product_facture(client_id: i64, _product_id: i64) -> FactureRow {
        FactureRow {
            id: 0, // Will be assigned by DB
            client_id,
            facture_type: Some("Product".to_string()),
            date: Some("2026-03-15".to_string()),
            event_id: None,
            fixed_total: None,
            cancelled: false,
            paper_ref: Some("INV-001".to_string()),
            created_at: "2026-03-15 10:00:00".to_string(),
            updated_at: "2026-03-15 10:00:00".to_string(),
        }
    }
}

#[derive(Clone)]
pub struct FactureItemFixture;

impl FactureItemFixture {
    /// A simple product item
    pub fn product_item(facture_id: i64, product_id: i64) -> FactureItemRow {
        FactureItemRow {
            facture_id,
            product_id,
            item_type: "Product".to_string(),
            price: Some(15000), // $150.00 in cents
            notes: Some("Beautiful red dress".to_string()),
            quantity: 1,
            extra_large_size: None,
            rebate_percent: None,
            size: Some("Medium".to_string()),
            chest: Some(36),
            waist: Some(28),
            hips: Some(38),
            color: Some("Red".to_string()),
            beneficiary: Some("Jane Doe".to_string()),
            floor_item: false,
            insurance: None,
            other_costs: None,
            rebate_dollar: None,
            created_at: "2026-03-15 10:00:00".to_string(),
            updated_at: "2026-03-15 10:00:00".to_string(),
        }
    }
}

#[derive(Clone)]
pub struct ProductFixture;

impl ProductFixture {
    pub fn evening_dress() -> ProductRow {
        ProductRow {
            name: "Evening Gown - Elegant Red".to_string(),
            price: Some(15000), // $150.00 in cents
            liquidation: false,
            visible_on_site: true,
            created_at: "2026-03-01 10:00:00".to_string(),
            updated_at: "2026-03-01 10:00:00".to_string(),
        }
    }
}

#[derive(Clone)]
pub struct ProductTypeFixture;

impl ProductTypeFixture {
    pub fn wedding_dress() -> ProductTypeRow {
        ProductTypeRow {
            name: "Robe de mariée".to_string(),
        }
    }

    pub fn evening() -> ProductTypeRow {
        ProductTypeRow {
            name: "Soirée".to_string(),
        }
    }
}

#[derive(Clone)]
pub struct StatutFixture;

impl StatutFixture {
    pub fn recording_out_date(facture_id: i64, facture_item_id: i64) -> StatutInsert {
        StatutInsert {
            facture_id,
            facture_item_id,
            statut_type: "RecordingOutDate".to_string(),
            date: "2026-03-20".to_string(),
            seamstress: None,
        }
    }
}
