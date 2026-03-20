use crate::server::models::{clients::ClientRowWithId, config::ConfigRow, product_types::ProductTypeRow};

pub trait HasTable {
    fn table_name() -> &'static str;
}

impl HasTable for ClientRowWithId {
    fn table_name() -> &'static str {
        "clients"
    }
}

impl HasTable for ConfigRow {
    fn table_name() -> &'static str {
        "config"
    }
}

impl HasTable for ProductTypeRow {
    fn table_name() -> &'static str {
        "product_types"
    }
}
