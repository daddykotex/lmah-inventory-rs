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
    pub state: State<String, String>,
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

//   1. DressToOrderFlow (Dresses made to order)

//   - ToOrder (value: 1) - "À commander"
//   - BackOrder (value: 1) - "Back Order"
//   - Ordered (value: 2) - "Commandé"
//   - ExpectingDelivery (value: 3) - "À recevoir"
//   - Received (value: 4) - "Reçu"
//   - TransferedToAlteration (value: 6) - "Transférer en altération"
//   - ItemOut (value: 7) - "Sortie"
//   - Cancelled (value: 8) - "Abandonné"

//   2. DressFloorItemFlow (Floor item dresses)

//   - FloorItem (value: 4) - "Item plancher"
//   - TransferedToAlteration (value: 6) - "Transférer en altération"
//   - ItemOut (value: 7) - "Sortie"

//   3. AccessoryItemFlow (Accessory items)

//   - ItemOut (value: 7) - "Sortie" (only state, items go directly out)

//   4. AlterationFlow (Alteration items)

//   - ToBeAltered (value: 1) - "À altérer"
//   - GivenToSeamstress (value: 2) - "Remis à la couturière"
//   - BackFromSeamstress (value: 4) - "Couture terminé"
//   - ItemOut (value: 7) - "Sortie"

//   5. LocationFlow (Rental items)

//   - ToOrder (value: 1) - "À commander"
//   - Ordered (value: 2) - "Commandé"
//   - ExpectingDelivery (value: 3) - "À recevoir"
//   - Received (value: 4) - "Reçu"
//   - WaitingAdjustment (value: 1) - "À ajuster"
//   - WaitingForSeamstress (value: 2) - "Remis à la couturière"
//   - BackFromSeamstress (value: 4) - "Couture terminé"
//   - OutForLocation (value: 3) - "Sortie pour location"
//   - BackFromLocation (value: 1) - "À retourner au locateur"
//   - ItemOut (value: 7) - "Retour locateur"
