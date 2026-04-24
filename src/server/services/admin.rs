use anyhow::Context;
use anyhow::Result;
use futures_core::stream::BoxStream;
use futures_util::StreamExt;
use serde::Serialize;
use sqlx::SqlitePool;

use crate::server::models::payments::PaymentReportRow;
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

#[derive(Serialize, Debug)]
pub struct PaymentReportRecord {
    #[serde(rename = "Num facture")]
    facture_id: i64,
    #[serde(rename = "Num facture ancienne")]
    paper_ref: Option<String>,
    #[serde(rename = "Type de facture")]
    facture_type: String,
    #[serde(rename = "Date")]
    date: Option<String>,
    #[serde(rename = "MontantF")]
    amount: String,
    #[serde(rename = "Type")]
    payment_type: String,
    #[serde(rename = "AnnuleeF")]
    cancelled: u8,
    #[serde(rename = "Transactions")]
    transaction_url: String,
}

pub fn load_payment_reports_data<'a>(
    pool: &'a SqlitePool,
    external_url: &'a str,
) -> BoxStream<'a, PaymentReportRecord> {
    let data = PaymentReportRow::stream_all_with_facture(pool);
    let data = data.filter_map(async |row| {
        row.map(|record| {
            let external_url = external_url.to_owned();
            let transaction_url = format!(
                "{}/factures/{}/transactions",
                external_url, record.facture_id
            );
            PaymentReportRecord {
                facture_id: record.facture_id,
                paper_ref: record.paper_ref,
                facture_type: record.facture_type,
                date: record.date,
                amount: record.amount.to_string(), //TODO format as ###.## $
                payment_type: record.payment_type,
                cancelled: record.cancelled,
                transaction_url,
            }
        })
        .ok()
    });
    Box::pin(data)
}
