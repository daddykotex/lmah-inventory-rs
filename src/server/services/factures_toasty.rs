use anyhow::Result;
use toasty::Db;

use crate::server::models::{
    FactureDashboardData,
    factures::Facture,
    statuts::{State, StateView, Statut},
};

/// Determines the flow type for a facture item based on its type, product types, and floor_item flag
fn calculate_flow_type(item_type: &str, product_type_names: &[String], floor_item: bool) -> String {
    match item_type {
        "Alteration" => "AlterationFlow".to_string(),
        "Location" => "LocationFlow".to_string(),
        _ => {
            // Check if it's a dress product type
            let is_dress = product_type_names.iter().any(|name| {
                matches!(
                    name.as_str(),
                    "Robe de mariée"
                        | "Robe de mère de la mariée"
                        | "Robe de bal"
                        | "Robe de bouquetière"
                )
            });

            if !is_dress {
                "AccessoryItemFlow".to_string()
            } else if floor_item {
                "DressFloorItemFlow".to_string()
            } else {
                "DressToOrderFlow".to_string()
            }
        }
    }
}

/// Creates the initial state for a given flow type
fn initial_state(flow_type: &str) -> Result<State<String, String>> {
    match flow_type {
        "AlterationFlow" => Ok(State::ToBeAltered),
        "LocationFlow" => Ok(State::ToOrder),
        "AccessoryItemFlow" => Ok(State::FloorItem),
        "DressFloorItemFlow" => Ok(State::FloorItem),
        "DressToOrderFlow" => Ok(State::ToOrder),
        _ => anyhow::bail!("Unsupported flow_type: {}", flow_type),
    }
}

/// Applies a statut transition to build the next state
fn apply_statut(
    old_state: &StateView,
    statut_type: &str,
    date: String,
    seamstress: Option<String>,
) -> Result<StateView> {
    let state: Result<State<String, String>> = match old_state.item_flow.as_ref() {
        "AlterationFlow" => match statut_type {
            "RecordingTransfertToSeamstressDate" => {
                let seamstress =
                    seamstress.ok_or_else(|| anyhow::Error::msg("Required seamstress."))?;
                Ok(State::GivenToSeamstress(date, seamstress))
            }
            "RecordingBackFromSeamstressDate" => Ok(State::BackFromSeamstress(date)),
            "RecordingOutDate" => Ok(State::ItemOut(date)),
            _ => anyhow::bail!("Invalid state transition for AlterationFlow"),
        },
        "LocationFlow" => match statut_type {
            "PlaceOrder" => Ok(State::Ordered(date)),
            "RecordExpectedDeliveryDate" => Ok(State::ExpectingDelivery(date)),
            "RecordReceptionDate" => Ok(State::Received(date)),
            "RecordAdjustDate" => Ok(State::WaitingAdjustment(date)),
            "RecordingTransfertToSeamstressDate" => Ok(State::WaitingForSeamstress(date)),
            "RecordingBackFromSeamstressDate" => Ok(State::BackFromSeamstress(date)),
            "RecordingOutForLocationDate" => Ok(State::OutForLocation(date)),
            "RecordingClientReturnDate" => Ok(State::BackFromLocation(date)),
            "RecordingOutDate" => Ok(State::LocationOut(date)),
            _ => anyhow::bail!("Invalid state transition for LocationFlow"),
        },
        "AccessoryItemFlow" => match statut_type {
            "TransfertToAlteration" => Ok(State::TransferredToAlteration(date)),
            "RecordingOutDate" => Ok(State::ItemOut(date)),
            _ => anyhow::bail!("Invalid state transition for AccessoryItemFlow"),
        },
        "DressFloorItemFlow" => match statut_type {
            "TransfertToAlteration" => Ok(State::TransferredToAlteration(date)),
            "RecordingOutDate" => Ok(State::ItemOut(date)),
            _ => Ok(State::Invalid(date)),
        },
        "DressToOrderFlow" => match statut_type {
            "RecordingBackOrderDate" => Ok(State::BackOrder(date)),
            "PlaceOrder" => Ok(State::Ordered(date)),
            "RecordExpectedDeliveryDate" => Ok(State::ExpectingDelivery(date)),
            "RecordReceptionDate" => Ok(State::Received(date)),
            "RecordingOutDate" => Ok(State::ItemOut(date)),
            "RecordingCancelDate" => Ok(State::Cancelled(date)),
            "TransfertToAlteration" => Ok(State::TransferredToAlteration(date)),
            _ => Ok(State::Invalid(date)),
        },
        _ => anyhow::bail!("Unsupported flow_type."),
    };

    let mut previous_states = old_state.previous_states.clone();
    previous_states.push(old_state.current_state.clone());

    Ok(StateView {
        current_state: state?,
        previous_states,
        item_flow: old_state.item_flow.clone(),
    })
}

/// Builds a StateView from a list of statuts for a facture item
fn build_state_view(flow_type: String, statuts: Vec<&Statut>) -> Result<StateView> {
    let mut state = StateView {
        item_flow: flow_type.clone(),
        current_state: initial_state(&flow_type)?,
        previous_states: vec![],
    };

    for statut in statuts {
        state = apply_statut(
            &state,
            &statut.statut_type,
            statut.date.clone(),
            statut.seamstress.clone(),
        )?;
    }

    Ok(state)
}

pub async fn select_all(db: &mut Db) -> Result<Vec<FactureDashboardData>> {
    let factures = Facture::all()
        .include(Facture::fields().client())
        .include(Facture::fields().facture_items())
        .include(Facture::fields().facture_items().product())
        .include(Facture::fields().facture_items().statuts())
        .include(Facture::fields().facture_items().product().product_types())
        .include(
            Facture::fields()
                .facture_items()
                .product()
                .product_types()
                .product_type(),
        )
        .exec(db)
        .await?;

    let result = factures
        .iter()
        .map(|facture| {
            // Collect all facture items with their states
            let state_per_item: Result<Vec<(u64, StateView)>> = facture
                .facture_items
                .get()
                .iter()
                .map(|item| {
                    // Get product and its types
                    let product = item.product.get();
                    let product_type_names: Vec<String> = product
                        .product_types
                        .get()
                        .iter()
                        .map(|pt| pt.product_type.get().name.clone())
                        .collect();

                    // Calculate flow type
                    let flow_type =
                        calculate_flow_type(&item.item_type, &product_type_names, item.floor_item);

                    // Get all statuts for this item, sorted by creation date
                    let mut statuts: Vec<&Statut> = item.statuts.get().iter().collect();
                    statuts.sort_by(|a, b| a.created_at.cmp(&b.created_at));

                    // Build state view
                    let state = build_state_view(flow_type, statuts)?;

                    Ok((item.id, state))
                })
                .collect();

            let state_per_item = state_per_item?;

            Ok(FactureDashboardData {
                facture: facture.into(),
                client: facture.client.get().into(),
                state_per_item,
            })
        })
        .collect();

    result
}
