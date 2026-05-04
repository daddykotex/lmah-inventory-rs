use std::{collections::HashMap, hash::Hash, vec};

use crate::server::{
    database::{insert::Insertable, select::Selectable},
    models::{
        FactureDashboardData, FactureInfo, FactureItemEntry, FactureItemFormConfig,
        FactureItemInfo, FactureItemsData, PageAddOneFactureItemData, PageAddProduct,
        PageFactureItemsData, PageOneFactureItemData, PagePrintData, PageTransactionsData,
        PrintConfig,
        clients::{ClientRow, ClientView},
        events::EventRow,
        facture_items::{
            FactureComputed, FactureItemComputed, FactureItemRow, FactureItemValue,
            FactureItemView, ItemFactureFlowType,
        },
        factures::{FactureInsert, FactureRow, FactureView},
        payments::{PaymentRow, PaymentView},
        product_types::{ProductTypeRow, ProductTypeView},
        products::{ProductInfo, ProductRow, ProductView},
        refunds::{RefundRow, RefundView},
        statuts::{State, StateView, StatutRow},
    },
    services::{
        config::{
            load_clauses, load_extra_large_amount, load_note_templates, load_seamstresses,
            load_signatures,
        },
        statuts::{load_one_item_statuts_flow, load_statuts_flow},
    },
};
use anyhow::{Context, Result};
use sqlx::SqlitePool;

use crate::server::utils::money::parse_money;

type ProductsOrRedirect = std::result::Result<Vec<ProductInfo>, ProductView>;

pub async fn load_products_to_add(
    pool: &SqlitePool,
    facture_id: i64,
) -> Result<ProductsOrRedirect> {
    let mut tx = pool.begin().await.context("Failed to begin transaction")?;

    let facture_info = select_one(pool, facture_id).await?;
    let facture_type = facture_info
        .facture_data
        .facture_info
        .facture
        .facture_type
        .as_deref()
        .unwrap_or("Product");

    let result: ProductsOrRedirect = match facture_type {
        "Alteration" => {
            let product = ProductRow::select_by_name("Altération", &mut *tx).await?;
            let product = product.ok_or(anyhow::Error::msg("Alteration product not found"))?;
            Err(ProductView::from(product))
        }
        "Location" => {
            let product = ProductRow::select_by_name("Location", &mut *tx).await?;
            let product = product.ok_or(anyhow::Error::msg("Location product not found"))?;
            Err(ProductView::from(product))
        }
        "Product" => {
            let products = ProductRow::select_only_products(&mut *tx).await?;
            let product_types = ProductTypeRow::select_only_products(&mut *tx).await?;
            let products = products.into_iter().map(ProductView::from).collect();
            let products = build_product_info(products, product_types);
            Ok(products)
        }
        _ => anyhow::bail!("Invalid facture type"),
    };
    tx.commit().await.context("Failed to commit transaction")?;

    Ok(result)
}

/// combine products and their types
/// it is assumed they are ordered by product it
fn build_product_info(products: Vec<ProductView>, types: Vec<(i64, String)>) -> Vec<ProductInfo> {
    let mut product_info_records = Vec::with_capacity(products.capacity());

    let mut p_iter = products.into_iter();
    let mut pt_iter = types.into_iter();
    let mut current_type: Option<(i64, String)> = None;

    loop {
        match p_iter.next() {
            // we're done iterating the product list (should be done iterating the types as well)
            None => break,
            // we're processing one product
            Some(p) => {
                // clear the types list accumulator (we are working with a new product)
                let mut pi = ProductInfo {
                    product: p,
                    types: vec![],
                };
                // continue iterate over types
                loop {
                    match current_type {
                        // if the product id match, we add it to the list
                        Some((p_id, p_type)) if p_id == pi.product.id => {
                            pi.types.push(ProductTypeView { name: p_type });
                            current_type = pt_iter.next();
                        }
                        // if the type product id is smaller, skip it
                        Some((p_id, _)) if p_id < pi.product.id => {
                            current_type = None;
                        }
                        // if it does not match, we have reached another the type for another product
                        Some(_) => {
                            break;
                        }
                        // if None, this mean we are done (confirmed by the `next` being None below)
                        // or it's the first time we arrive here
                        None => {
                            let next = pt_iter.next();
                            match next {
                                Some(n) => {
                                    current_type = Some(n);
                                }
                                None => break,
                            }
                        }
                    }
                }
                product_info_records.push(pi);
            }
        }
    }

    product_info_records
}

