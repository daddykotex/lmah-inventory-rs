use crate::server::models::{
    clients::ClientView,
    factures::FactureView,
    statuts::{State, StateView},
};

pub mod clients;
pub mod config;
pub mod events;
pub mod facture_items;
pub mod factures;
pub mod payments;
pub mod product_types;
pub mod products;
pub mod refunds;
pub mod statuts;

pub struct FactureWithRelatedData {
    pub facture: FactureView,
    pub client: ClientView,
    pub state_per_item: Vec<(i64, StateView)>,
}

impl FactureWithRelatedData {
    pub fn seamstresses(&self) -> Vec<String> {
        self.state_per_item
            .iter()
            .filter_map(|(_, state)| match &state.state {
                State::GivenToSeamstress(_, seamstress) => Some(seamstress.clone()),
                _ => None,
            })
            .collect()
    }

    pub fn smallest_state(&self) -> Option<StateView> {
        self.state_per_item
            .iter()
            .min_by_key(|&a| a.1.state.value())
            .map(|(_, state)| state.clone())
    }
}
