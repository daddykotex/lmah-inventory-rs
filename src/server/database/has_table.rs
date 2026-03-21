use crate::server::models::{
    clients::ClientRow, config::ConfigRow, events::EventRow, facture_items::FactureItemRow,
    factures::FactureRow, product_types::ProductTypeRow, products::ProductRow,
};

pub trait HasTable {
    fn table_name() -> &'static str;
}

impl HasTable for ClientRow {
    fn table_name() -> &'static str {
        "clients"
    }
}

impl HasTable for ConfigRow {
    fn table_name() -> &'static str {
        "config"
    }
}

impl HasTable for EventRow {
    fn table_name() -> &'static str {
        "events"
    }
}

impl HasTable for ProductTypeRow {
    fn table_name() -> &'static str {
        "product_types"
    }
}

impl HasTable for ProductRow {
    fn table_name() -> &'static str {
        "products"
    }
}

impl HasTable for FactureRow {
    fn table_name() -> &'static str {
        "factures"
    }
}

impl HasTable for FactureItemRow {
    fn table_name() -> &'static str {
        "facture_items"
    }
}