pub async fn blank_facture_item(
    pool: &SqlitePool,
    facture_id: i64,
    product_id: i64,
) -> Result<PageAddOneFactureItemData> {
    let mut tx = pool.begin().await.context("Failed to begin transaction")?;

    let facture_row = FactureRow::select_one(facture_id, &mut *tx)
        .await?
        .ok_or(anyhow::Error::msg("Facture not found."))?;
    let facture_view = FactureView::from(facture_row);

    let client_row = ClientRow::select_one(facture_view.client_id, &mut *tx)
        .await?
        .ok_or(anyhow::Error::msg("Client related to facture not found."))?;

    let event_row = match facture_view.event_id {
        Some(e_id) => EventRow::select_one(e_id, &mut *tx)
            .await?
            .ok_or(anyhow::Error::msg("Event related to facture not found."))
            .map(Some),
        None => Ok(None),
    };
    let event_row = event_row?;

    let facture_items = FactureItemRow::select_all_for_facture(facture_id, &mut *tx).await?;
    let facture_items: Result<Vec<FactureItemView>> = facture_items
        .into_iter()
        .map(FactureItemView::try_from)
        .collect();
    let facture_items = facture_items?;

    let payment_rows = PaymentRow::select_all_for_facture(facture_id, &mut *tx).await?;
    let refund_rows = RefundRow::select_all_for_facture(facture_id, &mut *tx).await?;

    let payment_views: Vec<PaymentView> = payment_rows.into_iter().map(PaymentView::from).collect();
    let refund_views: Vec<RefundView> = refund_rows.into_iter().map(RefundView::from).collect();
    let (facture_computed, _) =
        computed_facture_fields(&facture_view, &facture_items, &payment_views, &refund_views);

    let product_row =
        ProductRow::select_one(product_id, &mut *tx)
            .await?
            .ok_or(anyhow::Error::msg(
                "Product related to facture item not found.",
            ))?;
    let (state, item_flow) = match &facture_view.facture_type.as_deref() {
        Some("Alteration") => (State::<String, String>::ToBeAltered, "AlterationFlow"),
        None | Some(_) => (State::<String, String>::ToOrder, "DressToOrderFlow"),
    };
    let state = StateView {
        current_state: state,
        previous_states: vec![],
        item_flow: String::from(item_flow),
    };

    let product_type_row = ProductTypeRow::select_for_product(product_row.id, &mut *tx).await?;

    let config_note_templates = load_note_templates(&mut *tx).await?;
    let config_extra_large_amount = load_extra_large_amount(&mut *tx).await?;
    let config_seamstresses = load_seamstresses(&mut *tx).await?;

    let form_config = FactureItemFormConfig {
        note_templates: config_note_templates,
        extra_large_amount: config_extra_large_amount,
        seamstresses: config_seamstresses,
    };

    let facture_item = FactureItemView::blank(&product_row.name);

    tx.commit().await.context("Failed to commit transaction")?;

    let item_entry = FactureItemEntry {
        item: facture_item,
        state,
        product: ProductView::from(product_row),
    };
    let facture_info = FactureInfo {
        facture: facture_view,
        facture_computed,
        event: event_row.map(|e| e.into()),
        client: ClientView::from(client_row),
    };
    Ok(PageAddOneFactureItemData {
        facture_info,
        item: item_entry,
        product_type: ProductTypeView::from(product_type_row),
        form_config,
    })
}

pub async fn load_add_product_data(pool: &SqlitePool, facture_id: i64) -> Result<PageAddProduct> {
    let mut tx = pool.begin().await.context("Failed to begin transaction")?;

    let (facture_info, _, _, _) = load_facture_info(facture_id, &mut tx).await?;
    let product_types = ProductTypeRow::select_all(&mut *tx).await?;

    tx.commit().await.context("Failed to commit transaction")?;

    Ok(PageAddProduct {
        facture_info,
        product_types: product_types.into_iter().map(|a| a.into()).collect(),
    })
}

pub async fn load_print_data(pool: &SqlitePool, facture_id: i64) -> Result<PagePrintData> {
    let mut tx = pool.begin().await.context("Failed to begin transaction")?;

    let (facture_info, items, payments, refunds) = load_facture_info(facture_id, &mut tx).await?;

    let signatures = load_signatures(&mut *tx).await?;
    let clauses = load_clauses(&mut *tx).await?;
    let print_config = PrintConfig {
        signatures,
        clauses,
    };
    let products = ProductRow::select_for_facture(facture_id, &mut *tx).await?;
    let product_types = ProductTypeRow::select_for_facture(facture_id, &mut *tx).await?;
    let products = products.into_iter().map(ProductView::from).collect();
    let mut products = build_product_info(products, product_types);

    let mut facture_item_info = Vec::with_capacity(items.capacity());
    for (item, item_computed) in items {
        let idx = products
            .iter()
            .position(|value| value.product.id == item.product_id);
        let idx = idx.ok_or(anyhow::Error::msg(format!(
            "Product {} not found for facture item id: {}",
            item.product_id, item.facture_id
        )))?;
        let product_info = products.swap_remove(idx);
        facture_item_info.push(FactureItemInfo {
            item,
            item_computed,
            product_info,
        })
    }

    tx.commit().await.context("Failed to commit transaction")?;

    Ok(PagePrintData {
        facture_info,
        items: facture_item_info,
        payments,
        refunds,
        print_config,
    })
}

