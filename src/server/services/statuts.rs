use anyhow::Result;
use std::{collections::HashMap, vec};

use crate::server::models::{
    facture_items::ItemFactureFlowType,
    statuts::{State, StateView, StatutRow},
};

pub fn load_one_item_statuts_flow(
    facture_item_flow_type: ItemFactureFlowType,
    data: Vec<StatutRow>,
) -> Result<StateView> {
    let mut state = initial_state(&facture_item_flow_type.flow_type)?;

    for statut_row in data {
        let new_state = apply_statut(&state, &statut_row)?;
        state = new_state;
    }

    Ok(state)
}

pub fn load_statuts_flow(
    facture_item_flows: Vec<ItemFactureFlowType>,
    data: Vec<StatutRow>,
) -> Result<HashMap<(i64, i64), StateView>> {
    let result: Result<HashMap<(i64, i64), StateView>> = facture_item_flows
        .into_iter()
        .map(|f| {
            initial_state(&f.flow_type).map(|state| ((f.facture_id, f.facture_item_id), state))
        })
        .collect();

    let mut result = result?;

    for statut_row in data {
        let found = result.get_mut(&(statut_row.facture_id, statut_row.facture_item_id));
        match found {
            Some(state) => {
                let new_state = apply_statut(&state, &statut_row)?;
                result.insert(
                    (statut_row.facture_id, statut_row.facture_item_id),
                    new_state,
                );
            }
            None => anyhow::bail!(
                "No state recorded for {}, {}",
                statut_row.facture_id,
                statut_row.facture_item_id
            ),
        }
    }

    Ok(result)
}

fn initial_state(flow_type: &str) -> Result<StateView> {
    let state: Result<State<String, String>> = match flow_type {
        "AlterationFlow" => Ok(State::ToBeAltered),
        "LocationFlow" => Ok(State::ToOrder),
        "AccessoryItemFlow" => Ok(State::FloorItem),
        "DressFloorItemFlow" => Ok(State::FloorItem),
        "DressToOrderFlow" => Ok(State::ToOrder),
        _ => anyhow::bail!("Unsupported flow_type."),
    };
    Ok(StateView {
        item_flow: String::from(flow_type),
        current_state: state?,
        previous_states: vec![],
    })
}

