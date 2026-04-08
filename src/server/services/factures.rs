use std::{collections::HashMap, hash::Hash};

use crate::server::{
    database::select::Selectable,
    models::{
        FactureDashboardData, FactureItemEntry, FactureItemsData, PageFactureItemsData,
        PageOneFactureItemData,
        clients::{ClientRow, ClientView},
        events::{EventRow, EventView},
        facture_items::{
            FactureComputed, FactureItemComputed, FactureItemRow, FactureItemType, FactureItemView,
            ItemFactureFlowType,
        },
        factures::{FactureRow, FactureView},
        payments::{PaymentRow, PaymentView},
        products::{ProductRow, ProductView},
        refunds::RefundRow,
        statuts::{StateView, StatutRow},
    },
    services::statuts::{load_one_item_statuts_flow, load_statuts_flow},
};
use anyhow::{Context, Result};
use sqlx::SqlitePool;

pub async fn select_one_facture_item(
    pool: &SqlitePool,
    facture_id: i64,
    facture_item_id: i64,
) -> Result<PageOneFactureItemData> {
    let mut tx = pool.begin().await.context("Failed to begin transaction")?;

    let facture_row = FactureRow::select_one(facture_id, &mut tx)
        .await?
        .ok_or(anyhow::Error::msg("Facture not found."))?;

    let client_row = ClientRow::select_one(facture_row.client_id, &mut tx)
        .await?
        .ok_or(anyhow::Error::msg("Client related to facture not found."))?;

    let facture_item_row = FactureItemRow::select_one(facture_item_id, &mut tx)
        .await?
        .ok_or(anyhow::Error::msg("Facture item not found."))?;

    let product_row = ProductRow::select_one(facture_item_row.product_id, &mut tx)
        .await?
        .ok_or(anyhow::Error::msg(
            "Product related to facture item not found.",
        ))?;

    let facture_item_flow =
        ItemFactureFlowType::select_one_facture_item_flow_types(facture_item_id, &mut tx).await?;

    let statut_rows = StatutRow::select_all_for_facture_item(facture_item_id, &mut tx).await?;

    tx.commit().await.context("Failed to commit transaction")?;

    build_one_facture_item_data(
        facture_row,
        client_row,
        facture_item_row,
        product_row,
        facture_item_flow,
        statut_rows,
    )
}

pub async fn select_one(pool: &SqlitePool, facture_id: i64) -> Result<PageFactureItemsData> {
    let mut tx = pool.begin().await.context("Failed to begin transaction")?;

    let facture_row = FactureRow::select_one(facture_id, &mut tx)
        .await?
        .ok_or(anyhow::Error::msg("Facture not found."))?;

    let client_row = ClientRow::select_one(facture_row.client_id, &mut tx)
        .await?
        .ok_or(anyhow::Error::msg("Client related to facture not found."))?;

    let event_row = match facture_row.event_id {
        Some(e_id) => EventRow::select_one(e_id, &mut tx)
            .await?
            .ok_or(anyhow::Error::msg("Event related to facture not found."))
            .map(Some),
        None => Ok(None),
    };
    let event_row = event_row?;

    let alteration_product_row = ProductRow::select_by_name("Altération", &mut tx)
        .await?
        .ok_or(anyhow::Error::msg("Unable to locate Alteration product"))?;
    let location_product_row = ProductRow::select_by_name("Location", &mut tx)
        .await?
        .ok_or(anyhow::Error::msg("Unable to locate Location product"))?;
    let product_rows = ProductRow::select_for_facture(facture_id, &mut tx).await?;

    let facture_item_rows = FactureItemRow::select_all_for_facture(facture_id, &mut tx).await?;
    let facture_item_flows =
        ItemFactureFlowType::select_one_facture_flow_types(facture_id, &mut tx).await?;
    let statut_rows = StatutRow::select_all_for_facture(facture_id, &mut tx).await?;

    let payment_rows = PaymentRow::select_all_for_facture(facture_id, &mut tx).await?;
    let refund_rows = RefundRow::select_all_for_facture(facture_id, &mut tx).await?;

    tx.commit().await.context("Failed to commit transaction")?;

    let facture_data = build_one_facture_data(
        facture_row,
        client_row,
        event_row,
        product_rows,
        facture_item_rows,
        facture_item_flows,
        statut_rows,
        payment_rows,
        refund_rows,
    )?;
    Ok(PageFactureItemsData {
        facture_data,
        alteration_product: ProductView::from(alteration_product_row),
        location_product: ProductView::from(location_product_row),
    })
}

