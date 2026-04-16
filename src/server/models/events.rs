use serde::Deserialize;

/// Event model with Toasty ORM
#[derive(Debug, toasty::Model)]
pub struct Event {
    #[key]
    #[auto]
    id: u64,

    name: String,
    event_type: String,
    date: String,
    created_at: String,
    updated_at: String,

    #[has_many]
    factures: toasty::HasMany<crate::server::models::factures::Facture>,
}

impl From<EventForm> for EventInsert {
    fn from(value: EventForm) -> Self {
        EventInsert {
            name: value.name,
            event_type: value.event_type,
            date: value.date,
        }
    }
}

/// Database row structure for events table
#[derive(Debug)]
pub struct EventInsert {
    pub name: String,
    pub event_type: String,
    pub date: String,
}

/// Received from the UI, can be transformed into an EventInsert
#[derive(Deserialize, Debug)]
pub struct EventForm {
    pub name: String,
    pub date: String,
    #[serde(rename = "type")]
    pub event_type: String,
}

/// Used in the UI to display Event information
/// Very close if not the same to the Event model
#[derive(Clone)]
pub struct EventView {
    pub id: u64,
    pub name: String,
    pub event_type: String,
    pub date: String,
}

impl From<Event> for EventView {
    fn from(value: Event) -> Self {
        EventView {
            id: value.id,
            name: value.name,
            event_type: value.event_type,
            date: value.date,
        }
    }
}
