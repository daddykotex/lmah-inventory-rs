use std::fmt::Display;

use crate::server::models::{
    clients::{ClientInsert, ClientRow},
    config::ConfigRow,
    events::EventRow,
    facture_items::FactureItemRow,
    factures::FactureRow,
    payments::PaymentRow,
    product_types::ProductTypeRow,
    products::ProductRow,
    refunds::RefundRow,
    statuts::StatutRow,
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

pub trait TableName {
    fn table_name(&self) -> &'static str;
}

impl TableName for Table {
    fn table_name(&self) -> &'static str {
        match self {
            Table::Clients => "clients",
            Table::Config => "config",
            Table::Events => "events",
            Table::ProductTypes => "product_types",
            Table::Products => "products",
            Table::Factures => "factures",
            Table::FactureItems => "facture_items",
            Table::Payments => "payments",
            Table::Refunds => "refunds",
            Table::Statuts => "statuts",
        }
    }
}

pub trait HasTable {
    fn table() -> Table;
}

impl HasTable for ConfigRow {
    fn table() -> Table {
        Table::Config
    }
}

impl HasTable for ClientRow {
    fn table() -> Table {
        Table::Clients
    }
}

impl HasTable for ClientInsert {
    fn table() -> Table {
        Table::Clients
    }
}

impl HasTable for EventRow {
    fn table() -> Table {
        Table::Events
    }
}

impl HasTable for ProductTypeRow {
    fn table() -> Table {
        Table::ProductTypes
    }
}

impl HasTable for ProductRow {
    fn table() -> Table {
        Table::Products
    }
}

impl HasTable for FactureRow {
    fn table() -> Table {
        Table::Factures
    }
}

impl HasTable for FactureItemRow {
    fn table() -> Table {
        Table::FactureItems
    }
}

impl HasTable for PaymentRow {
    fn table() -> Table {
        Table::Payments
    }
}

impl HasTable for RefundRow {
    fn table() -> Table {
        Table::Refunds
    }
}

impl HasTable for StatutRow {
    fn table() -> Table {
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
