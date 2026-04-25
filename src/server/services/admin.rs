use std::collections::HashMap;
use std::sync::Arc;

use anyhow::Context;
use anyhow::Result;
use futures_core::Stream;
use futures_util::StreamExt;
use serde::Serialize;
use sqlx::SqlitePool;

use crate::server::models::facture_items::{FactureItemRow, FactureItemView};
use crate::server::models::payments::{PaymentReportRow, PaymentRow, PaymentView};
use crate::server::models::refunds::{RefundRow, RefundView};
use crate::server::services::factures::computed_facture_fields;
use crate::server::{
    database::select::Selectable,
    models::{
        FactureAndClient, PageAdmin,
        clients::{ClientRow, ClientView},
        factures::{FactureRow, FactureView},
    },
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

pub fn load_payment_reports_data(
    pool: &SqlitePool,
    external_url: Arc<str>,
) -> impl Stream<Item = PaymentReportRecord> + 'static {
    let data = PaymentReportRow::stream_all_with_facture(pool);
    data.filter_map(async |r| r.ok()).map(move |record| {
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
}

#[derive(Serialize, Debug)]
pub struct FactureReportRecord {
    #[serde(rename = "Numéro de facture")]
    facture_id: i64,
    #[serde(rename = "Ancienne référence")]
    paper_ref: Option<String>,
    #[serde(rename = "Date de facture")]
    date: Option<String>,
    #[serde(rename = "Solde à payer")]
    balance: String,
    #[serde(rename = "Nom")]
    client_name: String,
}
pub async fn load_facture_reports_data(pool: &SqlitePool) -> Result<Vec<FactureReportRecord>> {
    let mut tx = pool.begin().await.context("Failed to begin transaction")?;

    let facture_rows = FactureRow::select_all(&mut *tx).await?;
    let facture_item_rows = FactureItemRow::select_with_facture(&mut *tx).await?;
    let client_rows = ClientRow::select_with_facture(&mut *tx).await?;
    let payment_rows = PaymentRow::select_with_facture(&mut *tx).await?;
    let refund_rows = RefundRow::select_with_facture(&mut *tx).await?;

    tx.commit().await.context("Failed to commit transaction")?;

    // Group items/payments/refunds by facture_id as views
    let mut items_by_facture: HashMap<i64, Vec<FactureItemView>> = HashMap::new();
    for row in facture_item_rows {
        let facture_id = row.facture_id;
        if let Ok(view) = FactureItemView::try_from(row) {
            items_by_facture.entry(facture_id).or_default().push(view);
        }
    }

    let mut payments_by_facture: HashMap<i64, Vec<PaymentView>> = HashMap::new();
    for row in payment_rows {
        let facture_id = row.facture_id;
        payments_by_facture
            .entry(facture_id)
            .or_default()
            .push(PaymentView::from(row));
    }

    let mut refunds_by_facture: HashMap<i64, Vec<RefundView>> = HashMap::new();
    for row in refund_rows {
        let facture_id = row.facture_id;
        refunds_by_facture
            .entry(facture_id)
            .or_default()
            .push(RefundView::from(row));
    }

    let clients_by_id: HashMap<i64, ClientView> = client_rows
        .into_iter()
        .map(|r| (r.id, ClientView::from(r)))
        .collect();

    let empty_items = vec![];
    let empty_payments = vec![];
    let empty_refunds = vec![];

    let mut entries = Vec::with_capacity(facture_rows.len());
    for facture_row in facture_rows {
        let facture_id = facture_row.id;
        let client_id = facture_row.client_id;

        let facture_view = FactureView::from(facture_row);
        let items = items_by_facture.get(&facture_id).unwrap_or(&empty_items);
        let payments = payments_by_facture
            .get(&facture_id)
            .unwrap_or(&empty_payments);
        let refunds = refunds_by_facture
            .get(&facture_id)
            .unwrap_or(&empty_refunds);

        let (facture_computed, _) =
            computed_facture_fields(&facture_view, items, payments, refunds);

        let client = clients_by_id.get(&client_id).cloned().ok_or_else(|| {
            anyhow::anyhow!(
                "Unable to find client {} for facture {}",
                client_id,
                facture_id
            )
        })?;

        entries.push(FactureReportRecord {
            facture_id,
            paper_ref: facture_view.paper_ref,
            date: facture_view.date,
            balance: facture_computed.balance.to_string(), // Format as ###.## $
            client_name: client.name(),
        });
    }

    Ok(entries)
}
