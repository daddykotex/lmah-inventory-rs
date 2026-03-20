use crate::server::models::{clients::ClientRowWithId, config::ConfigRow};

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
