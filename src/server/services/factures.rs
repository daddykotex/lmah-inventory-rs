use std::{collections::HashMap, hash::Hash};

use crate::server::{
    database::select::Selectable,
    models::{
        FactureDashboardData, FactureItemEntry, FactureItemsData, PageFactureItemsData,
        clients::{ClientRow, ClientView},
        facture_items::{FactureItemRow, FactureItemType, FactureItemView, ItemFactureFlowType},
        factures::{FactureRow, FactureView},
        products::{ProductRow, ProductView},
        statuts::{StateView, StatutRow},
    },
    services::statuts::load_statuts_flow,
};
use anyhow::{Context, Result};
use sqlx::SqlitePool;

pub async fn select_one(pool: &SqlitePool, facture_id: i64) -> Result<PageFactureItemsData> {
    let mut tx = pool.begin().await.context("Failed to begin transaction")?;

    let facture_row = FactureRow::select_one(facture_id, &mut tx)
        .await?
        .ok_or(anyhow::Error::msg("Facture not found."))?;

    let client_row = ClientRow::select_one(facture_row.client_id, &mut tx)
        .await?
        .ok_or(anyhow::Error::msg("Client related to facture not found."))?;

    let alteration_product_row = ProductRow::select_by_name("Altération", &mut tx)
        .await?
        .ok_or(anyhow::Error::msg("Unable to locate Alteration product"))?;
    let location_product_row = ProductRow::select_by_name("Location`", &mut tx)
        .await?
        .ok_or(anyhow::Error::msg("Unable to locate Location product"))?;
    let product_rows = ProductRow::select_for_facture(facture_id, &mut tx).await?;

    let facture_item_rows = FactureItemRow::select_all_for_facture(facture_id, &mut tx).await?;
    let facture_item_flows =
        ItemFactureFlowType::select_one_facture_flow_types(facture_id, &mut tx).await?;
    let statut_rows = StatutRow::select_all_for_facture(facture_id, &mut tx).await?;

    tx.commit().await.context("Failed to commit transaction")?;

    let facture_data = build_one_facture_data(
        facture_row,
        client_row,
        product_rows,
        facture_item_rows,
        facture_item_flows,
        statut_rows,
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
    product: Vec<ProductRow>,
    facture_items: Vec<FactureItemRow>,
    facture_item_flows: Vec<ItemFactureFlowType>,
    statuts: Vec<StatutRow>,
) -> Result<FactureItemsData> {
    let mut product_per_id: HashMap<i64, ProductRow> =
        product.into_iter().map(|p| (p.id, p)).collect();
    let mut state_per_item = load_statuts_flow(facture_item_flows, statuts)?;
    let facture_items: Result<Vec<FactureItemView>> = facture_items
        .into_iter()
        .map(FactureItemView::try_from)
        .collect();
    let items: Result<Vec<FactureItemEntry>> = facture_items?
        .into_iter()
        .map(|item| {
            let facture_item_id = match &item.value {
                FactureItemType::FactureItemProduct(v) => v.id,
                FactureItemType::FactureItemLocation(v) => v.id,
                FactureItemType::FactureItemAlteration(v) => v.id,
            };
            let product_id = match &item.value {
                FactureItemType::FactureItemProduct(v) => v.product_id,
                FactureItemType::FactureItemLocation(v) => v.product_id,
                FactureItemType::FactureItemAlteration(v) => v.product_id,
            };
            let product = product_per_id
                .remove(&product_id)
                .ok_or(anyhow::Error::msg("Related product not found."));
            let state = state_per_item
                .remove(&(facture.id, facture_item_id))
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
        facture: FactureView::from(facture),
        client: ClientView::from(client),
        items: items?,
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
    let grouped_by_first_id = group_by_map(data, |a| a.0.0, |a| (a.0.1, a.1));
    assert_eq!(
        grouped_by_first_id.get(&3),
        Some(&vec![(1, "test - 3 - 1"), (2, "test - 3 - 2")])
    )
}