pub async fn select_transactions(
    pool: &SqlitePool,
    facture_id: i64,
) -> Result<PageTransactionsData> {
    let mut tx = pool.begin().await.context("Failed to begin transaction")?;

    let facture_row = FactureRow::select_one(facture_id, &mut *tx)
        .await?
        .ok_or(anyhow::Error::msg("Facture not found."))?;
    let facture_view = FactureView::from(facture_row);

    let client_row = ClientRow::select_one(facture_view.client_id, &mut *tx)
        .await?
        .ok_or(anyhow::Error::msg("Client related to facture not found."))?;

    let event_row = match facture_view.event_id {
        Some(e_id) => EventRow::select_one(e_id, &mut *tx)
            .await?
            .ok_or(anyhow::Error::msg("Event related to facture not found."))
            .map(Some),
        None => Ok(None),
    };
    let event_row = event_row?;

    let facture_items = FactureItemRow::select_all_for_facture(facture_id, &mut *tx).await?;
    let facture_items: Result<Vec<FactureItemView>> = facture_items
        .into_iter()
        .map(FactureItemView::try_from)
        .collect();
    let facture_items = facture_items?;

    let payment_rows = PaymentRow::select_all_for_facture(facture_id, &mut *tx).await?;
    let refund_rows = RefundRow::select_all_for_facture(facture_id, &mut *tx).await?;

    let payment_views: Vec<PaymentView> = payment_rows.into_iter().map(PaymentView::from).collect();
    let refund_views: Vec<RefundView> = refund_rows.into_iter().map(RefundView::from).collect();
    let (facture_computed, _) =
        computed_facture_fields(&facture_view, &facture_items, &payment_views, &refund_views);

    tx.commit().await.context("Failed to commit transaction")?;

    let facture_info = FactureInfo {
        facture: facture_view,
        facture_computed,
        event: event_row.map(|e| e.into()),
        client: ClientView::from(client_row),
    };
    Ok(PageTransactionsData {
        facture_info,
        payments: payment_views,
        refunds: refund_views,
    })
}

pub async fn select_one_facture_item(
    pool: &SqlitePool,
    facture_id: i64,
    facture_item_id: i64,
) -> Result<PageOneFactureItemData> {
    let mut tx = pool.begin().await.context("Failed to begin transaction")?;

    let facture_row = FactureRow::select_one(facture_id, &mut *tx)
        .await?
        .ok_or(anyhow::Error::msg("Facture not found."))?;

    let client_row = ClientRow::select_one(facture_row.client_id, &mut *tx)
        .await?
        .ok_or(anyhow::Error::msg("Client related to facture not found."))?;

    let facture_item_row = FactureItemRow::select_one(facture_item_id, &mut *tx)
        .await?
        .ok_or(anyhow::Error::msg("Facture item not found."))?;

    let product_row = ProductRow::select_one(facture_item_row.product_id, &mut *tx)
        .await?
        .ok_or(anyhow::Error::msg(
            "Product related to facture item not found.",
        ))?;

    let facture_item_flow =
        ItemFactureFlowType::select_one_facture_item_flow_types(facture_item_id, &mut *tx).await?;

    let statut_rows = StatutRow::select_all_for_facture_item(facture_item_id, &mut *tx).await?;

    let product_type_row = ProductTypeRow::select_for_product(product_row.id, &mut *tx).await?;

    let config_note_templates = load_note_templates(&mut *tx).await?;
    let config_extra_large_amount = load_extra_large_amount(&mut *tx).await?;
    let config_seamstresses = load_seamstresses(&mut *tx).await?;

    let form_config = FactureItemFormConfig {
        note_templates: config_note_templates,
        extra_large_amount: config_extra_large_amount,
        seamstresses: config_seamstresses,
    };

    tx.commit().await.context("Failed to commit transaction")?;

    let state = load_one_item_statuts_flow(facture_item_flow, statut_rows)?;
    let item_entry = FactureItemEntry {
        item: FactureItemView::try_from(facture_item_row)?,
        product: ProductView::from(product_row),
        state,
    };
    Ok(PageOneFactureItemData {
        facture: FactureView::from(facture_row),
        client: ClientView::from(client_row),
        product_type: ProductTypeView::from(product_type_row),
        item: item_entry,
        form_config,
    })
}