pub async fn select_all(pool: &SqlitePool) -> Result<Vec<FactureDashboardData>> {
    let mut tx = pool.begin().await.context("Failed to begin transaction")?;

    let facture_rows = FactureRow::select_all(&mut tx).await?;
    let client_rows = ClientRow::select_with_facture(&mut tx).await?;
    let facture_item_flows = ItemFactureFlowType::select_all_flow_types(&mut tx).await?;
    let statut_rows = StatutRow::select_all(&mut tx).await?;

    tx.commit().await.context("Failed to commit transaction")?;

    build_facture_dashboard_data(facture_rows, client_rows, facture_item_flows, statut_rows)
}

fn build_one_facture_data(
    facture: FactureRow,
    client: ClientRow,
    event: Option<EventRow>,
    product: Vec<ProductRow>,
    facture_items: Vec<FactureItemRow>,
    facture_item_flows: Vec<ItemFactureFlowType>,
    statuts: Vec<StatutRow>,
    payment_rows: Vec<PaymentRow>,
    refund_rows: Vec<RefundRow>,
) -> Result<FactureItemsData> {
    let mut product_per_id: HashMap<i64, ProductRow> =
        product.into_iter().map(|p| (p.id, p)).collect();
    let mut state_per_item = load_statuts_flow(facture_item_flows, statuts)?;
    let facture_items: Result<Vec<FactureItemView>> = facture_items
        .into_iter()
        .map(FactureItemView::try_from)
        .collect();
    let facture_items = facture_items?;

    let facture_view = FactureView::from(facture);
    let payment_views = payment_rows.into_iter().map(PaymentView::from).collect();
    let refund_views = refund_rows.into_iter().map(RefundRow::from).collect();
    let (facture_computed, items_computed) =
        computed_facture_fields(&facture_view, &facture_items, &payment_views, &refund_views);

    let items: Result<Vec<FactureItemEntry>> = facture_items
        .into_iter()
        .map(|item| {
            let facture_item_id = item.id();
            let product_id = item.product_id();
            let product = product_per_id
                .remove(&product_id)
                .ok_or(anyhow::Error::msg("Related product not found."));
            let state = state_per_item
                .remove(&(facture_view.id, facture_item_id))
                .ok_or(anyhow::Error::msg("Related state not found."));

            product.and_then(|p| {
                state.map(|s| FactureItemEntry {
                    item,
                    state: s,
                    product: ProductView::from(p),
                })
            })
        })
        .collect();

    Ok(FactureItemsData {
        facture: facture_view,
        facture_computed: facture_computed,
        client: ClientView::from(client),
        event: event.map(EventView::from),
        items: items?,
        items_computed: items_computed.into_values().collect(),
    })
}

fn build_facture_dashboard_data(
    factures: Vec<FactureRow>,
    clients: Vec<ClientRow>,
    facture_item_flows: Vec<ItemFactureFlowType>,
    statuts: Vec<StatutRow>,
) -> Result<Vec<FactureDashboardData>> {
    let state_per_item = load_statuts_flow(facture_item_flows, statuts)?;
    let mut state_per_facture: HashMap<i64, Vec<(i64, StateView)>> =
        group_by_map(state_per_item, |a| a.0.0, |a| (a.0.1, a.1.clone()));
    let mut res = Vec::new();

    for facture in factures {
        let found_client: Option<ClientRow> = clients.iter().find_map(|c| {
            if c.id == facture.client_id {
                Some(c.clone())
            } else {
                None
            }
        });
        let client = found_client.ok_or(anyhow::Error::msg(format!(
            "No client found for facture {}",
            facture.id
        )))?;

        let state_per_item = state_per_facture.remove(&facture.id).unwrap_or(Vec::new());

        let one = FactureDashboardData {
            facture: FactureView::from(facture),
            client: ClientView::from(client.clone()),
            state_per_item,
        };

        res.push(one);
    }

    Ok(res)
}

fn build_one_facture_item_data(
    facture: FactureRow,
    client: ClientRow,
    facture_item: FactureItemRow,
    product: ProductRow,
    item_flow_type: ItemFactureFlowType,
    statuts: Vec<StatutRow>,
) -> Result<PageOneFactureItemData> {
    let state = load_one_item_statuts_flow(item_flow_type, statuts)?;
    let item_entry = FactureItemEntry {
        item: FactureItemView::try_from(facture_item)?,
        product: ProductView::from(product),
        state: state,
    };
    Ok(PageOneFactureItemData {
        facture: FactureView::from(facture),
        client: ClientView::from(client),
        item: item_entry,
    })
}

