use anyhow::Result;
use sqlx::prelude::FromRow;

/// Database row structure for statuts table
#[derive(Debug, FromRow)]
pub struct StatutRow {
    pub id: i64,
    pub facture_id: i64,      // Required FK to factures
    pub facture_item_id: i64, // Required FK to facture_items
    pub statut_type: String,  // Type of status
    pub date: String,
    pub seamstress: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug)]
pub struct StatutInsert {
    pub facture_id: i64,      // Required FK to factures
    pub facture_item_id: i64, // Required FK to facture_items
    pub statut_type: String,  // Type of status
    pub date: String,
    pub seamstress: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum State<Date, Seamstress> {
    // initial states
    FloorItem,
    ToOrder,
    ToBeAltered,

    // state after n transitions
    BackFromLocation(Date),
    BackFromSeamstress(Date),
    BackOrder(Date),
    Cancelled(Date),
    ExpectingDelivery(Date),
    GivenToSeamstress(Date, Seamstress),
    Ordered(Date),
    OutForLocation(Date),
    Received(Date),
    TransferredToAlteration(Date),
    WaitingAdjustment(Date),
    WaitingForSeamstress(Date),

    // final states
    ItemOut(Date),
    LocationOut(Date),

    // invalid states, for example: if the user change a user record status for an item then change its type
    // by setting floor_item to true, then some states exists when they should not
    Invalid(Date),
}

impl State<String, String> {
    pub fn date(&self) -> Option<&str> {
        match self {
            State::FloorItem | State::ToOrder | State::ToBeAltered => None,
            State::BackFromLocation(date) => Some(date),
            State::BackFromSeamstress(date) => Some(date),
            State::BackOrder(date) => Some(date),
            State::Cancelled(date) => Some(date),
            State::ExpectingDelivery(date) => Some(date),
            State::GivenToSeamstress(date, _) => Some(date),
            State::Ordered(date) => Some(date),
            State::OutForLocation(date) => Some(date),
            State::Received(date) => Some(date),
            State::TransferredToAlteration(date) => Some(date),
            State::WaitingAdjustment(date) => Some(date),
            State::WaitingForSeamstress(date) => Some(date),
            State::ItemOut(date) => Some(date),
            State::LocationOut(date) => Some(date),
            State::Invalid(date) => Some(date),
        }
    }
    pub fn label(&self) -> &str {
        match self {
            State::BackFromLocation(_) => "À retourner au locateur",
            State::BackFromSeamstress(_) => "Couture terminé",
            State::BackOrder(_) => "Back Order",
            State::Cancelled(_) => "Abandonné",
            State::ExpectingDelivery(_) => "À recevoir",
            State::FloorItem => "Item plancher",
            State::GivenToSeamstress(_, _) => "Remis à la couturière",
            State::ItemOut(_) => "Sortie",
            State::LocationOut(_) => "Retour locateur",
            State::Ordered(_) => "Commandé",
            State::OutForLocation(_) => "Sortie pour location",
            State::Received(_) => "Reçu",
            State::ToBeAltered => "À altérer",
            State::ToOrder => "À commander",
            State::TransferredToAlteration(_) => "Transférer en altération",
            State::WaitingAdjustment(_) => "À ajuster",
            State::WaitingForSeamstress(_) => "Remis à la couturière",
            State::Invalid(_) => "État invalide",
        }
    }
    pub fn label_with_date(&self) -> String {
        match self {
            State::BackFromLocation(date) => format!("Retour par le client: {}", date),
            State::BackFromSeamstress(date) => format!("Couture terminé le: {}", date),
            State::BackOrder(date) => format!("Back Order le: {}", date),
            State::Cancelled(date) => format!("Annulé le: {}", date),
            State::ExpectingDelivery(date) => format!("Livraison attendue le:: {}", date),
            State::FloorItem => format!("Item plancher"),
            State::GivenToSeamstress(date, st) => format!("Remise à {} le:  {}", st, date),
            State::ItemOut(date) => format!("Sortie le: {}", date),
            State::LocationOut(date) => format!("Sortie le: {}", date),
            State::Ordered(date) => format!("Commande placée le: {}", date),
            State::OutForLocation(date) => format!("Sortie en location le: {}", date),
            State::Received(date) => format!("Date de réception: {}", date),
            State::ToBeAltered => "À altérer".to_string(),
            State::ToOrder => "À commander".to_string(),
            State::TransferredToAlteration(date) => format!("Transfert en altération le: {}", date),
            State::WaitingAdjustment(date) => format!("Passé en ajustement le: {}", date),
            State::WaitingForSeamstress(date) => format!("Remise à la couturière le: {}", date),
            State::Invalid(date) => format!("État invalide le : {}", date),
        }
    }
    pub fn value(&self) -> u8 {
        match self {
            State::FloorItem => 4,
            State::ToOrder => 1,
            State::ToBeAltered => 1,
            State::BackFromLocation(_) => 1,
            State::BackFromSeamstress(_) => 4,
            State::BackOrder(_) => 1,
            State::Cancelled(_) => 8,
            State::ExpectingDelivery(_) => 3,
            State::GivenToSeamstress(_, _) => 2,
            State::Ordered(_) => 2,
            State::OutForLocation(_) => 3,
            State::Received(_) => 4,
            State::TransferredToAlteration(_) => 6,
            State::WaitingAdjustment(_) => 1,
            State::WaitingForSeamstress(_) => 2,
            State::ItemOut(_) => 7,
            State::LocationOut(_) => 7,
            State::Invalid(_) => 7,
        }
    }
}

pub enum Statut {
    PlaceOrder,
    RecordExpectedDeliveryDate,
    RecordReceptionDate,
    RecordingBackFromSeamstressDate,
    RecordingClientReturnDate,
    RecordingCancelDate,
    RecordingOutDate,
    RecordingOutForLocationDate,
    RecordingBackOrderDate,
    RecordingTransfertToSeamstressDate,
    TransfertToAlteration,
}
pub enum StateType {
    AlterationFlow,
    LocationFlow,
    AccessoryItemFlow,
    DressFloorItemFlow,
    DressToOrderFlow,
}

#[derive(Clone, Debug)]
pub struct StateView {
    pub current_state: State<String, String>,
    pub previous_states: Vec<State<String, String>>,
    pub item_flow: String,
}

pub const FLOOR_ITEM_INITIAL_TRANSITIONS: [&'static str; 2] =
    ["RecordingOutDate", "TransfertToAlteration"];

impl StateView {
    pub fn available_transition(&self) -> Result<Vec<&str>> {
        match self.item_flow.as_ref() {
            "DressToOrderFlow" => Ok(match &self.current_state {
                State::ToOrder => vec!["PlaceOrder"],
                State::BackOrder(_) => vec!["RecordReceptionDate", "RecordingCancelDate"],
                State::Ordered(_) => vec!["RecordExpectedDeliveryDate"],
                State::ExpectingDelivery(_) => {
                    vec!["RecordReceptionDate", "RecordingBackOrderDate"]
                }
                State::Received(_) => vec![
                    "RecordingOutDate",
                    "TransfertToAlteration",
                    "RecordingCancelDate",
                ],
                // final states
                State::ItemOut(_) | State::TransferredToAlteration(_) | State::Cancelled(_) => {
                    vec![]
                }
                _ => vec![],
            }),

            "DressFloorItemFlow" => Ok(match &self.current_state {
                State::FloorItem => Vec::from(FLOOR_ITEM_INITIAL_TRANSITIONS),
                State::BackFromSeamstress(_) => vec!["RecordingOutDate"],
                // final states
                State::ItemOut(_) | State::TransferredToAlteration(_) => vec![],
                _ => vec![],
            }),

            "AccessoryItemFlow" => Ok(vec![]),

            "AlterationFlow" => Ok(match &self.current_state {
                State::ToBeAltered => vec!["RecordingTransfertToSeamstressDate"],
                State::GivenToSeamstress(_, _) => vec!["RecordingBackFromSeamstressDate"],
                State::BackFromSeamstress(_) => vec!["RecordingOutDate"],
                // final states
                State::ItemOut(_) => vec![],
                _ => vec![],
            }),

            "LocationFlow" => Ok(match &self.current_state {
                State::ToOrder => vec!["PlaceOrder"],
                State::Ordered(_) => vec!["RecordExpectedDeliveryDate"],
                State::ExpectingDelivery(_) => vec!["RecordReceptionDate"],
                State::Received(_) => vec!["RecordAdjustDate", "RecordingOutForLocationDate"],
                State::WaitingAdjustment(_) => vec!["RecordingTransfertToSeamstressDate"],
                State::WaitingForSeamstress(_) => vec!["RecordingBackFromSeamstressDate"],
                State::BackFromSeamstress(_) => vec!["RecordingOutForLocationDate"],
                State::OutForLocation(_) => vec!["RecordingClientReturnDate"],
                State::BackFromLocation(_) => vec!["RecordingOutDate"],
                // final states
                State::ItemOut(_) => vec![],
                _ => vec![],
            }),

            _ => anyhow::bail!("Invalid flow type"),
        }
    }
}

#[derive(Debug, FromRow)]
pub struct StatutsView {
    pub id: i64,
    pub facture_id: i64,      // Required FK to factures
    pub facture_item_id: i64, // Required FK to facture_items
    pub statut_type: String,  // Type of status
    pub date: String,
    pub seamstress: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}