pub async fn select_one(pool: &SqlitePool, facture_id: i64) -> Result<PageFactureItemsData> {
    let mut tx = pool.begin().await.context("Failed to begin transaction")?;

    let facture_row = FactureRow::select_one(facture_id, &mut *tx)
        .await?
        .ok_or(anyhow::Error::msg("Facture not found."))?;

    let client_row = ClientRow::select_one(facture_row.client_id, &mut *tx)
        .await?
        .ok_or(anyhow::Error::msg("Client related to facture not found."))?;

    let event_row = match facture_row.event_id {
        Some(e_id) => EventRow::select_one(e_id, &mut *tx)
            .await?
            .ok_or(anyhow::Error::msg("Event related to facture not found."))
            .map(Some),
        None => Ok(None),
    };
    let event_row = event_row?;

    let alteration_product_row = ProductRow::select_by_name("Altération", &mut *tx)
        .await?
        .ok_or(anyhow::Error::msg("Unable to locate Alteration product"))?;
    let location_product_row = ProductRow::select_by_name("Location", &mut *tx)
        .await?
        .ok_or(anyhow::Error::msg("Unable to locate Location product"))?;
    let product_rows = ProductRow::select_for_facture(facture_id, &mut *tx).await?;

    let facture_item_rows = FactureItemRow::select_all_for_facture(facture_id, &mut *tx).await?;
    let facture_item_flows =
        ItemFactureFlowType::select_one_facture_flow_types(facture_id, &mut *tx).await?;
    let statut_rows = StatutRow::select_all_for_facture(facture_id, &mut *tx).await?;

    let payment_rows = PaymentRow::select_all_for_facture(facture_id, &mut *tx).await?;
    let refund_rows = RefundRow::select_all_for_facture(facture_id, &mut *tx).await?;

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

    let facture_rows = FactureRow::select_all(&mut *tx).await?;
    let client_rows = ClientRow::select_with_facture(&mut *tx).await?;
    let facture_item_flows = ItemFactureFlowType::select_all_flow_types(&mut *tx).await?;
    let statut_rows = StatutRow::select_all(&mut *tx).await?;

    tx.commit().await.context("Failed to commit transaction")?;

    build_facture_dashboard_data(facture_rows, client_rows, facture_item_flows, statut_rows)
}