fn compute_item(item: &FactureItemView) -> FactureItemComputed {
    let price = item.price().unwrap_or(0);
    let calculated_rebate = match &item.value {
        FactureItemType::FactureItemProduct(i) => {
            let r = i.rebate_percent.unwrap_or(0);
            if r > 0 {
                i.quantity * (r / 100 * price)
            } else {
                0
            }
        }
        FactureItemType::FactureItemLocation(_) => 0,
        FactureItemType::FactureItemAlteration(i) => i.rebate_dollar.unwrap_or(0),
    };
    let total = match &item.value {
        FactureItemType::FactureItemProduct(i) => {
            let xl = i.extra_large_size.unwrap_or(0);
            let sub_total = (price + xl) * i.quantity;
            sub_total - calculated_rebate
        }
        FactureItemType::FactureItemLocation(i) => {
            price + i.other_costs.unwrap_or(0) + i.insurance.unwrap_or(0)
        }
        FactureItemType::FactureItemAlteration(i) => price - i.rebate_dollar.unwrap_or(0),
    };
    let measurements = match &item.value {
        FactureItemType::FactureItemProduct(i) => i
            .chest
            .and_then(|c| {
                i.waist.and_then(|w| {
                    i.hips
                        .map(|h| format!("B{} x T{} x H{}", c, w, h).to_string())
                })
            })
            .unwrap_or("-".to_string()),
        FactureItemType::FactureItemLocation(_) => "-".to_string(),
        FactureItemType::FactureItemAlteration(_) => "-".to_string(),
    };

    FactureItemComputed {
        calculated_rebate,
        total,
        measurements,
    }
}

const TPS_RATE: f64 = 5.0;
const TVQ_RATE: f64 = 9.975;

fn computed_facture_fields(
    facture: &FactureView,
    items: &Vec<FactureItemView>,
    payments: &Vec<PaymentView>,
    refunds: &Vec<RefundRow>,
) -> (FactureComputed, HashMap<i64, FactureItemComputed>) {
    let computed_per_items: HashMap<i64, FactureItemComputed> = items
        .iter()
        .map(|item| (item.id(), compute_item(item)))
        .collect();
    let effective_total = match facture.fixed_total {
        Some(t) => t,
        None => computed_per_items.values().map(|ic| ic.total).sum(),
    };

    let tps: i64 = (TPS_RATE / 100.0 * (effective_total as f64)).round() as i64;
    let tvq: i64 = (TVQ_RATE / 100.0 * (effective_total as f64)).round() as i64;

    let tax_total = effective_total + tps + tvq;

    let total_payments = payments.iter().map(|p| p.amount).sum();
    let total_refunds = refunds.iter().map(|p| p.amount).sum();
    let total_refundable = total_payments - total_refunds;

    let balance = tax_total - total_payments + total_refunds;

    let fc = FactureComputed {
        total: effective_total,
        tax_total,
        tvq,
        tps,
        balance,
        total_payments,
        total_refunds,
        total_refundable,
    };

    (fc, computed_per_items)
}

/// The order in the values vector is not guaranteed to be the same as the original iterator I.
fn group_by_map<A, I, F, K, G, V>(v: I, mut fk: F, mut fv: G) -> HashMap<K, Vec<V>>
where
    K: Hash,
    K: Eq,
    I: IntoIterator<Item = A>,
    F: FnMut(&A) -> K,
    G: FnMut(&A) -> V,
{
    let mut result = HashMap::<K, Vec<V>>::new();
    for a in v {
        let key = fk(&a);
        let value = fv(&a);
        result.entry(key).or_default().push(value);
    }
    result
}

#[cfg(test)]
#[test]
fn test_grouped_by_first_id() {
    let data = HashMap::from([
        ((1, 1), "test - 1 - 1"),
        ((1, 2), "test - 1 - 2"),
        ((2, 1), "test - 2 - 1"),
        ((3, 1), "test - 3 - 1"),
        ((3, 2), "test - 3 - 2"),
    ]);
    let mut grouped_by_first_id = group_by_map(data, |a| a.0.0, |a| (a.0.1, a.1));
    let mut vec = grouped_by_first_id.remove(&3).unwrap();
    vec.sort_by_key(|a| a.0); // sorting to assert
    assert_eq!(vec, vec![(1, "test - 3 - 1"), (2, "test - 3 - 2")])
}