/// The state argument is unused right now, but it could be used
/// to enforce only specific transitions, at specific states
fn apply_statut(old_state: &StateView, statut: &StatutRow) -> Result<StateView> {
    let date = statut.date.clone();
    let old_state = old_state.clone();
    let state: Result<State<String, String>> = match old_state.item_flow.as_ref() {
        "AlterationFlow" => match statut.statut_type.as_str() {
            "RecordingTransfertToSeamstressDate" => {
                let seamstress = statut
                    .seamstress
                    .clone()
                    .ok_or(anyhow::Error::msg("Required seamstress."))?;
                Ok(State::GivenToSeamstress(date, seamstress))
            }
            "RecordingBackFromSeamstressDate" => Ok(State::BackFromSeamstress(date)),
            "RecordingOutDate" => Ok(State::ItemOut(date)),
            _ => {
                // no invalid state possible
                anyhow::bail!("Invalid state transition for AlterationFlow")
            }
        },
        "LocationFlow" => match statut.statut_type.as_str() {
            "PlaceOrder" => Ok(State::Ordered(date)),
            "RecordExpectedDeliveryDate" => Ok(State::ExpectingDelivery(date)),
            "RecordReceptionDate" => Ok(State::Received(date)),
            "RecordAdjustDate" => Ok(State::WaitingAdjustment(date)),
            "RecordingTransfertToSeamstressDate" => Ok(State::WaitingForSeamstress(date)),
            "RecordingBackFromSeamstressDate" => Ok(State::BackFromSeamstress(date)),
            "RecordingOutForLocationDate" => Ok(State::OutForLocation(date)),
            "RecordingClientReturnDate" => Ok(State::BackFromLocation(date)),
            "RecordingOutDate" => Ok(State::LocationOut(date)),
            _ => {
                // no invalid state possible
                anyhow::bail!("Invalid state transition for LocationFlow")
            }
        },
        "AccessoryItemFlow" => match statut.statut_type.as_str() {
            "TransfertToAlteration" => Ok(State::TransferredToAlteration(date)),
            "RecordingOutDate" => Ok(State::ItemOut(date)),
            _ => {
                // no invalid state possible
                anyhow::bail!("Invalid state transition for AccessoryItemFlow")
            }
        },
        "DressFloorItemFlow" => match statut.statut_type.as_str() {
            "TransfertToAlteration" => Ok(State::TransferredToAlteration(date)),
            "RecordingOutDate" => Ok(State::ItemOut(date)),
            _ => {
                // invalid state possible (when toggling on floor_item)
                Ok(State::Invalid(date))
            }
        },
        "DressToOrderFlow" => match statut.statut_type.as_str() {
            "RecordingBackOrderDate" => Ok(State::BackOrder(date)),
            "PlaceOrder" => Ok(State::Ordered(date)),
            "RecordExpectedDeliveryDate" => Ok(State::ExpectingDelivery(date)),
            "RecordReceptionDate" => Ok(State::Received(date)),
            "RecordingOutDate" => Ok(State::ItemOut(date)),
            "RecordingCancelDate" => Ok(State::Cancelled(date)),
            "TransfertToAlteration" => Ok(State::TransferredToAlteration(date)),
            _ => {
                // invalid state possible (when toggling on floor_item)
                Ok(State::Invalid(date))
            }
        },
        _ => anyhow::bail!("Unsupported flow_type."),
    };

    let mut previous_states = old_state.previous_states;
    previous_states.push(old_state.current_state);

    Ok(StateView {
        current_state: state?,
        previous_states: previous_states,
        ..old_state
    })
}

#[cfg(test)]
#[test]
fn test_load_statuts_flow() {
    let facture_item_flows = vec![ItemFactureFlowType {
        facture_id: 159,
        facture_item_id: 2430,
        flow_type: "DressFloorItemFlow".to_string(),
    }];
    let statuts = vec![StatutRow {
        id: 1351,
        facture_id: 159,
        facture_item_id: 2430,
        statut_type: "RecordingOutDate".to_string(),
        date: "2020-07-28".to_string(),
        seamstress: None,
        created_at: "2026-03-30 15:51:52".to_string(),
        updated_at: "2026-03-30 15:51:52".to_string(),
    }];
    let result = load_statuts_flow(facture_item_flows, statuts).unwrap();
    let result = result.get(&(159, 2430)).unwrap();
    assert_eq!(
        result.current_state,
        State::ItemOut("2020-07-28".to_string())
    );
}

#[cfg(test)]
#[test]
fn test_load_statuts_flow_dress_to_order() {
    let facture_item_flows = vec![ItemFactureFlowType {
        facture_id: 2573,
        facture_item_id: 1226,
        flow_type: "DressToOrderFlow".to_string(),
    }];
    let statuts = vec![
        StatutRow {
            id: 4810,
            facture_id: 2573,
            facture_item_id: 1226,
            statut_type: "PlaceOrder".to_string(),
            date: "2026-02-06".to_string(),
            seamstress: None,
            created_at: "2026-04-06 14:31:40".to_string(),
            updated_at: "2026-04-06 14:31:40".to_string(),
        },
        StatutRow {
            id: 3934,
            facture_id: 2573,
            facture_item_id: 1226,
            statut_type: "RecordExpectedDeliveryDate".to_string(),
            date: "2026-02-06".to_string(),
            seamstress: None,
            created_at: "2026-04-06 14:31:43".to_string(),
            updated_at: "2026-04-06 14:31:43".to_string(),
        },
    ];
    let result = load_statuts_flow(facture_item_flows, statuts).unwrap();
    let result = result.get(&(2573, 1226)).unwrap();
    assert_eq!(
        result.current_state,
        State::ExpectingDelivery("2026-02-06".to_string())
    );
}
