use anyhow::Context;
use anyhow::Result;
use sqlx::SqlitePool;

use crate::server::{
    database::select::Selectable,
    models::{FactureAndClient, PageAdmin, clients::ClientRow, factures::FactureRow},
};

pub async fn load_all_factures(pool: &SqlitePool) -> Result<PageAdmin> {
    let mut tx = pool.begin().await.context("Failed to begin transaction")?;

    let factures = FactureRow::select_all(&mut *tx).await?;
    let mut clients = ClientRow::select_all_for_facture(&mut *tx).await?;

    let factures: Result<Vec<FactureAndClient>> = factures
        .into_iter()
        .map(|f| {
            let idx = clients.iter().position(|c| c.id == f.client_id);
            idx.ok_or(anyhow::Error::msg("Unable to find client for facture"))
                .map(|i| {
                    let client = clients.swap_remove(i);
                    FactureAndClient {
                        facture: f.into(),
                        client: client.into(),
                    }
                })
        })
        .collect();

    Ok(PageAdmin {
        factures: factures?,
    })
}
