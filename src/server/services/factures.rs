use std::{collections::HashMap, hash::Hash};

use crate::server::{
    database::select::Selectable,
    models::{
        FactureWithRelatedData,
        clients::{ClientRow, ClientView},
        facture_items::ItemFactureFlowType,
        factures::{FactureRow, FactureView},
        statuts::{StateView, StatutRow},
    },
    services::statuts::load_statuts_flow,
};
use anyhow::{Context, Result};
use sqlx::SqlitePool;

pub async fn select_all(pool: &SqlitePool) -> Result<Vec<FactureWithRelatedData>> {
    let mut tx = pool.begin().await.context("Failed to begin transaction")?;

    let facture_rows = FactureRow::select_all(&mut tx).await?;
    let client_rows = ClientRow::select_with_facture(&mut tx).await?;
    let facture_item_flows = ItemFactureFlowType::select_all_flow_types(&mut tx).await?;
    let statut_rows = StatutRow::select_all(&mut tx).await?;

    tx.commit().await.context("Failed to commit transaction")?;

    build_facture_dashboard_data(facture_rows, client_rows, facture_item_flows, statut_rows)
}

fn build_facture_dashboard_data(
    factures: Vec<FactureRow>,
    clients: Vec<ClientRow>,
    facture_item_flows: Vec<ItemFactureFlowType>,
    statuts: Vec<StatutRow>,
) -> Result<Vec<FactureWithRelatedData>> {
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

        let one = FactureWithRelatedData {
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
