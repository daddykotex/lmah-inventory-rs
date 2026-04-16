use anyhow::Result;
use toasty::Db;
use std::path::Path;

use crate::server::models::clients::Client;
use crate::server::models::events::Event;
use crate::server::models::factures::Facture;

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
        .models(toasty::models!(Client, Event, Facture))
        .connect(db_url)
        .await?;

    println!("Connected to database: {}", db_url);

    Ok(db)
}
