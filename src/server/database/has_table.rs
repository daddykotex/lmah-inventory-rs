use std::fmt::Display;

use crate::server::models::{
    clients::ClientRow, config::ConfigRow, events::EventRow, facture_items::FactureItemRow,
    factures::FactureRow, payments::PaymentRow, product_types::ProductTypeRow,
    products::ProductRow, refunds::RefundRow, statuts::StatutRow,
};

pub enum Table {
    Clients,
    Config,
    Events,
    ProductTypes,
    Products,
    Factures,
    FactureItems,
    Payments,
    Refunds,
    Statuts,
}

pub trait HasTable {
    fn table_name() -> Table;
}

impl HasTable for ClientRow {
    fn table_name() -> Table {
        Table::Clients
    }
}

impl HasTable for ConfigRow {
    fn table_name() -> Table {
        Table::Config
    }
}

impl HasTable for EventRow {
    fn table_name() -> Table {
        Table::Events
    }
}

impl HasTable for ProductTypeRow {
    fn table_name() -> Table {
        Table::ProductTypes
    }
}

impl HasTable for ProductRow {
    fn table_name() -> Table {
        Table::Products
    }
}

impl HasTable for FactureRow {
    fn table_name() -> Table {
        Table::Factures
    }
}

impl HasTable for FactureItemRow {
    fn table_name() -> Table {
        Table::FactureItems
    }
}

impl HasTable for PaymentRow {
    fn table_name() -> Table {
        Table::Payments
    }
}

impl HasTable for RefundRow {
    fn table_name() -> Table {
        Table::Refunds
    }
}

impl HasTable for StatutRow {
    fn table_name() -> Table {
        Table::Statuts
    }
}

impl Display for Table {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Table::Clients => write!(f, "clients"),
            Table::Config => write!(f, "config"),
            Table::Events => write!(f, "events"),
            Table::ProductTypes => write!(f, "product_types"),
            Table::Products => write!(f, "products"),
            Table::Factures => write!(f, "factures"),
            Table::FactureItems => write!(f, "facture_items"),
            Table::Payments => write!(f, "payments"),
            Table::Refunds => write!(f, "refunds"),
            Table::Statuts => write!(f, "statuts"),
        }
    }
}
