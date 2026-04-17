use anyhow::Result;
use std::path::Path;
use toasty::Db;

use crate::server::models::clients::Client;
use crate::server::models::config::Config;
use crate::server::models::events::Event;
use crate::server::models::facture_items::FactureItem;
use crate::server::models::factures::Facture;
use crate::server::models::payments::Payment;
use crate::server::models::product_types::ProductType;
use crate::server::models::products::{Product, ProductImage, ProductProductType};
use crate::server::models::refunds::Refund;
use crate::server::models::statuts::Statut;

// pub mod has_table;
// pub mod insert;
// pub mod queries;
// pub mod select;
// pub mod update;

pub async fn connect_to_path(db_path: &Path) -> Result<Db> {
    let connection_string = format!("sqlite://{}", db_path.display());
    connect_to_url(&connection_string).await
}

pub async fn connect_to_url(db_url: &String) -> Result<Db> {
    println!("Connecting to database: {}", db_url);
    let db = toasty::Db::builder()
        .models(toasty::models!(
            Client,
            Config,
            Event,
            Facture,
            FactureItem,
            Payment,
            Product,
            ProductImage,
            ProductProductType,
            ProductType,
            Refund,
            Statut
        ))
        .connect(db_url)
        .await?;

    println!("Connected to database: {}", db_url);

    Ok(db)
}