#[expect(clippy::too_many_arguments, reason = "Will reformat later.")]
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
    let payment_views: Vec<PaymentView> = payment_rows.into_iter().map(PaymentView::from).collect();
    let refund_views: Vec<RefundView> = refund_rows.into_iter().map(RefundView::from).collect();
    let (facture_computed, _) =
        computed_facture_fields(&facture_view, &facture_items, &payment_views, &refund_views);

    let items: Result<Vec<FactureItemEntry<FactureItemView>>> = facture_items
        .into_iter()
        .map(|item| {
            let facture_item_id = item.id;
            let product_id = item.product_id;
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

    let facture_info = FactureInfo {
        facture: facture_view,
        facture_computed,
        event: event.map(|e| e.into()),
        client: ClientView::from(client),
    };

    Ok(FactureItemsData {
        facture_info,
        items: items?,
    })
}

fn build_facture_dashboard_data(
    factures: Vec<FactureRow>,
    mut clients: Vec<ClientRow>,
    facture_item_flows: Vec<ItemFactureFlowType>,
    statuts: Vec<StatutRow>,
) -> Result<Vec<FactureDashboardData>> {
    let state_per_item = load_statuts_flow(facture_item_flows, statuts)?;
    let mut state_per_facture: HashMap<i64, Vec<(i64, StateView)>> =
        group_by_map(state_per_item, |a| a.0.0, |a| (a.0.1, a.1.clone()));
    let mut res = Vec::new();

    for facture in factures {
        let idx = clients.iter().position(|c| c.id == facture.client_id);
        let idx = idx.ok_or(anyhow::Error::msg(format!(
            "No client found for facture {}",
            facture.id
        )))?;
        let client = clients.swap_remove(idx);

        let state_per_item = state_per_facture.remove(&facture.id).unwrap_or_default();

        let one = FactureDashboardData {
            facture: FactureView::from(facture),
            client: ClientView::from(client.clone()),
            state_per_item,
        };

        res.push(one);
    }

    Ok(res)
}

fn compute_item(item: &FactureItemView) -> FactureItemComputed {
    let price = item.price().unwrap_or_default();
    let calculated_rebate = match &item.value {
        FactureItemValue::FactureItemProduct(i) => i.rebate_percent_applied().unwrap_or_default(),
        FactureItemValue::FactureItemLocation(_) => 0,
        FactureItemValue::FactureItemAlteration(i) => i.rebate_dollar.unwrap_or_default(),
    };
    let total = match &item.value {
        FactureItemValue::FactureItemProduct(i) => {
            let xl = i.extra_large_size.unwrap_or_default();
            let sub_total = (price + xl) * i.quantity;
            sub_total - calculated_rebate
        }
        FactureItemValue::FactureItemLocation(i) => {
            price + i.other_costs.unwrap_or_default() + i.insurance.unwrap_or_default()
        }
        FactureItemValue::FactureItemAlteration(i) => price - i.rebate_dollar.unwrap_or_default(),
    };
    let measurements = match &item.value {
        FactureItemValue::FactureItemProduct(i) => i
            .chest
            .and_then(|c| {
                i.waist.and_then(|w| {
                    i.hips
                        .map(|h| format!("B{} x T{} x H{}", c, w, h).to_string())
                })
            })
            .unwrap_or("-".to_string()),
        FactureItemValue::FactureItemLocation(_) => "-".to_string(),
        FactureItemValue::FactureItemAlteration(_) => "-".to_string(),
    };

    FactureItemComputed {
        calculated_rebate,
        total,
        measurements,
    }
}

const TPS_RATE: f64 = 5.0;
const TVQ_RATE: f64 = 9.975;

pub fn computed_facture_fields(
    facture: &FactureView,
    items: &[FactureItemView],
    payments: &[PaymentView],
    refunds: &[RefundView],
) -> (FactureComputed, HashMap<i64, FactureItemComputed>) {
    let computed_per_items: HashMap<i64, FactureItemComputed> = items
        .iter()
        .map(|item| (item.id, compute_item(item)))
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

async fn load_facture_info(
    facture_id: i64,
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
) -> Result<(
    FactureInfo,
    Vec<(FactureItemView, FactureItemComputed)>,
    Vec<PaymentView>,
    Vec<RefundView>,
)> {
    let facture_row = FactureRow::select_one(facture_id, &mut **tx)
        .await?
        .ok_or(anyhow::Error::msg("Facture not found."))?;
    let facture_view = FactureView::from(facture_row);

    let client_row = ClientRow::select_one(facture_view.client_id, &mut **tx)
        .await?
        .ok_or(anyhow::Error::msg("Client related to facture not found."))?;

    let event_row = match facture_view.event_id {
        Some(e_id) => EventRow::select_one(e_id, &mut **tx)
            .await?
            .ok_or(anyhow::Error::msg("Event related to facture not found."))
            .map(Some),
        None => Ok(None),
    };
    let event_row = event_row?;

    let facture_items = FactureItemRow::select_all_for_facture(facture_id, &mut **tx).await?;
    let facture_items: Result<Vec<FactureItemView>> = facture_items
        .into_iter()
        .map(FactureItemView::try_from)
        .collect();
    let facture_items = facture_items?;

    let payment_rows = PaymentRow::select_all_for_facture(facture_id, &mut **tx).await?;
    let refund_rows = RefundRow::select_all_for_facture(facture_id, &mut **tx).await?;

    let payment_views: Vec<PaymentView> = payment_rows.into_iter().map(PaymentView::from).collect();
    let refund_views: Vec<RefundView> = refund_rows.into_iter().map(RefundView::from).collect();

    let (facture_computed, mut items_computed) =
        computed_facture_fields(&facture_view, &facture_items, &payment_views, &refund_views);

    let facture_items_details: Result<Vec<(FactureItemView, FactureItemComputed)>> = facture_items
        .into_iter()
        .map(|item| {
            let item_computed = items_computed.remove(&item.id);
            let item_computed =
                item_computed.ok_or(anyhow::Error::msg("Unable to find computed item"));
            item_computed.map(|ir| (item, ir))
        })
        .collect();

    let facture_info = FactureInfo {
        facture: facture_view,
        facture_computed,
        event: event_row.map(|e| e.into()),
        client: ClientView::from(client_row),
    };
    Ok((
        facture_info,
        facture_items_details?,
        payment_views,
        refund_views,
    ))
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

#[cfg(test)]
#[test]
fn test_build_product_info() {
    fn dummy(id: i64) -> ProductView {
        ProductView {
            id,
            name: "dummy".to_string(),
            price: None,
            liquidation: false,
            visible_on_site: false,
            created_at: "".to_string(),
            updated_at: "".to_string(),
        }
    }
    let products = Vec::from([dummy(1), dummy(2), dummy(3), dummy(4)]);
    let types = Vec::from([
        (1_i64, "robe".to_string()),
        (1_i64, "robe mere".to_string()),
        (2_i64, "gaine".to_string()),
        (3_i64, "robe".to_string()),
        (4_i64, "other".to_string()),
    ]);

    let expected = Vec::from([
        ProductInfo {
            product: dummy(1),
            types: Vec::from([
                ProductTypeView {
                    name: "robe".to_string(),
                },
                ProductTypeView {
                    name: "robe mere".to_string(),
                },
            ]),
        },
        ProductInfo {
            product: dummy(2),
            types: Vec::from([ProductTypeView {
                name: "gaine".to_string(),
            }]),
        },
        ProductInfo {
            product: dummy(3),
            types: Vec::from([ProductTypeView {
                name: "robe".to_string(),
            }]),
        },
        ProductInfo {
            product: dummy(4),
            types: Vec::from([ProductTypeView {
                name: "other".to_string(),
            }]),
        },
    ]);
    let result = build_product_info(products, types);
    assert_eq!(result, expected)
}

#[cfg(test)]
#[test]
fn test_build_product_info_more_types_than_product() {
    fn dummy(id: i64) -> ProductView {
        ProductView {
            id,
            name: "dummy".to_string(),
            price: None,
            liquidation: false,
            visible_on_site: false,
            created_at: "".to_string(),
            updated_at: "".to_string(),
        }
    }
    let products = Vec::from([dummy(2)]);
    let types = Vec::from([
        (1_i64, "robe".to_string()),
        (2_i64, "gaine".to_string()),
        (3_i64, "robe".to_string()),
        (4_i64, "other".to_string()),
    ]);

    let expected = Vec::from([ProductInfo {
        product: dummy(2),
        types: Vec::from([ProductTypeView {
            name: "gaine".to_string(),
        }]),
    }]);
    let result = build_product_info(products, types);
    assert_eq!(result, expected)
}

// Phase 1 POST endpoint service functions

/// Mark a facture as cancelled
pub async fn cancel_facture(pool: &SqlitePool, facture_id: i64) -> Result<u64> {
    let mut tx = pool.begin().await.context("Failed to begin transaction")?;

    // Verify facture exists
    let maybe_facture = FactureRow::select_one(facture_id, &mut *tx).await?;
    maybe_facture.ok_or(anyhow::Error::msg(format!(
        "Facture with id {} not found",
        facture_id
    )))?;

    let result =
        sqlx::query("UPDATE factures SET cancelled = 1, updated_at = datetime('now') WHERE id = ?")
            .bind(facture_id)
            .execute(&mut *tx)
            .await
            .with_context(|| format!("Failed to cancel facture {}", facture_id))?;

    tx.commit().await.context("Failed to commit transaction")?;

    Ok(result.rows_affected())
}

/// Unmark a facture as cancelled
pub async fn uncancel_facture(pool: &SqlitePool, facture_id: i64) -> Result<u64> {
    let mut tx = pool.begin().await.context("Failed to begin transaction")?;

    // Verify facture exists
    let maybe_facture = FactureRow::select_one(facture_id, &mut *tx).await?;
    maybe_facture.ok_or(anyhow::Error::msg(format!(
        "Facture with id {} not found",
        facture_id
    )))?;

    let result =
        sqlx::query("UPDATE factures SET cancelled = 0, updated_at = datetime('now') WHERE id = ?")
            .bind(facture_id)
            .execute(&mut *tx)
            .await
            .with_context(|| format!("Failed to uncancel facture {}", facture_id))?;

    tx.commit().await.context("Failed to commit transaction")?;

    Ok(result.rows_affected())
}

/// Remove event link from a facture
pub async fn unlink_event(pool: &SqlitePool, facture_id: i64) -> Result<u64> {
    let mut tx = pool.begin().await.context("Failed to begin transaction")?;

    // Verify facture exists
    let maybe_facture = FactureRow::select_one(facture_id, &mut *tx).await?;
    maybe_facture.ok_or(anyhow::Error::msg(format!(
        "Facture with id {} not found",
        facture_id
    )))?;

    let result = sqlx::query(
        "UPDATE factures SET event_id = NULL, updated_at = datetime('now') WHERE id = ?",
    )
    .bind(facture_id)
    .execute(&mut *tx)
    .await
    .with_context(|| format!("Failed to unlink event from facture {}", facture_id))?;

    tx.commit().await.context("Failed to commit transaction")?;

    Ok(result.rows_affected())
}

/// Link an existing event to a facture (within an existing transaction)
pub async fn link_event(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    facture_id: i64,
    event_id: i64,
) -> Result<u64> {
    // Verify facture exists
    let maybe_facture = FactureRow::select_one(facture_id, &mut **tx).await?;
    maybe_facture.ok_or(anyhow::Error::msg(format!(
        "Facture with id {} not found",
        facture_id
    )))?;

    // Verify event exists
    let maybe_event = EventRow::select_one(event_id, &mut **tx).await?;
    maybe_event.ok_or(anyhow::Error::msg(format!(
        "Event with id {} not found",
        event_id
    )))?;

    let result =
        sqlx::query("UPDATE factures SET event_id = ?, updated_at = datetime('now') WHERE id = ?")
            .bind(event_id)
            .bind(facture_id)
            .execute(&mut **tx)
            .await
            .with_context(|| {
                format!(
                    "Failed to link event {} to facture {}",
                    event_id, facture_id
                )
            })?;

    Ok(result.rows_affected())
}

/// Get the facture type for a given facture
pub async fn get_facture_type(pool: &SqlitePool, facture_id: i64) -> Result<String> {
    let mut tx = pool.begin().await.context("Failed to begin transaction")?;

    let maybe_facture = FactureRow::select_one(facture_id, &mut *tx).await?;
    let facture = maybe_facture.ok_or(anyhow::Error::msg(format!(
        "Facture with id {} not found",
        facture_id
    )))?;

    tx.commit().await.context("Failed to commit transaction")?;

    Ok(facture
        .facture_type
        .unwrap_or_else(|| "Product".to_string()))
}

pub async fn insert_facture(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    client_id: i64,
    facture_type: String,
) -> Result<i64> {
    let to_insert = FactureInsert {
        client_id,
        facture_type: Some(facture_type),
        event_id: None,
        fixed_total: None,
        cancelled: false,
        paper_ref: None,
    };

    let inserted_id = to_insert
        .insert_one(&mut *tx)
        .await?
        .expect("An ID should be generated for a new Facture");

    Ok(inserted_id)
}

/// Update facture details (date and paper_ref)
pub async fn update_facture_details(
    pool: &SqlitePool,
    facture_id: i64,
    date: String,
    paper_ref: Option<String>,
) -> Result<u64> {
    let mut tx = pool.begin().await.context("Failed to begin transaction")?;

    // Verify facture exists
    let maybe_facture = FactureRow::select_one(facture_id, &mut *tx).await?;
    maybe_facture.ok_or(anyhow::Error::msg(format!(
        "Facture with id {} not found",
        facture_id
    )))?;

    let result = sqlx::query(
        "UPDATE factures SET date = ?, paper_ref = ?, updated_at = datetime('now') WHERE id = ?",
    )
    .bind(date)
    .bind(paper_ref)
    .bind(facture_id)
    .execute(&mut *tx)
    .await
    .with_context(|| format!("Failed to update facture {}", facture_id))?;

    tx.commit().await.context("Failed to commit transaction")?;

    Ok(result.rows_affected())
}

// Phase 6: Facture Items CRUD operations

use crate::server::models::facture_items::{FactureItemForm, FactureItemInsert};

/// Insert a new facture item
pub async fn insert_facture_item(
    pool: &SqlitePool,
    facture_id: i64,
    form: FactureItemForm,
) -> Result<i64> {
    let mut tx = pool.begin().await.context("Failed to begin transaction")?;

    // Verify facture exists and get its type
    let maybe_facture = FactureRow::select_one(facture_id, &mut *tx).await?;
    let facture = maybe_facture.ok_or(anyhow::Error::msg(format!(
        "Facture with id {} not found",
        facture_id
    )))?;

    let item_type = facture
        .facture_type
        .unwrap_or_else(|| "Product".to_string());

    let quantity: Option<i64> = form
        .quantity
        .as_deref()
        .map(|r| r.parse())
        .transpose()
        .map_err(anyhow::Error::msg)?;

    let price = form
        .price
        .as_deref()
        .map(parse_money)
        .transpose()
        .map_err(anyhow::Error::msg)?;
    let insurance = form
        .insurance
        .as_deref()
        .map(parse_money)
        .transpose()
        .map_err(anyhow::Error::msg)?;
    let other_costs = form
        .other_costs
        .as_deref()
        .map(parse_money)
        .transpose()
        .map_err(anyhow::Error::msg)?;
    let rebate_dollar = form
        .rebate_dollar
        .as_deref()
        .map(parse_money)
        .transpose()
        .map_err(anyhow::Error::msg)?;
    let rebate_percent: Option<i64> = form
        .rebate_percent
        .as_deref()
        .filter(|r| !r.is_empty())
        .map(|r| r.parse())
        .transpose()
        .map_err(anyhow::Error::msg)?;
    let extra_large_size = form
        .extra_large_size
        .filter(|_| form.extra_large_size_checked.unwrap_or(false));

    let chest: Option<i64> = form
        .chest
        .as_deref()
        .filter(|r| !r.is_empty())
        .map(|r| r.parse())
        .transpose()
        .map_err(anyhow::Error::msg)?;
    let waist: Option<i64> = form
        .waist
        .as_deref()
        .filter(|r| !r.is_empty())
        .map(|r| r.parse())
        .transpose()
        .map_err(anyhow::Error::msg)?;
    let hips: Option<i64> = form
        .hips
        .as_deref()
        .filter(|r| !r.is_empty())
        .map(|r| r.parse())
        .transpose()
        .map_err(anyhow::Error::msg)?;

    let to_insert = FactureItemInsert {
        facture_id,
        product_id: form.product_id,
        item_type,
        price,
        notes: form.notes,
        quantity: quantity.unwrap_or(1),
        extra_large_size,
        rebate_percent,
        size: form.size,
        chest,
        waist,
        hips,
        color: form.color,
        beneficiary: form.beneficiary,
        floor_item: form.floor_item.unwrap_or(false),
        insurance,
        other_costs,
        rebate_dollar,
    };

    let inserted_id = to_insert
        .insert_one(&mut tx)
        .await?
        .expect("An ID should be generated for a new FactureItem");

    // If floor_item is true, create initial status record
    if to_insert.floor_item {
        use crate::server::models::statuts::StatutInsert;

        // Get today's date from SQLite
        let today: (String,) = sqlx::query_as("SELECT date('now')")
            .fetch_one(&mut *tx)
            .await
            .context("Failed to get current date")?;

        let initial_status = StatutInsert {
            facture_id,
            facture_item_id: inserted_id,
            statut_type: "RecordingOutDate".to_string(), // Initial status for floor items
            date: today.0,
            seamstress: None,
        };
        initial_status.insert_one(&mut tx).await?;
    }

    tx.commit().await.context("Failed to commit transaction")?;

    Ok(inserted_id)
}

/// Update an existing facture item
pub async fn update_facture_item(
    pool: &SqlitePool,
    item_id: i64,
    form: FactureItemForm,
) -> Result<u64> {
    let mut tx = pool.begin().await.context("Failed to begin transaction")?;

    // Verify item exists
    let maybe_item = FactureItemRow::select_one(item_id, &mut *tx).await?;
    let _existing_item = maybe_item.ok_or(anyhow::Error::msg(format!(
        "Facture item with id {} not found",
        item_id
    )))?;

    let floor_item = form.floor_item.unwrap_or(false);

    let quantity: Option<i64> = form
        .quantity
        .as_deref()
        .map(|r| r.parse())
        .transpose()
        .map_err(anyhow::Error::msg)?;
    let price = form
        .price
        .as_deref()
        .map(parse_money)
        .transpose()
        .map_err(anyhow::Error::msg)?;
    let insurance = form
        .insurance
        .as_deref()
        .map(parse_money)
        .transpose()
        .map_err(anyhow::Error::msg)?;
    let other_costs = form
        .other_costs
        .as_deref()
        .map(parse_money)
        .transpose()
        .map_err(anyhow::Error::msg)?;
    let rebate_dollar = form
        .rebate_dollar
        .as_deref()
        .map(parse_money)
        .transpose()
        .map_err(anyhow::Error::msg)?;
    let rebate_percent: Option<i64> = form
        .rebate_percent
        .as_deref()
        .filter(|r| !r.is_empty())
        .map(|r| r.parse::<i64>())
        .transpose()
        .map_err(anyhow::Error::msg)?;
    let extra_large_size = form
        .extra_large_size
        .filter(|_| form.extra_large_size_checked.unwrap_or(false));

    let chest: Option<i64> = form
        .chest
        .as_deref()
        .filter(|r| !r.is_empty())
        .map(|r| r.parse())
        .transpose()
        .map_err(anyhow::Error::msg)?;

    let waist: Option<i64> = form
        .waist
        .as_deref()
        .filter(|r| !r.is_empty())
        .map(|r| r.parse())
        .transpose()
        .map_err(anyhow::Error::msg)?;

    let hips: Option<i64> = form
        .hips
        .as_deref()
        .filter(|r| !r.is_empty())
        .map(|r| r.parse())
        .transpose()
        .map_err(anyhow::Error::msg)?;

    // If floor_item changes to true, sanitize detail fields
    let (size, chest, waist, hips, color) = if floor_item {
        (None, None, None, None, None)
    } else {
        (form.size, chest, waist, hips, form.color)
    };

    let result = sqlx::query(
        "UPDATE facture_items SET
            product_id = ?, quantity = ?, price = ?, notes = ?,
            size = ?, chest = ?, waist = ?, hips = ?, color = ?,
            beneficiary = ?, floor_item = ?, extra_large_size = ?, rebate_percent = ?,
            insurance = ?, other_costs = ?, rebate_dollar = ?,
            updated_at = datetime('now')
         WHERE id = ?",
    )
    .bind(form.product_id)
    .bind(quantity.unwrap_or(1))
    .bind(price)
    .bind(form.notes)
    .bind(size)
    .bind(chest)
    .bind(waist)
    .bind(hips)
    .bind(color)
    .bind(form.beneficiary)
    .bind(if floor_item { 1 } else { 0 })
    .bind(extra_large_size)
    .bind(rebate_percent)
    .bind(insurance)
    .bind(other_costs)
    .bind(rebate_dollar)
    .bind(item_id)
    .execute(&mut *tx)
    .await
    .with_context(|| format!("Failed to update facture item {}", item_id))?;

    tx.commit().await.context("Failed to commit transaction")?;

    Ok(result.rows_affected())
}

/// Delete a facture item and its associated status records
pub async fn delete_facture_item(pool: &SqlitePool, item_id: i64) -> Result<u64> {
    let mut tx = pool.begin().await.context("Failed to begin transaction")?;

    // Verify item exists
    let maybe_item = FactureItemRow::select_one(item_id, &mut *tx).await?;
    maybe_item.ok_or(anyhow::Error::msg(format!(
        "Facture item with id {} not found",
        item_id
    )))?;

    // Delete associated status records first
    sqlx::query("DELETE FROM statuts WHERE facture_item_id = ?")
        .bind(item_id)
        .execute(&mut *tx)
        .await
        .with_context(|| format!("Failed to delete statuses for facture item {}", item_id))?;

    // Delete facture item
    let result = sqlx::query("DELETE FROM facture_items WHERE id = ?")
        .bind(item_id)
        .execute(&mut *tx)
        .await
        .with_context(|| format!("Failed to delete facture item {}", item_id))?;

    tx.commit().await.context("Failed to commit transaction")?;

    Ok(result.rows_affected())
}
